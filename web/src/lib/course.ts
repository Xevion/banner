import type { DbMeetingTime, CourseResponse, InstructorResponse } from "$lib/api";

/** Convert "0900" to "9:00 AM" */
export function formatTime(time: string | null): string {
  if (!time || time.length !== 4) return "TBA";
  const hours = parseInt(time.slice(0, 2), 10);
  const minutes = time.slice(2);
  const period = hours >= 12 ? "PM" : "AM";
  const display = hours > 12 ? hours - 12 : hours === 0 ? 12 : hours;
  return `${display}:${minutes} ${period}`;
}

/** Get day abbreviation string like "MWF" from a meeting time */
export function formatMeetingDays(mt: DbMeetingTime): string {
  const days: [boolean, string][] = [
    [mt.monday, "M"],
    [mt.tuesday, "T"],
    [mt.wednesday, "W"],
    [mt.thursday, "R"],
    [mt.friday, "F"],
    [mt.saturday, "S"],
    [mt.sunday, "U"],
  ];
  return days
    .filter(([active]) => active)
    .map(([, abbr]) => abbr)
    .join("");
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

/** Condensed meeting time: "MWF 9:00 AM–9:50 AM" */
export function formatMeetingTime(mt: DbMeetingTime): string {
  const days = formatMeetingDays(mt);
  if (!days) return "TBA";
  const begin = formatTime(mt.begin_time);
  const end = formatTime(mt.end_time);
  if (begin === "TBA") return `${days} TBA`;
  return `${days} ${begin}–${end}`;
}

/** Abbreviate instructor name: "Heaps, John" → "Heaps, J." */
export function abbreviateInstructor(name: string): string {
  const commaIdx = name.indexOf(", ");
  if (commaIdx === -1) return name;
  const last = name.slice(0, commaIdx);
  const first = name.slice(commaIdx + 2);
  return `${last}, ${first.charAt(0)}.`;
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

/** Format credit hours display */
export function formatCreditHours(course: CourseResponse): string {
  if (course.creditHours != null) return String(course.creditHours);
  if (course.creditHourLow != null && course.creditHourHigh != null) {
    return `${course.creditHourLow}–${course.creditHourHigh}`;
  }
  return "—";
}
