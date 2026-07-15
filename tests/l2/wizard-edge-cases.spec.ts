import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN || path.join(REPO_ROOT, "target/debug/wyvern");
const UI_ROOT = path.join(REPO_ROOT, "examples/wizards/single-page");
const WIZARD_JSON = path.join(UI_ROOT, "wizard.json");

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

async function spawnSinglePage(page: import("@playwright/test").Page) {
  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-wizard-edge-${process.pid}-${Date.now()}.txt`,
  );
  const wizardJson = fs.readFileSync(WIZARD_JSON, "utf8");

  let stdout = "";
  let stderr = "";
  const child = spawn(
    WYVERN_BIN,
    [wizardJson, "--viewer", "none", "--ui-root", UI_ROOT],
    {
      cwd: REPO_ROOT,
      env: {
        ...process.env,
        WYVERN_DIALOG_URL_FILE: urlFile,
        WYVERN_LOG: "off",
      },
    },
  );
  child.stdout.on("data", (chunk: Buffer) => {
    stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk: Buffer) => {
    stderr += chunk.toString();
  });
  const exitPromise = waitForExit(child);

  const dialogUrl = await waitForUrlFile(urlFile);
  await gotoDialog(page, dialogUrl);
  await expect(page.getByTestId("single-page-root")).toBeVisible();

  return {
    child,
    urlFile,
    exitPromise,
    getStdout: () => stdout,
    getStderr: () => stderr,
  };
}

test("first-page back is hidden; terminal next labeled Finish (N=1)", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);

  const session = await spawnSinglePage(page);
  try {
    const back = page.getByTestId("wizard-back");
    await expect(back).toBeHidden();

    const next = page.getByTestId("wizard-next");
    await expect(next).toBeVisible();
    await expect(next).toHaveText("Finish");
    await expect(next).toHaveAttribute("data-wizard-action", "finish");

    await expect(page.locator("[data-wizard-terminal='true']")).toHaveCount(1);
  } finally {
    if (session.child.exitCode === null && !session.child.killed) {
      session.child.kill("SIGTERM");
    }
    try {
      fs.unlinkSync(session.urlFile);
    } catch {
      // ignore
    }
  }
});

test("N=1 finish end-to-end via shared wizard chrome", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);

  const session = await spawnSinglePage(page);
  try {
    await page.getByTestId("single-page-note").fill("hello-chrome");
    await page.getByTestId("wizard-next").click();

    const exitCode = await session.exitPromise;
    expect(exitCode, `stderr=${session.getStderr()}`).toBe(0);
    const result = JSON.parse(session.getStdout().trim());
    expect(result.button).toBe("finish");
    expect(result.data).toEqual({ note: "hello-chrome" });
    expect(result.stack).toHaveLength(1);
    expect(result.stack[0].page.id).toBe("only");
    expect(result.stack[0].data).toEqual({ note: "hello-chrome" });
  } finally {
    if (session.child.exitCode === null && !session.child.killed) {
      session.child.kill("SIGTERM");
    }
    try {
      fs.unlinkSync(session.urlFile);
    } catch {
      // ignore
    }
  }
});

test("empty page data submits {} without console errors", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);

  const consoleErrors: string[] = [];
  page.on("pageerror", (err) => {
    consoleErrors.push(String(err));
  });
  page.on("console", (msg) => {
    if (msg.type() === "error") {
      consoleErrors.push(msg.text());
    }
  });

  const session = await spawnSinglePage(page);
  try {
    // Leave the note empty — collector returns {}.
    await expect(page.getByTestId("single-page-note")).toHaveValue("");
    await page.getByTestId("wizard-next").click();

    const exitCode = await session.exitPromise;
    expect(exitCode, `stderr=${session.getStderr()}`).toBe(0);
    const result = JSON.parse(session.getStdout().trim());
    expect(result.button).toBe("finish");
    expect(result.data).toEqual({});
    expect(result.stack).toHaveLength(1);
    expect(result.stack[0].data).toEqual({});
    expect(consoleErrors, `console errors: ${consoleErrors.join("; ")}`).toEqual(
      [],
    );
  } finally {
    if (session.child.exitCode === null && !session.child.killed) {
      session.child.kill("SIGTERM");
    }
    try {
      fs.unlinkSync(session.urlFile);
    } catch {
      // ignore
    }
  }
});
