<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import {
  DAY_OPTIONS,
  toggleDay as _toggleDay,
  parseTimeInput,
  formatTime,
  toggleValue,
} from "$lib/filters";
import { getPartOfTermFilterLabel } from "$lib/labels";
import FilterPopover from "./FilterPopover.svelte";

let {
  days = $bindable<string[]>([]),
  timeStart = $bindable<string | null>(null),
  timeEnd = $bindable<string | null>(null),
  partOfTerm = $bindable<string[]>([]),
  referenceData,
}: {
  days: string[];
  timeStart: string | null;
  timeEnd: string | null;
  partOfTerm: string[];
  referenceData: {
    partsOfTerm: CodeDescription[];
  };
} = $props();

const hasActiveFilters = $derived(
  days.length > 0 || timeStart !== null || timeEnd !== null || partOfTerm.length > 0
);

function toggleDay(day: string) {
  days = _toggleDay(days, day);
}
</script>

<FilterPopover label="Schedule" active={hasActiveFilters}>
  {#snippet content()}
    <div class="flex flex-col gap-1.5">
      <span class="text-xs font-medium text-muted-foreground select-none">Days of week</span>
      <div class="flex gap-1 whitespace-nowrap">
        {#each DAY_OPTIONS as { label, value } (value)}
          <button
            type="button"
            aria-label={value.charAt(0).toUpperCase() + value.slice(1)}
            aria-pressed={days.includes(value)}
            class="flex flex-1 items-center justify-center rounded-md px-2 py-1 text-xs font-medium transition-colors cursor-pointer select-none
                   {days.includes(value)
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
            onclick={() => toggleDay(value)}
          >
            {label}
          </button>
        {/each}
      </div>
    </div>

    <div class="h-px bg-border"></div>

    <div class="flex flex-col gap-1.5">
      <span class="text-xs font-medium text-muted-foreground select-none">Time range</span>
      <div class="flex items-center gap-2">
        <input
          type="text"
          placeholder="10:00 AM"
          autocomplete="off"
          value={formatTime(timeStart)}
          onchange={(e) => {
            timeStart = parseTimeInput(e.currentTarget.value);
            e.currentTarget.value = formatTime(timeStart);
          }}
          class="h-8 w-24 border border-border bg-card text-foreground rounded-md px-2 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
        />
        <span class="text-xs text-muted-foreground select-none">to</span>
        <input
          type="text"
          placeholder="3:00 PM"
          autocomplete="off"
          value={formatTime(timeEnd)}
          onchange={(e) => {
            timeEnd = parseTimeInput(e.currentTarget.value);
            e.currentTarget.value = formatTime(timeEnd);
          }}
          class="h-8 w-24 border border-border bg-card text-foreground rounded-md px-2 text-sm
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
        />
      </div>
    </div>

    {#if referenceData.partsOfTerm.length > 0}
      <div class="h-px bg-border"></div>

      <div class="flex flex-col gap-1.5">
        <span class="text-xs font-medium text-muted-foreground select-none">Part of Term</span>
        <div class="flex gap-1 whitespace-nowrap">
          {#each referenceData.partsOfTerm as item (item.filterValue)}
            <button
              type="button"
              aria-pressed={partOfTerm.includes(item.filterValue)}
              class="flex flex-1 items-center justify-center rounded-md px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer select-none
                     {partOfTerm.includes(item.filterValue)
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
              onclick={() => (partOfTerm = toggleValue(partOfTerm, item.filterValue))}
              title={item.description}
            >
              {getPartOfTermFilterLabel(item.filterValue)}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/snippet}
</FilterPopover>
