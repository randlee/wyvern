import { test, expect } from "@playwright/test";
import { spawn, ChildProcessWithoutNullStreams } from "node:child_process";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { gotoDialog } from "./helpers";

// Contract test for the atm-core Send-To `PickerInput`/`PickerOutput`
// round trip (see examples/wizards/atm-pick-member/README.md and
// https://github.com/randlee/atm-core/blob/develop/docs/plans/phase-aq/fixtures/wyvern-pick-member-contract.md).

const REPO_ROOT = path.resolve(__dirname, "../..");
const WYVERN_BIN =
  process.env.WYVERN_BIN || path.join(REPO_ROOT, "target/debug/wyvern");
const UI_ROOT = path.join(REPO_ROOT, "examples/wizards/atm-pick-member");
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

function spawnWizard(urlFile: string): ChildProcessWithoutNullStreams {
  const wizardJson = fs.readFileSync(WIZARD_JSON, "utf8");
  return spawn(WYVERN_BIN, [wizardJson, "--viewer", "none", "--ui-root", UI_ROOT], {
    cwd: REPO_ROOT,
    env: {
      ...process.env,
      WYVERN_DIALOG_URL_FILE: urlFile,
      WYVERN_LOG: "off",
    },
  });
}

test("atm-pick-member disables dead/idle rows and returns only the active recipient", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);
  test.skip(!fs.existsSync(WIZARD_JSON), `missing fixture at ${WIZARD_JSON}`);

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-atm-pick-member-${process.pid}-${Date.now()}.txt`,
  );

  let stdout = "";
  let stderr = "";
  let child: ChildProcessWithoutNullStreams | null = null;

  try {
    child = spawnWizard(urlFile);
    child.stdout.on("data", (chunk: Buffer) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk: Buffer) => {
      stderr += chunk.toString();
    });
    const exitPromise = waitForExit(child);

    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoDialog(page, dialogUrl);
    await expect(page.getByTestId("atm-pick-member-heading")).toBeVisible();

    // Active member: selectable.
    const activeCheckbox = page.getByTestId("atm-picker-checkbox-cipher@atm-dev");
    await expect(activeCheckbox).toBeEnabled();

    // Idle and dead members: rendered, but genuinely non-selectable (R4).
    const idleCheckbox = page.getByTestId("atm-picker-checkbox-fenix@atm-dev");
    const deadCheckbox = page.getByTestId("atm-picker-checkbox-offline@atm-dev");
    await expect(idleCheckbox).toBeVisible();
    await expect(idleCheckbox).toBeDisabled();
    await expect(deadCheckbox).toBeVisible();
    await expect(deadCheckbox).toBeDisabled();
    await expect(page.getByTestId("atm-picker-status-fenix@atm-dev")).toContainText("idle");
    await expect(page.getByTestId("atm-picker-status-offline@atm-dev")).toContainText("dead");

    await activeCheckbox.check();
    await page.getByTestId("atm-picker-note").fill("see the attached plan");
    await page.getByTestId("wizard-next").click();

    const exitCode = await exitPromise;
    expect(exitCode, `stderr=${stderr}`).toBe(0);
    const result = JSON.parse(stdout.trim());
    // The wizard envelope, not a bare PickerOutput -- see this example's
    // README.md "Real Wyvern invocation" note. `.data` is the PickerOutput.
    expect(result.button).toBe("finish");
    expect(result.data).toEqual({
      schema_version: 1,
      recipients: ["cipher@atm-dev"],
      note: "see the attached plan",
    });
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

test("atm-pick-member rejects an unrecognized PickerInput schema_version", async ({
  page,
}) => {
  test.skip(!fs.existsSync(WYVERN_BIN), `missing wyvern binary at ${WYVERN_BIN}`);

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "wyvern-atm-pick-member-schema-"));
  const uiRoot = path.join(tempDir, "atm-pick-member");
  fs.mkdirSync(path.join(uiRoot, "pages"), { recursive: true });
  fs.copyFileSync(
    path.join(UI_ROOT, "pages", "pick-member.html"),
    path.join(uiRoot, "pages", "pick-member.html"),
  );
  const wizardJson = JSON.stringify({
    type: "wizard",
    page: { id: "pick-member", title: "ATM Send-To", html: "pages/pick-member.html" },
    config: { schema_version: 99, teams: [] },
  });

  const urlFile = path.join(
    os.tmpdir(),
    `wyvern-l2-atm-pick-member-schema-${process.pid}-${Date.now()}.txt`,
  );
  let child: ChildProcessWithoutNullStreams | null = null;

  try {
    child = spawn(WYVERN_BIN, [wizardJson, "--viewer", "none", "--ui-root", uiRoot], {
      cwd: REPO_ROOT,
      env: { ...process.env, WYVERN_DIALOG_URL_FILE: urlFile, WYVERN_LOG: "off" },
    });
    const exitPromise = waitForExit(child);
    const dialogUrl = await waitForUrlFile(urlFile);
    await gotoDialog(page, dialogUrl);

    await expect(page.getByTestId("wizard-error")).toBeVisible();
    await expect(page.getByTestId("wizard-error")).toContainText("schema_version");
    await expect(page.getByTestId("atm-picker-teams")).toBeEmpty();

    child.kill("SIGTERM");
    await exitPromise;
  } finally {
    if (child && child.exitCode === null && !child.killed) {
      child.kill("SIGTERM");
    }
    try {
      fs.unlinkSync(urlFile);
    } catch {
      // ignore
    }
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
