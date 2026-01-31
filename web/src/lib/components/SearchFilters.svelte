<script lang="ts">
import type { CodeDescription, Subject, Term } from "$lib/api";
import AttributesPopover from "./AttributesPopover.svelte";
import MorePopover from "./MorePopover.svelte";
import SchedulePopover from "./SchedulePopover.svelte";
import StatusPopover from "./StatusPopover.svelte";
import SubjectCombobox from "./SubjectCombobox.svelte";
import TermCombobox from "./TermCombobox.svelte";

let {
  terms,
  subjects,
  selectedTerm = $bindable(),
  selectedSubjects = $bindable(),
  query = $bindable(),
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
  courseNumberLow = $bindable(),
  courseNumberHigh = $bindable(),
  referenceData,
}: {
  terms: Term[];
  subjects: Subject[];
  selectedTerm: string;
  selectedSubjects: string[];
  query: string;
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
  courseNumberLow: number | null;
  courseNumberHigh: number | null;
  referenceData: {
    instructionalMethods: CodeDescription[];
    campuses: CodeDescription[];
    partsOfTerm: CodeDescription[];
    attributes: CodeDescription[];
  };
} = $props();
</script>

<!-- Row 1: Primary filters -->
<div class="flex flex-wrap gap-3 items-start">
  <TermCombobox {terms} bind:value={selectedTerm} />

  <SubjectCombobox {subjects} bind:value={selectedSubjects} />

  <input
    type="text"
    placeholder="Search courses..."
    aria-label="Search courses"
    bind:value={query}
    class="h-9 border border-border bg-card text-foreground rounded-md px-3 text-sm flex-1 min-w-[200px]
           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background
           transition-colors"
  />
</div>

<!-- Row 2: Category popovers -->
<div class="flex flex-wrap gap-2 items-center">
  <StatusPopover bind:openOnly bind:waitCountMax />
  <SchedulePopover bind:days bind:timeStart bind:timeEnd />
  <AttributesPopover
    bind:instructionalMethod
    bind:campus
    bind:partOfTerm
    bind:attributes
    {referenceData}
  />
  <MorePopover
    bind:creditHourMin
    bind:creditHourMax
    bind:instructor
    bind:courseNumberLow
    bind:courseNumberHigh
  />
</div>
