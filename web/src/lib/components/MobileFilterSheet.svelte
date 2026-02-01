<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import {
  DAY_OPTIONS,
  toggleDay as _toggleDay,
  parseTimeInput,
  formatTime,
  toggleValue,
} from "$lib/filters";
import { ChevronDown } from "@lucide/svelte";
import BottomSheet from "./BottomSheet.svelte";
import RangeSlider from "./RangeSlider.svelte";

let {
  open = $bindable(false),
  openOnly = $bindable(),
  waitCountMax = $bindable(),
  days = $bindable(),
  timeStart = $bindable(),
  timeEnd = $bindable(),
  instructionalMethod = $bindable(),
  campus = $bindable(),
  partOfTerm = $bindable(),
  attributes = $bindable(),
  creditHourMin = $bindable(),
  creditHourMax = $bindable(),
  instructor = $bindable(),
  courseNumberMin = $bindable(),
  courseNumberMax = $bindable(),
  referenceData,
  ranges,
}: {
  open: boolean;
  openOnly: boolean;
  waitCountMax: number | null;
  days: string[];
  timeStart: string | null;
  timeEnd: string | null;
  instructionalMethod: string[];
  campus: string[];
  partOfTerm: string[];
  attributes: string[];
  creditHourMin: number | null;
  creditHourMax: number | null;
  instructor: string;
  courseNumberMin: number | null;
  courseNumberMax: number | null;
  referenceData: {
    instructionalMethods: CodeDescription[];
    campuses: CodeDescription[];
    partsOfTerm: CodeDescription[];
    attributes: CodeDescription[];
  };
  ranges: {
    courseNumber: { min: number; max: number };
    creditHours: { min: number; max: number };
    waitCount: { max: number };
  };
} = $props();

let expandedSection = $state<string | null>(null);

function toggleSection(id: string) {
  expandedSection = expandedSection === id ? null : id;
}

function toggleDay(day: string) {
  days = _toggleDay(days, day);
}

const attributeSections: {
  label: string;
  key: "instructionalMethod" | "campus" | "partOfTerm" | "attributes";
  dataKey: "instructionalMethods" | "campuses" | "partsOfTerm" | "attributes";
}[] = [
  { label: "Instructional Method", key: "instructionalMethod", dataKey: "instructionalMethods" },
  { label: "Campus", key: "campus", dataKey: "campuses" },
  { label: "Part of Term", key: "partOfTerm", dataKey: "partsOfTerm" },
  { label: "Course Attributes", key: "attributes", dataKey: "attributes" },
];

function getAttrSelected(
  key: "instructionalMethod" | "campus" | "partOfTerm" | "attributes"
): string[] {
  if (key === "instructionalMethod") return instructionalMethod;
  if (key === "campus") return campus;
  if (key === "partOfTerm") return partOfTerm;
  return attributes;
}

function toggleAttr(
  key: "instructionalMethod" | "campus" | "partOfTerm" | "attributes",
  code: string
) {
  if (key === "instructionalMethod") instructionalMethod = toggleValue(instructionalMethod, code);
  else if (key === "campus") campus = toggleValue(campus, code);
  else if (key === "partOfTerm") partOfTerm = toggleValue(partOfTerm, code);
  else attributes = toggleValue(attributes, code);
}
</script>

