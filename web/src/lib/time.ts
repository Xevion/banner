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
 * Compute a compact relative time string and the interval until it next changes.
 *
 * Format tiers:
 * - < 60s: seconds only ("45s")
 * - < 1h: minutes + seconds ("9m 35s")
 * - < 24h: hours + minutes ("1h 23m")
 * - >= 24h: days only ("3d")
 */
export function relativeTime(date: Date, ref: Date): RelativeTimeResult {
  const diffMs = ref.getTime() - date.getTime();
  const totalSeconds = Math.floor(diffMs / 1000);

  if (totalSeconds < 1) {
    return { text: "now", nextUpdateMs: 1000 - (diffMs % 1000) || 1000 };
  }

  if (totalSeconds < 60) {
    const remainder = 1000 - (diffMs % 1000);
    return {
      text: `${totalSeconds}s`,
      nextUpdateMs: remainder || 1000,
    };
  }

  const totalMinutes = Math.floor(totalSeconds / 60);
  if (totalMinutes < 60) {
    const secs = totalSeconds % 60;
    const remainder = 1000 - (diffMs % 1000);
    return {
      text: `${totalMinutes}m ${secs}s`,
      nextUpdateMs: remainder || 1000,
    };
  }

  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 24) {
    const mins = totalMinutes % 60;
    const msIntoCurrentMinute = diffMs % 60_000;
    const msUntilNextMinute = 60_000 - msIntoCurrentMinute;
    return {
      text: `${totalHours}h ${mins}m`,
      nextUpdateMs: msUntilNextMinute || 60_000,
    };
  }

  const days = Math.floor(totalHours / 24);
  const msIntoCurrentDay = diffMs % 86_400_000;
  const msUntilNextDay = 86_400_000 - msIntoCurrentDay;
  return {
    text: `${days}d`,
    nextUpdateMs: msUntilNextDay || 86_400_000,
  };
}
