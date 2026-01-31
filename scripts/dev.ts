/**
 * Dev server orchestrator.
 *
 * Usage: bun scripts/dev.ts [flags] [-- passthrough-args]
 *
 * Flags:
 *   -f, --frontend-only   Frontend only (Vite dev server)
 *   -b, --backend-only    Backend only (bacon watch)
 *   -W, --no-watch        Build once + run (no watch)
 *   -n, --no-build        Run last compiled binary (no rebuild)
 *   -r, --release         Use release profile
 *   -e, --embed           Embed assets (implies -b)
 *   --tracing <fmt>       Tracing format (default: pretty)
 */

import { existsSync } from "fs";
import { parseFlags, c } from "./lib/fmt";
import { run, ProcessGroup } from "./lib/proc";

const { flags, passthrough } = parseFlags(
  process.argv.slice(2),
  {
    "frontend-only": "bool",
    "backend-only": "bool",
    "no-watch": "bool",
    "no-build": "bool",
    release: "bool",
    embed: "bool",
    tracing: "string",
  } as const,
  { f: "frontend-only", b: "backend-only", W: "no-watch", n: "no-build", r: "release", e: "embed" },
  {
    "frontend-only": false,
    "backend-only": false,
    "no-watch": false,
    "no-build": false,
    release: false,
    embed: false,
    tracing: "pretty",
  },
);

let frontendOnly = flags["frontend-only"];
let backendOnly = flags["backend-only"];
let noWatch = flags["no-watch"];
const noBuild = flags["no-build"];
const release = flags.release;
const embed = flags.embed;
const tracing = flags.tracing as string;

// -e implies -b
if (embed) backendOnly = true;
// -n implies -W
if (noBuild) noWatch = true;

if (frontendOnly && backendOnly) {
  console.error("Cannot use -f and -b together (or -e implies -b)");
  process.exit(1);
}

const runFrontend = !backendOnly;
const runBackend = !frontendOnly;
const profile = release ? "release" : "dev";
const profileDir = release ? "release" : "debug";
const group = new ProcessGroup();

// Build frontend first when embedding assets
if (embed && !noBuild) {
  console.log(c("1;36", "→ Building frontend (for embedding)..."));
  run(["bun", "run", "--cwd", "web", "build"]);
}

// Frontend: Vite dev server
if (runFrontend) {
  group.spawn(["bun", "run", "--cwd", "web", "dev"]);
}

// Backend
if (runBackend) {
  const backendArgs = ["--tracing", tracing, ...passthrough];
  const bin = `target/${profileDir}/banner`;

  if (noWatch) {
    if (!noBuild) {
      console.log(c("1;36", `→ Building backend (${profile})...`));
      const cargoArgs = ["cargo", "build", "--bin", "banner"];
      if (!embed) cargoArgs.push("--no-default-features");
      if (release) cargoArgs.push("--release");
      run(cargoArgs);
    }

    if (!existsSync(bin)) {
      console.error(`Binary not found: ${bin}`);
      console.error(`Run 'just build${release ? "" : " -d"}' first, or remove -n to use bacon.`);
      await group.killAll();
      process.exit(1);
    }

    console.log(c("1;36", `→ Running ${bin} (no watch)`));
    group.spawn([bin, ...backendArgs]);
  } else {
    // Bacon watch mode
    const baconArgs = ["bacon", "--headless", "run", "--"];
    if (!embed) baconArgs.push("--no-default-features");
    if (release) baconArgs.push("--profile", "release");
    baconArgs.push("--", ...backendArgs);
    group.spawn(baconArgs);
  }
}

const code = await group.waitForFirst();
process.exit(code);
