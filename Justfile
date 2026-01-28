default_services := "bot,web,scraper"

default:
    just --list

# Run all checks (format, clippy, tests, lint)
check:
    cargo fmt --all -- --check
    cargo clippy --all-features -- --deny warnings
    cargo nextest run
    bun run --cwd web typecheck
    bun run --cwd web lint

# Format all Rust and TypeScript code
format:
    cargo fmt --all
    bun run --cwd web format

# Check formatting without modifying (CI-friendly)
format-check:
    cargo fmt --all -- --check
    bun run --cwd web format:check

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

# Auto-reloading frontend server
frontend:
    bun run --cwd web dev

# Production build of frontend
build-frontend:
    bun run --cwd web build

# Auto-reloading backend server (with embedded assets)
backend *ARGS:
    bacon --headless run -- -- {{ARGS}}

# Auto-reloading backend server (no embedded assets, for dev proxy mode)
backend-dev *ARGS:
    bacon --headless run -- --no-default-features -- {{ARGS}}

# Production build
build:
    bun run --cwd web build
    cargo build --release --bin banner

# Run auto-reloading development build with release characteristics
dev-build *ARGS='--services web --tracing pretty': build-frontend
    bacon --headless run -- --profile dev-release -- {{ARGS}}

# Auto-reloading development build: Vite frontend + backend (no embedded assets, proxies to Vite)
[parallel]
dev *ARGS='--services web,bot': frontend (backend-dev ARGS)
