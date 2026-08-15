import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN || path.join(REPO_ROOT, "target/debug/wyvern");
const UI_ROOT = path.join(REPO_ROOT, "examples/wizards/turbo-flow");
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

type WyvernChild = {
  child: ChildProcessWithoutNullStreams;
  exitPromise: Promise<number>;
  urlFile: string;
  stdout: string;
  stderr: string;
};

function spawnTurboFlowWizard(suffix: string, theme: "dark" | "light" = "dark"): WyvernChild {
  const wizardPath =
    theme === "light" ? path.join(UI_ROOT, "wizard.light.json") : WIZARD_JSON;
  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-turbo-flow-${suffix}-${process.pid}-${Date.now()}.txt`,
  );
  const wizardJson = fs.readFileSync(wizardPath, "utf8");
  const capture = { stdout: "", stderr: "" };
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
    capture.stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk: Buffer) => {
    capture.stderr += chunk.toString();
  });
  return {
    child,
    exitPromise: waitForExit(child),
    urlFile,
    get stdout() {
      return capture.stdout;
    },
    get stderr() {
      return capture.stderr;
    },
  };
}

async function cleanupWizard(run: WyvernChild) {
  if (run.child.exitCode === null && !run.child.killed) {
    run.child.kill("SIGTERM");
  }
  try {
    fs.unlinkSync(run.urlFile);
  } catch {
    // ignore
  }
}

async function waitForCanvas(page: import("@playwright/test").Page) {
  await expect(page.getByTestId("turbo-flow-workspace")).toBeVisible({ timeout: 15_000 });
  await expect(page.getByTestId("turbo-flow-add-node")).toBeVisible();
  await expect(page.getByTestId("turbo-flow-configure")).toBeVisible();
  await expect(page.getByTestId("turbo-flow-review")).toBeVisible();
  await expect(page.locator(".svelte-flow")).toBeVisible();
  await expect(page.locator(".svelte-flow__controls")).toBeVisible();
  await expect(page.getByTestId("turbo-node-node-1")).toBeVisible();
  await expect(page.getByTestId("turbo-node-node-2")).toBeVisible();
}

async function selectNodeOne(page: import("@playwright/test").Page) {
  await page.getByTestId("turbo-node-node-1").click();
  await expect(page.getByTestId("turbo-flow-configure")).toBeEnabled();
}

test.beforeEach(() => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);
  test.skip(
    !fs.existsSync(path.join(UI_ROOT, "dist/canvas.js")),
    "missing built turbo-flow canvas bundle — run npm run build in examples/wizards/turbo-flow",
  );
});

test("turbo-flow canvas workspace layout and controls render", async ({ page }) => {
  const run = spawnTurboFlowWizard("layout");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await waitForCanvas(page);

    const workspace = page.locator("#dialog.dialog--workspace");
    await expect(workspace).toBeVisible();
    const box = await workspace.boundingBox();
    expect(box?.width ?? 0).toBeGreaterThan(320);
    expect(box?.height ?? 0).toBeGreaterThan(240);

    await expect(page.getByTestId("turbo-flow-configure")).toBeDisabled();
    await selectNodeOne(page);

    await page.getByTestId("turbo-flow-add-node").click();
    await expect(page.getByTestId("turbo-node-node-3")).toBeVisible();
  } finally {
    await cleanupWizard(run);
  }
});

test("turbo-flow quick configure path finishes with graph and node details", async ({
  page,
}) => {
  const run = spawnTurboFlowWizard("quick");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await waitForCanvas(page);
    await selectNodeOne(page);

    await page.getByTestId("turbo-flow-configure").click();
    await expect(page.getByTestId("node-detail-root")).toBeVisible({ timeout: 15_000 });
    const frame = page.locator("#dialog.dialog--frame");
    await expect(frame).toBeVisible();
    const frameBox = await frame.boundingBox();
    expect(frameBox?.width ?? 0).toBeGreaterThan(280);

    await page.getByTestId("node-detail-name").fill("Researcher");
    await page.getByTestId("node-detail-role").fill("analysis");
    await page.getByTestId("node-detail-description").fill("Collects sources");
    await page.getByTestId("node-detail-back-graph").click();

    await expect(page.getByTestId("turbo-flow-workspace")).toBeVisible({ timeout: 15_000 });
    await waitForCanvas(page);
    await expect(page.getByTestId("turbo-node-node-1")).toContainText("Researcher");

    await page.getByTestId("turbo-flow-review").click();
    await expect(page.getByTestId("review-heading")).toBeVisible();
    await expect(page.getByTestId("review-node-summary")).toContainText("Researcher");
    await expect(page.getByTestId("review-node-summary")).toContainText("analysis");
    await expect(page.getByTestId("review-finish")).toBeInViewport();

    await page.getByTestId("review-finish").click();
    await expect(page.getByTestId("wizard-error")).toBeHidden({ timeout: 5_000 });

    const exitCode = await run.exitPromise;
    expect(exitCode, `stderr=${run.stderr}`).toBe(0);
    const stdout = run.stdout.trim();
    if (stdout) {
      const result = JSON.parse(stdout);
      expect(result.button).toBe("finish");
    }
  } finally {
    await cleanupWizard(run);
  }
});

test("turbo-flow extras path preserves optional fields", async ({ page }) => {
  const run = spawnTurboFlowWizard("extras");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await waitForCanvas(page);
    await selectNodeOne(page);

    await page.getByTestId("turbo-flow-configure").click();
    await page.getByTestId("node-detail-name").fill("Planner");
    await page.getByTestId("node-detail-role").fill("orchestrator");
    await page.getByTestId("node-detail-description").fill("Coordinates work");
    await page.getByTestId("node-detail-next").click();

    await expect(page.getByTestId("node-extras-root")).toBeVisible({ timeout: 15_000 });
    await page.getByTestId("node-extras-prompt").fill("Be concise");
    await page.getByTestId("node-extras-tool").fill("web-search");
    await page.getByTestId("node-extras-back").click();

    await expect(page.getByTestId("node-detail-root")).toBeVisible();
    await page.getByTestId("node-detail-back-graph").click();
    await waitForCanvas(page);

    await page.getByTestId("turbo-flow-review").click();
    await expect(page.getByTestId("review-node-summary")).toContainText("Be concise");
    await expect(page.getByTestId("review-node-summary")).toContainText("web-search");
    await page.getByTestId("review-finish").click();
    await expect(page.getByTestId("wizard-error")).toBeHidden({ timeout: 5_000 });

    const exitCode = await run.exitPromise;
    expect(exitCode, `stderr=${run.stderr}`).toBe(0);
    const stdout = run.stdout.trim();
    if (stdout) {
      const result = JSON.parse(stdout);
      expect(result.button).toBe("finish");
    }
  } finally {
    await cleanupWizard(run);
  }
});

test("turbo-flow double-click opens configure for node", async ({ page }) => {
  const run = spawnTurboFlowWizard("dblclick");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await waitForCanvas(page);

    await page.getByTestId("turbo-node-node-2").dblclick();
    await expect(page.getByTestId("node-detail-root")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("node-detail-heading")).toContainText("node-2");
  } finally {
    await cleanupWizard(run);
  }
});

test("turbo-flow configure restores per-node values", async ({ page }) => {
  const run = spawnTurboFlowWizard("per-node");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await waitForCanvas(page);

    await page.getByTestId("turbo-node-node-1").click();
    await page.getByTestId("turbo-flow-configure").click();
    await page.getByTestId("node-detail-name").fill("Alpha");
    await page.getByTestId("node-detail-role").fill("role-a");
    await page.getByTestId("node-detail-description").fill("desc-a");
    await page.getByTestId("node-detail-back-graph").click();
    await waitForCanvas(page);

    await page.getByTestId("turbo-node-node-2").click();
    await page.getByTestId("turbo-flow-configure").click();
    await expect(page.getByTestId("node-detail-name")).toHaveValue("");
    await expect(page.getByTestId("node-detail-role")).toHaveValue("");
    await expect(page.getByTestId("node-detail-description")).toHaveValue("");
    await page.getByTestId("node-detail-name").fill("Beta");
    await page.getByTestId("node-detail-description").fill("desc-b");
    await page.getByTestId("node-detail-back-graph").click();
    await waitForCanvas(page);

    await page.getByTestId("turbo-node-node-1").click();
    await page.getByTestId("turbo-flow-configure").click();
    await expect(page.getByTestId("node-detail-name")).toHaveValue("Alpha");
    await expect(page.getByTestId("node-detail-role")).toHaveValue("role-a");
    await expect(page.getByTestId("node-detail-description")).toHaveValue("desc-a");
  } finally {
    await cleanupWizard(run);
  }
});

test("turbo-flow dark theme keeps readable detail text", async ({ page }) => {
  const run = spawnTurboFlowWizard("theme-dark", "dark");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await expect(page.locator("html")).toHaveAttribute("data-wyvern-theme", "dark");
    await waitForCanvas(page);
    await selectNodeOne(page);
    await page.getByTestId("turbo-flow-configure").click();
    await expect(page.getByTestId("node-detail-root")).toBeVisible({ timeout: 15_000 });

    const headingColor = await page.getByTestId("node-detail-heading").evaluate((el) => {
      return getComputedStyle(el).color;
    });
    const bgColor = await page.locator("#dialog.dialog--frame").evaluate((el) => {
      return getComputedStyle(el).backgroundColor;
    });
    expect(headingColor).not.toBe(bgColor);
    expect(headingColor).toMatch(/rgb/);
  } finally {
    await cleanupWizard(run);
  }
});

test("turbo-flow light theme keeps readable detail text", async ({ page }) => {
  const run = spawnTurboFlowWizard("theme-light", "light");
  try {
    const dialogUrl = await waitForUrlFile(run.urlFile);
    await gotoDialog(page, dialogUrl);
    await expect(page.locator("html")).toHaveAttribute("data-wyvern-theme", "light");
    await waitForCanvas(page);
    await selectNodeOne(page);
    await page.getByTestId("turbo-flow-configure").click();
    await expect(page.getByTestId("node-detail-root")).toBeVisible({ timeout: 15_000 });

    const headingColor = await page.getByTestId("node-detail-heading").evaluate((el) => {
      return getComputedStyle(el).color;
    });
    const bgColor = await page.locator("#dialog.dialog--frame").evaluate((el) => {
      return getComputedStyle(el).backgroundColor;
    });
    expect(headingColor).not.toBe(bgColor);
    expect(headingColor).toMatch(/rgb/);
  } finally {
    await cleanupWizard(run);
  }
});
