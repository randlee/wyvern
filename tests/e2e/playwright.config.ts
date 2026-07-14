import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  timeout: 60_000,
  retries: 0,
  use: {
    headless: true,
    trace: "on-first-retry",
  },
  reporter: [["list"]],
});
