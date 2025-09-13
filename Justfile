default_services := "bot,web,scraper"

# Auto-reloading frontend server
frontend:
    pnpm run -C web dev

# Production build of frontend
build-frontend:
    pnpm run -C web build

# Auto-reloading backend server
backend *ARGS:
    bacon --headless run -- -- {{ARGS}}

# Production build
build:
    pnpm run -C web build
    cargo build --release --bin banner

# Run auto-reloading development build with release characteristics (frontend is embedded, non-auto-reloading)
# This is useful for testing backend release-mode details.
dev-build *ARGS='--services web --tracing pretty': build-frontend
    bacon --headless run -- --profile dev-release -- {{ARGS}}

# Auto-reloading development build for both frontend and backend
# Will not notice if either the frontend/backend crashes, but will generally be resistant to stopping on their own.
[parallel]
dev: frontend backend