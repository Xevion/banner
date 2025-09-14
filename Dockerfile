# Build arguments
ARG RUST_VERSION=1.89.0

# Frontend Build Stage
FROM node:22-bookworm-slim AS frontend-builder

# Install pnpm
RUN npm install -g pnpm

WORKDIR /app

# Copy frontend package files
COPY ./web/package.json ./web/pnpm-lock.yaml ./

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy frontend source code
COPY ./web ./

# Build frontend
RUN pnpm run build

# Rust Build Stage
FROM rust:${RUST_VERSION}-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src
RUN USER=root cargo new --bin banner
WORKDIR /usr/src/banner

# Copy dependency files for better layer caching
COPY ./Cargo.toml ./Cargo.lock* ./

# Copy .git directory for build.rs to access Git information (if available)
# This will copy .git (and .gitignore) if it exists, but won't fail if it doesn't
# While normally a COPY requires at least one file, .gitignore should still be available, so this wildcard should always work
COPY ./.git* ./

# Copy build.rs early so it can run during the first build
COPY ./build.rs ./

# Build empty app with downloaded dependencies to produce a stable image layer for next build
RUN cargo build --release

# Copy source code
RUN rm src/*.rs
COPY ./src ./src/

# Copy built frontend assets
COPY --from=frontend-builder /app/dist ./web/dist

# Build web app with embedded assets
RUN rm ./target/release/deps/banner*
RUN cargo build --release

# Strip the binary to reduce size
RUN strip target/release/banner

# Runtime Stage - Debian slim for glibc compatibility
FROM debian:12-slim

ARG APP=/usr/src/app
ARG APP_USER=appuser
ARG UID=1000
ARG GID=1000

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata \
    wget \
    && rm -rf /var/lib/apt/lists/*

ARG TZ=Etc/UTC
ENV TZ=${TZ}

# Create user with specific UID/GID
RUN addgroup --gid $GID $APP_USER \
    && adduser --uid $UID --disabled-password --gecos "" --ingroup $APP_USER $APP_USER \
    && mkdir -p ${APP}

# Copy application binary
COPY --from=builder --chown=$APP_USER:$APP_USER /usr/src/banner/target/release/banner ${APP}/banner

# Set proper permissions
RUN chmod +x ${APP}/banner

USER $APP_USER
WORKDIR ${APP}

# Build-time arg for PORT, default to 8000
ARG PORT=8000
# Runtime environment var for PORT, default to build-time arg
ENV PORT=${PORT}
EXPOSE ${PORT}

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT}/health || exit 1

# Can be explicitly overriden with different hosts & ports
ENV HOSTS=0.0.0.0,[::]

# Implicitly uses PORT environment variable
# temporary: running without 'scraper' service
CMD ["sh", "-c", "exec ./banner --services web,bot"]