# banner

A complex multi-service system providing a Discord bot and browser-based interface to UTSA's course data.

## Services

The application consists of three modular services that can be run independently or together:

- Discord Bot ([`bot`][src-bot])

  - Primary interface for course monitoring and data queries
  - Built with [Serenity][serenity] and [Poise][poise] frameworks for robust command handling
  - Uses slash commands with comprehensive error handling and logging

- Web Server ([`web`][src-web])

  - [Axum][axum]-based server with Vite/React-based frontend
  - [Embeds static assets][rust-embed] at compile time with E-Tags & Cache-Control headers

- Scraper ([`scraper`][src-scraper])

  - Intelligent data collection system with priority-based queuing inside PostgreSQL via [`sqlx`][sqlx]
  - Rate-limited scraping with burst handling to respect UTSA's systems
  - Handles course data updates, availability changes, and metadata synchronization

## Quick Start

```bash
pnpm install -C web  # Install frontend dependencies
cargo build  # Build the backend

just dev # Runs auto-reloading dev build
just dev bot,web # Runs auto-reloading dev build, running only the bot and web services
just dev-build # Development build with release characteristics (frontend is embedded, non-auto-reloading)

just build # Production build that embeds assets
```

## Documentation

Comprehensive documentation is available in the [`docs/`][documentation] folder.

[documentation]: docs/README.md
[src-bot]: src/bot
[src-web]: src/web
[src-scraper]: src/scraper
[serenity]: https://github.com/serenity-rs/serenity
[poise]: https://github.com/serenity-rs/poise
[axum]: https://github.com/tokio-rs/axum
[rust-embed]: https://lib.rs/crates/rust-embed
[sqlx]: https://github.com/launchbadge/sqlx
