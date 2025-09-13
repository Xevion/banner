import { defineConfig } from "vite";
import viteReact from "@vitejs/plugin-react";
import tanstackRouter from "@tanstack/router-plugin/vite";
import { resolve } from "node:path";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [tanstackRouter({ autoCodeSplitting: true }), viteReact()],
  test: {
    globals: true,
    environment: "jsdom",
  },
  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },
  server: {
    port: 3000,
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
      },
    },
  },
  build: {
    outDir: "dist",
    sourcemap: true,
  },
});
