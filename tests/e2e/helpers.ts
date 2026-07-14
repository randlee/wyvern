import type { Page } from "@playwright/test";

const DIALOG_READY_BUDGET_MS = 15_000;
const RETRY_DELAY_MS = 100;

/** Retry page.goto for transient connection races before axum accepts (FTQ-005). */
export async function gotoDialog(
  page: Page,
  url: string,
  budgetMs = DIALOG_READY_BUDGET_MS,
): Promise<void> {
  const deadline = Date.now() + budgetMs;
  let lastError: unknown;
  while (Date.now() < deadline) {
    try {
      await page.goto(url, { waitUntil: "domcontentloaded" });
      // Align with L1 readiness: URL alone is not enough — wait for /api/dialog.
      await waitForDialogApi(page, url, deadline - Date.now());
      return;
    } catch (err) {
      lastError = err;
      await new Promise((r) => setTimeout(r, RETRY_DELAY_MS));
    }
  }
  throw lastError;
}

/** Poll GET /api/dialog until HTTP 200 or budget exhausted. */
async function waitForDialogApi(
  page: Page,
  dialogUrl: string,
  remainingMs: number,
): Promise<void> {
  const apiUrl = new URL("/api/dialog", dialogUrl).toString();
  const deadline = Date.now() + Math.max(remainingMs, 0);
  let lastStatus = 0;
  while (Date.now() < deadline) {
    try {
      const resp = await page.request.get(apiUrl);
      lastStatus = resp.status();
      if (resp.ok()) {
        return;
      }
    } catch {
      // keep polling
    }
    await new Promise((r) => setTimeout(r, RETRY_DELAY_MS));
  }
  throw new Error(
    `timed out waiting for ${apiUrl} (last status ${lastStatus || "none"})`,
  );
}
