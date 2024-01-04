# banner

A discord bot for executing queries & searches on the Ellucian Banner instance hosting all of UTSA's class data.

All data is publicly available and 

## Features

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