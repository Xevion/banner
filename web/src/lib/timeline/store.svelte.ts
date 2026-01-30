/**
 * Reactive timeline data store with gap-aware on-demand loading.
 *
 * Tracks which time ranges have already been fetched and only requests
 * the missing segments when the view expands into unloaded territory.
 * Fetches are throttled so rapid panning/zooming doesn't flood the
 * (currently mock) API.
 */
import { generateSlots } from "./data";
import { SLOT_INTERVAL_MS } from "./constants";
import type { TimeSlot } from "./types";

/** Inclusive range of aligned slot timestamps [start, end]. */
type Range = [start: number, end: number];

const FETCH_THROTTLE_MS = 500;
const BUFFER_RATIO = 0.15;

// Mock network latency bounds (ms).
const MOCK_DELAY_MIN = 40;
const MOCK_DELAY_MAX = 120;

/**
 * Simulate an API call that returns slots for an arbitrary time range.
 * The delay makes loading behaviour visible during development.
 */
async function mockFetch(startMs: number, endMs: number): Promise<TimeSlot[]> {
  const delay = MOCK_DELAY_MIN + Math.random() * (MOCK_DELAY_MAX - MOCK_DELAY_MIN);
  await new Promise((r) => setTimeout(r, delay));
  return generateSlots(startMs, endMs);
}

/** Align a timestamp down to the nearest slot boundary. */
function alignFloor(ms: number): number {
  return Math.floor(ms / SLOT_INTERVAL_MS) * SLOT_INTERVAL_MS;
}

/** Align a timestamp up to the nearest slot boundary. */
function alignCeil(ms: number): number {
  return Math.ceil(ms / SLOT_INTERVAL_MS) * SLOT_INTERVAL_MS;
}

/**
 * Given a requested range and a sorted list of already-loaded ranges,
 * return the sub-ranges that still need fetching.
 */
function findGaps(start: number, end: number, loaded: Range[]): Range[] {
  const s = alignFloor(start);
  const e = alignCeil(end);

  if (loaded.length === 0) return [[s, e]];

  const gaps: Range[] = [];
  let cursor = s;

  for (const [ls, le] of loaded) {
    if (le < cursor) continue; // entirely before cursor
    if (ls > e) break; // entirely past our range

    if (ls > cursor) {
      gaps.push([cursor, Math.min(ls, e)]);
    }
    cursor = Math.max(cursor, le);
  }

  if (cursor < e) {
    gaps.push([cursor, e]);
  }

  return gaps;
}

/** Merge a new range into a sorted, non-overlapping range list (mutates nothing). */
function mergeRange(ranges: Range[], added: Range): Range[] {
  const all = [...ranges, added].sort((a, b) => a[0] - b[0]);
  const merged: Range[] = [];
  for (const r of all) {
    if (merged.length === 0 || merged[merged.length - 1][1] < r[0]) {
      merged.push([r[0], r[1]]);
    } else {
      merged[merged.length - 1][1] = Math.max(merged[merged.length - 1][1], r[1]);
    }
  }
  return merged;
}

/**
 * Create a reactive timeline store.
 *
 * Call `requestRange(viewStart, viewEnd)` whenever the visible window
 * changes. The store applies a 15 % buffer, computes which sub-ranges
 * are missing, and fetches them (throttled to 500 ms).
 *
 * The `data` getter returns a sorted `TimeSlot[]` that reactively
 * updates as new segments arrive.
 */
export function createTimelineStore() {
  // All loaded slots keyed by aligned timestamp (ms).
  let slotMap: Map<number, TimeSlot> = $state(new Map());

  // Sorted, non-overlapping list of fetched ranges.
  let loadedRanges: Range[] = [];

  let throttleTimer: ReturnType<typeof setTimeout> | undefined;
  let pendingStart = 0;
  let pendingEnd = 0;
  let hasFetchedOnce = false;

  // Sorted array derived from the map. The O(n log n) sort only runs when
  // slotMap is reassigned, which happens on fetch completion — not per frame.
  const data: TimeSlot[] = $derived(
    [...slotMap.values()].sort((a, b) => a.time.getTime() - b.time.getTime())
  );

  async function fetchGaps(start: number, end: number): Promise<void> {
    const gaps = findGaps(start, end, loadedRanges);
    if (gaps.length === 0) return;

    // Fetch all gap segments in parallel.
    const results = await Promise.all(gaps.map(([gs, ge]) => mockFetch(gs, ge)));

    // Merge results into the slot map.
    const next = new Map(slotMap);
    for (const slots of results) {
      for (const slot of slots) {
        next.set(slot.time.getTime(), slot);
      }
    }

    // Update loaded-range bookkeeping.
    for (const gap of gaps) {
      loadedRanges = mergeRange(loadedRanges, gap);
    }

    // Single reactive assignment.
    slotMap = next;
  }

  /**
   * Notify the store that the viewport now covers [viewStart, viewEnd] (ms).
   * Automatically buffers by 15 % each side and throttles fetches.
   *
   * The first call fetches immediately. Subsequent calls update the pending
   * range but don't reset an existing timer, so continuous view changes
   * (follow mode, momentum pan) don't starve the fetch.
   */
  function requestRange(viewStart: number, viewEnd: number): void {
    const span = viewEnd - viewStart;
    const buffer = span * BUFFER_RATIO;
    pendingStart = viewStart - buffer;
    pendingEnd = viewEnd + buffer;

    if (!hasFetchedOnce) {
      hasFetchedOnce = true;
      fetchGaps(pendingStart, pendingEnd);
      return;
    }

    if (throttleTimer === undefined) {
      throttleTimer = setTimeout(() => {
        throttleTimer = undefined;
        fetchGaps(pendingStart, pendingEnd);
      }, FETCH_THROTTLE_MS);
    }
  }

  /** Clean up the throttle timer (call on component destroy). */
  function dispose(): void {
    clearTimeout(throttleTimer);
  }

  return {
    get data() {
      return data;
    },
    requestRange,
    dispose,
  };
}
