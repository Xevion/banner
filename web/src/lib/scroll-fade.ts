/** Distance in pixels before the fade is fully opaque */
export const FADE_DISTANCE = 60;

/** Percentage of container width used for each fade zone */
export const FADE_PERCENT = 8;

export interface ScrollMetrics {
  scrollLeft: number;
  scrollWidth: number;
  clientWidth: number;
}

/** Compute left edge opacity: 0 when scrollLeft = 0, 1 when scrollLeft >= FADE_DISTANCE */
export function leftOpacity(metrics: ScrollMetrics): number {
  return Math.min(metrics.scrollLeft / FADE_DISTANCE, 1);
}

/** Compute right edge opacity: 0 when at max scroll, 1 when remaining >= FADE_DISTANCE */
export function rightOpacity(metrics: ScrollMetrics): number {
  const { scrollLeft, scrollWidth, clientWidth } = metrics;
  if (scrollWidth <= clientWidth) return 0;
  const maxScroll = scrollWidth - clientWidth;
  const remainingScroll = maxScroll - scrollLeft;
  return Math.min(remainingScroll / FADE_DISTANCE, 1);
}

/** Build a CSS mask-image gradient from scroll metrics */
export function maskGradient(metrics: ScrollMetrics): string {
  const left = leftOpacity(metrics);
  const right = rightOpacity(metrics);
  const leftEnd = left * FADE_PERCENT;
  const rightStart = 100 - right * FADE_PERCENT;
  return `linear-gradient(to right, transparent 0%, black ${leftEnd}%, black ${rightStart}%, transparent 100%)`;
}
