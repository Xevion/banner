# Architecture

## System Overview

The Banner project is built as a multi-service application with the following components:

- **Discord Bot Service**: Handles Discord interactions and commands (Serenity/Poise)
- **Web Service**: Axum HTTP server serving the SvelteKit frontend and REST API endpoints
- **Scraper Service**: Background data collection and synchronization with job queue
- **Database Layer**: PostgreSQL 17 for persistent storage (SQLx with compile-time verification)
- **RateMyProfessors Client**: GraphQL-based bulk sync of professor ratings

### Frontend Stack

- **SvelteKit** with Svelte 5 runes (`$state`, `$derived`, `$effect`)
- **Tailwind CSS v4** via `@tailwindcss/vite`
- **bits-ui** for headless UI primitives (comboboxes, tooltips, dropdowns)
- **TanStack Table** for interactive data tables with sorting and column control
- **OverlayScrollbars** for styled, theme-aware scrollable areas
- **ts-rs** generates TypeScript type bindings from Rust structs

### API Endpoints

| Endpoint | Description |
|---|---|
| `GET /api/health` | Health check |
| `GET /api/status` | Service status, version, and commit hash |
| `GET /api/metrics` | Basic metrics |
| `GET /api/courses/search` | Paginated course search with filters (term, subject, query, open-only, sort) |
| `GET /api/courses/:term/:crn` | Single course detail with instructors and RMP ratings |
| `GET /api/terms` | Available terms from reference cache |
| `GET /api/subjects?term=` | Subjects for a term, ordered by enrollment |
| `GET /api/reference/:category` | Reference data lookups (campuses, instructional methods, etc.) |

## Technical Analysis

### Banner System Integration

Some of the features and architecture of Ellucian's Banner system are not clear.
The following features, JSON, and more require validation & analysis:

- Struct Nullability
  - Much of the responses provided by Ellucian contain nulls, and most of them are uncertain as to when and why they're null.
  - Analysis must be conducted to be sure of when to use a string and when it should nillable (pointer).
- Multiple Professors / Primary Indicator
- Multiple Meeting Times
- Meeting Schedule Types
  - AFF vs AIN vs AHB etc.
- Do CRNs repeat between years?
- Check whether partOfTerm is always filled in, and it's meaning for various class results.
- Check which API calls are affected by change in term/sessionID term select
- SessionIDs
  - How long does a session ID work?
  - Do I really require a separate one per term?
  - How many can I activate, are there any restrictions?
  - How should session IDs be checked as 'invalid'?
  - What action(s) keep a session ID 'active', if any?
- Are there any courses with multiple meeting times?
- Google Calendar link generation, as an alternative to ICS file generation

## Change Identification

- Important attributes of a class will be parsed on both the old and new data.
- These attributes will be compared and given identifiers that can be subscribed to.
- When a user subscribes to one of these identifiers, any changes identified will be sent to the user.

## Real-time Suggestions

Various commands arguments have the ability to have suggestions appear.

- They must be fast. As ephemeral suggestions that are only relevant for seconds or less, they need to be delivered in less than a second.
- They need to be easy to acquire. With as many commands & arguments to search as I do, it is paramount that the API be easy to understand & use.
- It cannot be complicated. I only have so much time to develop this.
- It does not need to be persistent. Since the data is scraped and rolled periodically from the Banner system, the data used will be deleted and re-requested occasionally.

For these reasons, I believe PostgreSQL to be the ideal place for this data to be stored.
It is exceptionally fast, works well in-memory, and is less complicated compared to most other solutions.

- Only required data about the class will be stored, along with the JSON-encoded string.
  - For now, this would only be the CRN (and possibly the Term).
  - Potentially, a binary encoding could be used for performance, but it is unlikely to be better.
- Database dumping into R2 would be good to ensure that over-scraping of the Banner system does not occur.
  - Upon a safe close requested
    - Must be done quickly (<8 seconds)
  - Every 30 minutes, if any scraping ocurred.
    - May cause locking of commands.

## Scraping System

In order to keep the in-memory database of the bot up-to-date with the Banner system, the API must be scraped.
Scraping will be separated by major to allow for priority majors (namely, Computer Science) to be scraped more often compared to others.
This will lower the overall load on the Banner system while ensuring that data presented by the app is still relevant.

For now, all majors will be scraped fully every 4 hours with at least 5 minutes between each one.

- On startup, priority majors will be scraped first (if required).
- Other majors will be scraped in arbitrary order (if required).
- Scrape timing will be stored in database.
- CRNs will be the Primary Key within database
  - If CRNs are duplicated between terms, then the primary key will be (CRN, Term)

Considerations

- Change in metadata should decrease the interval
- The number of courses scraped should change the interval (2 hours per 500 courses involved)

## Rate Limiting, Costs & Bursting

Ideally, this application would implement dynamic rate limiting to ensure overload on the server does not occur.
Better, it would also ensure that priority requests (commands) are dispatched faster than background processes (scraping), while making sure different requests are weighted differently.
For example, a recent scrape of 350 classes should be weighted 5x more than a search for 8 classes by a user.
Still, even if the cap does not normally allow for this request to be processed immediately, the small user search should proceed with a small bursting cap.

The requirements to this hypothetical system would be:

- Conditional Bursting: background processes or other requests deemed "low priority" are not allowed to use bursting.
- Arbitrary Costs: rate limiting is considered in the form of the request size/speed more or less, such that small simple requests can be made more frequently, unlike large requests.
