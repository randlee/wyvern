import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  // Hang detector only — every spec must actively drive the dialog (~1s). Never the pass path.
  timeout: 90_000,
  workers: 1,
  retries: 1,
  use: {
    headless: true,
    trace: "on-first-retry",
  },
  reporter: [["list"]],
});
