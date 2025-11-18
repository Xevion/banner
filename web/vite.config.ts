import { defineConfig } from "vite";
import viteReact from "@vitejs/plugin-react";
import tanstackRouter from "@tanstack/router-plugin/vite";
import { resolve } from "node:path";
import { readFileSync, existsSync } from "node:fs";

// Extract version from Cargo.toml
function getVersion() {
  const filename = "Cargo.toml";
  const paths = [resolve(__dirname, filename), resolve(__dirname, "..", filename)];

  for (const path of paths) {
    try {
      // Check if file exists before reading
      if (!existsSync(path)) {
        console.log("Skipping ", path, " because it does not exist");
        continue;
      }

      const cargoTomlContent = readFileSync(path, "utf8");
      const versionMatch = cargoTomlContent.match(/^version\s*=\s*"([^"]+)"/m);
      if (versionMatch) {
        console.log("Found version in ", path, ": ", versionMatch[1]);
        return versionMatch[1];
      }
    } catch (error) {
      console.warn("Failed to read Cargo.toml at path: ", path, error);
      // Continue to next path
    }
  }

  console.warn("Could not read version from Cargo.toml in any location");
  return "unknown";
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
