import type {
  CourseResponse,
  DayOfWeek,
  DbMeetingTime,
  DeliveryMode,
  InstructorResponse,
} from "$lib/api";

/** Convert ISO time string "08:30:00" to "8:30 AM" */
export function formatTime(time: string | null): string {
  if (!time) return "TBA";
  // ISO format: "HH:MM:SS"
  const parts = time.split(":");
  if (parts.length < 2) return "TBA";
  const hours = parseInt(parts[0], 10);
  const minutes = parts[1];
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
const DAY_SHORT: Record<DayOfWeek, [string, string]> = {
  monday: ["M", "Mon"],
  tuesday: ["T", "Tue"],
  wednesday: ["W", "Wed"],
  thursday: ["Th", "Thu"],
  friday: ["F", "Fri"],
  saturday: ["Sa", "Sat"],
  sunday: ["Su", "Sun"],
};

export function formatMeetingDays(mt: DbMeetingTime): string {
  const days = mt.days;
  if (days.length === 0) return "";
  if (days.length === 1) return DAY_SHORT[days[0]][1];
  return days.map((d) => DAY_SHORT[d][0]).join("");
}

/** Longer day names for detail view: single day → "Thursdays", multiple → "Mon, Wed, Fri" */
const DAY_LONG: Record<DayOfWeek, [string, string]> = {
  monday: ["Mon", "Mondays"],
  tuesday: ["Tue", "Tuesdays"],
  wednesday: ["Wed", "Wednesdays"],
  thursday: ["Thur", "Thursdays"],
  friday: ["Fri", "Fridays"],
  saturday: ["Sat", "Saturdays"],
  sunday: ["Sun", "Sundays"],
};

export function formatMeetingDaysLong(mt: DbMeetingTime): string {
  const days = mt.days;
  if (days.length === 0) return "";
  if (days.length === 1) return DAY_LONG[days[0]][1];
  return days.map((d) => DAY_LONG[d][0]).join(", ");
}

/**
 * Format a time range with smart AM/PM elision.
 *
 * Same period:  "9:00–9:50 AM"
 * Cross-period: "11:30 AM–12:20 PM"
 * Missing:      "TBA"
 */
export function formatTimeRange(begin: string | null, end: string | null): string {
  if (!begin || !end) return "TBA";

  const bParts = begin.split(":");
  const eParts = end.split(":");
  if (bParts.length < 2 || eParts.length < 2) return "TBA";

  const bHours = parseInt(bParts[0], 10);
  const eHours = parseInt(eParts[0], 10);
  const bPeriod = bHours >= 12 ? "PM" : "AM";
  const ePeriod = eHours >= 12 ? "PM" : "AM";

  const bDisplay = bHours > 12 ? bHours - 12 : bHours === 0 ? 12 : bHours;
  const eDisplay = eHours > 12 ? eHours - 12 : eHours === 0 ? 12 : eHours;

  const endStr = `${eDisplay}:${eParts[1]} ${ePeriod}`;
  if (bPeriod === ePeriod) {
    return `${bDisplay}:${bParts[1]}–${endStr}`;
  }
  return `${bDisplay}:${bParts[1]} ${bPeriod}–${endStr}`;
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
export function abbreviateInstructor(name: string, maxLen = 18): string {
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

/**
 * Get the primary instructor from a course.
 *
 * When `primaryInstructorId` is available (from the backend), does a direct
 * lookup. Falls back to iterating `isPrimary` / first instructor for safety.
 */
export function getPrimaryInstructor(
  instructors: InstructorResponse[],
  primaryInstructorId?: number | null
): InstructorResponse | undefined {
  if (primaryInstructorId != null) {
    return instructors.find((i) => i.instructorId === primaryInstructorId) ?? instructors[0];
  }
  return instructors.find((i) => i.isPrimary) ?? instructors[0];
}

/** Format an ISO-8601 date (YYYY-MM-DD) to "January 20, 2026". */
export function formatDate(dateStr: string): string {
  const [year, month, day] = dateStr.split("-").map(Number);
  if (!year || !month || !day) return dateStr;
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" });
}

/** Longer location string using building description: "Main Hall 2.206" */
function formatLocationLong(mt: DbMeetingTime): string | null {
  const name = mt.location?.buildingDescription ?? mt.location?.building;
  if (!name) return null;
  return mt.location?.room ? `${name} ${mt.location.room}` : name;
}

/** Format an ISO-8601 date (YYYY-MM-DD) as "Aug 26, 2024". */
export function formatDateShort(dateStr: string): string {
  const [year, month, day] = dateStr.split("-").map(Number);
  if (!year || !month || !day) return dateStr;
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString("en-US", { year: "numeric", month: "short", day: "numeric" });
}

/**
 * Verbose day names for tooltips: "Tuesdays & Thursdays", "Mondays, Wednesdays & Fridays".
 * Single day → plural: "Thursdays".
 */
const DAY_VERBOSE: Record<DayOfWeek, string> = {
  monday: "Mondays",
  tuesday: "Tuesdays",
  wednesday: "Wednesdays",
  thursday: "Thursdays",
  friday: "Fridays",
  saturday: "Saturdays",
  sunday: "Sundays",
};

export function formatMeetingDaysVerbose(mt: DbMeetingTime): string {
  const names = mt.days.map((d) => DAY_VERBOSE[d]);
  if (names.length === 0) return "";
  if (names.length === 1) return names[0];
  return names.slice(0, -1).join(", ") + " & " + names[names.length - 1];
}

/**
 * Full verbose tooltip for a single meeting time:
 * "Tuesdays & Thursdays, 4:15–5:30 PM\nMain Hall 2.206 · Aug 26 – Dec 12, 2024"
 */
export function formatMeetingTimeTooltip(mt: DbMeetingTime): string {
  const days = formatMeetingDaysVerbose(mt);
  const range = formatTimeRange(mt.timeRange?.start ?? null, mt.timeRange?.end ?? null);
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
  const dateRange = `${formatDateShort(mt.dateRange.start)} – ${formatDateShort(mt.dateRange.end)}`;

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

/** Border accent color for each delivery mode type. */
export function concernAccentColor(concern: DeliveryMode | null): string | null {
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

/** Tooltip text for the location column: long-form location + delivery note */
export function formatLocationTooltip(course: CourseResponse): string | null {
  const parts: string[] = [];

  for (const mt of course.meetingTimes) {
    const loc = formatLocationLong(mt);
    if (loc && !parts.includes(loc)) parts.push(loc);
  }

  const locationLine = parts.length > 0 ? parts.join(", ") : null;

  const c = course.deliveryMode;
  let deliveryNote: string | null = null;
  if (c === "online") deliveryNote = "Online";
  else if (c === "internet") deliveryNote = "Internet";
  else if (c === "hybrid") deliveryNote = "Hybrid";
  else if (c === "off-campus") deliveryNote = "Off-campus";

  if (locationLine && deliveryNote) return `${locationLine}\n${deliveryNote}`;
  if (locationLine) return locationLine;
  if (deliveryNote) return deliveryNote;
  return null;
}

/** Text color class for seat availability: red (full), yellow (low), green (open) */
export function seatsColor(openSeats: number): string {
  if (openSeats === 0) return "text-status-red";
  if (openSeats <= 5) return "text-yellow-500";
  return "text-status-green";
}

/** Background dot color class for seat availability */
export function seatsDotColor(openSeats: number): string {
  if (openSeats === 0) return "bg-red-500";
  if (openSeats <= 5) return "bg-yellow-500";
  return "bg-green-500";
}

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
  if (course.creditHours == null) return "—";
  if (course.creditHours.type === "fixed") {
    return String(course.creditHours.hours);
  }
  return `${course.creditHours.low}–${course.creditHours.high}`;
}

/**
 * Format an instructor's name for display.
 *
 * When an `InstructorResponse` object with `firstName` and `lastName` is
 * provided, uses them directly: "First Last". Otherwise falls back to parsing
 * `displayName` from Banner's "Last, First Middle" format.
 */
export function formatInstructorName(
  nameOrInstructor: string | Pick<InstructorResponse, "displayName" | "firstName" | "lastName">
): string {
  if (typeof nameOrInstructor !== "string") {
    const { firstName, lastName, displayName } = nameOrInstructor;
    if (firstName && lastName) return `${firstName} ${lastName}`;
    return formatInstructorName(displayName);
  }

  const displayName = nameOrInstructor;
  const commaIdx = displayName.indexOf(",");
  if (commaIdx === -1) return displayName.trim();

  const last = displayName.slice(0, commaIdx).trim();
  const rest = displayName.slice(commaIdx + 1).trim();
  if (!rest) return last;

  return `${rest} ${last}`;
}

/** Compact meeting time summary for mobile cards: "MWF 9:00–9:50 AM", "Async", or "TBA" */
export function formatMeetingTimeSummary(course: CourseResponse): string {
  if (course.isAsyncOnline) return "Async";
  if (course.meetingTimes.length === 0) return "TBA";
  const mt = course.meetingTimes[0];
  if (mt.days.length === 0 && mt.timeRange === null) return "TBA";
  return `${formatMeetingDays(mt)} ${formatTimeRange(mt.timeRange?.start ?? null, mt.timeRange?.end ?? null)}`;
}
