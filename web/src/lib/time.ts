/**
 * Relative time formatting with adaptive refresh intervals.
 *
 * The key insight: a timestamp showing "3s" needs to update every second,
 * but "2h 15m" only needs to update every minute. This module provides both
 * the formatted string and the optimal interval until the next meaningful change.
 */

interface RelativeTimeResult {
  /** Compact relative time string (e.g. "9m 35s", "1h 23m", "3d") */
  text: string;
  /** Milliseconds until the displayed text would change */
  nextUpdateMs: number;
}

/**
 * Format a duration in milliseconds as a compact human-readable string.
 *
 * Format tiers:
 * - < 60s: seconds only ("45s")
 * - < 1h: minutes + seconds ("9m 35s")
 * - < 24h: hours + minutes ("1h 23m")
 * - >= 24h: days only ("3d")
 */
export function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(Math.abs(ms) / 1000);

  if (totalSeconds < 60) return `${totalSeconds}s`;

  const totalMinutes = Math.floor(totalSeconds / 60);
  if (totalMinutes < 60) {
    const secs = totalSeconds % 60;
    return `${totalMinutes}m ${secs}s`;
  }

  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 24) {
    const mins = totalMinutes % 60;
    return `${totalHours}h ${mins}m`;
  }

  const days = Math.floor(totalHours / 24);
  return `${days}d`;
}

/**
 * Compute a compact relative time string and the interval until it next changes.
 *
 * Uses {@link formatDuration} for the text, plus computes the optimal refresh
 * interval so callers can schedule the next update efficiently.
 */
export function relativeTime(date: Date, ref: Date): RelativeTimeResult {
  const diffMs = ref.getTime() - date.getTime();
  const totalSeconds = Math.floor(diffMs / 1000);

  if (totalSeconds < 1) {
    return { text: "now", nextUpdateMs: 1000 - (diffMs % 1000) || 1000 };
  }

  const text = formatDuration(diffMs);

  // Compute optimal next-update interval based on the current tier
  const totalMinutes = Math.floor(totalSeconds / 60);
  const totalHours = Math.floor(totalMinutes / 60);

  let nextUpdateMs: number;
  if (totalHours >= 24) {
    const msIntoCurrentDay = diffMs % 86_400_000;
    nextUpdateMs = 86_400_000 - msIntoCurrentDay || 86_400_000;
  } else if (totalMinutes >= 60) {
    const msIntoCurrentMinute = diffMs % 60_000;
    nextUpdateMs = 60_000 - msIntoCurrentMinute || 60_000;
  } else {
    nextUpdateMs = 1000 - (diffMs % 1000) || 1000;
  }

  return { text, nextUpdateMs };
}
