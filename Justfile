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
db:
    #!/usr/bin/env bash
    set -euo pipefail

    # Find available port
    PORT=$(shuf -i 49152-65535 -n 1)
    while ss -tlnp 2>/dev/null | grep -q ":$PORT "; do
        PORT=$(shuf -i 49152-65535 -n 1)
    done

    # Start PostgreSQL container
    docker run -d \
        --name banner-postgres \
        -e POSTGRES_PASSWORD=banner \
        -e POSTGRES_USER=banner \
        -e POSTGRES_DB=banner \
        -p "$PORT:5432" \
        postgres:17-alpine

    # Update .env file
    DB_URL="postgresql://banner:banner@localhost:$PORT/banner"
    if [ -f .env ]; then
        sed -i.bak "s|^DATABASE_URL=.*|DATABASE_URL=$DB_URL|" .env
    else
        echo "DATABASE_URL=$DB_URL" > .env
    fi

    echo "PostgreSQL started on port $PORT"
    echo "DATABASE_URL=$DB_URL"
    echo "Run: sqlx migrate run"

# Auto-reloading frontend server
frontend:
    bun run --cwd web dev

# Production build of frontend
build-frontend:
    bun run --cwd web build

# Auto-reloading backend server
backend *ARGS:
    bacon --headless run -- -- {{ARGS}}

# Production build
build:
    bun run --cwd web build
    cargo build --release --bin banner

# Run auto-reloading development build with release characteristics
dev-build *ARGS='--services web --tracing pretty': build-frontend
    bacon --headless run -- --profile dev-release -- {{ARGS}}

# Auto-reloading development build for both frontend and backend
[parallel]
dev *ARGS='--services web,bot': frontend (backend ARGS)
