/**
 * Utilities for ISO-8601 date string validation and conversion.
 *
 * All DateTime<Utc> fields from Rust are serialized as ISO-8601 strings.
 */

/**
 * Validates if a string is a valid ISO-8601 date string.
 *
 * @param value - The string to validate
 * @returns True if the string is a valid ISO-8601 date
 */
export function isValidISODate(value: string): boolean {
  try {
    const date = new Date(value);
    return !isNaN(date.getTime()) && date.toISOString() === value;
  } catch {
    return false;
  }
}

/**
 * Parses an ISO-8601 date string to a Date object.
 *
 * @param value - The ISO-8601 string to parse
 * @returns Date object, or null if invalid
 */
export function parseISODate(value: string): Date | null {
  try {
    const date = new Date(value);
    if (isNaN(date.getTime())) {
      return null;
    }
    return date;
  } catch {
    return null;
  }
}

/**
 * Asserts that a string is a valid ISO-8601 date, throwing if not.
 *
 * @param value - The string to validate
 * @param fieldName - Name of the field for error messages
 * @throws Error if the string is not a valid ISO-8601 date
 */
export function assertISODate(value: string, fieldName = "date"): void {
  if (!isValidISODate(value)) {
    throw new Error(`Invalid ISO-8601 date for ${fieldName}: ${value}`);
  }
}

/**
 * Converts a Date to an ISO-8601 UTC string.
 *
 * @param date - The Date object to convert
 * @returns ISO-8601 string in UTC (e.g., "2024-01-15T10:30:00Z")
 */
export function toISOString(date: Date): string {
  return date.toISOString();
}
