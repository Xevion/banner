<script lang="ts">
import FilterPopover from "./FilterPopover.svelte";
import RangeSlider from "./RangeSlider.svelte";

let {
  creditHourMin = $bindable<number | null>(null),
  creditHourMax = $bindable<number | null>(null),
  instructor = $bindable(""),
  courseNumberMin = $bindable<number | null>(null),
  courseNumberMax = $bindable<number | null>(null),
  ranges,
}: {
  creditHourMin: number | null;
  creditHourMax: number | null;
  instructor: string;
  courseNumberMin: number | null;
  courseNumberMax: number | null;
  ranges: { courseNumber: { min: number; max: number }; creditHours: { min: number; max: number } };
} = $props();

const hasActiveFilters = $derived(
  creditHourMin !== null ||
    creditHourMax !== null ||
    instructor !== "" ||
    courseNumberMin !== null ||
    courseNumberMax !== null
);
</script>

<FilterPopover label="More" active={hasActiveFilters}>
  {#snippet content()}
    <RangeSlider
      min={ranges.creditHours.min}
      max={ranges.creditHours.max}
      step={1}
      bind:valueLow={creditHourMin}
      bind:valueHigh={creditHourMax}
      label="Credit hours"
      pips
      all="label"
    />

    <div class="h-px bg-border"></div>

    <div class="flex flex-col gap-1.5">
      <label for="instructor-input" class="text-xs font-medium text-muted-foreground select-none">
        Instructor
      </label>
      <input
        id="instructor-input"
        type="text"
        placeholder="Search by name..."
        autocomplete="off"
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
      bind:valueLow={courseNumberMin}
      bind:valueHigh={courseNumberMax}
      label="Course number"
      pips
      pipstep={10}
    />
  {/snippet}
</FilterPopover>
