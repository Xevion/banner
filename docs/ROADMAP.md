# Roadmap

## Now

- **Discord bot revival** - Audit and fix all existing commands (search, terms, ics, gcal) against the current data model. Add test coverage. Bot has been untouched since ~0.3.4 and commands may be broken.
- **Notification and subscription system** - Subscribe to courses and get alerts on seat availability, waitlist movement, and detail changes (time, location, professor, seats). Deliver via Discord bot and web dashboard.
- **Mobile/responsive redesign** - Hamburger nav for sidebar, responsive table column hiding, mobile-friendly admin pages. Timeline is the only area with solid mobile support; most pages need work.
- **Professor name search filter** - Filter search results by instructor. Backend code exists but is commented out.
- **Search field autocomplete** - Typeahead for course titles, course numbers, professors, and terms.
- **Large component extraction** - Break down CourseTable, Instructors page, and TimelineCanvas into smaller, testable subcomponents.

## Soon

- **Bot slash command parity** - Keep Discord bot commands in sync with web features: timeline summaries, RMP lookups, audit log highlights, notification management via bot.
- **E2E test suite** - Playwright tests for critical user flows: search, login, admin pages, timeline interaction.
- **Settings page** - Replace placeholder with theme preferences, notification settings, default term/subject selection.
- **Profile enhancements** - Expand from read-only stub to subscription management, saved searches, and course watchlists.
- **Smart time-of-day search parsing** - Support natural queries like "2 PM", "ends by 2 PM", "after 2 PM" mapped to time ranges.
- **Multi-term querying** - Query across multiple terms in a single search instead of one at a time.
- **Historical analytics visualization** - Build trend UI on top of existing course metrics and timeline API. Fill-rate charts per course or professor.
- **Schedule builder** - Visual weekly schedule tool for assembling a conflict-free course lineup. Timeline visualization serves as a foundation.

## Eventually

- **API rate limiting** - Rate limiter on public API endpoints. Needed before any public or external exposure.
- **Bulk admin operations** - Batch RMP match/reject, bulk user management, data export from admin pages.
- **Degree audit helper** - Map available courses to degree requirements and suggest what to take next.
- **DM support** - Allow the Discord bot to respond in direct messages, not just guild channels.
- **"Classes Now" command** - Find classes currently in session based on the current day and time.
- **Privileged error feedback** - Detailed error information surfaced to bot admins when commands fail.

## Done

- **Interactive timeline visualization** - D3 canvas with pan/zoom, touch gestures, and enrollment aggregation API. *(0.6.0)*
- **Scraper analytics dashboard** - Timeseries charts, subject monitoring, adaptive scheduling, and admin endpoints. *(0.6.0)*
- **WebSocket job monitoring** - Real-time scrape job queue with live connection status indicators. *(0.6.0)*
- **Course change audit log** - Field-level change tracking with smart diffing, conditional caching, and auto-refresh. *(0.6.0)*
- **User authentication system** - Discord OAuth, sessions, admin roles, and login page. *(0.6.0)*
- **Dynamic scraper scheduling** - Adaptive scrape intervals based on change frequency and course volume. *(0.6.0)*
- **Metrics dashboard** - Scraper and service metrics surfaced on the web dashboard. *(0.6.0)*
- **Subject/major search filter** - Multi-select subject filtering with searchable comboboxes. *(0.5.0)*
- **Web course search UI** - Browser-based course search with interactive data table, sorting, pagination, and column controls. *(0.4.0)*
- **RateMyProfessor integration** - Bulk professor sync via GraphQL with inline ratings in search results. *(0.4.0)*
- **Test coverage expansion** - Unit tests for course formatting, API client, query builder, CLI args, and config parsing. *(0.3.4--0.4.0)*
