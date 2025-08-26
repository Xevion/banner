# banner

A discord bot for executing queries & searches on the Ellucian Banner instance hosting all of UTSA's class data.

## Feature Wishlist

- Commands
  - ICS Download (get a ICS download of your classes with location & timing perfectly - set for every class you're in)
  - Classes Now (find classes happening)
- Autocomplete
  - Class Title
  - Course Number
  - Term/Part of Term
  - Professor
  - Attribute
- Component Pagination
- RateMyProfessor Integration (Linked/Embedded)
- Smart term selection (i.e. Summer 2024 will be selected automatically when opened)
- Rate Limiting (bursting with global/user limits)
- DMs Integration (allow usage of the bot in DMs)
- Class Change Notifications (get notified when details about a class change)
- Multi-term Querying (currently the backend for searching is kinda weird)
- Full Autocomplete for Every Search Option
- Metrics, Log Query, Privileged Error Feedback
- Search for Classes
  - Major, Professor, Location, Name, Time of Day
- Subscribe to Classes
  - Availability (seat, pre-seat)
  - Waitlist Movement
  - Detail Changes (meta, time, location, seats, professor)
    - `time` Start, End, Days of Week
    - `seats` Any change in seat/waitlist data
    - `meta`
- Lookup via Course Reference Number (CRN)
- Smart Time of Day Handling
  - "2 PM" -> Start within 2:00 PM to 2:59 PM
  - "2-3 PM" -> Start within 2:00 PM to 3:59 PM
  - "ends by 2 PM" -> Ends within 12:00 AM to 2:00 PM
  - "after 2 PM" -> Start within 2:01 PM to 11:59 PM
  - "before 2 PM" -> Ends within 12:00 AM to 1:59 PM
- Get By Section Command
  - CS 4393 001 =>
  - Will require SQL to be able to search for a class by its section number

## Analysis Required

Some of the features and architecture of Ellucian's Banner system are not clear.
The follow features, JSON, and more require validation & analysis:

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

For these reasons, I believe SQLite to be the ideal place for this data to be stored.
It is exceptionally fast, works well in-memory, and is less complicated compared to most other solutions.

- Only required data about the class will be stored, along with the JSON-encoded string.
  - For now, this would only be the CRN (and possibly the Term).
  - Potentially, a binary encoding could be used for performance, but it is unlikely to be better.
- Database dumping into R2 would be good to ensure that over-scraping of the Banner system does not occur.
  - Upon a safe close requested
    - Must be done quickly (<8 seconds)
  - Every 30 minutes, if any scraping ocurred.
    - May cause locking of commands.

## Scraping

In order to keep the in-memory database of the bot up-to-date with the Banner system, the API must be scraped.
Scraping will be separated by major to allow for priority majors (namely, Computer Science) to be scraped more often compared to others.
This will lower the overall load on the Banner system while ensuring that data presented by the app is still relevant.

For now, all majors will be scraped fully every 4 hours with at least 5 minutes between each one.

- On startup, priority majors will be scraped first (if required).
- Other majors will be scraped in arbitrary order (if required).
- Scrape timing will be stored in Redis.
- CRNs will be the Primary Key within SQLite
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
