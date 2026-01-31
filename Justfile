set dotenv-load

default:
    just --list

# Run all checks in parallel. Pass -f/--fix to auto-format and fix first.
[script("bun")]
check *flags:
    const args = "{{flags}}".split(/\s+/).filter(Boolean);
    let fix = false;
    for (const arg of args) {
      if (arg === "-f" || arg === "--fix") fix = true;
      else { console.error(`Unknown flag: ${arg}`); process.exit(1); }
    }

    // --- Helpers ---

    const useColor = process.stdout.isTTY ?? false;
    const stderrTTY = process.stderr.isTTY ?? false;
    const c = (code, text) => useColor ? `\x1b[${code}m${text}\x1b[0m` : text;
    const since = (t) => ((Date.now() - t) / 1000).toFixed(1);

    /** Sync spawn with inherited stdio (for --fix path). */
    const run = (cmd) => {
      const proc = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
      if (proc.exitCode !== 0) process.exit(proc.exitCode);
    };

    /**
     * Spawn a command, collect stdout/stderr, return a result object.
     * Catches spawn failures (e.g. missing binary) instead of throwing.
     */
    const spawnCollect = async (cmd, startTime) => {
      try {
        const proc = Bun.spawn(cmd, {
          env: { ...process.env, FORCE_COLOR: "1" },
          stdout: "pipe", stderr: "pipe",
        });
        const [stdout, stderr] = await Promise.all([
          new Response(proc.stdout).text(),
          new Response(proc.stderr).text(),
        ]);
        await proc.exited;
        return { stdout, stderr, exitCode: proc.exitCode, elapsed: since(startTime) };
      } catch (err) {
        return { stdout: "", stderr: String(err), exitCode: 1, elapsed: since(startTime) };
      }
    };

    /**
     * Sync spawn with piped stdio. Returns { exitCode, stdout, stderr }.
     * Used for Phase 2 formatters so output doesn't spill into structured results.
     */
    const runPiped = (cmd) => {
      const proc = Bun.spawnSync(cmd, { stdout: "pipe", stderr: "pipe" });
      return {
        exitCode: proc.exitCode,
        stdout: proc.stdout?.toString() ?? "",
        stderr: proc.stderr?.toString() ?? "",
      };
    };

    /**
     * Race all promises, yielding results in completion order via callback.
     * Every promise gets a .catch() wrapper so spawn failures become results, not unhandled rejections.
     */
    const raceInOrder = async (promises, fallbacks, onResult) => {
      const tagged = promises.map((p, i) =>
        p.then(r => ({ i, r }))
         .catch(err => ({ i, r: {
           ...fallbacks[i], exitCode: 1, stdout: "", stderr: String(err), elapsed: "?",
         }}))
      );
      for (let n = 0; n < promises.length; n++) {
        const { i, r } = await Promise.race(tagged);
        tagged[i] = new Promise(() => {}); // sentinel: never resolves
        onResult(r);
      }
    };

    // --- Fix path ---

    if (fix) {
      console.log(c("1;36", "→ Fixing..."));
      run(["cargo", "fmt", "--all"]);
      run(["bun", "run", "--cwd", "web", "format"]);
      run(["cargo", "clippy", "--all-features", "--fix", "--allow-dirty", "--allow-staged",
           "--", "--deny", "warnings"]);
      console.log(c("1;36", "→ Verifying..."));
    }

    // --- Domain groups: formatter → { peers, format command, sanity rechecks } ---

    const domains = {
      rustfmt: {
        peers: ["clippy", "cargo-check", "rust-test"],
        format: () => runPiped(["cargo", "fmt", "--all"]),
        recheck: [
          { name: "rustfmt",     cmd: ["cargo", "fmt", "--all", "--", "--check"] },
          { name: "cargo-check", cmd: ["cargo", "check", "--all-features"] },
        ],
      },
      biome: {
        peers: ["svelte-check", "biome-lint", "web-test"],
        format: () => runPiped(["bun", "run", "--cwd", "web", "format"]),
        recheck: [
          { name: "biome",        cmd: ["bun", "run", "--cwd", "web", "format:check"] },
          { name: "svelte-check", cmd: ["bun", "run", "--cwd", "web", "check"] },
        ],
      },
    };

    // --- Ensure TypeScript bindings are up-to-date before frontend checks ---

    {
      const { statSync, existsSync, readdirSync, writeFileSync, rmSync } = await import("fs");
      const BINDINGS_DIR = "web/src/lib/bindings";

      // Find newest Rust source mtime (src/**/*.rs + Cargo.toml + Cargo.lock)
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

      // Find newest binding output mtime
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
        process.stdout.write(c("1;36", "→ Regenerating TypeScript bindings (Rust sources changed)...") + "\n");
        // Build test binary first (slow part) — fail before deleting anything
        const build = Bun.spawnSync(["cargo", "test", "--no-run"], {
          stdio: ["inherit", "inherit", "inherit"],
        });
        if (build.exitCode !== 0) process.exit(build.exitCode);
        // Clean slate, then run export (fast, already compiled)
        rmSync(BINDINGS_DIR, { recursive: true, force: true });
        const gen = Bun.spawnSync(["cargo", "test", "export_bindings"], {
          stdio: ["inherit", "inherit", "inherit"],
        });
        if (gen.exitCode !== 0) process.exit(gen.exitCode);

        // Auto-generate index.ts
        const types = readdirSync(BINDINGS_DIR)
          .filter(f => f.endsWith(".ts") && f !== "index.ts")
          .map(f => f.replace(/\.ts$/, ""))
          .sort();
        writeFileSync(`${BINDINGS_DIR}/index.ts`, types.map(t => `export type { ${t} } from "./${t}";`).join("\n") + "\n");

        process.stdout.write(c("32", `✓ bindings`) + ` (${since(t)}s, ${types.length} types)\n`);
      } else {
        process.stdout.write(c("2", "· bindings up-to-date, skipped") + "\n");
      }
    }

    // --- Check definitions ---

    const checks = [
      { name: "rustfmt",      cmd: ["cargo", "fmt", "--all", "--", "--check"],
        hint: "Run 'cargo fmt --all' to see and fix formatting issues." },
      { name: "clippy",       cmd: ["cargo", "clippy", "--all-features", "--", "--deny", "warnings"] },
      { name: "cargo-check",  cmd: ["cargo", "check", "--all-features"] },
      { name: "rust-test",    cmd: ["cargo", "nextest", "run", "-E", "not test(export_bindings)"] },
      { name: "svelte-check", cmd: ["bun", "run", "--cwd", "web", "check"] },
      { name: "biome",        cmd: ["bun", "run", "--cwd", "web", "format:check"] },
      { name: "biome-lint",   cmd: ["bun", "run", "--cwd", "web", "lint"] },
      { name: "web-test",     cmd: ["bun", "run", "--cwd", "web", "test"] },
      { name: "actionlint",   cmd: ["actionlint"] },
      // { name: "sqlx-prepare", cmd: ["cargo", "sqlx", "prepare", "--check"] },
    ];

    // --- Phase 1: run all checks in parallel, display results in completion order ---

    const start = Date.now();
    const remaining = new Set(checks.map(ch => ch.name));

    const promises = checks.map(async (check) => {
      if (check.fn) {
        return { ...check, ...(await check.fn(start)) };
      }
      return { ...check, ...(await spawnCollect(check.cmd, start)) };
    });

    const interval = stderrTTY ? setInterval(() => {
      process.stderr.write(`\r\x1b[K${since(start)}s [${Array.from(remaining).join(", ")}]`);
    }, 100) : null;

    const results = {};
    await raceInOrder(promises, checks, (r) => {
      results[r.name] = r;
      remaining.delete(r.name);
      if (stderrTTY) process.stderr.write(`\r\x1b[K`);

      if (r.exitCode !== 0) {
        process.stdout.write(c("31", `✗ ${r.name}`) + ` (${r.elapsed}s)\n`);
        if (r.hint) {
          process.stdout.write(c("2", `  ${r.hint}`) + `\n`);
        } else {
          if (r.stdout) process.stdout.write(r.stdout);
          if (r.stderr) process.stderr.write(r.stderr);
        }
      } else {
        process.stdout.write(c("32", `✓ ${r.name}`) + ` (${r.elapsed}s)\n`);
      }
    });

    if (interval) clearInterval(interval);
    if (stderrTTY) process.stderr.write(`\r\x1b[K`);

    // --- Phase 2: auto-fix formatting if it's the only failure in its domain ---

    const autoFixedDomains = new Set();
    for (const [fmtName, domain] of Object.entries(domains)) {
      const fmtResult = results[fmtName];
      if (!fmtResult || fmtResult.exitCode === 0) continue;
      if (!domain.peers.every(p => results[p]?.exitCode === 0)) continue;

      process.stdout.write(`\n` + c("1;36", `→ Auto-formatting ${fmtName} (peers passed, only formatting failed)...`) + `\n`);
      const fmtOut = domain.format();
      if (fmtOut.exitCode !== 0) {
        process.stdout.write(c("31", `  ✗ ${fmtName} formatter failed`) + `\n`);
        if (fmtOut.stdout) process.stdout.write(fmtOut.stdout);
        if (fmtOut.stderr) process.stderr.write(fmtOut.stderr);
        continue;
      }

      // Re-verify in parallel, display in completion order
      const recheckStart = Date.now();
      const recheckPromises = domain.recheck.map(async (ch) => ({
        ...ch, ...(await spawnCollect(ch.cmd, recheckStart)),
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
        process.stdout.write(c("32", `  ✓ ${fmtName} auto-fix succeeded`) + `\n`);
        autoFixedDomains.add(fmtName);
      } else {
        process.stdout.write(c("31", `  ✗ ${fmtName} auto-fix failed sanity check`) + `\n`);
      }
    }

    // --- Final verdict ---

    const finalFailed = Object.entries(results).some(
      ([name, r]) => r.exitCode !== 0 && !autoFixedDomains.has(name)
    );
    if (autoFixedDomains.size > 0 && !finalFailed) {
      process.stdout.write(`\n` + c("1;32", "✓ All checks passed (formatting was auto-fixed)") + `\n`);
    }
    process.exit(finalFailed ? 1 : 0);

# Format all Rust and TypeScript code
format:
    cargo fmt --all
    bun run --cwd web format

# Run tests. Usage: just test [rust|web|<nextest filter args>]
[script("bun")]
test *args:
    const input = "{{args}}".trim();
    const run = (cmd) => {
      const proc = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
      if (proc.exitCode !== 0) process.exit(proc.exitCode);
    };
    if (input === "web") {
      run(["bun", "run", "--cwd", "web", "test"]);
    } else if (input === "rust") {
      run(["cargo", "nextest", "run", "-E", "not test(export_bindings)"]);
    } else if (input === "") {
      run(["cargo", "nextest", "run", "-E", "not test(export_bindings)"]);
      run(["bun", "run", "--cwd", "web", "test"]);
    } else {
      run(["cargo", "nextest", "run", ...input.split(/\s+/)]);
    }

# Generate TypeScript bindings from Rust types (ts-rs)
[script("bun")]
bindings:
    const { readdirSync, writeFileSync, rmSync } = await import("fs");
    const dir = "web/src/lib/bindings";
    const run = (cmd) => {
      const r = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
      if (r.exitCode !== 0) process.exit(r.exitCode);
    };

    // Build test binary first (slow part) — fail before deleting anything
    run(["cargo", "test", "--no-run"]);
    // Clean slate
    rmSync(dir, { recursive: true, force: true });
    // Run the export (fast, already compiled)
    run(["cargo", "test", "export_bindings"]);

    // Auto-generate index.ts from emitted .ts files
    const types = readdirSync(dir)
      .filter(f => f.endsWith(".ts") && f !== "index.ts")
      .map(f => f.replace(/\.ts$/, ""))
      .sort();
    writeFileSync(`${dir}/index.ts`, types.map(t => `export type { ${t} } from "./${t}";`).join("\n") + "\n");
    console.log(`Generated ${dir}/index.ts (${types.length} types)`);

# Run the Banner API search demo (hits live UTSA API, ~20s)
search *ARGS:
    cargo run -q --bin search -- {{ARGS}}

# Pass args to binary after --: just dev -n -- --some-flag
# Dev server. Flags: -f(rontend) -b(ackend) -W(no-watch) -n(o-build) -r(elease) -e(mbed) --tracing <fmt>
[script("bun")]
dev *flags:
    const argv = "{{flags}}".split(/\s+/).filter(Boolean);

    let frontendOnly = false, backendOnly = false;
    let noWatch = false, noBuild = false, release = false, embed = false;
    let tracing = "pretty";
    const passthrough = [];

    let i = 0;
    let seenDashDash = false;
    while (i < argv.length) {
      const arg = argv[i];
      if (seenDashDash) { passthrough.push(arg); i++; continue; }
      if (arg === "--") { seenDashDash = true; i++; continue; }
      if (arg.startsWith("--")) {
        if (arg === "--frontend-only") frontendOnly = true;
        else if (arg === "--backend-only") backendOnly = true;
        else if (arg === "--no-watch") noWatch = true;
        else if (arg === "--no-build") noBuild = true;
        else if (arg === "--release") release = true;
        else if (arg === "--embed") embed = true;
        else if (arg === "--tracing") { tracing = argv[++i] || "pretty"; }
        else { console.error(`Unknown flag: ${arg}`); process.exit(1); }
      } else if (arg.startsWith("-") && arg.length > 1) {
        for (const c of arg.slice(1)) {
          if (c === "f") frontendOnly = true;
          else if (c === "b") backendOnly = true;
          else if (c === "W") noWatch = true;
          else if (c === "n") noBuild = true;
          else if (c === "r") release = true;
          else if (c === "e") embed = true;
          else { console.error(`Unknown flag: -${c}`); process.exit(1); }
        }
      } else { console.error(`Unknown argument: ${arg}`); process.exit(1); }
      i++;
    }

    // -e implies -b (no point running Vite if assets are embedded)
    if (embed) backendOnly = true;
    // -n implies -W (no build means no watch)
    if (noBuild) noWatch = true;

    // Validate conflicting flags
    if (frontendOnly && backendOnly) {
      console.error("Cannot use -f and -b together (or -e implies -b)");
      process.exit(1);
    }

    const runFrontend = !backendOnly;
    const runBackend = !frontendOnly;
    const profile = release ? "release" : "dev";
    const profileDir = release ? "release" : "debug";

    const procs = [];
    const cleanup = async () => {
      for (const p of procs) p.kill();
      await Promise.all(procs.map(p => p.exited));
    };
    process.on("SIGINT", async () => { await cleanup(); process.exit(0); });
    process.on("SIGTERM", async () => { await cleanup(); process.exit(0); });

    // Build frontend first when embedding assets (backend will bake them in)
    if (embed && !noBuild) {
      console.log(`\x1b[1;36m→ Building frontend (for embedding)...\x1b[0m`);
      const fb = Bun.spawnSync(["bun", "run", "--cwd", "web", "build"], {
        stdio: ["inherit", "inherit", "inherit"],
      });
      if (fb.exitCode !== 0) process.exit(fb.exitCode);
    }

    // Frontend: Vite dev server
    if (runFrontend) {
      const proc = Bun.spawn(["bun", "run", "--cwd", "web", "dev"], {
        stdio: ["inherit", "inherit", "inherit"],
      });
      procs.push(proc);
    }

    // Backend
    if (runBackend) {
      const backendArgs = [`--tracing`, tracing, ...passthrough];
      const bin = `target/${profileDir}/banner`;

      if (noWatch) {
        // Build first unless -n (skip build)
        if (!noBuild) {
          console.log(`\x1b[1;36m→ Building backend (${profile})...\x1b[0m`);
          const cargoArgs = ["cargo", "build", "--bin", "banner"];
          if (!embed) cargoArgs.push("--no-default-features");
          if (release) cargoArgs.push("--release");
          const build = Bun.spawnSync(cargoArgs, { stdio: ["inherit", "inherit", "inherit"] });
          if (build.exitCode !== 0) { cleanup(); process.exit(build.exitCode); }
        }

        // Run the binary directly (no watch)
        const { existsSync } = await import("fs");
        if (!existsSync(bin)) {
          console.error(`Binary not found: ${bin}`);
          console.error(`Run 'just build${release ? "" : " -d"}' first, or remove -n to use bacon.`);
          cleanup();
          process.exit(1);
        }
        console.log(`\x1b[1;36m→ Running ${bin} (no watch)\x1b[0m`);
        const proc = Bun.spawn([bin, ...backendArgs], {
          stdio: ["inherit", "inherit", "inherit"],
        });
        procs.push(proc);
      } else {
        // Bacon watch mode
        const baconArgs = ["bacon", "--headless", "run", "--"];
        if (!embed) baconArgs.push("--no-default-features");
        if (release) baconArgs.push("--profile", "release");
        baconArgs.push("--", ...backendArgs);
        const proc = Bun.spawn(baconArgs, {
          stdio: ["inherit", "inherit", "inherit"],
        });
        procs.push(proc);
      }
    }

    // Wait for any process to exit, then kill the rest
    const results = procs.map((p, i) => p.exited.then(code => ({ i, code })));
    const first = await Promise.race(results);
    cleanup();
    process.exit(first.code);

# Production build. Flags: -d(ebug) -f(rontend-only) -b(ackend-only)
[script("bun")]
build *flags:
    const argv = "{{flags}}".split(/\s+/).filter(Boolean);

    let debug = false, frontendOnly = false, backendOnly = false;
    for (const arg of argv) {
      if (arg.startsWith("--")) {
        if (arg === "--debug") debug = true;
        else if (arg === "--frontend-only") frontendOnly = true;
        else if (arg === "--backend-only") backendOnly = true;
        else { console.error(`Unknown flag: ${arg}`); process.exit(1); }
      } else if (arg.startsWith("-") && arg.length > 1) {
        for (const c of arg.slice(1)) {
          if (c === "d") debug = true;
          else if (c === "f") frontendOnly = true;
          else if (c === "b") backendOnly = true;
          else { console.error(`Unknown flag: -${c}`); process.exit(1); }
        }
      } else { console.error(`Unknown argument: ${arg}`); process.exit(1); }
    }

    if (frontendOnly && backendOnly) {
      console.error("Cannot use -f and -b together");
      process.exit(1);
    }

    const run = (cmd) => {
      const proc = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
      if (proc.exitCode !== 0) process.exit(proc.exitCode);
    };

    const buildFrontend = !backendOnly;
    const buildBackend = !frontendOnly;
    const profile = debug ? "debug" : "release";

    if (buildFrontend) {
      console.log("\x1b[1;36m→ Building frontend...\x1b[0m");
      run(["bun", "run", "--cwd", "web", "build"]);
    }

    if (buildBackend) {
      console.log(`\x1b[1;36m→ Building backend (${profile})...\x1b[0m`);
      const cmd = ["cargo", "build", "--bin", "banner"];
      if (!debug) cmd.push("--release");
      run(cmd);
    }

# Start PostgreSQL in Docker and update .env with connection string
# Commands: start (default), reset, rm
[script("bun")]
db cmd="start":
    const fs = await import("fs/promises");
    const { spawnSync } = await import("child_process");

    const NAME = "banner-postgres";
    const USER = "banner";
    const PASS = "banner";
    const DB = "banner";
    const PORT = "59489";
    const ENV_FILE = ".env";
    const CMD = "{{cmd}}";

    const run = (args) => spawnSync("docker", args, { encoding: "utf8" });
    const getContainer = () => {
      const res = run(["ps", "-a", "--filter", `name=^${NAME}$`, "--format", "json"]);
      return res.stdout.trim() ? JSON.parse(res.stdout) : null;
    };

    const updateEnv = async () => {
      const url = `postgresql://${USER}:${PASS}@localhost:${PORT}/${DB}`;
      try {
        let content = await fs.readFile(ENV_FILE, "utf8");
        content = content.includes("DATABASE_URL=")
          ? content.replace(/DATABASE_URL=.*$/m, `DATABASE_URL=${url}`)
          : content.trim() + `\nDATABASE_URL=${url}\n`;
        await fs.writeFile(ENV_FILE, content);
      } catch {
        await fs.writeFile(ENV_FILE, `DATABASE_URL=${url}\n`);
      }
    };

    const create = () => {
      run(["run", "-d", "--name", NAME, "-e", `POSTGRES_USER=${USER}`,
           "-e", `POSTGRES_PASSWORD=${PASS}`, "-e", `POSTGRES_DB=${DB}`,
           "-p", `${PORT}:5432`, "postgres:17-alpine"]);
      console.log("created");
    };

    const container = getContainer();

    if (CMD === "rm") {
      if (!container) process.exit(0);
      run(["stop", NAME]);
      run(["rm", NAME]);
      console.log("removed");
    } else if (CMD === "reset") {
      if (!container) create();
      else {
        run(["exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `DROP DATABASE IF EXISTS ${DB}`]);
        run(["exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `CREATE DATABASE ${DB}`]);
        console.log("reset");
      }
      await updateEnv();
    } else {
      if (!container) {
        create();
      } else if (container.State !== "running") {
        run(["start", NAME]);
        console.log("started");
      } else {
        console.log("running");
      }
      await updateEnv();
    }

alias b := bun
bun *ARGS:
	cd web && bun {{ ARGS }}

sql *ARGS:
	lazysql ${DATABASE_URL}
