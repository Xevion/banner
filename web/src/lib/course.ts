import type { CourseResponse, DbMeetingTime, InstructorResponse } from "$lib/api";

/** Convert "0900" to "9:00 AM" */
export function formatTime(time: string | null): string {
  if (!time || time.length !== 4) return "TBA";
  const hours = parseInt(time.slice(0, 2), 10);
  const minutes = time.slice(2);
  const period = hours >= 12 ? "PM" : "AM";
  const display = hours > 12 ? hours - 12 : hours === 0 ? 12 : hours;
  return `${display}:${minutes} ${period}`;
}

/**
 * Compact day abbreviation for table cells.
 *
 * Single day → 3-letter: "Mon", "Thu"
 * Multi-day  → concatenated codes: "MWF", "TTh", "MTWTh", "TSa"
 *
 * Codes use single letters where unambiguous (M/T/W/F) and
 * two letters where needed (Th/Sa/Su).
 */
export function formatMeetingDays(mt: DbMeetingTime): string {
  const dayDefs: [boolean, string, string][] = [
    [mt.monday, "M", "Mon"],
    [mt.tuesday, "T", "Tue"],
    [mt.wednesday, "W", "Wed"],
    [mt.thursday, "Th", "Thu"],
    [mt.friday, "F", "Fri"],
    [mt.saturday, "Sa", "Sat"],
    [mt.sunday, "Su", "Sun"],
  ];
  const active = dayDefs.filter(([a]) => a);
  if (active.length === 0) return "";
  if (active.length === 1) return active[0][2];
  return active.map(([, code]) => code).join("");
}

/** Longer day names for detail view: single day → "Thursdays", multiple → "Mon, Wed, Fri" */
export function formatMeetingDaysLong(mt: DbMeetingTime): string {
  const days: [boolean, string, string][] = [
    [mt.monday, "Mon", "Mondays"],
    [mt.tuesday, "Tue", "Tuesdays"],
    [mt.wednesday, "Wed", "Wednesdays"],
    [mt.thursday, "Thur", "Thursdays"],
    [mt.friday, "Fri", "Fridays"],
    [mt.saturday, "Sat", "Saturdays"],
    [mt.sunday, "Sun", "Sundays"],
  ];
  const active = days.filter(([a]) => a);
  if (active.length === 0) return "";
  if (active.length === 1) return active[0][2];
  return active.map(([, short]) => short).join(", ");
}

/**
 * Format a time range with smart AM/PM elision.
 *
 * Same period:  "9:00–9:50 AM"
 * Cross-period: "11:30 AM–12:20 PM"
 * Missing:      "TBA"
 */
export function formatTimeRange(begin: string | null, end: string | null): string {
  if (!begin || begin.length !== 4 || !end || end.length !== 4) return "TBA";

  const bHours = parseInt(begin.slice(0, 2), 10);
  const eHours = parseInt(end.slice(0, 2), 10);
  const bPeriod = bHours >= 12 ? "PM" : "AM";
  const ePeriod = eHours >= 12 ? "PM" : "AM";

  const bDisplay = bHours > 12 ? bHours - 12 : bHours === 0 ? 12 : bHours;
  const eDisplay = eHours > 12 ? eHours - 12 : eHours === 0 ? 12 : eHours;

  const endStr = `${eDisplay}:${end.slice(2)} ${ePeriod}`;
  if (bPeriod === ePeriod) {
    return `${bDisplay}:${begin.slice(2)}–${endStr}`;
  }
  return `${bDisplay}:${begin.slice(2)} ${bPeriod}–${endStr}`;
}

/** Condensed meeting time: "MWF 9:00–9:50 AM" */
export function formatMeetingTime(mt: DbMeetingTime): string {
  const days = formatMeetingDays(mt);
  if (!days) return "TBA";
  const range = formatTimeRange(mt.begin_time, mt.end_time);
  if (range === "TBA") return `${days} TBA`;
  return `${days} ${range}`;
}

/**
 * Progressively abbreviate an instructor name to fit within a character budget.
 *
 * Tries each level until the result fits `maxLen`:
 *   1. Full name: "Ramirez, Maria Elena"
 *   2. Abbreviate trailing given names: "Ramirez, Maria E."
 *   3. Abbreviate all given names: "Ramirez, M. E."
 *   4. First initial only: "Ramirez, M."
 *
 * Names without a comma (e.g. "Staff") are returned as-is.
 */
