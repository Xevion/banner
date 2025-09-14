import { defineConfig } from "vite";
import viteReact from "@vitejs/plugin-react";
import tanstackRouter from "@tanstack/router-plugin/vite";
import { resolve } from "node:path";
import { readFileSync } from "fs";

// Extract version from Cargo.toml
function getVersion() {
  try {
    const cargoTomlPath = resolve(__dirname, "..", "Cargo.toml");
    const cargoTomlContent = readFileSync(cargoTomlPath, "utf8");
    const versionMatch = cargoTomlContent.match(/^version\s*=\s*"([^"]+)"/m);
    return versionMatch ? versionMatch[1] : "unknown";
  } catch (error) {
    console.warn("Could not read version from Cargo.toml:", error);
    return "unknown";
  }
}

const version = getVersion();

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
  define: {
    __APP_VERSION__: JSON.stringify(version),
  },
});
