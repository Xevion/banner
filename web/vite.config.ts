import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

function getVersion() {
  const filename = "Cargo.toml";
  const paths = [resolve(__dirname, filename), resolve(__dirname, "..", filename)];

  for (const path of paths) {
    try {
      if (!existsSync(path)) continue;
      const content = readFileSync(path, "utf8");
      const match = /^version\s*=\s*"([^"]+)"/m.exec(content);
      if (match) return match[1];
    } catch {
      // Continue to next path
    }
  }

  return "unknown";
}

const version = getVersion();

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  test: {
    globals: true,
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
  server: {
    port: 3000,
    watch: {
      ignored: ["**/.svelte-kit/generated/**"],
    },
    proxy: {
      "/api": {
        target: "http://localhost:8080",
        changeOrigin: true,
        secure: false,
        ws: true,
      },
    },
  },
  build: {
    sourcemap: true,
  },
  define: {
    __APP_VERSION__: JSON.stringify(version),
  },
});
