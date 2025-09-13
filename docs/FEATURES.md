# Features

## Current Features

### Discord Bot Commands

- **search** - Search for courses with various filters (title, course code, keywords)
- **terms** - List available terms or search for a specific term
- **time** - Get meeting times for a specific course (CRN)
- **ics** - Generate ICS calendar file for a course with holiday exclusions
- **gcal** - Generate Google Calendar link for a course

### Data Pipeline

- Intelligent scraping system with priority queues
- Rate limiting and burst handling
- Background data synchronization

## Feature Wishlist

### Commands

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
