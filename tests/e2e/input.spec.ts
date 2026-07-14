import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

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

test("input text ok via --viewer none", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-input-url-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"input","title":"Name","message":"Enter name","buttons":"ok_cancel"}';

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
    await page.goto(dialogUrl);
    await expect(page.getByTestId("input-field")).toBeVisible();
    await page.getByTestId("input-field").fill("Ada Lovelace");
    await page.getByTestId("btn-ok").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    expect(stdout.trim()).toBe('{"button":"ok","input":"Ada Lovelace"}');
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

test("input file mode with WYVERN_MOCK_PICKER_PATH", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const fixture = path.join(
    os.tmpdir(),
    `wyvern-e2e-fixture-${process.pid}-${Date.now()}.txt`,
  );
  fs.writeFileSync(fixture, "fixture\n");

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-input-file-url-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"input","title":"File","message":"Pick","mode":"file","buttons":"ok_cancel"}';

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
          WYVERN_MOCK_PICKER_PATH: fixture,
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
    await page.goto(dialogUrl);
    await expect(page.getByTestId("btn-ok")).toBeVisible();
    await page.getByTestId("btn-ok").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const parsed = JSON.parse(stdout.trim());
    expect(parsed.button).toBe("ok");
    expect(parsed.input).toBe(fixture);
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
      fs.unlinkSync(fixture);
    } catch {
      // ignore
    }
  }
});