<BottomSheet bind:open>
  <div class="flex flex-col">
    <!-- Status section -->
    <button
      onclick={() => toggleSection("status")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span class="flex items-center gap-2">
        Status
        {#if openOnly || waitCountMax !== null}
          <span class="size-1.5 rounded-full bg-primary"></span>
        {/if}
      </span>
      <ChevronDown
        class="size-4 text-muted-foreground transition-transform {expandedSection === 'status'
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSection === "status"}
      <div class="px-4 pb-3 flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-muted-foreground select-none">Availability</span>
          <button
            type="button"
            aria-pressed={openOnly}
            class="inline-flex items-center justify-center rounded-full px-3 py-1 text-xs font-medium transition-colors cursor-pointer select-none
                   {openOnly
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
            onclick={() => (openOnly = !openOnly)}
          >
            Open only
          </button>
        </div>

        <div class="h-px bg-border"></div>

        {#if ranges.waitCount.max > 0}
          <RangeSlider
            min={0}
            max={ranges.waitCount.max}
            step={5}
            bind:value={waitCountMax}
            label="Max waitlist"
            dual={false}
            pips
            pipstep={2}
            formatValue={(v) => (v === 0 ? "Off" : String(v))}
          />
        {:else}
          <div class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-muted-foreground select-none">Max waitlist</span>
            <span class="text-xs text-muted-foreground select-none">No waitlisted courses</span>
          </div>
        {/if}
      </div>
    {/if}
    <div class="h-px bg-border mx-4"></div>

    <!-- Schedule section -->
    <button
      onclick={() => toggleSection("schedule")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span class="flex items-center gap-2">
        Schedule
        {#if days.length > 0 || timeStart !== null || timeEnd !== null}
          <span class="size-1.5 rounded-full bg-primary"></span>
        {/if}
      </span>
      <ChevronDown
        class="size-4 text-muted-foreground transition-transform {expandedSection === 'schedule'
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSection === "schedule"}
      <div class="px-4 pb-3 flex flex-col gap-3">
        <div class="flex flex-col gap-1.5">
          <span class="text-xs font-medium text-muted-foreground select-none">Days of week</span>
          <div class="flex gap-1">
            {#each DAY_OPTIONS as { label, value } (value)}
              <button
                type="button"
                aria-label={value.charAt(0).toUpperCase() + value.slice(1)}
                aria-pressed={days.includes(value)}
                class="flex items-center justify-center rounded-md px-2 py-1 text-xs font-medium transition-colors cursor-pointer select-none min-w-[2rem]
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
      </div>
    {/if}
    <div class="h-px bg-border mx-4"></div>

    <!-- Attributes section -->
    <button
      onclick={() => toggleSection("attributes")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span class="flex items-center gap-2">
        Attributes
        {#if instructionalMethod.length > 0 || campus.length > 0 || partOfTerm.length > 0 || attributes.length > 0}
          <span class="size-1.5 rounded-full bg-primary"></span>
        {/if}
      </span>
      <ChevronDown
        class="size-4 text-muted-foreground transition-transform {expandedSection === 'attributes'
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSection === "attributes"}
      <div class="px-4 pb-3 flex flex-col gap-3">
        {#each attributeSections as { label, key, dataKey }, i (key)}
          {#if i > 0}
            <div class="h-px bg-border"></div>
          {/if}
          <div class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-muted-foreground select-none">{label}</span>
            <div class="flex flex-wrap gap-1">
              {#each referenceData[dataKey] as item (item.code)}
                {@const selected = getAttrSelected(key)}
                <button
                  type="button"
                  aria-pressed={selected.includes(item.code)}
                  class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                         {selected.includes(item.code)
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                  onclick={() => toggleAttr(key, item.code)}
                  title={item.description}
                >
                  {item.description}
                </button>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
    <div class="h-px bg-border mx-4"></div>

    <!-- More section -->
    <button
      onclick={() => toggleSection("more")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span class="flex items-center gap-2">
        More
        {#if creditHourMin !== null || creditHourMax !== null || instructor !== "" || courseNumberMin !== null || courseNumberMax !== null}
          <span class="size-1.5 rounded-full bg-primary"></span>
        {/if}
      </span>
      <ChevronDown
        class="size-4 text-muted-foreground transition-transform {expandedSection === 'more'
          ? 'rotate-180'
          : ''}"
      />
    </button>
    {#if expandedSection === "more"}
      <div class="px-4 pb-3 flex flex-col gap-3">
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
          <label
            for="mobile-instructor-input"
            class="text-xs font-medium text-muted-foreground select-none"
          >
            Instructor
          </label>
          <input
            id="mobile-instructor-input"
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
      </div>
    {/if}
  </div>
</BottomSheet>
