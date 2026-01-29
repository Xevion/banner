set dotenv-load
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
    bun run --cwd web test --run

# Run all tests (Rust + frontend)
test: test-rust test-web

# Run only Rust tests
test-rust *ARGS:
    cargo nextest run {{ARGS}}

# Run only frontend tests
test-web:
    bun run --cwd web test --run

# Quick check: clippy + tests only (skips formatting)
check-quick:
    cargo clippy --all-features -- --deny warnings
    cargo nextest run
    bun run --cwd web typecheck

# Run the Banner API search demo (hits live UTSA API, ~20s)
search *ARGS:
    cargo run -q --bin search -- {{ARGS}}

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

# Smoke test: start web server, hit API endpoints, verify responses
[script("bash")]
test-smoke port="18080":
    set -euo pipefail
    PORT={{port}}

    cleanup() { kill "$SERVER_PID" 2>/dev/null; wait "$SERVER_PID" 2>/dev/null; }

    # Start server in background
    PORT=$PORT cargo run -q --no-default-features -- --services web --tracing json &
    SERVER_PID=$!
    trap cleanup EXIT

    # Wait for server to be ready (up to 15s)
    for i in $(seq 1 30); do
        if curl -sf "http://localhost:$PORT/api/health" >/dev/null 2>&1; then break; fi
        if ! kill -0 "$SERVER_PID" 2>/dev/null; then echo "FAIL: server exited early"; exit 1; fi
        sleep 0.5
    done

    PASS=0; FAIL=0
    check() {
        local label="$1" url="$2" expected="$3"
        body=$(curl -sf "$url") || { echo "FAIL: $label - request failed"; FAIL=$((FAIL+1)); return; }
        if echo "$body" | grep -q "$expected"; then
            echo "PASS: $label"
            PASS=$((PASS+1))
        else
            echo "FAIL: $label - expected '$expected' in: $body"
            FAIL=$((FAIL+1))
        fi
    }

    check "GET /api/health" "http://localhost:$PORT/api/health" '"status":"healthy"'
    check "GET /api/status" "http://localhost:$PORT/api/status" '"version"'
    check "GET /api/metrics" "http://localhost:$PORT/api/metrics" '"banner_api"'

    # Test 404
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:$PORT/api/nonexistent")
    if [ "$STATUS" = "404" ]; then
        echo "PASS: 404 on unknown route"
        PASS=$((PASS+1))
    else
        echo "FAIL: expected 404, got $STATUS"
        FAIL=$((FAIL+1))
    fi

    echo ""
    echo "Results: $PASS passed, $FAIL failed"
    [ "$FAIL" -eq 0 ]

alias b := bun
bun *ARGS:
	cd web && bun {{ ARGS }}

sql *ARGS:
	lazysql ${DATABASE_URL}
