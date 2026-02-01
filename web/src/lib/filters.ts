export const DAY_OPTIONS: { label: string; value: string }[] = [
  { label: "M", value: "monday" },
  { label: "T", value: "tuesday" },
  { label: "W", value: "wednesday" },
  { label: "Th", value: "thursday" },
  { label: "F", value: "friday" },
  { label: "Sa", value: "saturday" },
  { label: "Su", value: "sunday" },
];

export function toggleDay(days: string[], day: string): string[] {
  return days.includes(day) ? days.filter((d) => d !== day) : [...days, day];
}

export function parseTimeInput(input: string): string | null {
  const trimmed = input.trim();
  if (trimmed === "") return null;

  const ampmMatch = trimmed.match(/^(\d{1,2}):(\d{2})\s*(AM|PM)$/i);
  if (ampmMatch) {
    let hours = parseInt(ampmMatch[1], 10);
    const minutes = parseInt(ampmMatch[2], 10);
    const period = ampmMatch[3].toUpperCase();
    if (period === "PM" && hours !== 12) hours += 12;
    if (period === "AM" && hours === 12) hours = 0;
    return String(hours).padStart(2, "0") + String(minutes).padStart(2, "0");
  }

  const militaryMatch = trimmed.match(/^(\d{1,2}):(\d{2})$/);
  if (militaryMatch) {
    const hours = parseInt(militaryMatch[1], 10);
    const minutes = parseInt(militaryMatch[2], 10);
    return String(hours).padStart(2, "0") + String(minutes).padStart(2, "0");
  }

  return null;
}

export function formatTime(time: string | null): string {
  if (time === null || time.length !== 4) return "";
  const hours = parseInt(time.slice(0, 2), 10);
  const minutes = time.slice(2);
  const period = hours >= 12 ? "PM" : "AM";
  const displayHours = hours === 0 ? 12 : hours > 12 ? hours - 12 : hours;
  return `${displayHours}:${minutes} ${period}`;
}

export function toggleValue(arr: string[], code: string): string[] {
  return arr.includes(code) ? arr.filter((v) => v !== code) : [...arr, code];
}
