import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  timeout: 90_000,
  workers: 1,
  retries: 1,
  use: {
    headless: true,
    trace: "on-first-retry",
  },
  reporter: [["list"]],
});
