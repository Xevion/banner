/**
 * Data types, constants, and deterministic slot generation for the class timeline.
 * Each 15-minute slot is seeded by its timestamp, so the same slot always produces
 * identical data regardless of when or in what order it's fetched.
 */
import { SLOT_INTERVAL_MS } from "./constants";
import type { TimeSlot } from "./types";

export type { TimeSlot };

export const SUBJECTS = [
  "CS",
  "MATH",
  "BIO",
  "ENG",
  "PHYS",
  "HIST",
  "CHEM",
  "PSY",
  "ECE",
  "ART",
] as const;

export type Subject = (typeof SUBJECTS)[number];

/** Subject colors — distinct, accessible palette */
export const SUBJECT_COLORS: Record<Subject, string> = {
  CS: "#6366f1", // indigo
  MATH: "#f59e0b", // amber
  BIO: "#10b981", // emerald
  ENG: "#ef4444", // red
  PHYS: "#3b82f6", // blue
  HIST: "#8b5cf6", // violet
  CHEM: "#f97316", // orange
  PSY: "#ec4899", // pink
  ECE: "#14b8a6", // teal
  ART: "#a855f7", // purple
};

/**
 * Bell-curve-like distribution centered at a given hour.
 * Returns a value 0..1 representing relative class density.
 */
function bellCurve(hour: number, center: number, spread: number): number {
  const x = (hour - center) / spread;
  return Math.exp(-0.5 * x * x);
}

/**
 * Each subject has characteristic scheduling patterns:
 * peak hours, relative popularity, and spread.
 */
const SUBJECT_PROFILES: Record<Subject, { peaks: number[]; weight: number; spread: number }> = {
  CS: { peaks: [10, 14, 16], weight: 12, spread: 2.0 },
  MATH: { peaks: [8, 10, 13], weight: 10, spread: 1.8 },
  BIO: { peaks: [9, 11, 14], weight: 8, spread: 1.5 },
  ENG: { peaks: [9, 11, 14, 16], weight: 7, spread: 2.2 },
  PHYS: { peaks: [8, 13, 15], weight: 6, spread: 1.6 },
  HIST: { peaks: [10, 13, 15], weight: 5, spread: 2.0 },
  CHEM: { peaks: [8, 10, 14], weight: 6, spread: 1.5 },
  PSY: { peaks: [11, 14, 16], weight: 7, spread: 2.0 },
  ECE: { peaks: [9, 13, 15], weight: 5, spread: 1.8 },
  ART: { peaks: [10, 14, 17], weight: 4, spread: 2.5 },
};

/**
 * Seeded pseudo-random number generator (LCG) for reproducible data.
 */
function seededRandom(seed: number): () => number {
  let s = seed;
  return () => {
    s = (s * 1664525 + 1013904223) & 0xffffffff;
    return (s >>> 0) / 0xffffffff;
  };
}

/**
 * Integer hash so adjacent slot timestamps produce very different seeds.
 */
function hashTimestamp(ms: number): number {
  let h = ms | 0;
  h = ((h >> 16) ^ h) * 0x45d9f3b;
  h = ((h >> 16) ^ h) * 0x45d9f3b;
  h = (h >> 16) ^ h;
  return h >>> 0;
}

/** Generate a single TimeSlot for the given aligned timestamp. */
function generateSlot(timeMs: number): TimeSlot {
  const rand = seededRandom(hashTimestamp(timeMs));
  const time = new Date(timeMs);
  const hour = time.getHours() + time.getMinutes() / 60;

  const subjects = {} as Record<Subject, number>;
  for (const subject of SUBJECTS) {
    const profile = SUBJECT_PROFILES[subject];
    let density = 0;
    for (const peak of profile.peaks) {
      density += bellCurve(hour, peak, profile.spread);
    }
    const base = density * profile.weight;
    const noise = (rand() - 0.5) * 2;
    subjects[subject] = Math.max(0, Math.round(base + noise));
  }

  return { time, subjects };
}

/**
 * Generate TimeSlots covering [startMs, endMs], aligned to 15-minute boundaries.
 * Each slot is deterministically seeded by its timestamp.
 */
export function generateSlots(startMs: number, endMs: number): TimeSlot[] {
  const alignedStart = Math.floor(startMs / SLOT_INTERVAL_MS) * SLOT_INTERVAL_MS;
  const alignedEnd = Math.ceil(endMs / SLOT_INTERVAL_MS) * SLOT_INTERVAL_MS;

  const slots: TimeSlot[] = [];
  for (let t = alignedStart; t <= alignedEnd; t += SLOT_INTERVAL_MS) {
    slots.push(generateSlot(t));
  }
  return slots;
}
