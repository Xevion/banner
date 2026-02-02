<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import {
  DAY_OPTIONS,
  toggleDay as _toggleDay,
  parseTimeInput,
  formatTime,
  toggleValue,
} from "$lib/filters";
import { InstructionalMethodValues as IM, AttributeValues as AV } from "$lib/filterValues";
import { getPartOfTermFilterLabel, getAttributeFilterLabel } from "$lib/labels";
import { ChevronDown } from "@lucide/svelte";
import BottomSheet from "./BottomSheet.svelte";
import CompoundFilterButton from "./CompoundFilterButton.svelte";
import AvailabilityFilter from "./AvailabilityFilter.svelte";
import RangeSlider from "./RangeSlider.svelte";

type AttributeReferenceData = Record<
  "instructionalMethods" | "campuses" | "partsOfTerm" | "attributes",
  CodeDescription[]
>;

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
  referenceData: AttributeReferenceData;
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

// Instructional method compound button config (same as FormatPopover)
const imButtons = [
  { label: "In Person", codes: [IM.InPerson] },
  {
    label: "Online",
    codes: [IM.Online.Async, IM.Online.Sync, IM.Online.Mixed],
    variants: [
      { code: IM.Online.Async, label: "Async" },
      { code: IM.Online.Sync, label: "Sync" },
    ],
  },
  {
    label: "Hybrid",
    codes: [IM.Hybrid.Half, IM.Hybrid.OneThird, IM.Hybrid.TwoThirds],
    variants: [
      { code: IM.Hybrid.Half, label: "Half" },
      { code: IM.Hybrid.OneThird, label: "One Third" },
      { code: IM.Hybrid.TwoThirds, label: "Two Thirds" },
    ],
  },
  { label: "Independent", codes: [IM.Independent] },
];

// Attribute grouping (same as CatalogPopover)
const CORE_CODES = new Set<string>([
  AV.CoreCommunication,
  AV.CoreMathematics,
  AV.CoreLifePhysicalSciences,
  AV.CoreLanguagePhilosophy,
  AV.CoreCreativeArts,
  AV.CoreAmericanHistory,
  AV.CoreGovernment,
  AV.CoreSocialBehavioral,
  AV.CoreComponentArea,
]);
const LEVEL_CODES = new Set<string>([
  AV.Developmental,
  AV.LowerDivision,
  AV.UpperDivision,
  AV.Graduate,
]);

const coreAttributes = $derived(
  referenceData.attributes.filter((a) => CORE_CODES.has(a.filterValue))
);
const levelAttributes = $derived(
  referenceData.attributes.filter((a) => LEVEL_CODES.has(a.filterValue))
);
const specialAttributes = $derived(
  referenceData.attributes.filter(
    (a) => !CORE_CODES.has(a.filterValue) && !LEVEL_CODES.has(a.filterValue)
  )
);

function toggleAttr(code: string) {
  attributes = toggleValue(attributes, code);
}

// Format course number pips as "0", "1k", "2k", etc.
function formatCourseNumberPip(v: number): string {
  if (v === 0) return "0";
  return `${v / 1000}k`;
}

// Active state for each filter section
const statusActive = $derived(openOnly || waitCountMax !== null);
const formatActive = $derived(instructionalMethod.length > 0);
const scheduleActive = $derived(
  days.length > 0 || timeStart !== null || timeEnd !== null || partOfTerm.length > 0
);
const catalogActive = $derived(campus.length > 0 || attributes.length > 0);
const moreActive = $derived(
  creditHourMin !== null ||
    creditHourMax !== null ||
    instructor !== "" ||
    courseNumberMin !== null ||
    courseNumberMax !== null
);

// Show chevron when section is expanded OR when filter is not active
const showStatusChevron = $derived(expandedSection === "status" || !statusActive);
const showFormatChevron = $derived(expandedSection === "format" || !formatActive);
const showScheduleChevron = $derived(expandedSection === "schedule" || !scheduleActive);
const showCatalogChevron = $derived(expandedSection === "catalog" || !catalogActive);
const showMoreChevron = $derived(expandedSection === "more" || !moreActive);
</script>

