<script lang="ts">
import type { ComponentProps } from "svelte";
import LibRangeSlider from "svelte-range-slider-pips";

type LibProps = ComponentProps<LibRangeSlider>;

/**
 * Two modes:
 * - `dual` (default): bind `valueLow` and `valueHigh` for a two-thumb range.
 * - `dual={false}`: bind `value` for a single-thumb slider. `valueLow`/`valueHigh` are ignored.
 *
 * All three are null when at their default (boundary) position.
 */
type Props = Omit<
  LibProps,
  | "values"
  | "value"
  | "formatter"
  | "range"
  | "min"
  | "max"
  | "float"
  | "hoverable"
  | "springValues"
> & {
  min: number;
  max: number;
  label: string;
  formatValue?: (v: number) => string;
  dual?: boolean;
  float?: boolean;
  hoverable?: boolean;
  springValues?: { stiffness?: number; damping?: number };
  valueLow?: number | null;
  valueHigh?: number | null;
  value?: number | null;
};

let {
  min,
  max,
  valueLow = $bindable(null),
  valueHigh = $bindable(null),
  value = $bindable(null),
  label,
  formatValue = (v: number) => String(v),
  dual = true,
  float = true,
  hoverable = true,
  // Intentionally snappier than library defaults (0.15/0.4)
  springValues = { stiffness: 0.3, damping: 0.7 },
  ...libProps
}: Props = $props();

let internalValues = $state<number[]>([min, max]);
let internalValue = $state(max);

if (import.meta.env.DEV) {
  $effect(() => {
    if (min >= max) {
      console.warn(`RangeSlider "${label}": min (${min}) must be less than max (${max})`);
    }
  });
}

// Sync external -> internal (equality guards prevent loops)
$effect(() => {
  if (dual) {
    const nextLow = valueLow ?? min;
    const nextHigh = valueHigh ?? max;
    if (internalValues[0] !== nextLow || internalValues[1] !== nextHigh) {
      internalValues = [nextLow, nextHigh];
    }
  } else {
    const next = value ?? max;
    if (internalValue !== next) {
      internalValue = next;
    }
  }
});

const isDefault = $derived(dual ? valueLow === null && valueHigh === null : value === null);

function handleDualChange(event: CustomEvent<{ values: number[] }>) {
  const [low, high] = event.detail.values;
  const nextLow = low === min && high === max ? null : low;
  const nextHigh = low === min && high === max ? null : high;
  if (nextLow === valueLow && nextHigh === valueHigh) return;
  valueLow = nextLow;
  valueHigh = nextHigh;
}

function handleSingleChange(event: CustomEvent<{ value: number }>) {
  const next = event.detail.value === max ? null : event.detail.value;
  if (next === value) return;
  value = next;
}
</script>

<div class="range-slider-wrapper flex flex-col gap-1.5" role="group" aria-label={label}>
  <div class="flex items-center justify-between">
    <span class="text-xs font-medium text-muted-foreground">{label}</span>
    {#if !isDefault}
      <span class="text-xs text-muted-foreground">
        {#if dual}
          {formatValue(valueLow ?? min)} – {formatValue(valueHigh ?? max)}
        {:else}
          ≤ {formatValue(value ?? max)}
        {/if}
      </span>
    {/if}
  </div>

  <div class="pt-0.5">
    {#if dual}
      <LibRangeSlider
        bind:values={internalValues}
        {min}
        {max}
        {float}
        {hoverable}
        {springValues}
        range
        formatter={formatValue}
        {...libProps}
        on:change={handleDualChange}
      />
    {:else}
      <LibRangeSlider
        bind:value={internalValue}
        {min}
        {max}
        {float}
        {hoverable}
        {springValues}
        formatter={formatValue}
        {...libProps}
        on:change={handleSingleChange}
      />
    {/if}
  </div>
</div>

<style>
/* Theme color mapping */
.range-slider-wrapper :global(.rangeSlider) {
  --range-slider: var(--border);
  --range-handle-inactive: var(--muted-foreground);
  --range-handle: var(--muted-foreground);
  --range-handle-focus: var(--foreground);
  --range-handle-border: var(--muted-foreground);
  --range-range-inactive: var(--muted-foreground);
  --range-range: var(--foreground);
  --range-range-hover: var(--foreground);
  --range-range-press: var(--foreground);
  --range-float-inactive: var(--card);
  --range-float: var(--card);
  --range-float-text: var(--card-foreground);
  --range-range-limit: var(--muted);
  font-size: 0.75rem;
  margin: 0.5em;
  height: 0.375em;
}

/* Smaller handles, plain circles */
.range-slider-wrapper :global(.rangeSlider .rangeHandle) {
  height: 1em;
  width: 1em;
}

.range-slider-wrapper :global(.rangeSlider.rsRange:not(.rsMin):not(.rsMax) .rangeNub) {
  border-radius: 9999px;
}

.range-slider-wrapper :global(.rangeSlider.rsRange .rangeHandle .rangeNub) {
  transform: none;
}

/* Hover / press effects */
.range-slider-wrapper :global(.rangeSlider.rsHoverable:not(.rsDisabled) .rangeHandle:hover::before) {
  box-shadow: 0 0 0 6px var(--handle-border);
  opacity: 0.15;
}

.range-slider-wrapper :global(.rangeSlider.rsHoverable:not(.rsDisabled) .rangeHandle.rsPress::before),
.range-slider-wrapper :global(.rangeSlider.rsHoverable:not(.rsDisabled) .rangeHandle.rsPress:hover::before) {
  box-shadow: 0 0 0 8px var(--handle-border);
  opacity: 0.25;
}

/* Track bar */
.range-slider-wrapper :global(.rangeSlider .rangeBar),
.range-slider-wrapper :global(.rangeSlider .rangeLimit) {
  height: 0.375em;
}

/* Float label */
.range-slider-wrapper :global(.rangeSlider .rangeFloat) {
  font-size: 0.7em;
  font-weight: 400;
  line-height: 1;
  padding: 0.25em 0.4em 0.35em;
  border-radius: 0.375em;
  border: 1px solid var(--border);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

/* Pip labels */
.range-slider-wrapper :global(.rangeSlider .rangePip .pipVal) {
  color: var(--muted-foreground);
  font-size: 0.6em;
  font-weight: 400;
}

/* Pip spacing */
.range-slider-wrapper :global(.rangeSlider.rsPips) {
  margin-bottom: 1.2em;
}

.range-slider-wrapper :global(.rangeSlider.rsPipLabels) {
  margin-bottom: 2em;
}
</style>
