<script lang="ts">
import { goto } from "$app/navigation";
import {
  type CodeDescription,
  type SearchOptionsResponse,
  type SearchResponse,
  type SortColumn,
  type SortDirection,
  type Subject,
  client,
} from "$lib/api";
import CourseTable from "$lib/components/CourseTable.svelte";
import FilterChip from "$lib/components/FilterChip.svelte";
import Footer from "$lib/components/Footer.svelte";
import Pagination from "$lib/components/Pagination.svelte";
import SearchFilters from "$lib/components/SearchFilters.svelte";
import SearchStatus, { type SearchMeta } from "$lib/components/SearchStatus.svelte";
import SegmentedChip from "$lib/components/SegmentedChip.svelte";
import { Check, Columns3, RotateCcw } from "@lucide/svelte";
import type { SortingState, VisibilityState } from "@tanstack/table-core";
import { DropdownMenu } from "bits-ui";
import { tick, untrack } from "svelte";
import { fly } from "svelte/transition";

let { data } = $props();

// Read initial state from URL params (intentionally captured once)
const initialParams = untrack(() => new URLSearchParams(data.url.search));

// The default term is the first one returned by the backend (most current)
const defaultTermSlug = untrack(() => data.searchOptions?.terms[0]?.slug ?? "");

// Helper to parse a URL param as a number or null
function parseNumParam(key: string): number | null {
  const v = initialParams.get(key);
  if (v === null || v === "") return null;
  const n = Number(v);
  return Number.isNaN(n) ? null : n;
}

// Default to the first term when no URL param is present
const urlTerm = initialParams.get("term");
let selectedTerm = $state(
  untrack(() => {
    const terms = data.searchOptions?.terms ?? [];
    return urlTerm && terms.some((t) => t.slug === urlTerm) ? urlTerm : defaultTermSlug;
  })
);
let selectedSubjects: string[] = $state(untrack(() => initialParams.getAll("subject")));
let query = $state(initialParams.get("q") ?? "");
let openOnly = $state(initialParams.get("open") === "true");
let offset = $state(Number(initialParams.get("offset")) || 0);
const limit = 25;

// New filter state from URL
let waitCountMax = $state<number | null>(parseNumParam("wait_count_max"));
let days: string[] = $state(initialParams.getAll("days"));
let timeStart = $state<string | null>(initialParams.get("time_start"));
let timeEnd = $state<string | null>(initialParams.get("time_end"));
let instructionalMethod: string[] = $state(initialParams.getAll("instructional_method"));
let campus: string[] = $state(initialParams.getAll("campus"));
let partOfTerm: string[] = $state(initialParams.getAll("part_of_term"));
let attributes: string[] = $state(initialParams.getAll("attributes"));
let creditHourMin = $state<number | null>(parseNumParam("credit_hour_min"));
let creditHourMax = $state<number | null>(parseNumParam("credit_hour_max"));
let instructor = $state(initialParams.get("instructor") ?? "");
let courseNumberMin = $state<number | null>(parseNumParam("course_number_low"));
let courseNumberMax = $state<number | null>(parseNumParam("course_number_high"));

let searchOptions = $state<SearchOptionsResponse | null>(null);

// Sync data prop to local state
$effect(() => {
  searchOptions = data.searchOptions;
});

// Derived from search options
const terms = $derived(searchOptions?.terms ?? []);
const subjects: Subject[] = $derived(searchOptions?.subjects ?? []);
let subjectMap: Record<string, string> = $derived(
  Object.fromEntries(subjects.map((s) => [s.code, s.description]))
);

const referenceData = $derived({
  instructionalMethods: searchOptions?.reference.instructionalMethods ?? [],
  campuses: searchOptions?.reference.campuses ?? [],
  partsOfTerm: searchOptions?.reference.partsOfTerm ?? [],
  attributes: searchOptions?.reference.attributes ?? [],
});

const ranges = $derived(
  searchOptions?.ranges ?? {
    courseNumberMin: 1000,
    courseNumberMax: 9000,
    creditHourMin: 0,
    creditHourMax: 8,
    waitCountMax: 0,
  }
);

