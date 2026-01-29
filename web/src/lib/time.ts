/**
 * Relative time formatting with adaptive refresh intervals.
 *
 * The key insight: a timestamp showing "3 seconds ago" needs to update every second,
 * but "2 hours ago" only needs to update every minute. This module provides both
 * the formatted string and the optimal interval until the next meaningful change.
 */

interface RelativeTimeResult {
  /** The human-readable relative time string (e.g. "3 seconds ago") */
  text: string;
  /** Milliseconds until the displayed text would change */
  nextUpdateMs: number;
}

/**
 * Compute a relative time string and the interval until it next changes.
 *
 * Granularity tiers:
 * - < 60s: per-second ("1 second ago", "45 seconds ago")
 * - < 60m: per-minute ("1 minute ago", "12 minutes ago")
 * - < 24h: per-hour ("1 hour ago", "5 hours ago")
 * - >= 24h: per-day ("1 day ago", "3 days ago")
 */
export function relativeTime(date: Date, ref: Date): RelativeTimeResult {
  const diffMs = ref.getTime() - date.getTime();
  const seconds = Math.round(diffMs / 1000);

  if (seconds < 1) {
    return { text: "just now", nextUpdateMs: 1000 - (diffMs % 1000) || 1000 };
  }

  if (seconds < 60) {
    const remainder = 1000 - (diffMs % 1000);
    return {
      text: seconds === 1 ? "1 second ago" : `${seconds} seconds ago`,
      nextUpdateMs: remainder || 1000,
    };
  }

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    // Update when the next minute boundary is crossed
    const msIntoCurrentMinute = diffMs % 60_000;
    const msUntilNextMinute = 60_000 - msIntoCurrentMinute;
    return {
      text: minutes === 1 ? "1 minute ago" : `${minutes} minutes ago`,
      nextUpdateMs: msUntilNextMinute || 60_000,
    };
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    const msIntoCurrentHour = diffMs % 3_600_000;
    const msUntilNextHour = 3_600_000 - msIntoCurrentHour;
    return {
      text: hours === 1 ? "1 hour ago" : `${hours} hours ago`,
      nextUpdateMs: msUntilNextHour || 3_600_000,
    };
  }

  const days = Math.floor(hours / 24);
  const msIntoCurrentDay = diffMs % 86_400_000;
  const msUntilNextDay = 86_400_000 - msIntoCurrentDay;
  return {
    text: days === 1 ? "1 day ago" : `${days} days ago`,
    nextUpdateMs: msUntilNextDay || 86_400_000,
  };
}
