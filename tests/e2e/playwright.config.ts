import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  timeout: 60_000,
  // Question (and other) L2 specs each spawn a wyvern subprocess; serialize
  // workers and allow one retry for transient listen/connect races on CI.
  workers: 1,
  retries: 1,
  use: {
    headless: true,
    trace: "on-first-retry",
  },
  reporter: [["list"]],
});
