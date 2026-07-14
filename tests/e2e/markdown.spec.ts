import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN ||
  path.join(REPO_ROOT, "target/debug/wyvern");

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

test("markdown JSON ok via --viewer none", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-md-url-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"markdown","title":"Notes","content":"# Hello\\n\\nBody text","buttons":"ok"}';

  let stdout = "";
  let stderr = "";
  let child: ChildProcessWithoutNullStreams | null = null;

  try {
    child = spawn(
      WYVERN_BIN,
      [json, "--viewer", "none", "--ui-root", path.join(REPO_ROOT, "ui")],
      {
        cwd: REPO_ROOT,
        env: {
          ...process.env,
          WYVERN_DIALOG_URL_FILE: urlFile,
          WYVERN_LOG: "off",
        },
      },
    );
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    const exitPromise = waitForExit(child);

    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoDialog(page, dialogUrl);
    await expect(page.getByTestId("markdown-content")).toBeVisible();
    await expect(page.locator("#markdown-body h1")).toHaveText("Hello");
    await page.getByTestId("btn-ok").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    expect(stdout.trim()).toBe('{"button":"ok"}');
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

test("markdown file path via --viewer none", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const mdFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-doc-${process.pid}-${Date.now()}.md`,
  );
  fs.writeFileSync(mdFile, "# From File\n\nLoaded via path.\n");

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-md-file-url-${process.pid}-${Date.now()}.txt`,
  );

  let stdout = "";
  let stderr = "";
  let child: ChildProcessWithoutNullStreams | null = null;

  try {
    child = spawn(
      WYVERN_BIN,
      [mdFile, "--viewer", "none", "--ui-root", path.join(REPO_ROOT, "ui")],
      {
        cwd: REPO_ROOT,
        env: {
          ...process.env,
          WYVERN_DIALOG_URL_FILE: urlFile,
          WYVERN_LOG: "off",
        },
      },
    );
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    const exitPromise = waitForExit(child);

    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoDialog(page, dialogUrl);
    await expect(page.getByTestId("markdown-content")).toBeVisible();
    await expect(page.locator("#markdown-body h1")).toHaveText("From File");
    await page.getByTestId("btn-ok").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    expect(stdout.trim()).toBe('{"button":"ok"}');
  } finally {
    if (child && child.exitCode === null && !child.killed) {
      child.kill("SIGTERM");
    }
    try {
      fs.unlinkSync(urlFile);
    } catch {
      // ignore
    }
    try {
      fs.unlinkSync(mdFile);
    } catch {
      // ignore
    }
  }
});