<BottomSheet bind:open>
  <div class="flex flex-col">
    <!-- Status section -->
    <button
      onclick={() => toggleSection("status")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span>Status</span>
      <div class="relative size-4 flex items-center justify-center">
        <ChevronDown
          class="size-4 text-muted-foreground absolute transition-all duration-150 {showStatusChevron
            ? 'opacity-100 scale-100'
            : 'opacity-0 scale-75'} {expandedSection === 'status' ? 'rotate-180' : ''}"
        />
        <span
          class="size-1.5 rounded-full bg-primary absolute transition-all duration-150 {showStatusChevron
            ? 'opacity-0 scale-75'
            : 'opacity-100 scale-100'}"
        ></span>
      </div>
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

    <!-- Format section (was part of Attributes) -->
    <button
      onclick={() => toggleSection("format")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span>Format</span>
      <div class="relative size-4 flex items-center justify-center">
        <ChevronDown
          class="size-4 text-muted-foreground absolute transition-all duration-150 {showFormatChevron
            ? 'opacity-100 scale-100'
            : 'opacity-0 scale-75'} {expandedSection === 'format' ? 'rotate-180' : ''}"
        />
        <span
          class="size-1.5 rounded-full bg-primary absolute transition-all duration-150 {showFormatChevron
            ? 'opacity-0 scale-75'
            : 'opacity-100 scale-100'}"
        ></span>
      </div>
    </button>
    {#if expandedSection === "format"}
      <div class="px-4 pb-3 flex flex-col gap-1.5">
        {#each imButtons as btn (btn.label)}
          <CompoundFilterButton
            label={btn.label}
            codes={btn.codes}
            variants={btn.variants}
            bind:selected={instructionalMethod}
          />
        {/each}
      </div>
    {/if}
    <div class="h-px bg-border mx-4"></div>

    <!-- Schedule section (now includes Part of Term) -->
    <button
      onclick={() => toggleSection("schedule")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span>Schedule</span>
      <div class="relative size-4 flex items-center justify-center">
        <ChevronDown
          class="size-4 text-muted-foreground absolute transition-all duration-150 {showScheduleChevron
            ? 'opacity-100 scale-100'
            : 'opacity-0 scale-75'} {expandedSection === 'schedule' ? 'rotate-180' : ''}"
        />
        <span
          class="size-1.5 rounded-full bg-primary absolute transition-all duration-150 {showScheduleChevron
            ? 'opacity-0 scale-75'
            : 'opacity-100 scale-100'}"
        ></span>
      </div>
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

        {#if referenceData.partsOfTerm.length > 0}
          <div class="h-px bg-border"></div>

          <div class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-muted-foreground select-none">Part of Term</span>
            <div class="flex flex-wrap gap-1">
              {#each referenceData.partsOfTerm as item (item.filterValue)}
                <button
                  type="button"
                  aria-pressed={partOfTerm.includes(item.filterValue)}
                  class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
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
      </div>
    {/if}
    <div class="h-px bg-border mx-4"></div>

    <!-- Catalog section (Availability + Course Attributes) -->
    <button
      onclick={() => toggleSection("catalog")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span>Catalog</span>
      <div class="relative size-4 flex items-center justify-center">
        <ChevronDown
          class="size-4 text-muted-foreground absolute transition-all duration-150 {showCatalogChevron
            ? 'opacity-100 scale-100'
            : 'opacity-0 scale-75'} {expandedSection === 'catalog' ? 'rotate-180' : ''}"
        />
        <span
          class="size-1.5 rounded-full bg-primary absolute transition-all duration-150 {showCatalogChevron
            ? 'opacity-0 scale-75'
            : 'opacity-100 scale-100'}"
        ></span>
      </div>
    </button>
    {#if expandedSection === "catalog"}
      <div class="px-4 pb-3 flex flex-col gap-3">
        <AvailabilityFilter bind:campus />

        {#if levelAttributes.length > 0}
          <div class="h-px bg-border"></div>
          <div class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-muted-foreground select-none">Course Level</span>
            <div class="flex flex-wrap gap-1">
              {#each levelAttributes as item (item.filterValue)}
                <button
                  type="button"
                  aria-pressed={attributes.includes(item.filterValue)}
                  class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                         {attributes.includes(item.filterValue)
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                  onclick={() => toggleAttr(item.filterValue)}
                  title={item.description}
                >
                  {getAttributeFilterLabel(item.filterValue)}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        {#if coreAttributes.length > 0}
          <div class="h-px bg-border"></div>
          <div class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-muted-foreground select-none">Core Curriculum</span>
            <div class="flex flex-wrap gap-1">
              {#each coreAttributes as item (item.filterValue)}
                <button
                  type="button"
                  aria-pressed={attributes.includes(item.filterValue)}
                  class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                         {attributes.includes(item.filterValue)
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                  onclick={() => toggleAttr(item.filterValue)}
                  title={item.description}
                >
                  {getAttributeFilterLabel(item.filterValue)}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        {#if specialAttributes.length > 0}
          <div class="h-px bg-border"></div>
          <div class="flex flex-col gap-1.5">
            <span class="text-xs font-medium text-muted-foreground select-none">Designations</span>
            <div class="flex flex-wrap gap-1">
              {#each specialAttributes as item (item.filterValue)}
                <button
                  type="button"
                  aria-pressed={attributes.includes(item.filterValue)}
                  class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                         {attributes.includes(item.filterValue)
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                  onclick={() => toggleAttr(item.filterValue)}
                  title={item.description}
                >
                  {getAttributeFilterLabel(item.filterValue)}
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
    <div class="h-px bg-border mx-4"></div>

    <!-- More section -->
    <button
      onclick={() => toggleSection("more")}
      class="flex items-center justify-between px-4 py-3 text-sm font-medium text-foreground"
    >
      <span>More</span>
      <div class="relative size-4 flex items-center justify-center">
        <ChevronDown
          class="size-4 text-muted-foreground absolute transition-all duration-150 {showMoreChevron
            ? 'opacity-100 scale-100'
            : 'opacity-0 scale-75'} {expandedSection === 'more' ? 'rotate-180' : ''}"
        />
        <span
          class="size-1.5 rounded-full bg-primary absolute transition-all duration-150 {showMoreChevron
            ? 'opacity-0 scale-75'
            : 'opacity-100 scale-100'}"
        ></span>
      </div>
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
          formatPip={formatCourseNumberPip}
          pips
          pipstep={10}
          all="label"
        />
      </div>
    {/if}
  </div>
</BottomSheet>