export function abbreviateInstructor(name: string, maxLen: number = 18): string {
  if (name.length <= maxLen) return name;

  const commaIdx = name.indexOf(", ");
  if (commaIdx === -1) return name;

  const last = name.slice(0, commaIdx);
  const parts = name.slice(commaIdx + 2).split(" ");

  // Level 2: abbreviate trailing given names, keep first given name intact
  // "Maria Elena" → "Maria E."
  if (parts.length > 1) {
    const abbreviated = [parts[0], ...parts.slice(1).map((p) => `${p[0]}.`)].join(" ");
    const result = `${last}, ${abbreviated}`;
    if (result.length <= maxLen) return result;
  }

  // Level 3: abbreviate all given names
  // "Maria Elena" → "M. E."
  if (parts.length > 1) {
    const allInitials = parts.map((p) => `${p[0]}.`).join(" ");
    const result = `${last}, ${allInitials}`;
    if (result.length <= maxLen) return result;
  }

  // Level 4: first initial only
  // "Maria Elena" → "M."  or  "John" → "J."
  return `${last}, ${parts[0][0]}.`;
}

/** Get primary instructor from a course, or first instructor */
export function getPrimaryInstructor(
  instructors: InstructorResponse[]
): InstructorResponse | undefined {
  return instructors.find((i) => i.isPrimary) ?? instructors[0];
}

/** Check if a meeting time has no scheduled days */
export function isMeetingTimeTBA(mt: DbMeetingTime): boolean {
  return (
    !mt.monday &&
    !mt.tuesday &&
    !mt.wednesday &&
    !mt.thursday &&
    !mt.friday &&
    !mt.saturday &&
    !mt.sunday
  );
}

/** Check if a meeting time has no begin/end times */
export function isTimeTBA(mt: DbMeetingTime): boolean {
  return !mt.begin_time || mt.begin_time.length !== 4;
}

/** Format a date string to "January 20, 2026". Accepts YYYY-MM-DD or MM/DD/YYYY. */
export function formatDate(dateStr: string): string {
  let year: number, month: number, day: number;
  if (dateStr.includes("-")) {
    [year, month, day] = dateStr.split("-").map(Number);
  } else if (dateStr.includes("/")) {
    [month, day, year] = dateStr.split("/").map(Number);
  } else {
    return dateStr;
  }
  if (!year || !month || !day) return dateStr;
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" });
}

/** Short location string from first meeting time: "MH 2.206" or campus fallback */
export function formatLocation(course: CourseResponse): string | null {
  for (const mt of course.meetingTimes) {
    if (mt.building && mt.room) return `${mt.building} ${mt.room}`;
    if (mt.building) return mt.building;
  }
  return course.campus ?? null;
}

/** Longer location string using building description: "Main Hall 2.206" */
export function formatLocationLong(mt: DbMeetingTime): string | null {
  const name = mt.building_description ?? mt.building;
  if (!name) return null;
  return mt.room ? `${name} ${mt.room}` : name;
}

/** Format a date as "Aug 26, 2024". Accepts YYYY-MM-DD or MM/DD/YYYY. */
export function formatDateShort(dateStr: string): string {
  let year: number, month: number, day: number;
  if (dateStr.includes("-")) {
    [year, month, day] = dateStr.split("-").map(Number);
  } else if (dateStr.includes("/")) {
    [month, day, year] = dateStr.split("/").map(Number);
  } else {
    return dateStr;
  }
  if (!year || !month || !day) return dateStr;
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString("en-US", { year: "numeric", month: "short", day: "numeric" });
}

/**
 * Verbose day names for tooltips: "Tuesdays & Thursdays", "Mondays, Wednesdays & Fridays".
 * Single day → plural: "Thursdays".
 */
export function formatMeetingDaysVerbose(mt: DbMeetingTime): string {
  const dayDefs: [boolean, string][] = [
    [mt.monday, "Mondays"],
    [mt.tuesday, "Tuesdays"],
    [mt.wednesday, "Wednesdays"],
    [mt.thursday, "Thursdays"],
    [mt.friday, "Fridays"],
    [mt.saturday, "Saturdays"],
    [mt.sunday, "Sundays"],
  ];
  const active = dayDefs.filter(([a]) => a).map(([, name]) => name);
  if (active.length === 0) return "";
  if (active.length === 1) return active[0];
  return active.slice(0, -1).join(", ") + " & " + active[active.length - 1];
}

