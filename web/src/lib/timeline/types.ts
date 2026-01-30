/**
 * Shared types for the timeline feature.
 */
import type { ScaleLinear, ScaleTime } from "d3-scale";

import type { Subject } from "./data";

export type { Subject };

/** A single 15-minute time slot with per-subject class counts. */
export interface TimeSlot {
  time: Date;
  subjects: Record<Subject, number>;
}

/** Lerped animation entry for a single subject within a slot. */
export interface AnimEntry {
  current: number;
  target: number;
}

/**
 * Shared context passed to all canvas rendering functions.
 * Computed once per frame in the orchestrator and threaded through.
 */
export interface ChartContext {
  ctx: CanvasRenderingContext2D;
  xScale: ScaleTime<number, number>;
  yScale: ScaleLinear<number, number>;
  width: number;
  chartTop: number;
  chartBottom: number;
  viewSpan: number;
  viewStart: number;
  viewEnd: number;
}
