import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  base: "/wizard/dist/",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: "src/canvas/main.ts",
      output: {
        entryFileNames: "canvas.js",
        assetFileNames: "canvas.[ext]",
      },
    },
  },
});
