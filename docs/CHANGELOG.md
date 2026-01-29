# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
