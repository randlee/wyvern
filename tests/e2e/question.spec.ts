import { test, expect, Page } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN ||
  path.join(REPO_ROOT, "target/debug/wyvern");

function waitForDialogUrl(
  filePath: string,
  getStderr: () => string,
  timeoutMs = 15_000,
): Promise<string> {
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
      // Harness fallback: scrape WYVERN_DIALOG_URL= from stderr (http-dialog-contract).
      const match = getStderr().match(/WYVERN_DIALOG_URL=(http:\/\/\S+)/);
      if (match?.[1]) {
        resolve(match[1].trim());
        return;
      }
      if (Date.now() - start > timeoutMs) {
        reject(new Error(`timed out waiting for dialog URL: ${filePath}`));
        return;
      }
      setTimeout(tick, 50);
    };
    tick();
  });
}

/** Retry page.goto for transient connection races before axum accepts. */
async function gotoDialog(page: Page, url: string, attempts = 15): Promise<void> {
  let lastError: unknown;
  for (let i = 0; i < attempts; i++) {
    try {
      await page.goto(url, { waitUntil: "domcontentloaded" });
      return;
    } catch (err) {
      lastError = err;
      await new Promise((r) => setTimeout(r, 100));
    }
  }
  throw lastError;
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

test("question single-select submit via --viewer none", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-q-single-url-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"question","questions":[{"question":"Output format?","header":"Format","options":[{"label":"JSON","description":"Structured","preview":"<pre>{\\"ok\\":true}</pre>"},{"label":"Plain","description":"Text only"}],"multiSelect":false}]}';

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

    const dialogUrl = await waitForDialogUrl(urlFile, () => stderr);
    await gotoDialog(page, dialogUrl);
    await expect(page.getByTestId("question-cards")).toBeVisible();
    await expect(page.getByTestId("option-q0-o0")).toBeVisible();
    await expect(page.getByTestId("preview-q0-o0")).toContainText("ok");
    await page.getByTestId("option-q0-o0").click();
    await page.getByTestId("btn-submit").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const wire = JSON.parse(stdout.trim());
    expect(wire.button).toBeUndefined();
    expect(wire.answers["Output format?"]).toBe("JSON");
    expect(wire.response).toBe("");
    expect(wire.questions[0].question).toBe("Output format?");
    expect(wire.questions[0].options[0].preview).toBe('<pre>{"ok":true}</pre>');
    expect(wire.questions[0].options[0].preview_html).toBeUndefined();
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

test("question multi-select submit via --viewer none", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-q-multi-url-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"question","questions":[{"question":"Pick tools","header":"Tools","options":[{"label":"JSON","description":"A"},{"label":"Plain","description":"B"}],"multiSelect":true}]}';

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

    const dialogUrl = await waitForDialogUrl(urlFile, () => stderr);
    await gotoDialog(page, dialogUrl);
    await expect(page.getByTestId("question-cards")).toBeVisible();
    await page.getByTestId("option-q0-o0").click();
    await page.getByTestId("option-q0-o1").click();
    await page.getByTestId("btn-submit").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const wire = JSON.parse(stdout.trim());
    expect(wire.button).toBeUndefined();
    expect(wire.answers["Pick tools"]).toBe("JSON, Plain");
    expect(wire.response).toBe("");
    expect(wire.questions[0].multiSelect).toBe(true);
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

test("question dismiss returns REQ-0068 shape", async ({ page }) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-e2e-q-dismiss-url-${process.pid}-${Date.now()}.txt`,
  );
  const json =
    '{"type":"question","questions":[{"question":"Output format?","header":"Format","options":[{"label":"JSON","description":"Structured"},{"label":"Plain","description":"Text only"}],"multiSelect":false}]}';

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

    const dialogUrl = await waitForDialogUrl(urlFile, () => stderr);
    await gotoDialog(page, dialogUrl);
    await expect(page.getByTestId("question-cards")).toBeVisible();

    // Extended dismiss shape (REQ-0068) — same body beforeunload beacon uses.
    await page.evaluate(async () => {
      const api = (
        window as unknown as {
          WyvernApi: {
            fetchDialog: () => Promise<{
              questions: Array<{
                question: string;
                header: string;
                options: Array<{
                  label: string;
                  description: string;
                  preview?: string;
                }>;
                multiSelect: boolean;
              }>;
            }>;
            postResult: (body: unknown) => Promise<unknown>;
          };
        }
      ).WyvernApi;
      const dialog = await api.fetchDialog();
      const questions = dialog.questions.map((card) => ({
        question: card.question,
        header: card.header,
        options: card.options.map((opt) => {
          const o: {
            label: string;
            description: string;
            preview?: string;
          } = {
            label: opt.label,
            description: opt.description,
          };
          if (opt.preview != null) o.preview = opt.preview;
          return o;
        }),
        multiSelect: card.multiSelect,
      }));
      await api.postResult({
        button: "dismissed",
        questions,
        answers: {},
        response: "",
      });
    });

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const wire = JSON.parse(stdout.trim());
    expect(wire.button).toBe("dismissed");
    expect(wire.answers).toEqual({});
    expect(wire.response).toBe("");
    expect(Array.isArray(wire.questions)).toBe(true);
    expect(wire.questions[0].question).toBe("Output format?");
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
