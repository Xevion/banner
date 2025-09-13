# Documentation

This folder contains detailed documentation for the Banner project. This file acts as the index.

## Files

- [`FEATURES.md`](FEATURES.md) - Current features, implemented functionality, and future roadmap
- [`BANNER.md`](BANNER.md) - General API documentation on the Banner system
- [`ARCHITECTURE.md`](ARCHITECTURE.md) - Technical implementation details, system design, and analysis

## Samples

The `samples/` folder contains real Banner API response examples:

- `search/` - Course search API responses with various filters
  - [`searchResults.json`](samples/search/searchResults.json)
  - [`searchResults_500.json`](samples/search/searchResults_500.json)
  - [`searchResults_CS500.json`](samples/search/searchResults_CS500.json)
  - [`searchResults_malware.json`](samples/search/searchResults_malware.json)
- `meta/` - Metadata API responses (terms, subjects, instructors, etc.)
  - [`get_attribute.json`](samples/meta/get_attribute.json)
  - [`get_campus.json`](samples/meta/get_campus.json)
  - [`get_instructionalMethod.json`](samples/meta/get_instructionalMethod.json)
  - [`get_instructor.json`](samples/meta/get_instructor.json)
  - [`get_partOfTerm.json`](samples/meta/get_partOfTerm.json)
  - [`get_subject.json`](samples/meta/get_subject.json)
  - [`getTerms.json`](samples/meta/getTerms.json)
- `course/` - Course detail API responses (HTML and JSON)
  - [`getFacultyMeetingTimes.json`](samples/course/getFacultyMeetingTimes.json)
  - [`getClassDetails.html`](samples/course/getClassDetails.html)
  - [`getCorequisites.html`](samples/course/getCorequisites.html)
  - [`getCourseDescription.html`](samples/course/getCourseDescription.html)
  - [`getEnrollmentInfo.html`](samples/course/getEnrollmentInfo.html)
  - [`getFees.html`](samples/course/getFees.html)
  - [`getLinkedSections.html`](samples/course/getLinkedSections.html)
  - [`getRestrictions.html`](samples/course/getRestrictions.html)
  - [`getSectionAttributes.html`](samples/course/getSectionAttributes.html)
  - [`getSectionBookstoreDetails.html`](samples/course/getSectionBookstoreDetails.html)
  - [`getSectionPrerequisites.html`](samples/course/getSectionPrerequisites.html)
  - [`getXlistSections.html`](samples/course/getXlistSections.html)

These samples are used for development, testing, and understanding the Banner API structure.
