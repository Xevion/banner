# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.6.1](https://github.com/Xevion/Banner/compare/v0.6.0...v0.6.1) (2026-01-31)


### Features

* **build:** Auto-regenerate TypeScript bindings on source changes ([e203e8e](https://github.com/Xevion/Banner/commit/e203e8e182f7a0b0224a8f9e6bf79d15259215a2))
* **course:** Distinguish async from synchronous online courses ([8bfc14e](https://github.com/Xevion/Banner/commit/8bfc14e55c1bdf5acc2006096476e0b1eb1b7cc6))
* **scraper:** Improve dashboard clarity with stat tooltips ([1ad614d](https://github.com/Xevion/Banner/commit/1ad614dad03d3631a8d119203786718c814e72c7))
* **scraper:** Improve results visibility and loading states ([c533768](https://github.com/Xevion/Banner/commit/c53376836238f3aca92ac82cd5fd59a077bcceff))


### Bug Fixes

* Avoid status flickering on subjects table ([2689587](https://github.com/Xevion/Banner/commit/2689587dd53c572a65eeb91f74c737662e1f148b))
* **ci:** Add postgres container service for rust tests ([ebb7a97](https://github.com/Xevion/Banner/commit/ebb7a97c113fa1d4b61b8637dfe97cae5260075c))
* **ci:** Fix rust/frontend/security job failures and expand local checks ([dd148e0](https://github.com/Xevion/Banner/commit/dd148e08a0b6d5b7afe4ff614d7d6e4e4d0dfce6))
* **data:** Decode HTML entities in course titles and instructor names ([7d2255a](https://github.com/Xevion/Banner/commit/7d2255a988a23f6e1b1c8e7cb5a8ead833ad34da))
* **metrics:** Always emit baseline metrics on initial course insertion ([16039e0](https://github.com/Xevion/Banner/commit/16039e02a999c668d4969a43eb9ed1d4e8d370e1))


### Code Refactoring

* **terms:** Move term formatting from frontend to backend ([cbb0a51](https://github.com/Xevion/Banner/commit/cbb0a51bca9e4e0d6a8fcee90465c93943f2a30e))
* Use friendly term codes in URL query parameters ([550401b](https://github.com/Xevion/Banner/commit/550401b85ceb8a447e316209b479c69062c5b658))


### Continuous Integration

* Add Release Please automation for changelog and version management ([6863ee5](https://github.com/Xevion/Banner/commit/6863ee58d0a5778303af1b7626b2a9eda3043ca0))
* Split quality checks into parallel jobs with security scanning ([3494341](https://github.com/Xevion/Banner/commit/3494341e3fbe9ffd96b6fcd8abbe7f95ecec6f45))


### Miscellaneous

* Add ts-rs generated bindings ([2df0ba0](https://github.com/Xevion/Banner/commit/2df0ba0ec58155d73830a66132cb635dc819e8a9))
* Update frontend packages ([acccaa5](https://github.com/Xevion/Banner/commit/acccaa54d4455500db60d1b6437cad1c592445f1))

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
