import type { Page } from "@playwright/test";

/** Retry page.goto for transient connection races before axum accepts. */
export async function gotoDialog(
  page: Page,
  url: string,
  attempts = 15,
): Promise<void> {
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
