# Banner

All notes on the internal workings of the Banner system by Ellucian.

## Sessions

All notes on the internal workings of Sessions in the Banner system.

- Sessions are generated on demand with a random string of characters.
  - The format `{5 random characters}{milliseconds since epoch}`
  - Example: ``
- Sessions are invalidated after 30 minutes, but may change.
  - This delay can be found in the original HTML returned, find `meta[name="maxInactiveInterval"]` and read the `content` attribute.
  - This is read at runtime (in the browser, by javascript) on initialization.
- Multiple timers exist, one is for the Inactivity Timer.
  - A dialog will appear asking the user to continue their session.
  - If they click the button, the session will be extended via the keepAliveURL (see `meta[name="keepAliveURL"]`).
  - The `keepAliveURL` does not seem to care whether the session is or was ever valid, it will always return a 200 OK with `I am Alive` as the content.
- When searching with an invalid session (or none at all, as the case may be), the server will return 200 OK, but with an empty result response structure.

```json
{
  "success": true,
  "totalCount": 0,
  "data": null, // always an array, even if empty
  "pageOffset": 0, //
  "pageMaxSize": 10,
  "sectionsFetchedCount": 0,
  "pathMode": "registration", // normally "search"
  "searchResultsConfigs": null, // normally an array
  "ztcEncodedImage": null // normally a static string in base64
}
```

- This is only the handling for the search endpoint, more research is required to see how other endpoints handle invalid/expired sessions.
- TODO: How is `pathMode` affected by an expired session, rather than an invalid/non-existent one?