// Sorting state — maps TanStack column IDs to server sort params
const SORT_COLUMN_MAP: Record<string, SortColumn> = {
  course_code: "course_code",
  title: "title",
  instructor: "instructor",
  time: "time",
  seats: "seats",
};

let sorting: SortingState = $state(
  (() => {
    const sortBy = initialParams.get("sort_by");
    const sortDir = initialParams.get("sort_dir");
    if (!sortBy) return [];
    return [{ id: sortBy, desc: sortDir === "desc" }];
  })()
);

function handleSortingChange(newSorting: SortingState) {
  sorting = newSorting;
  offset = 0;
}

// Data state
let searchResult: SearchResponse | null = $state(null);
let searchMeta: SearchMeta | null = $state(null);
let loading = $state(false);
let error = $state<string | null>(null);

// Track if we're validating subjects to prevent cascading search
let validatingSubjects = false;

// Fetch new search options when term changes
$effect(() => {
  const term = selectedTerm;
  if (!term) return;
  client
    .getSearchOptions(term)
    .then((opts) => {
      searchOptions = opts;
      // Validate selected subjects against new term's subjects
      const validCodes = new Set(opts.subjects.map((s) => s.code));
      const filtered = selectedSubjects.filter((code) => validCodes.has(code));
      if (filtered.length !== selectedSubjects.length) {
        validatingSubjects = true;
        selectedSubjects = filtered;
        validatingSubjects = false;
      }
    })
    .catch((e) => {
      console.error("Failed to fetch search options:", e);
    });
});

// Centralized throttle configuration - maps trigger source to throttle delay (ms)
const THROTTLE_MS = {
  term: 0,
  subjects: 100,
  query: 300,
  openOnly: 0,
  offset: 0,
  sorting: 0,
  waitCountMax: 300,
  days: 100,
  timeStart: 300,
  timeEnd: 300,
  instructionalMethod: 100,
  campus: 100,
  partOfTerm: 100,
  attributes: 100,
  creditHourMin: 300,
  creditHourMax: 300,
  instructor: 300,
  courseNumberMin: 300,
  courseNumberMax: 300,
} as const;

let searchTimeout: ReturnType<typeof setTimeout> | undefined;
let lastSearchKey = "";
let lastNavigationTime = 0;

function buildSearchKey(): string {
  return [
    selectedTerm,
    selectedSubjects.join(","),
    query,
    openOnly,
    offset,
    JSON.stringify(sorting),
    waitCountMax,
    days.join(","),
    timeStart,
    timeEnd,
    instructionalMethod.join(","),
    campus.join(","),
    partOfTerm.join(","),
    attributes.join(","),
    creditHourMin,
    creditHourMax,
    instructor,
    courseNumberMin,
    courseNumberMax,
  ].join("|");
}

function scheduleSearch(source: keyof typeof THROTTLE_MS) {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    const key = buildSearchKey();
    if (key === lastSearchKey) return;
    performSearch();
  }, THROTTLE_MS[source]);
}

// Separate effects for each trigger source with appropriate throttling
$effect(() => {
  selectedTerm;
  scheduleSearch("term");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  selectedSubjects;
  if (!validatingSubjects) {
    scheduleSearch("subjects");
  }
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  query;
  scheduleSearch("query");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  openOnly;
  scheduleSearch("openOnly");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  offset;
  scheduleSearch("offset");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  sorting;
  scheduleSearch("sorting");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  waitCountMax;
  scheduleSearch("waitCountMax");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  days;
  scheduleSearch("days");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  timeStart;
  scheduleSearch("timeStart");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  timeEnd;
  scheduleSearch("timeEnd");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  instructionalMethod;
  scheduleSearch("instructionalMethod");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  campus;
  scheduleSearch("campus");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  partOfTerm;
  scheduleSearch("partOfTerm");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  attributes;
  scheduleSearch("attributes");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  creditHourMin;
  scheduleSearch("creditHourMin");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  creditHourMax;
  scheduleSearch("creditHourMax");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  instructor;
  scheduleSearch("instructor");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  courseNumberMin;
  scheduleSearch("courseNumberMin");
  return () => clearTimeout(searchTimeout);
});

