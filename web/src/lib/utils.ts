import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Shared tooltip content styling for bits-ui Tooltip.Content */
export const tooltipContentClass =
  "z-50 bg-card text-card-foreground text-xs border border-border rounded-md px-2.5 py-1.5 shadow-md max-w-72";
