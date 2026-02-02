/**
 * Run all project checks in parallel. Auto-fixes formatting when safe.
 *
 * Usage: bun scripts/check.ts [--fix|-f]
 */

import { c, elapsed, isStderrTTY } from "./lib/fmt";
import { run, runPiped, spawnCollect, raceInOrder, type CollectResult } from "./lib/proc";
import { existsSync, statSync, readdirSync, writeFileSync, rmSync } from "fs";

const args = process.argv.slice(2);
let fix = false;

for (const arg of args) {
  if (arg === "-f" || arg === "--fix") {
    fix = true;
  } else {
    console.error(`Unknown flag: ${arg}`);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// Fix path: format + clippy fix, then fall through to verification
// ---------------------------------------------------------------------------

if (fix) {
  console.log(c("1;36", "→ Fixing..."));
  run(["cargo", "fmt", "--all"]);
  run(["bun", "run", "--cwd", "web", "format"]);
  run([
    "cargo", "clippy", "--all-features", "--fix", "--allow-dirty", "--allow-staged",
    "--", "--deny", "warnings",
  ]);
  console.log(c("1;36", "→ Verifying..."));
}

// ---------------------------------------------------------------------------
// Ensure TypeScript bindings are up-to-date before frontend checks
// ---------------------------------------------------------------------------

{
  const BINDINGS_DIR = "web/src/lib/bindings";

  let newestSrcMtime = 0;
  for (const file of new Bun.Glob("src/**/*.rs").scanSync(".")) {
    const mt = statSync(file).mtimeMs;
    if (mt > newestSrcMtime) newestSrcMtime = mt;
  }
  for (const f of ["Cargo.toml", "Cargo.lock"]) {
    if (existsSync(f)) {
      const mt = statSync(f).mtimeMs;
      if (mt > newestSrcMtime) newestSrcMtime = mt;
    }
  }

  let newestBindingMtime = 0;
  if (existsSync(BINDINGS_DIR)) {
    for (const file of new Bun.Glob("**/*").scanSync(BINDINGS_DIR)) {
      const mt = statSync(`${BINDINGS_DIR}/${file}`).mtimeMs;
      if (mt > newestBindingMtime) newestBindingMtime = mt;
    }
  }

  const stale = newestBindingMtime === 0 || newestSrcMtime > newestBindingMtime;
  if (stale) {
    const t = Date.now();
    process.stdout.write(
      c("1;36", "→ Regenerating TypeScript bindings (Rust sources changed)...") + "\n",
    );
    run(["cargo", "test", "--no-run"]);
    rmSync(BINDINGS_DIR, { recursive: true, force: true });
    run(["cargo", "test", "export_bindings"]);

    const types = readdirSync(BINDINGS_DIR)
      .filter((f) => f.endsWith(".ts") && f !== "index.ts")
      .map((f) => f.replace(/\.ts$/, ""))
      .sort();
    writeFileSync(
      `${BINDINGS_DIR}/index.ts`,
      types.map((t) => `export type { ${t} } from "./${t}";`).join("\n") + "\n",
    );

    process.stdout.write(c("32", "✓ bindings") + ` (${elapsed(t)}s, ${types.length} types)\n`);
  } else {
    process.stdout.write(c("2", "· bindings up-to-date, skipped") + "\n");
  }
}

// ---------------------------------------------------------------------------
// Check definitions
// ---------------------------------------------------------------------------

interface Check {
  name: string;
  cmd: string[];
  hint?: string;
}

const checks: Check[] = [
  {
    name: "rust-format",
    cmd: ["cargo", "fmt", "--all", "--", "--check"],
    hint: "Run 'cargo fmt --all' to see and fix formatting issues.",
  },
  { name: "rust-lint", cmd: ["cargo", "clippy", "--all-features", "--", "--deny", "warnings"] },
  { name: "rust-check", cmd: ["cargo", "check", "--all-features"] },
  { name: "rust-test", cmd: ["cargo", "nextest", "run", "-E", "not test(export_bindings)"] },
  { name: "svelte-check", cmd: ["bun", "run", "--cwd", "web", "check"] },
  { name: "web-lint", cmd: ["bun", "run", "--cwd", "web", "lint"] },
  { name: "web-format", cmd: ["bun", "run", "--cwd", "web", "format:check"] },
  { name: "web-test", cmd: ["bun", "run", "--cwd", "web", "test"] },
  { name: "actionlint", cmd: ["actionlint"] },
];

// ---------------------------------------------------------------------------
// Domain groups: formatter → { peers, format command, sanity rechecks }
// ---------------------------------------------------------------------------

const domains: Record<
  string,
  {
    peers: string[];
    format: () => ReturnType<typeof runPiped>;
    recheck: Check[];
  }
> = {
  "rust-format": {
    peers: ["rust-lint", "rust-check", "rust-test"],
    format: () => runPiped(["cargo", "fmt", "--all"]),
    recheck: [
      { name: "rust-format", cmd: ["cargo", "fmt", "--all", "--", "--check"] },
      { name: "rust-check", cmd: ["cargo", "check", "--all-features"] },
    ],
  },
  "web-format": {
    peers: ["svelte-check", "web-lint", "web-test"],
    format: () => runPiped(["bun", "run", "--cwd", "web", "format"]),
    recheck: [
      { name: "web-format", cmd: ["bun", "run", "--cwd", "web", "format:check"] },
      { name: "svelte-check", cmd: ["bun", "run", "--cwd", "web", "check"] },
    ],
  },
};

// ---------------------------------------------------------------------------
// Phase 1: run all checks in parallel, display in completion order
// ---------------------------------------------------------------------------

const start = Date.now();
const remaining = new Set(checks.map((ch) => ch.name));

const promises = checks.map(async (check) => ({
  ...check,
  ...(await spawnCollect(check.cmd, start)),
}));

const interval = isStderrTTY
  ? setInterval(() => {
      process.stderr.write(`\r\x1b[K${elapsed(start)}s [${Array.from(remaining).join(", ")}]`);
    }, 100)
  : null;

const results: Record<string, Check & CollectResult> = {};

await raceInOrder(promises, checks, (r) => {
  results[r.name] = r;
  remaining.delete(r.name);
  if (isStderrTTY) process.stderr.write("\r\x1b[K");

  if (r.exitCode !== 0) {
    process.stdout.write(c("31", `✗ ${r.name}`) + ` (${r.elapsed}s)\n`);
    if (r.hint) {
      process.stdout.write(c("2", `  ${r.hint}`) + "\n");
    } else {
      if (r.stdout) process.stdout.write(r.stdout);
      if (r.stderr) process.stderr.write(r.stderr);
    }
  } else {
    process.stdout.write(c("32", `✓ ${r.name}`) + ` (${r.elapsed}s)\n`);
  }
});

if (interval) clearInterval(interval);
if (isStderrTTY) process.stderr.write("\r\x1b[K");

// ---------------------------------------------------------------------------
// Phase 2: auto-fix formatting if it's the only failure in its domain
// ---------------------------------------------------------------------------

const autoFixedDomains = new Set<string>();

for (const [fmtName, domain] of Object.entries(domains)) {
  const fmtResult = results[fmtName];
  if (!fmtResult || fmtResult.exitCode === 0) continue;
  if (!domain.peers.every((p) => results[p]?.exitCode === 0)) continue;

  process.stdout.write(
    "\n" +
      c("1;36", `→ Auto-formatting ${fmtName} (peers passed, only formatting failed)...`) +
      "\n",
  );
  const fmtOut = domain.format();
  if (fmtOut.exitCode !== 0) {
    process.stdout.write(c("31", `  ✗ ${fmtName} formatter failed`) + "\n");
    if (fmtOut.stdout) process.stdout.write(fmtOut.stdout);
    if (fmtOut.stderr) process.stderr.write(fmtOut.stderr);
    continue;
  }

  const recheckStart = Date.now();
  const recheckPromises = domain.recheck.map(async (ch) => ({
    ...ch,
    ...(await spawnCollect(ch.cmd, recheckStart)),
  }));

  let recheckFailed = false;
  await raceInOrder(recheckPromises, domain.recheck, (r) => {
    if (r.exitCode !== 0) {
      recheckFailed = true;
      process.stdout.write(c("31", `  ✗ ${r.name}`) + ` (${r.elapsed}s)\n`);
      if (r.stdout) process.stdout.write(r.stdout);
      if (r.stderr) process.stderr.write(r.stderr);
    } else {
      process.stdout.write(c("32", `  ✓ ${r.name}`) + ` (${r.elapsed}s)\n`);
    }
  });

  if (!recheckFailed) {
    process.stdout.write(c("32", `  ✓ ${fmtName} auto-fix succeeded`) + "\n");
    autoFixedDomains.add(fmtName);
  } else {
    process.stdout.write(c("31", `  ✗ ${fmtName} auto-fix failed sanity check`) + "\n");
  }
}

// ---------------------------------------------------------------------------
// Final verdict
// ---------------------------------------------------------------------------

const finalFailed = Object.entries(results).some(
  ([name, r]) => r.exitCode !== 0 && !autoFixedDomains.has(name),
);

if (autoFixedDomains.size > 0 && !finalFailed) {
  process.stdout.write(
    "\n" + c("1;32", "✓ All checks passed (formatting was auto-fixed)") + "\n",
  );
}

process.exit(finalFailed ? 1 : 0);
