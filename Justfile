default_services := "bot,web,scraper"

# Auto-reloading frontend server
frontend:
    pnpm run -C web dev

# Production build of frontend
build-frontend:
    pnpm run -C web build

# Auto-reloading backend server
backend services=default_services:
    bacon --headless run -- -- --services "{{services}}"

# Production build
build:
    pnpm run -C web build
    cargo build --release --bin banner

# Run auto-reloading development build with release characteristics (frontend is embedded, non-auto-reloading)
# This is useful for testing backend release-mode details.
dev-build services=default_services: build-frontend
    bacon --headless run -- --profile dev-release -- --services "{{services}}" --tracing pretty

# Auto-reloading development build for both frontend and backend
# Will not notice if either the frontend/backend crashes, but will generally be resistant to stopping on their own.
[parallel]
dev services=default_services: frontend (backend services)