import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN || path.join(REPO_ROOT, "target/debug/wyvern");
const UI_ROOT = path.join(REPO_ROOT, "examples/wizards/layout-picker");
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

async function waitForCards(page: import("@playwright/test").Page) {
  await expect(page.getByTestId("layout-picker")).toBeVisible();
  await expect(page.getByTestId("layout-card-solo")).toBeVisible();
  await expect(page.getByTestId("layout-card-pair")).toBeVisible();
  await expect(page.getByTestId("layout-card-trio")).toBeVisible();
}

test("layout-picker pair flow finishes with full stack via --viewer none", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-layout-picker-url-${process.pid}-${Date.now()}.txt`,
  );
  const wizardJson = fs.readFileSync(WIZARD_JSON, "utf8");

  let stdout = "";
  let stderr = "";
  let child: ChildProcessWithoutNullStreams | null = null;

  try {
    child = spawn(
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
    await waitForCards(page);

    await expect(page.getByTestId("layout-card-pair")).toContainText("Pair");
    await expect(page.getByTestId("layout-card-pair")).toContainText("2 agents");
    await page.getByTestId("layout-card-pair").click();

    await expect(page.getByTestId("agent-heading")).toContainText("Agent 1");
    await page.getByTestId("agent-name").fill("Alpha");
    await page.getByTestId("agent-description").fill("First agent");
    await page.getByTestId("agent-next").click();

    await expect(page.getByTestId("agent-heading")).toContainText("Agent 2");
    await page.getByTestId("agent-name").fill("Beta");
    await page.getByTestId("agent-description").fill("Second agent");
    await page.getByTestId("agent-next").click();

    await expect(page.getByTestId("finish-heading")).toBeVisible();
    await expect(page.getByTestId("finish-summary")).toContainText("Pair");
    await expect(page.getByTestId("finish-summary")).toContainText("Alpha");
    await expect(page.getByTestId("finish-summary")).toContainText("Beta");
    await page.getByTestId("finish-submit").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const result = JSON.parse(stdout.trim());
    expect(result.button).toBe("finish");
    expect(result.stack).toHaveLength(4);
    expect(result.stack[0].data.layout_id).toBe("pair");
    expect(result.stack[0].data.agent_count).toBe(2);
    expect(result.stack[1].data.name).toBe("Alpha");
    expect(result.stack[2].data.name).toBe("Beta");
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

test("layout-picker back-navigation restores then branches to solo", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-layout-picker-branch-${process.pid}-${Date.now()}.txt`,
  );
  const wizardJson = fs.readFileSync(WIZARD_JSON, "utf8");

  let stdout = "";
  let stderr = "";
  let child: ChildProcessWithoutNullStreams | null = null;

  try {
    child = spawn(
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
    await waitForCards(page);

    await page.getByTestId("layout-card-pair").click();
    await expect(page.getByTestId("agent-heading")).toContainText("Agent 1");
    await page.getByTestId("agent-name").fill("Temp");
    await page.getByTestId("agent-description").fill("will branch away");
    await page.getByTestId("agent-back").click();

    await waitForCards(page);
    await page.getByTestId("layout-card-solo").click();
    await expect(page.getByTestId("agent-heading")).toContainText("Agent 1 of 1");
    await page.getByTestId("agent-name").fill("Soloist");
    await page.getByTestId("agent-description").fill("after branch");
    await page.getByTestId("agent-next").click();

    await expect(page.getByTestId("finish-heading")).toBeVisible();
    await page.getByTestId("finish-submit").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const result = JSON.parse(stdout.trim());
    expect(result.button).toBe("finish");
    expect(result.stack).toHaveLength(3);
    expect(result.stack[0].data.layout_id).toBe("solo");
    expect(result.stack[1].data.name).toBe("Soloist");
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