/**
 * Full verbose tooltip for a single meeting time:
 * "Tuesdays & Thursdays, 4:15–5:30 PM\nMain Hall 2.206 · Aug 26 – Dec 12, 2024"
 */
export function formatMeetingTimeTooltip(mt: DbMeetingTime): string {
  const days = formatMeetingDaysVerbose(mt);
  const range = formatTimeRange(mt.begin_time, mt.end_time);
  let line1: string;
  if (!days && range === "TBA") {
    line1 = "TBA";
  } else if (!days) {
    line1 = range;
  } else if (range === "TBA") {
    line1 = `${days}, TBA`;
  } else {
    line1 = `${days}, ${range}`;
  }

  const parts = [line1];

  const loc = formatLocationLong(mt);
  const dateRange =
    mt.start_date && mt.end_date
      ? `${formatDateShort(mt.start_date)} – ${formatDateShort(mt.end_date)}`
      : null;

  if (loc && dateRange) {
    parts.push(`${loc}, ${dateRange}`);
  } else if (loc) {
    parts.push(loc);
  } else if (dateRange) {
    parts.push(dateRange);
  }

  return parts.join("\n");
}

/** Full verbose tooltip for all meeting times on a course, newline-separated. */
export function formatMeetingTimesTooltip(meetingTimes: DbMeetingTime[]): string {
  if (meetingTimes.length === 0) return "TBA";
  return meetingTimes.map(formatMeetingTimeTooltip).join("\n\n");
}

/**
 * Delivery concern category for visual accent on location cells.
 * - "online": fully online with no physical location (OA, OS, OH without INT building)
 * - "internet": internet campus with INT building code
 * - "hybrid": mix of online and in-person (HB, H1, H2)
 * - "off-campus": in-person but not on Main Campus
 * - null: normal in-person on main campus (no accent)
 */
export type DeliveryConcern = "online" | "internet" | "hybrid" | "off-campus" | null;

const ONLINE_METHODS = new Set(["OA", "OS", "OH"]);
const HYBRID_METHODS = new Set(["HB", "H1", "H2"]);
const MAIN_CAMPUS = "11";
const ONLINE_CAMPUSES = new Set(["9", "ONL"]);

export function getDeliveryConcern(course: CourseResponse): DeliveryConcern {
  const method = course.instructionalMethod;
  if (method && ONLINE_METHODS.has(method)) {
    const hasIntBuilding = course.meetingTimes.some((mt: DbMeetingTime) => mt.building === "INT");
    return hasIntBuilding ? "internet" : "online";
  }
  if (method && HYBRID_METHODS.has(method)) return "hybrid";
  if (course.campus && course.campus !== MAIN_CAMPUS && !ONLINE_CAMPUSES.has(course.campus)) {
    return "off-campus";
  }
  return null;
}

/** Border accent color for each delivery concern type. */
export function concernAccentColor(concern: DeliveryConcern): string | null {
  switch (concern) {
    case "online":
      return "#3b82f6"; // blue-500
    case "internet":
      return "#06b6d4"; // cyan-500
    case "hybrid":
      return "#a855f7"; // purple-500
    case "off-campus":
      return "#f59e0b"; // amber-500
    default:
      return null;
  }
}

/**
 * Location display text for the table cell.
 * Falls back to "Online" for online courses instead of showing a dash.
 */
export function formatLocationDisplay(course: CourseResponse): string | null {
  const loc = formatLocation(course);
  if (loc) return loc;
  const concern = getDeliveryConcern(course);
  if (concern === "online") return "Online";
  return null;
}

