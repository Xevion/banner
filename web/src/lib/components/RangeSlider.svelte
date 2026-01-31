<script lang="ts">
let {
  min,
  max,
  step = 1,
  valueLow = $bindable<number | null>(null),
  valueHigh = $bindable<number | null>(null),
  label,
  formatValue = (v: number) => String(v),
  dual = true,
}: {
  min: number;
  max: number;
  step?: number;
  valueLow: number | null;
  valueHigh: number | null;
  label: string;
  formatValue?: (v: number) => string;
  dual?: boolean;
} = $props();

// Internal slider values — full range when filter is null (inactive)
let internalLow = $state(0);
let internalHigh = $state(0);

// Sync external → internal when props change (e.g., reset)
$effect(() => {
  internalLow = valueLow ?? min;
  internalHigh = valueHigh ?? max;
});

// Whether the slider is at its default (full range) position
const isDefault = $derived(internalLow === min && internalHigh === max);

function commitLow(value: number) {
  internalLow = value;
  // At full range = no filter
  if (value === min && internalHigh === max) {
    valueLow = null;
    valueHigh = null;
  } else {
    valueLow = value;
    if (valueHigh === null) valueHigh = internalHigh;
  }
}

function commitHigh(value: number) {
  internalHigh = value;
  if (internalLow === min && value === max) {
    valueLow = null;
    valueHigh = null;
  } else {
    valueHigh = value;
    if (valueLow === null) valueLow = internalLow;
  }
}

function commitSingle(value: number) {
  internalHigh = value;
  valueHigh = value === 0 ? null : value;
}
</script>

<div class="flex flex-col gap-1.5">
  <div class="flex items-center justify-between">
    <span class="text-xs font-medium text-muted-foreground">{label}</span>
    {#if !isDefault}
      <span class="text-xs text-muted-foreground">
        {#if dual}
          {formatValue(internalLow)} – {formatValue(internalHigh)}
        {:else}
          ≤ {formatValue(internalHigh)}
        {/if}
      </span>
    {/if}
  </div>

  {#if dual}
    <div class="flex items-center gap-2">
      <input
        type="range"
        {min}
        max={internalHigh}
        {step}
        value={internalLow}
        oninput={(e) => commitLow(Number(e.currentTarget.value))}
        class="flex-1 accent-primary h-1.5"
      />
      <input
        type="range"
        min={internalLow}
        {max}
        {step}
        value={internalHigh}
        oninput={(e) => commitHigh(Number(e.currentTarget.value))}
        class="flex-1 accent-primary h-1.5"
      />
    </div>
  {:else}
    <input
      type="range"
      min={0}
      {max}
      {step}
      value={internalHigh}
      oninput={(e) => commitSingle(Number(e.currentTarget.value))}
      class="w-full accent-primary h-1.5"
    />
  {/if}
</div>