$effect(() => {
  courseNumberMax;
  scheduleSearch("courseNumberMax");
  return () => clearTimeout(searchTimeout);
});

// Build a filter key that excludes offset/sorting — used to detect filter changes for offset reset
function buildFilterKey(): string {
  return [
    selectedTerm,
    selectedSubjects.join(","),
    query,
    openOnly,
    waitCountMax,
    days.join(","),
    timeStart,
    timeEnd,
    instructionalMethod.join(","),
    campus.join(","),
    partOfTerm.join(","),
    attributes.join(","),
    creditHourMin,
    creditHourMax,
    instructor,
    courseNumberMin,
    courseNumberMax,
  ].join("|");
}

// Reset offset when filters change (not offset itself)
let prevFilters = $state("");
$effect(() => {
  const key = buildFilterKey();
  if (prevFilters && key !== prevFilters) {
    offset = 0;
  }
  prevFilters = key;
});

async function performSearch() {
  if (!selectedTerm) return;
  const key = buildSearchKey();
  lastSearchKey = key;
  loading = true;
  error = null;

  const sortBy = sorting.length > 0 ? SORT_COLUMN_MAP[sorting[0].id] : undefined;
  const sortDir: SortDirection | undefined =
    sorting.length > 0 ? (sorting[0].desc ? "desc" : "asc") : undefined;

  // Build URL params for browser URL sync
  const params = new URLSearchParams();
  for (const s of selectedSubjects) {
    params.append("subject", s);
  }
  if (query) params.set("q", query);
  if (openOnly) params.set("open", "true");
  if (offset > 0) params.set("offset", String(offset));
  if (sortBy) params.set("sort_by", sortBy);
  if (sortDir && sortBy) params.set("sort_dir", sortDir);
  if (waitCountMax !== null) params.set("wait_count_max", String(waitCountMax));
  for (const d of days) params.append("days", d);
  if (timeStart) params.set("time_start", timeStart);
  if (timeEnd) params.set("time_end", timeEnd);
  for (const m of instructionalMethod) params.append("instructional_method", m);
  for (const c of campus) params.append("campus", c);
  for (const p of partOfTerm) params.append("part_of_term", p);
  for (const a of attributes) params.append("attributes", a);
  if (creditHourMin !== null) params.set("credit_hour_min", String(creditHourMin));
  if (creditHourMax !== null) params.set("credit_hour_max", String(creditHourMax));
  if (instructor) params.set("instructor", instructor);
  if (courseNumberMin !== null) params.set("course_number_low", String(courseNumberMin));
  if (courseNumberMax !== null) params.set("course_number_high", String(courseNumberMax));

  // Include term in URL only when it differs from the default or other params are active
  const hasOtherParams = params.size > 0;
  if (selectedTerm !== defaultTermSlug || hasOtherParams) {
    params.set("term", selectedTerm);
  }

  // Smart batching: batch rapid changes (<2.5s) into one history entry,
  // but create new entries for deliberate filter changes
  const BATCH_WINDOW_MS = 2500;
  const now = Date.now();
  const shouldBatch = now - lastNavigationTime < BATCH_WINDOW_MS;

  goto(`?${params.toString()}`, {
    replaceState: shouldBatch,
    noScroll: true,
    keepFocus: true,
  });

  lastNavigationTime = now;

  const t0 = performance.now();
  try {
    const result = await client.searchCourses({
      term: selectedTerm,
      subject: selectedSubjects.length > 0 ? selectedSubjects : [],
      q: query || undefined,
      openOnly: openOnly || false,
      limit,
      offset,
      sortBy,
      sortDir,
      waitCountMax: waitCountMax ?? undefined,
      days: days.length > 0 ? days : undefined,
      timeStart: timeStart ?? undefined,
      timeEnd: timeEnd ?? undefined,
      instructionalMethod: instructionalMethod.length > 0 ? instructionalMethod : undefined,
      campus: campus.length > 0 ? campus : undefined,
      partOfTerm: partOfTerm.length > 0 ? partOfTerm : undefined,
      attributes: attributes.length > 0 ? attributes : undefined,
      creditHourMin: creditHourMin ?? undefined,
      creditHourMax: creditHourMax ?? undefined,
      instructor: instructor || undefined,
      courseNumberLow: courseNumberMin ?? undefined,
      courseNumberHigh: courseNumberMax ?? undefined,
    });

    const applyUpdate = () => {
      searchResult = result;
      searchMeta = {
        totalCount: result.totalCount,
        durationMs: performance.now() - t0,
        timestamp: new Date(),
      };
    };

    // Scoped view transitions only affect the table element, so filters and
    // other controls remain fully interactive. Document-level transitions
    // apply visibility:hidden to the entire page for the transition duration,
    // blocking all pointer interactions — so we skip those entirely and let
    // Svelte's animate:flip / in:fade handle the visual update instead.
    const tableEl = document.querySelector("[data-search-results]") as HTMLElement | null;

    if (tableEl && "startViewTransition" in tableEl) {
      const transition = (tableEl as any).startViewTransition(async () => {
        applyUpdate();
        await tick();
      });
      await transition.updateCallbackDone;
    } else {
      applyUpdate();
    }
  } catch (e) {
    error = e instanceof Error ? e.message : "Search failed";
  } finally {
    loading = false;
  }
}

