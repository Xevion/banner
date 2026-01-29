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
export function getPrimaryInstructor(instructors: InstructorResponse[]): InstructorResponse | undefined {
  return instructors.find((i) => i.isPrimary) ?? instructors[0];
}

/** Format credit hours display */
export function formatCreditHours(course: CourseResponse): string {
  if (course.creditHours != null) return String(course.creditHours);
  if (course.creditHourLow != null && course.creditHourHigh != null) {
    return `${course.creditHourLow}–${course.creditHourHigh}`;
  }
  return "—";
}
