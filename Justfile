set dotenv-load

default:
    just --list

# Run all checks in parallel. Pass -f/--fix to auto-format and fix first.
check *flags:
    bun scripts/check.ts {{flags}}

# Format all Rust and TypeScript code
format:
    cargo fmt --all
    bun run --cwd web format

# Run tests. Usage: just test [rust|web|<nextest filter args>]
test *args:
    bun scripts/test.ts {{args}}

# Generate TypeScript bindings from Rust types (ts-rs)
bindings:
    bun scripts/bindings.ts

# Run the Banner API search demo (hits live UTSA API, ~20s)
search *ARGS:
    cargo run -q --bin search -- {{ARGS}}

# Dev server. Flags: -f(rontend) -b(ackend) -W(no-watch) -n(o-build) -r(elease) -e(mbed) --tracing <fmt>
# Pass args to binary after --: just dev -n -- --some-flag
dev *flags:
    bun scripts/dev.ts {{flags}}

# Production build. Flags: -d(ebug) -f(rontend-only) -b(ackend-only)
build *flags:
    bun scripts/build.ts {{flags}}

# Start PostgreSQL in Docker and update .env with connection string
# Commands: start (default), reset, rm
db cmd="start":
    bun scripts/db.ts {{cmd}}

alias b := bun
bun *ARGS:
	cd web && bun {{ ARGS }}

sql *ARGS:
	lazysql ${DATABASE_URL}
