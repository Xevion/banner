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

    const run = (cmd) => {
      const proc = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
      if (proc.exitCode !== 0) process.exit(proc.exitCode);
    };

    if (fix) {
      console.log("\x1b[1;36m→ Fixing...\x1b[0m");
      run(["cargo", "fmt", "--all"]);
      run(["bun", "run", "--cwd", "web", "format"]);
      run(["cargo", "clippy", "--all-features", "--fix", "--allow-dirty", "--allow-staged",
           "--", "--deny", "warnings"]);
      console.log("\x1b[1;36m→ Verifying...\x1b[0m");
    }

    const checks = [
      { name: "rustfmt",      cmd: ["cargo", "fmt", "--all", "--", "--check"] },
      { name: "clippy",       cmd: ["cargo", "clippy", "--all-features", "--", "--deny", "warnings"] },
      { name: "rust-test",    cmd: ["cargo", "nextest", "run", "-E", "not test(export_bindings)"] },
      { name: "svelte-check", cmd: ["bun", "run", "--cwd", "web", "check"] },
      { name: "biome",        cmd: ["bun", "run", "--cwd", "web", "format:check"] },
      { name: "web-test",     cmd: ["bun", "run", "--cwd", "web", "test"] },
      // { name: "sqlx-prepare", cmd: ["cargo", "sqlx", "prepare", "--check"] },
    ];

    const isTTY = process.stderr.isTTY;
    const start = Date.now();
    const remaining = new Set(checks.map(c => c.name));

    const promises = checks.map(async (check) => {
      const proc = Bun.spawn(check.cmd, {
        env: { ...process.env, FORCE_COLOR: "1" },
        stdout: "pipe", stderr: "pipe",
      });
      const [stdout, stderr] = await Promise.all([
        new Response(proc.stdout).text(),
        new Response(proc.stderr).text(),
      ]);
      await proc.exited;
      return { ...check, stdout, stderr, exitCode: proc.exitCode,
               elapsed: ((Date.now() - start) / 1000).toFixed(1) };
    });

    const interval = isTTY ? setInterval(() => {
      const elapsed = ((Date.now() - start) / 1000).toFixed(1);
      process.stderr.write(`\r\x1b[K${elapsed}s [${Array.from(remaining).join(", ")}]`);
    }, 100) : null;

    let anyFailed = false;
    for (const promise of promises) {
      const r = await promise;
      remaining.delete(r.name);
      if (isTTY) process.stderr.write(`\r\x1b[K`);
      if (r.exitCode !== 0) {
        anyFailed = true;
        process.stdout.write(`\x1b[31m✗ ${r.name}\x1b[0m (${r.elapsed}s)\n`);
        if (r.stdout) process.stdout.write(r.stdout);
        if (r.stderr) process.stderr.write(r.stderr);
      } else {
        process.stdout.write(`\x1b[32m✓ ${r.name}\x1b[0m (${r.elapsed}s)\n`);
      }
    }

    if (interval) clearInterval(interval);
    if (isTTY) process.stderr.write(`\r\x1b[K`);
    process.exit(anyFailed ? 1 : 0);

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
bindings:
    cargo test export_bindings

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
