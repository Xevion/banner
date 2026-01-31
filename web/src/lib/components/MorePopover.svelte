<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import { fly } from "svelte/transition";
import RangeSlider from "./RangeSlider.svelte";

let {
  creditHourMin = $bindable<number | null>(null),
  creditHourMax = $bindable<number | null>(null),
  instructor = $bindable(""),
  courseNumberLow = $bindable<number | null>(null),
  courseNumberHigh = $bindable<number | null>(null),
  ranges,
}: {
  creditHourMin: number | null;
  creditHourMax: number | null;
  instructor: string;
  courseNumberLow: number | null;
  courseNumberHigh: number | null;
  ranges: { courseNumber: { min: number; max: number }; creditHours: { min: number; max: number } };
} = $props();

const hasActiveFilters = $derived(
  creditHourMin !== null ||
    creditHourMax !== null ||
    instructor !== "" ||
    courseNumberLow !== null ||
    courseNumberHigh !== null
);
</script>

<Popover.Root>
  <Popover.Trigger
    class="inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium transition-colors cursor-pointer
           {hasActiveFilters
      ? 'border-primary/50 bg-primary/10 text-primary hover:bg-primary/20'
      : 'border-border bg-background text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
  >
    {#if hasActiveFilters}
      <span class="size-1.5 rounded-full bg-primary"></span>
    {/if}
    More
    <ChevronDown class="size-3" />
  </Popover.Trigger>
  <Popover.Content
    class="z-50 rounded-md border border-border bg-card p-3 text-card-foreground shadow-lg w-72"
    sideOffset={4}
    forceMount
  >
    {#snippet child({ wrapperProps, props, open })}
      {#if open}
        <div {...wrapperProps}>
          <div {...props} transition:fly={{ duration: 150, y: -4 }}>
            <div class="flex flex-col gap-3">
              <RangeSlider
                min={ranges.creditHours.min}
                max={ranges.creditHours.max}
                step={1}
                bind:valueLow={creditHourMin}
                bind:valueHigh={creditHourMax}
                label="Credit hours"
              />

              <div class="h-px bg-border"></div>

              <div class="flex flex-col gap-1.5">
                <label for="instructor-input" class="text-xs font-medium text-muted-foreground">
                  Instructor
                </label>
                <input
                  id="instructor-input"
                  type="text"
                  placeholder="Search by name..."
                  bind:value={instructor}
                  class="h-8 border border-border bg-card text-foreground rounded-md px-2 text-sm
                         focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                />
              </div>

              <div class="h-px bg-border"></div>

              <RangeSlider
                min={ranges.courseNumber.min}
                max={ranges.courseNumber.max}
                step={100}
                bind:valueLow={courseNumberLow}
                bind:valueHigh={courseNumberHigh}
                label="Course number"
              />
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </Popover.Content>
</Popover.Root>
