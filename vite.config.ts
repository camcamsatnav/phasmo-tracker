import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  base: "./",
  plugins: [react()],
  server: {
    port: 5173,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
    setupFiles: "frontend/src/test/setup.ts",
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
