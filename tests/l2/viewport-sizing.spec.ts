import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";
import type { Page } from "@playwright/test";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN || path.join(REPO_ROOT, "target/debug/wyvern");
const SHARED_API = path.join(REPO_ROOT, "ui/shared/wyvern-api.js");
const UI_ROOT = path.join(REPO_ROOT, "ui");
const WORKSPACE_HINT_UI = path.join(REPO_ROOT, "examples/wizards/workspace-hint");
const WORKSPACE_HINT_JSON = path.join(WORKSPACE_HINT_UI, "wizard.json");
const DIALOG_SLACK = 1.25;
const VIEWPORT_CLAMP = 0.92;

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

/** Navigate a blocking dialog URL; wait for /api/dialog (not wizard state). */
async function gotoBlockingDialog(page: Page, url: string, budgetMs = 15_000) {
  const deadline = Date.now() + budgetMs;
  let lastError: unknown;
  while (Date.now() < deadline) {
    try {
      await page.goto(url, { waitUntil: "domcontentloaded" });
      const apiUrl = new URL("/api/dialog", url).toString();
      const resp = await page.request.get(apiUrl);
      if (resp.ok()) {
        return;
      }
      lastError = new Error(`GET ${apiUrl} → ${resp.status()}`);
    } catch (err) {
      lastError = err;
    }
    await new Promise((r) => setTimeout(r, 100));
  }
  throw lastError;
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

type FitResult = {
  w: number;
  h: number;
  clamped: boolean;
  contentW: number;
  contentH: number;
  log: string[];
};

/** Install ipc stub + viewport bounds, then apply dialog fit against live DOM measure. */
async function firstOpenFitWithViewport(
  page: Page,
  viewport: { available_width: number; available_height: number },
): Promise<FitResult> {
  return page.evaluate((vp) => {
    const w = window as unknown as {
      __resizeLog: string[];
      __presentedAfterResize: boolean;
      ipc: { postMessage: (m: string) => void };
      WyvernApi: {
        measureNaturalContent: () => { contentW: number; contentH: number };
        applyDialogFitWithSlack: (
          m: { contentW: number; contentH: number },
          v: { available_width: number; available_height: number },
          s: number,
        ) => { w: number; h: number; clamped: boolean };
      };
    };
    w.__resizeLog = [];
    w.__presentedAfterResize = false;
    w.ipc = {
      postMessage(msg: string) {
        w.__resizeLog.push(msg);
        if (msg.startsWith("resize:")) {
          w.__presentedAfterResize = true;
        }
      },
    };
    window.dispatchEvent(
      new CustomEvent("wyvern:viewport-bounds", {
        detail: vp,
      }),
    );
    const measure = w.WyvernApi.measureNaturalContent();
    const sized = w.WyvernApi.applyDialogFitWithSlack(measure, vp, 1.25);
    return {
      ...sized,
      contentW: measure.contentW,
      contentH: measure.contentH,
      log: w.__resizeLog.slice(),
    };
  }, viewport);
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

test("message dialog first-open resize applies slack (golden L2)", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-viewport-message-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"message","title":"Viewport sizing","message":"Representative message payload for first-open auto-size with slack.","buttons":"ok"}';

  let child: ChildProcessWithoutNullStreams | null = null;
  try {
    child = spawn(WYVERN_BIN, [json, "--viewer", "none", "--ui-root", UI_ROOT], {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        WYVERN_DIALOG_URL_FILE: urlFile,
        WYVERN_LOG: "off",
      },
    });
    const exitPromise = waitForExit(child);
    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoBlockingDialog(page, dialogUrl);
    await expect(page.getByTestId("btn-ok")).toBeVisible();
    await expect(page.locator("#dialog")).toBeVisible();

    const vp = { available_width: 1920, available_height: 1080 };
    const fit = await firstOpenFitWithViewport(page, vp);

    expect(fit.contentW).toBeGreaterThan(0);
    expect(fit.contentH).toBeGreaterThan(0);
    // Slack: sized ≥ ceil(content × 1.25) when unclamped (large viewport).
    expect(fit.clamped).toBe(false);
    expect(fit.w).toBe(Math.ceil(fit.contentW * DIALOG_SLACK));
    expect(fit.h).toBe(Math.ceil(fit.contentH * DIALOG_SLACK));
    expect(fit.w).toBeLessThanOrEqual(Math.floor(vp.available_width * VIEWPORT_CLAMP));
    expect(fit.h).toBeLessThanOrEqual(Math.floor(vp.available_height * VIEWPORT_CLAMP));
    expect(fit.log.some((m) => m.startsWith(`resize:${fit.w}x${fit.h}`))).toBe(
      true,
    );

    // Hidden-until-first-resize: present only after resize IPC (L2 stand-in).
    const presented = await page.evaluate(
      () =>
        (window as unknown as { __presentedAfterResize?: boolean })
          .__presentedAfterResize === true,
    );
    expect(presented).toBe(true);

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

test("input dialog first-open resize clamps to viewport × 0.92 (golden L2)", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-viewport-input-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"input","title":"Name","message":"Enter a representative value for viewport clamp sizing.","buttons":"ok_cancel"}';

  let child: ChildProcessWithoutNullStreams | null = null;
  try {
    child = spawn(WYVERN_BIN, [json, "--viewer", "none", "--ui-root", UI_ROOT], {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        WYVERN_DIALOG_URL_FILE: urlFile,
        WYVERN_LOG: "off",
      },
    });
    const exitPromise = waitForExit(child);
    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoBlockingDialog(page, dialogUrl);
    await expect(page.getByTestId("btn-ok")).toBeVisible();
    await expect(page.locator("#dialog")).toBeVisible();

    // Tiny viewport forces clamp on first-open fit.
    const vp = { available_width: 400, available_height: 300 };
    const fit = await firstOpenFitWithViewport(page, vp);
    const maxW = Math.floor(vp.available_width * VIEWPORT_CLAMP);
    const maxH = Math.floor(vp.available_height * VIEWPORT_CLAMP);

    expect(fit.clamped).toBe(true);
    expect(fit.w).toBe(maxW);
    expect(fit.h).toBeLessThanOrEqual(maxH);
    expect(fit.w).toBeLessThan(Math.ceil(fit.contentW * DIALOG_SLACK));
    expect(fit.log.some((m) => m.startsWith("resize:"))).toBe(true);

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
