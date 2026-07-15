import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN || path.join(REPO_ROOT, "target/debug/wyvern");
const SHARED_API = path.join(REPO_ROOT, "ui/shared/wyvern-api.js");
const WORKSPACE_HINT_UI = path.join(REPO_ROOT, "examples/wizards/workspace-hint");
const WORKSPACE_HINT_JSON = path.join(WORKSPACE_HINT_UI, "wizard.json");

function waitForUrlFile(filePath: string, timeoutMs = 15_000): Promise<string> {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    const tick = () => {
      try {
        if (fs.existsSync(filePath)) {
          const url = fs.readFileSync(filePath, "utf8").trim();
          if (url.startsWith("http://")) {
            resolve(url);
            return;
          }
        }
      } catch {
        // retry
      }
      if (Date.now() - start > timeoutMs) {
        reject(new Error(`timed out waiting for dialog URL file: ${filePath}`));
        return;
      }
      setTimeout(tick, 50);
    };
    tick();
  });
}

function waitForExit(child: ChildProcessWithoutNullStreams): Promise<number> {
  if (child.exitCode !== null) {
    return Promise.resolve(child.exitCode);
  }
  return new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("close", (code) => resolve(code ?? -1));
  });
}

async function loadWyvernApi(page: import("@playwright/test").Page) {
  const apiSource = fs.readFileSync(SHARED_API, "utf8");
  await page.setContent(`<!DOCTYPE html>
<html><head></head>
<body>
  <div id="dialog" class="dialog dialog--compact">
    <h1>Hello</h1>
    <p>Short message body for auto-size.</p>
    <div class="content"><p>content</p></div>
    <div class="buttons"><button type="button">OK</button></div>
  </div>
</body></html>`);
  await page.addScriptTag({ content: apiSource });
  await page.waitForFunction(
    () => typeof (window as unknown as { WyvernApi: unknown }).WyvernApi !== "undefined",
  );
}

test("dialog fit-with-slack clamps to viewport × 0.92 (golden)", async ({
  page,
}) => {
  test.skip(!fs.existsSync(SHARED_API), `missing ${SHARED_API}`);

  await loadWyvernApi(page);

  const result = await page.evaluate(() => {
    const w = window as unknown as {
      __resizeLog: string[];
      ipc: { postMessage: (m: string) => void };
      WyvernApi: {
        applyDialogFitWithSlack: (
          m: { contentW: number; contentH: number },
          v: { available_width: number; available_height: number },
          s: number,
        ) => { w: number; h: number; clamped: boolean };
      };
    };
    w.__resizeLog = [];
    w.ipc = {
      postMessage(msg: string) {
        w.__resizeLog.push(msg);
      },
    };
    const sized = w.WyvernApi.applyDialogFitWithSlack(
      { contentW: 400, contentH: 200 },
      { available_width: 1000, available_height: 800 },
      1.25,
    );
    const oversized = w.WyvernApi.applyDialogFitWithSlack(
      { contentW: 2000, contentH: 1600 },
      { available_width: 1000, available_height: 800 },
      1.25,
    );
    return {
      sized,
      oversized,
      log: w.__resizeLog.slice(),
    };
  });

  expect(result.sized.w).toBe(500);
  expect(result.sized.h).toBe(250);
  expect(result.sized.clamped).toBe(false);
  expect(result.oversized.w).toBe(920); // floor(1000 * 0.92)
  expect(result.oversized.h).toBe(736); // floor(800 * 0.92)
  expect(result.oversized.clamped).toBe(true);
  expect(result.log.some((m) => m.startsWith("resize:"))).toBe(true);
});

test("workspace-hint honors estimated_size with viewport clamp", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WORKSPACE_HINT_JSON), `missing ${WORKSPACE_HINT_JSON}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-viewport-workspace-${process.pid}-${Date.now()}.txt`,
  );
  const wizardJson = fs.readFileSync(WORKSPACE_HINT_JSON, "utf8");

  let child: ChildProcessWithoutNullStreams | null = null;
  try {
    child = spawn(
      WYVERN_BIN,
      [wizardJson, "--viewer", "none", "--ui-root", WORKSPACE_HINT_UI],
      {
        cwd: REPO_ROOT,
        env: {
          ...process.env,
          WYVERN_DIALOG_URL_FILE: urlFile,
          WYVERN_LOG: "off",
        },
      },
    );
    const exitPromise = waitForExit(child);
    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoDialog(page, dialogUrl);

    await expect(page.getByTestId("workspace-canvas")).toBeVisible();
    await expect(page.getByTestId("estimated-size")).toContainText("960");
    await expect(page.getByTestId("estimated-size")).toContainText("640");

    const applied = await page.evaluate(async () => {
      const api = (window as unknown as {
        WyvernApi: {
          applyWorkspaceLayout: (
            state: unknown,
            vp: { available_width: number; available_height: number },
          ) => { w: number; h: number; layout: string };
          wyvernWizardState: () => Promise<unknown>;
        };
      }).WyvernApi;
      const state = await api.wyvernWizardState();
      return api.applyWorkspaceLayout(state, {
        available_width: 800,
        available_height: 600,
      });
    });

    expect(applied.layout).toBe("workspace");
    expect(applied.w).toBe(736); // floor(800 * 0.92)
    expect(applied.h).toBe(552); // floor(600 * 0.92)

    const unclamped = await page.evaluate(async () => {
      const api = (window as unknown as {
        WyvernApi: {
          applyWorkspaceLayout: (
            state: unknown,
            vp: { available_width: number; available_height: number },
          ) => { w: number; h: number };
          wyvernWizardState: () => Promise<unknown>;
        };
      }).WyvernApi;
      const state = await api.wyvernWizardState();
      return api.applyWorkspaceLayout(state, {
        available_width: 1920,
        available_height: 1080,
      });
    });
    expect(unclamped.w).toBe(960);
    expect(unclamped.h).toBe(640);

    if (child && child.exitCode === null && !child.killed) {
      child.kill("SIGTERM");
    }
    await exitPromise.catch(() => -1);
  } finally {
    if (child && child.exitCode === null && !child.killed) {
      child.kill("SIGTERM");
    }
    try {
      fs.unlinkSync(urlFile);
    } catch {
      // ignore
    }
  }
});

test("viewport-bounds event feeds applyWizardLayout", async ({ page }) => {
  test.skip(!fs.existsSync(SHARED_API), `missing ${SHARED_API}`);

  await loadWyvernApi(page);

  const sized = await page.evaluate(() => {
    const w = window as unknown as {
      ipc: { postMessage: (m: string) => void };
      WyvernApi: {
        applyWizardLayout: (
          state: { page: { layout: string }; config: Record<string, unknown> },
          vp?: { available_width: number; available_height: number },
        ) => { w: number; h: number };
      };
    };
    w.ipc = { postMessage() {} };
    window.dispatchEvent(
      new CustomEvent("wyvern:viewport-bounds", {
        detail: { available_width: 1200, available_height: 900 },
      }),
    );
    return w.WyvernApi.applyWizardLayout(
      {
        page: { layout: "dialog" },
        config: {},
      },
      { available_width: 1200, available_height: 900 },
    );
  });

  expect(sized.w).toBeGreaterThan(0);
  expect(sized.h).toBeGreaterThan(0);
  expect(sized.w).toBeLessThanOrEqual(Math.floor(1200 * 0.92));
  expect(sized.h).toBeLessThanOrEqual(Math.floor(900 * 0.92));
});
