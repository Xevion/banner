import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Shared tooltip content styling for bits-ui Tooltip.Content */
export const tooltipContentClass =
  "z-50 bg-card text-card-foreground text-xs border border-border rounded-md px-2.5 py-1.5 shadow-md max-w-72";

export interface FormatNumberOptions {
  /** Include sign for positive numbers (default: false) */
  sign?: boolean;
  /** Maximum fraction digits (default: 0 for integers) */
  maximumFractionDigits?: number;
}

/**
 * Format a number with locale-aware thousands separators.
 * Uses browser locale via Intl.NumberFormat.
 */
export function formatNumber(num: number, options: FormatNumberOptions = {}): string {
  const { sign = false, maximumFractionDigits = 0 } = options;
  const formatted = new Intl.NumberFormat(undefined, {
    maximumFractionDigits,
  }).format(num);

  if (sign && num >= 0) {
    return `+${formatted}`;
  }
  return formatted;
}