/** Tooltip text for the location column: long-form location + delivery note */
export function formatLocationTooltip(course: CourseResponse): string | null {
  const parts: string[] = [];

  for (const mt of course.meetingTimes) {
    const loc = formatLocationLong(mt);
    if (loc && !parts.includes(loc)) parts.push(loc);
  }

  const locationLine = parts.length > 0 ? parts.join(", ") : null;

  const concern = getDeliveryConcern(course);
  let deliveryNote: string | null = null;
  if (concern === "online") deliveryNote = "Online";
  else if (concern === "internet") deliveryNote = "Internet";
  else if (concern === "hybrid") deliveryNote = "Hybrid";
  else if (concern === "off-campus") deliveryNote = "Off-campus";

  if (locationLine && deliveryNote) return `${locationLine}\n${deliveryNote}`;
  if (locationLine) return locationLine;
  if (deliveryNote) return deliveryNote;
  return null;
}

/** Number of open seats in a course section */
export function openSeats(course: CourseResponse): number {
  return Math.max(0, course.maxEnrollment - course.enrollment);
}

/** Text color class for seat availability: red (full), yellow (low), green (open) */
export function seatsColor(course: CourseResponse): string {
  const open = openSeats(course);
  if (open === 0) return "text-status-red";
  if (open <= 5) return "text-yellow-500";
  return "text-status-green";
}

/** Background dot color class for seat availability */
export function seatsDotColor(course: CourseResponse): string {
  const open = openSeats(course);
  if (open === 0) return "bg-red-500";
  if (open <= 5) return "bg-yellow-500";
  return "bg-green-500";
}

/** Minimum number of ratings needed to consider RMP data reliable */
export const RMP_CONFIDENCE_THRESHOLD = 7;

/** RMP professor page URL from legacy ID */
export function rmpUrl(legacyId: number): string {
  return `https://www.ratemyprofessors.com/professor/${legacyId}`;
}

/**
 * Smooth OKLCH color + text-shadow for a RateMyProfessors rating.
 *
 * Three-stop gradient interpolated in OKLCH:
 *   1.0 → red, 3.0 → amber, 5.0 → green
 * with separate light/dark mode tuning.
 */
export function ratingStyle(rating: number, isDark: boolean): string {
  const clamped = Math.max(1, Math.min(5, rating));

  // OKLCH stops: [lightness, chroma, hue]
  const stops: { light: [number, number, number]; dark: [number, number, number] }[] = [
    { light: [0.63, 0.2, 25], dark: [0.7, 0.19, 25] }, // 1.0 – red
    { light: [0.7, 0.16, 85], dark: [0.78, 0.15, 85] }, // 3.0 – amber
    { light: [0.65, 0.2, 145], dark: [0.72, 0.19, 145] }, // 5.0 – green
  ];

  let t: number;
  let fromIdx: number;
  if (clamped <= 3) {
    t = (clamped - 1) / 2;
    fromIdx = 0;
  } else {
    t = (clamped - 3) / 2;
    fromIdx = 1;
  }

  const from = isDark ? stops[fromIdx].dark : stops[fromIdx].light;
  const to = isDark ? stops[fromIdx + 1].dark : stops[fromIdx + 1].light;

  const l = from[0] + (to[0] - from[0]) * t;
  const c = from[1] + (to[1] - from[1]) * t;
  const h = from[2] + (to[2] - from[2]) * t;

  return `color: oklch(${l.toFixed(3)} ${c.toFixed(3)} ${h.toFixed(1)}); text-shadow: 0 0 4px oklch(${l.toFixed(3)} ${c.toFixed(3)} ${h.toFixed(1)} / 0.3);`;
}

/** Format credit hours display */
export function formatCreditHours(course: CourseResponse): string {
  if (course.creditHours != null) return String(course.creditHours);
  if (course.creditHourLow != null && course.creditHourHigh != null) {
    return `${course.creditHourLow}–${course.creditHourHigh}`;
  }
  return "—";
}

/**
 * Convert Banner "Last, First Middle" → "First Middle Last".
 * Handles: no comma (returned as-is), trailing/leading spaces,
 * middle names/initials preserved.
 */
export function formatInstructorName(displayName: string): string {
  const commaIdx = displayName.indexOf(",");
  if (commaIdx === -1) return displayName.trim();

  const last = displayName.slice(0, commaIdx).trim();
  const rest = displayName.slice(commaIdx + 1).trim();
  if (!rest) return last;

  return `${rest} ${last}`;
}

/** Check if a rating value represents real data (not the 0.0 placeholder for unrated professors). */
export function isRatingValid(avgRating: number | null, numRatings: number): boolean {
  return avgRating !== null && !(avgRating === 0 && numRatings === 0);
}
