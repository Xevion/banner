/**
 * Convert between Banner's internal term codes (e.g., "202620") and human-friendly format (e.g., "summer-26")
 */

export type SemesterName = "spring" | "summer" | "fall";

const SEMESTER_CODES: Record<string, SemesterName> = {
  "10": "spring",
  "20": "summer",
  "30": "fall",
};

const SEMESTER_TO_CODE: Record<SemesterName, string> = {
  spring: "10",
  summer: "20",
  fall: "30",
};

/**
 * Convert Banner term code (e.g., "202620") to friendly format (e.g., "summer-26")
 */
export function termToFriendly(bannerCode: string): string | null {
  if (bannerCode.length !== 6) return null;

  const year = bannerCode.substring(0, 4);
  const semesterCode = bannerCode.substring(4, 6);
  const semester = SEMESTER_CODES[semesterCode];

  if (!semester) return null;

  const shortYear = year.substring(2, 4);
  return `${semester}-${shortYear}`;
}

/**
 * Convert friendly format (e.g., "summer-26") to Banner term code (e.g., "202620")
 */
export function termToBanner(friendly: string): string | null {
  const match = friendly.match(/^(spring|summer|fall)-(\d{2})$/);
  if (!match) return null;

  const [, semester, shortYear] = match;
  const semesterCode = SEMESTER_TO_CODE[semester as SemesterName];
  if (!semesterCode) return null;

  const fullYear = `20${shortYear}`;
  return `${fullYear}${semesterCode}`;
}