function handlePageChange(newOffset: number) {
  offset = newOffset;
}

// Column visibility state (lifted from CourseTable)
let columnVisibility: VisibilityState = $state({});

function resetColumnVisibility() {
  columnVisibility = {};
}

let hasCustomVisibility = $derived(Object.values(columnVisibility).some((v) => v === false));

// All possible column IDs for the View dropdown
const columnDefs = [
  { id: "crn", label: "CRN" },
  { id: "course_code", label: "Course" },
  { id: "title", label: "Title" },
  { id: "instructor", label: "Instructor" },
  { id: "time", label: "Time" },
  { id: "location", label: "Location" },
  { id: "seats", label: "Seats" },
];

// Chip helpers
const DAY_ABBREV: Record<string, string> = {
  monday: "M",
  tuesday: "T",
  wednesday: "W",
  thursday: "Th",
  friday: "F",
  saturday: "Sa",
  sunday: "Su",
};

function formatDaysChip(d: string[]): string {
  return d.map((day) => DAY_ABBREV[day] ?? day).join("");
}

function formatTimeChip(start: string | null, end: string | null): string {
  const fmt = (t: string) => {
    if (t.length !== 4) return t;
    const h = parseInt(t.slice(0, 2), 10);
    const m = t.slice(2);
    const period = h >= 12 ? "PM" : "AM";
    const dh = h === 0 ? 12 : h > 12 ? h - 12 : h;
    return `${dh}:${m} ${period}`;
  };
  if (start && end) return `${fmt(start)} – ${fmt(end)}`;
  if (start) return `After ${fmt(start)}`;
  if (end) return `Before ${fmt(end)}`;
  return "";
}

function formatMultiChip(codes: string[], refItems: CodeDescription[]): string {
  const lookup = new Map(refItems.map((r) => [r.code, r.description]));
  const first = lookup.get(codes[0]) ?? codes[0];
  if (codes.length === 1) return first;
  return `${first} + ${codes.length - 1} more`;
}

let activeFilterCount = $derived(
  (selectedSubjects.length > 0 ? 1 : 0) +
    (openOnly ? 1 : 0) +
    (waitCountMax !== null ? 1 : 0) +
    (days.length > 0 ? 1 : 0) +
    (timeStart !== null || timeEnd !== null ? 1 : 0) +
    (instructionalMethod.length > 0 ? 1 : 0) +
    (campus.length > 0 ? 1 : 0) +
    (partOfTerm.length > 0 ? 1 : 0) +
    (attributes.length > 0 ? 1 : 0) +
    (creditHourMin !== null || creditHourMax !== null ? 1 : 0) +
    (instructor !== "" ? 1 : 0) +
    (courseNumberMin !== null || courseNumberMax !== null ? 1 : 0)
);

function removeSubject(code: string) {
  selectedSubjects = selectedSubjects.filter((s) => s !== code);
}

