# Roadmap

## Now

- **Notification and subscription system** - Subscribe to courses and get alerts on seat availability, waitlist movement, and detail changes (time, location, professor, seats). DB schema exists.
- **Professor name search filter** - Filter search results by instructor. Backend code exists but is commented out.
- **Autocomplete for search fields** - Typeahead for course titles, course numbers, professors, and terms.
- **Test coverage expansion** - Broaden coverage with session/rate-limiter tests and more DB integration tests.

## Soon

- **Smart time-of-day search parsing** - Support natural queries like "2 PM", "2-3 PM", "ends by 2 PM", "after 2 PM", "before 2 PM" mapped to time ranges.
- **Section-based lookup** - Search by full section identifier, e.g. "CS 4393 001".
- **Search result pagination** - Paginated embeds for large result sets in Discord.
- **Multi-term querying** - Query across multiple terms in a single search instead of one at a time.
- **Historical analytics** - Track seat availability over time and visualize fill-rate trends per course or professor.
- **Schedule builder** - Visual weekly schedule tool for assembling a conflict-free course lineup.
- **Professor stats** - Aggregate data views: average class size, typical waitlist length, schedule patterns across semesters.

## Eventually

- **Degree audit helper** - Map available courses to degree requirements and suggest what to take next.
- **Dynamic scraper scheduling** - Adjust scrape intervals based on change frequency and course count (e.g. 2 hours per 500 courses, shorter intervals when changes are detected).
- **DM support** - Allow the Discord bot to respond in direct messages, not just guild channels.
- **"Classes Now" command** - Find classes currently in session based on the current day and time.
- **CRN direct lookup** - Look up a course by its CRN without going through search.
- **Metrics dashboard** - Surface scraper and service metrics visually on the web dashboard.
- **Privileged error feedback** - Detailed error information surfaced to bot admins when commands fail.

## Done

- **Web course search UI** - Browser-based course search with interactive data table, sorting, pagination, and column controls. *(0.4.0)*
- **RateMyProfessor integration** - Bulk professor sync via GraphQL with inline ratings in search results. *(0.4.0)*
- **Subject/major search filter** - Multi-select subject filtering with searchable comboboxes. *(0.5.0)*
- **Test coverage expansion** - Unit tests for course formatting, API client, query builder, CLI args, and config parsing. *(0.3.4–0.4.0)*
