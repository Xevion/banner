frontend:
    pnpm run -C web dev

backend:
    cargo run --bin banner

build-frontend:
    pnpm run -C web build

build-backend:
    cargo build --release --bin banner

build: build-frontend build-backend

# Production build that embeds assets
build-prod:
    pnpm run -C web build
    cargo build --release --bin banner

[parallel]
dev: frontend backend