function clearAllFilters() {
  selectedSubjects = [];
  openOnly = false;
  waitCountMax = null;
  days = [];
  timeStart = null;
  timeEnd = null;
  instructionalMethod = [];
  campus = [];
  partOfTerm = [];
  attributes = [];
  creditHourMin = null;
  creditHourMax = null;
  instructor = "";
  courseNumberMin = null;
  courseNumberMax = null;
}
</script>

<div class="min-h-screen flex flex-col items-center px-5 pb-5 pt-20">
    <div class="w-full max-w-6xl flex flex-col pt-2">
        <!-- Chips bar: status | chips | view button -->
        <div class="flex items-end gap-3 min-h-7">
            <SearchStatus meta={searchMeta} {loading} />

            <!-- Active filter chips -->
            <div
                class="flex items-center gap-1.5 flex-1 min-w-0 flex-wrap pb-1.5"
            >
                {#if selectedSubjects.length > 0}
                    <SegmentedChip
                        segments={selectedSubjects}
                        onRemoveSegment={removeSubject}
                    />
                {/if}
                {#if openOnly}
                    <FilterChip
                        label="Open only"
                        onRemove={() => (openOnly = false)}
                    />
                {/if}
                {#if waitCountMax !== null}
                    <FilterChip
                        label="Waitlist ≤ {waitCountMax}"
                        onRemove={() => (waitCountMax = null)}
                    />
                {/if}
                {#if days.length > 0}
                    <FilterChip
                        label={formatDaysChip(days)}
                        onRemove={() => (days = [])}
                    />
                {/if}
                {#if timeStart !== null || timeEnd !== null}
                    <FilterChip
                        label={formatTimeChip(timeStart, timeEnd)}
                        onRemove={() => {
                            timeStart = null;
                            timeEnd = null;
                        }}
                    />
                {/if}
                {#if instructionalMethod.length > 0}
                    <FilterChip
                        label={formatMultiChip(
                            instructionalMethod,
                            referenceData.instructionalMethods,
                        )}
                        onRemove={() => (instructionalMethod = [])}
                    />
                {/if}
                {#if campus.length > 0}
                    <FilterChip
                        label={formatMultiChip(campus, referenceData.campuses)}
                        onRemove={() => (campus = [])}
                    />
                {/if}
                {#if partOfTerm.length > 0}
                    <FilterChip
                        label={formatMultiChip(
                            partOfTerm,
                            referenceData.partsOfTerm,
                        )}
                        onRemove={() => (partOfTerm = [])}
                    />
                {/if}
                {#if attributes.length > 0}
                    <FilterChip
                        label={formatMultiChip(
                            attributes,
                            referenceData.attributes,
                        )}
                        onRemove={() => (attributes = [])}
                    />
                {/if}
                {#if creditHourMin !== null || creditHourMax !== null}
                    <FilterChip
                        label={creditHourMin !== null && creditHourMax !== null
                            ? `${creditHourMin}–${creditHourMax} credits`
                            : creditHourMin !== null
                              ? `≥ ${creditHourMin} credits`
                              : `≤ ${creditHourMax} credits`}
                        onRemove={() => {
                            creditHourMin = null;
                            creditHourMax = null;
                        }}
                    />
                {/if}
                {#if instructor !== ""}
                    <FilterChip
                        label="Instructor: {instructor}"
                        onRemove={() => (instructor = "")}
                    />
                {/if}
                {#if courseNumberMin !== null || courseNumberMax !== null}
                    <FilterChip
                        label={courseNumberMin !== null &&
                        courseNumberMax !== null
                            ? `Course ${courseNumberMin}–${courseNumberMax}`
                            : courseNumberMin !== null
                              ? `Course ≥ ${courseNumberMin}`
                              : `Course ≤ ${courseNumberMax}`}
                        onRemove={() => {
                            courseNumberMin = null;
                            courseNumberMax = null;
                        }}
                    />
                {/if}
                {#if activeFilterCount >= 2}
                    <button
                        type="button"
                        class="text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer ml-1"
                        onclick={clearAllFilters}
                    >
                        Clear all
                    </button>
                {/if}
            </div>

            <!-- View columns dropdown (moved from CourseTable) -->
            <div class="pb-1.5">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger
                        class="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer shrink-0"
                    >
                        <Columns3 class="size-3.5" />
                        View
                    </DropdownMenu.Trigger>
                <DropdownMenu.Portal>
                    <DropdownMenu.Content
                        class="z-50 min-w-40 rounded-md border border-border bg-card p-1 text-card-foreground shadow-lg"
                        align="end"
                        sideOffset={4}
                        forceMount
                    >
                        {#snippet child({ wrapperProps, props, open })}
                            {#if open}
                                <div {...wrapperProps}>
                                    <div
                                        {...props}
                                        transition:fly={{
                                            duration: 150,
                                            y: -10,
                                        }}
                                    >
                                        <DropdownMenu.Group>
                                            <DropdownMenu.GroupHeading
                                                class="px-2 py-1.5 text-xs font-medium text-muted-foreground"
                                            >
                                                Toggle columns
                                            </DropdownMenu.GroupHeading>
                                            {#each columnDefs as col}
                                                <DropdownMenu.CheckboxItem
                                                    checked={columnVisibility[
                                                        col.id
                                                    ] !== false}
                                                    closeOnSelect={false}
                                                    onCheckedChange={(
                                                        checked,
                                                    ) => {
                                                        columnVisibility = {
                                                            ...columnVisibility,
                                                            [col.id]: checked,
                                                        };
                                                    }}
                                                    class="relative flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground"
                                                >
                                                    {#snippet children({
                                                        checked,
                                                    })}
                                                        <span
                                                            class="flex size-4 items-center justify-center rounded-sm border border-border"
                                                        >
                                                            {#if checked}
                                                                <Check
                                                                    class="size-3"
                                                                />
                                                            {/if}
                                                        </span>
                                                        {col.label}
                                                    {/snippet}
                                                </DropdownMenu.CheckboxItem>
                                            {/each}
                                        </DropdownMenu.Group>
                                        {#if hasCustomVisibility}
                                            <DropdownMenu.Separator
                                                class="mx-1 my-1 h-px bg-border"
                                            />
                                            <DropdownMenu.Item
                                                class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground"
                                                onSelect={resetColumnVisibility}
                                            >
                                                <RotateCcw class="size-3.5" />
                                                Reset to default
                                            </DropdownMenu.Item>
                                        {/if}
                                    </div>
                                </div>
                            {/if}
                        {/snippet}
                    </DropdownMenu.Content>
                </DropdownMenu.Portal>
            </DropdownMenu.Root>
            </div>
        </div>

        <!-- Filter bar -->
        <div class="flex flex-col gap-2 pb-4">
            <SearchFilters
                {terms}
                {subjects}
                bind:selectedTerm
                bind:selectedSubjects
                bind:query
                bind:openOnly
                bind:waitCountMax
                bind:days
                bind:timeStart
                bind:timeEnd
                bind:instructionalMethod
                bind:campus
                bind:partOfTerm
                bind:attributes
                bind:creditHourMin
                bind:creditHourMax
                bind:instructor
                bind:courseNumberMin
                bind:courseNumberMax
                {referenceData}
                ranges={{
                    courseNumber: { min: ranges.courseNumberMin, max: ranges.courseNumberMax },
                    creditHours: { min: ranges.creditHourMin, max: ranges.creditHourMax },
                    waitCount: { max: ranges.waitCountMax },
                }}
            />
        </div>

        <!-- Results -->
        {#if error}
            <div class="text-center py-8">
                <p class="text-status-red">{error}</p>
                <button
                    onclick={() => performSearch()}
                    class="mt-2 text-sm text-muted-foreground hover:underline"
                >
                    Retry
                </button>
            </div>
        {:else}
            <CourseTable
                courses={searchResult?.courses ?? []}
                {loading}
                {sorting}
                onSortingChange={handleSortingChange}
                manualSorting={true}
                {subjectMap}
                {limit}
                bind:columnVisibility
            />

            {#if searchResult}
                <Pagination
                    totalCount={searchResult.totalCount}
                    {offset}
                    {limit}
                    {loading}
                    onPageChange={handlePageChange}
                />
            {/if}
        {/if}

        <!-- Footer -->
        <Footer />
    </div>
</div>
