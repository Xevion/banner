# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.6.0] - 2026-01-30

### Added

- User authentication system with Discord OAuth, sessions, admin roles, and login page with FAQ.
- Interactive timeline visualization with D3 canvas, pan/zoom, touch gestures, and enrollment aggregation API.
- Scraper analytics dashboard with timeseries charts, subject monitoring, and per-subject detail views.
- Adaptive scraper scheduling with admin endpoints for monitoring and configuration.
- Scrape job result persistence for effectiveness tracking.
- WebSocket support for real-time scrape job monitoring with connection status indicators.
- Course change auditing with field-level tracking and time-series metrics endpoint.
- Audit log UI with smart JSON diffing, conditional request caching, and auto-refresh.
- Calendar export web endpoints for ICS download and Google Calendar redirect.
- Confidence-based RMP matching with manual review workflow and admin instructor UI.
- RMP profile links and confidence-aware rating display.
- Name parsing and normalization for improved instructor-RMP matching.
- Mobile touch controls with gesture detection for timeline.
- Worker timeout protection and crash recovery for job queue.
- Build-time asset compression with encoding negotiation (gzip, brotli, zstd).
- Smart page transitions with theme-aware element transitions.
- Search duration and result count feedback.
- Root error page handling.
- Login page with FAQ section and improved styling.

### Changed

- Consolidated navigation with top nav bar and route groups.
- Centralized number formatting with locale-aware utility.
- Modernized Justfile commands and simplified service management.
- Persisted audit log state in module scope for cross-navigation caching.
- Relative time feedback and improved tooltip customization.

### Fixed

- Instructor/course mismatching via build-order-independent map for association.
- Page content clipping.
- Backend startup delays with retry logic in auth.
- Banner API timeouts increased to handle slow responses.
- i64 serialization for JavaScript compatibility, fixing avatar URL display.
- Frontend build ordering with `-e` embed flag in Justfile.
- Login page centering and unnecessary scrollbar.
- ts-rs serde warnings.

## [0.5.0] - 2026-01-29

### Added

- Multi-select subject filtering with searchable comboboxes.
- Smart instructor name abbreviation for compact table display.
- Delivery mode indicators and tooltips in location column.
- Page selector dropdown with animated pagination controls.
- FLIP animations for smooth table row transitions during pagination.
- Time tooltip with detailed meeting schedule and day abbreviations.
- Reusable SimpleTooltip component for consistent UI hints.

### Changed

- Consolidated query logic and eliminated N+1 instructor loads via batch fetching.
- Consolidated menu snippets and strengthened component type safety.
- Enhanced table scrolling with OverlayScrollbars and theme-aware styling.
- Eliminated initial theme flash on page load.

## [0.4.0] - 2026-01-28

### Added

- Web-based course search UI with interactive data table, multi-column sorting, and column visibility controls.
- TypeScript type bindings generated from Rust types via ts-rs.
- RateMyProfessors integration: bulk professor sync via GraphQL and inline rating display in search results.
- Course detail expansion panel with enrollment, meeting times, and instructor info.
- OverlayScrollbars integration for styled, theme-aware scrollable areas.
- Pagination component for navigating large search result sets.
- Footer component with version display.
- API endpoints: `/api/courses/search`, `/api/courses/:term/:crn`, `/api/terms`, `/api/subjects`, `/api/reference/:category`.
- Frontend API client with typed request/response handling and test coverage.
- Course formatting utilities with comprehensive unit tests.

## [0.3.4] - 2026-01

### Added

- Live service status tracking on web dashboard with auto-refresh and health indicators.
- DB operation extraction for improved testability.
- Unit test suite foundation covering core functionality.
- Docker support for PostgreSQL development environment.
- ICS calendar export with comprehensive holiday exclusion coverage.
- Google Calendar link generation with recurrence rules and meeting details.
- Job queue with priority-based scheduling for background scraping.
- Rate limiting with burst allowance for Banner API requests.
- Session management and caching for Banner API interactions.
- Discord bot commands: search, terms, ics, gcal.
- Intelligent scraping system with priority queues and retry tracking.

### Changed

- Type consolidation and dead code removal across the codebase.
