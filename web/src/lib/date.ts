import { format, formatDistanceToNow } from "date-fns";

/** Returns a relative time string like "3 minutes ago" or "in 2 hours". */
export function formatRelativeDate(date: string | Date): string {
  const d = typeof date === "string" ? new Date(date) : date;
  return formatDistanceToNow(d, { addSuffix: true });
}

/** Returns a full absolute datetime string for tooltip display, e.g. "Jan 29, 2026, 3:45:12 PM". */
export function formatAbsoluteDate(date: string | Date): string {
  const d = typeof date === "string" ? new Date(date) : date;
  return format(d, "MMM d, yyyy, h:mm:ss a");
}
