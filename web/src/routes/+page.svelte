<script lang="ts">
import { goto } from "$app/navigation";
import { type SearchOptionsResponse, type SearchResponse, type Subject, client } from "$lib/api";
import { CourseTable } from "$lib/components/course-table";
import { buildAttributeMap, setCourseDetailContext } from "$lib/components/course-detail/context";
import FilterChip from "$lib/components/FilterChip.svelte";
import Footer from "$lib/components/Footer.svelte";
import Pagination from "$lib/components/Pagination.svelte";
import SearchFilters from "$lib/components/SearchFilters.svelte";
import SearchStatus from "$lib/components/SearchStatus.svelte";
import SegmentedChip from "$lib/components/SegmentedChip.svelte";
import { Check, Columns3, RotateCcw } from "@lucide/svelte";
import type { SortingState, VisibilityState } from "@tanstack/table-core";
import { DropdownMenu } from "bits-ui";
import { SvelteSet } from "svelte/reactivity";
import { tick, untrack } from "svelte";
import { type ScrollMetrics, maskGradient as computeMaskGradient } from "$lib/scroll-fade";
import {
  getCampusFilterLabel,
  getAttributeFilterLabel,
  getPartOfTermFilterLabel,
} from "$lib/labels";
import { fly } from "svelte/transition";
import {
  SearchFilters as SearchFiltersStore,
  setFiltersContext,
} from "$lib/stores/search-filters.svelte";

interface PageLoadData {
  searchOptions: SearchOptionsResponse | null;
  url: URL;
}

let { data }: { data: PageLoadData } = $props();

/** No-op function to register Svelte reactivity dependencies for `$effect` tracking */
function track(..._deps: unknown[]) {
  /* noop */
}

let courseTableRef: CourseTable | undefined = $state();

// Read initial state from URL params (intentionally captured once)
const initialParams = untrack(() => new URLSearchParams(data.url.search));

// The default term is the first one returned by the backend (most current)
const defaultTermSlug = untrack(() => data.searchOptions?.terms[0]?.slug ?? "");

// Default to the first term when no URL param is present
const urlTerm = initialParams.get("term");
let selectedTerm = $state(
  untrack(() => {
    const terms = data.searchOptions?.terms ?? [];
    return urlTerm && terms.some((t) => t.slug === urlTerm) ? urlTerm : defaultTermSlug;
  })
);

// Initialize filters from URL params and provide via context
const filters = new SearchFiltersStore();
setFiltersContext(filters);
untrack(() => {
  const validSubjects = new Set(data.searchOptions?.subjects.map((s) => s.code));
  filters.fromURLParams(initialParams, validSubjects);
});

let offset = $state(Number(initialParams.get("offset")) || 0);
const limit = 25;

// svelte-ignore state_referenced_locally
let searchOptions = $state<SearchOptionsResponse | null>(data.searchOptions);

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

// Provide attribute descriptions and navigation to CourseDetail components via context
const attributeMap = $derived(buildAttributeMap(referenceData.attributes));
const courseDetailCtx: import("$lib/components/course-detail/context").CourseDetailContext = {
  get attributeMap() {
    return attributeMap;
  },
  navigateToSection: null,
};
setCourseDetailContext(courseDetailCtx);

// Wire up navigation callback once CourseTable is mounted
$effect(() => {
  if (courseTableRef) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-return, @typescript-eslint/no-unsafe-call
    courseDetailCtx.navigateToSection = (crn: string) => courseTableRef?.navigateToSection(crn);
  }
});

const ranges = $derived(
  searchOptions?.ranges ?? {
    courseNumberMin: 0,
    courseNumberMax: 9000,
    creditHourMin: 0,
    creditHourMax: 8,
    waitCountMax: 0,
  }
);

// Sorting state
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
let searchMeta: { totalCount: number; durationMs: number; timestamp: Date } | null = $state(null);
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
      const filtered = filters.subject.filter((code) => validCodes.has(code));
      if (filtered.length !== filters.subject.length) {
        validatingSubjects = true;
        filters.subject = filtered;
        validatingSubjects = false;
      }
    })
    .catch((e) => {
      console.error("Failed to fetch search options:", e);
    });
});

// Search throttling
let searchTimeout: ReturnType<typeof setTimeout> | undefined;
let lastSearchKey = "";
let lastNavigationTime = 0;

// Unified search effect - watches all filter properties, term, offset, and sorting
$effect(() => {
  // Track dependencies
  const term = selectedTerm;
  const filterKey = filters.toSearchKey();
  track(offset, sorting);

  // Skip search if we're in subject validation
  if (validatingSubjects) return;

  // Build complete search key including pagination and sorting
  const searchKey = [term, filterKey, offset, JSON.stringify(sorting)].join("|");

  // Throttle based on what changed
  const THROTTLE_MS = 300;
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    if (searchKey === lastSearchKey) return;
    void performSearch();
  }, THROTTLE_MS);

  return () => clearTimeout(searchTimeout);
});

// Reset offset when filters change (not offset/sorting itself)
let prevFilters = $state("");
$effect(() => {
  const key = filters.toFilterKey();
  if (prevFilters && key !== prevFilters) {
    offset = 0;
  }
  prevFilters = key;
});

async function performSearch() {
  if (!selectedTerm) return;
  const key = [selectedTerm, filters.toSearchKey(), offset, JSON.stringify(sorting)].join("|");
  lastSearchKey = key;
  loading = true;
  error = null;

  // Build URL params for browser URL sync
  const params = filters.toURLParams();
  if (offset > 0) params.set("offset", String(offset));
  if (sorting.length > 0) {
    params.set("sort_by", sorting[0].id);
    params.set("sort_dir", sorting[0].desc ? "desc" : "asc");
  }

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

  void goto(`?${params.toString()}`, {
    replaceState: shouldBatch,
    noScroll: true,
    keepFocus: true,
  });

  lastNavigationTime = now;

  const t0 = performance.now();
  try {
    const apiParams = filters.toAPIParams(selectedTerm, limit, offset, sorting);
    const result = await client.searchCourses(apiParams);

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
    const tableEl = document.querySelector("[data-search-results]");

    if (tableEl && "startViewTransition" in tableEl) {
      const startViewTransition = (
        tableEl as unknown as {
          startViewTransition: (cb: () => Promise<void>) => {
            updateCallbackDone: Promise<void>;
          };
        }
      ).startViewTransition;
      const transition = startViewTransition(async () => {
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

// Responsive column hiding: hide CRN and Location in the sm-to-md range (640-768px)
let isCompactTable = $state(false);
// Track columns the user has explicitly toggled so we don't override their choices
let userToggledColumns = new SvelteSet<string>();

$effect(() => {
  if (typeof window === "undefined") return;
  const mql = window.matchMedia("(min-width: 640px) and (max-width: 767px)");
  isCompactTable = mql.matches;
  const handler = (e: MediaQueryListEvent) => {
    isCompactTable = e.matches;
  };
  mql.addEventListener("change", handler);
  return () => mql.removeEventListener("change", handler);
});

// Auto-hide/show columns based on compact mode (only for columns the user hasn't manually toggled)
const autoHideColumns = ["crn", "location"];
$effect(() => {
  const compact = isCompactTable;
  const toggled = userToggledColumns;
  const current = untrack(() => columnVisibility);
  let changed = false;
  const next = { ...current };
  for (const col of autoHideColumns) {
    if (toggled.has(col)) continue;
    const shouldHide = compact;
    if (shouldHide && next[col] !== false) {
      next[col] = false;
      changed = true;
    } else if (!shouldHide && next[col] === false) {
      delete next[col];
      changed = true;
    }
  }
  if (changed) columnVisibility = next;
});

function resetColumnVisibility() {
  columnVisibility = {};
  userToggledColumns = new SvelteSet();
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

function formatMultiChip(codes: string[], labelFn: (filterValue: string) => string): string {
  const first = labelFn(codes[0]);
  if (codes.length === 1) return first;
  return `${first} + ${codes.length - 1} more`;
}

// Group instructional methods by type for separate chips
interface FormatChipGroup {
  type: "InPerson" | "Online" | "Hybrid" | "Independent";
  codes: string[];
  label: string;
}

const VARIANT_LABELS: Record<string, string> = {
  "Online.Async": "Async",
  "Online.Sync": "Sync",
  "Online.Mixed": "Mix",
  "Hybrid.Half": "Half",
  "Hybrid.OneThird": "One Third",
  "Hybrid.TwoThirds": "Two Thirds",
};

function groupInstructionalMethods(methods: string[]): FormatChipGroup[] {
  const groups: FormatChipGroup[] = [];

  // Check for simple types
  if (methods.includes("InPerson")) {
    groups.push({ type: "InPerson", codes: ["InPerson"], label: "In Person" });
  }
  if (methods.includes("Independent")) {
    groups.push({ type: "Independent", codes: ["Independent"], label: "Independent" });
  }

  // Check for Online variants
  const onlineCodes = methods.filter((m) => m.startsWith("Online."));
  if (onlineCodes.length > 0) {
    const variantLabels = onlineCodes.map((c) => VARIANT_LABELS[c] || c);
    groups.push({
      type: "Online",
      codes: onlineCodes,
      label: `Online: ${variantLabels.join(", ")}`,
    });
  }

  // Check for Hybrid variants
  const hybridCodes = methods.filter((m) => m.startsWith("Hybrid."));
  if (hybridCodes.length > 0) {
    const variantLabels = hybridCodes.map((c) => VARIANT_LABELS[c] || c);
    groups.push({
      type: "Hybrid",
      codes: hybridCodes,
      label: `Hybrid: ${variantLabels.join(", ")}`,
    });
  }

  return groups;
}

function removeFormatGroup(group: FormatChipGroup) {
  filters.instructionalMethod = filters.instructionalMethod.filter((m) => !group.codes.includes(m));
}

let formatChipGroups = $derived(groupInstructionalMethods(filters.instructionalMethod));

function removeSubject(code: string) {
  filters.subject = filters.subject.filter((s) => s !== code);
}

// Scroll-based fade mask for chips container
let chipsContainer: HTMLDivElement | undefined = $state();
let scrollMetrics = $state<ScrollMetrics>({ scrollLeft: 0, scrollWidth: 0, clientWidth: 0 });

const maskGradient = $derived(computeMaskGradient(scrollMetrics));

function updateScrollMetrics() {
  if (!chipsContainer) return;
  scrollMetrics = {
    scrollLeft: chipsContainer.scrollLeft,
    scrollWidth: chipsContainer.scrollWidth,
    clientWidth: chipsContainer.clientWidth,
  };
}

$effect(() => {
  if (!chipsContainer) return;
  const el = chipsContainer; // capture for cleanup

  const ro = new ResizeObserver(updateScrollMetrics);
  ro.observe(el);

  el.addEventListener("scroll", updateScrollMetrics, { passive: true });
  updateScrollMetrics(); // initial measurement

  return () => {
    ro.disconnect();
    el.removeEventListener("scroll", updateScrollMetrics);
  };
});
</script>

<div class="min-h-screen flex flex-col items-center px-3 md:px-5 pb-5 pt-20">
    <div class="w-full max-w-6xl flex flex-col pt-2">
        <!-- Chips bar: status | chips | view button -->
        <div class="flex flex-col md:flex-row md:items-end gap-1 md:gap-3 min-h-7">
            <SearchStatus meta={searchMeta} {loading} />

            <!-- Active filter chips -->
            <div
                bind:this={chipsContainer}
                class="flex items-center gap-1.5 flex-1 min-w-0
                       flex-nowrap overflow-x-auto md:flex-wrap md:overflow-x-visible
                       -mx-3 px-3 md:mx-0 md:px-0
                       pb-1.5 scrollbar-none"
                style:mask-image={maskGradient}
                style:-webkit-mask-image={maskGradient}
            >
                {#if filters.subject.length > 0}
                    <SegmentedChip segments={filters.subject} onRemoveSegment={removeSubject} />
                {/if}
                {#if filters.openOnly}
                    <FilterChip label="Open only" onRemove={() => (filters.openOnly = false)} />
                {/if}
                {#if filters.waitCountMax !== null}
                    <FilterChip label="Waitlist ≤ {filters.waitCountMax}" onRemove={() => (filters.waitCountMax = null)} />
                {/if}
                {#if filters.days.length > 0}
                    <FilterChip label={formatDaysChip(filters.days)} onRemove={() => (filters.days = [])} />
                {/if}
                {#if filters.timeStart !== null || filters.timeEnd !== null}
                    <FilterChip
                        label={formatTimeChip(filters.timeStart, filters.timeEnd)}
                        onRemove={() => {
                            filters.timeStart = null;
                            filters.timeEnd = null;
                        }}
                    />
                {/if}
                {#each formatChipGroups as group (group.type)}
                    <FilterChip label={group.label} onRemove={() => removeFormatGroup(group)} />
                {/each}
                {#if filters.campus.length > 0}
                    <FilterChip
                        label={formatMultiChip(filters.campus, getCampusFilterLabel)}
                        onRemove={() => (filters.campus = [])}
                    />
                {/if}
                {#if filters.partOfTerm.length > 0}
                    <FilterChip
                        label={formatMultiChip(filters.partOfTerm, getPartOfTermFilterLabel)}
                        onRemove={() => (filters.partOfTerm = [])}
                    />
                {/if}
                {#if filters.attributes.length > 0}
                    <FilterChip
                        label={formatMultiChip(filters.attributes, getAttributeFilterLabel)}
                        onRemove={() => (filters.attributes = [])}
                    />
                {/if}
                {#if filters.creditHourMin !== null || filters.creditHourMax !== null}
                    <FilterChip
                        label={filters.creditHourMin !== null && filters.creditHourMax !== null
                            ? `${filters.creditHourMin}–${filters.creditHourMax} credits`
                            : filters.creditHourMin !== null
                              ? `≥ ${filters.creditHourMin} credits`
                              : `≤ ${filters.creditHourMax} credits`}
                        onRemove={() => {
                            filters.creditHourMin = null;
                            filters.creditHourMax = null;
                        }}
                    />
                {/if}
                {#if filters.instructor !== ""}
                    <FilterChip label="Instructor: {filters.instructor}" onRemove={() => (filters.instructor = "")} />
                {/if}
                {#if filters.courseNumberLow !== null || filters.courseNumberHigh !== null}
                    <FilterChip
                        label={filters.courseNumberLow !== null && filters.courseNumberHigh !== null
                            ? `Course ${filters.courseNumberLow}–${filters.courseNumberHigh}`
                            : filters.courseNumberLow !== null
                              ? `Course ≥ ${filters.courseNumberLow}`
                              : `Course ≤ ${filters.courseNumberHigh}`}
                        onRemove={() => {
                            filters.courseNumberLow = null;
                            filters.courseNumberHigh = null;
                        }}
                    />
                {/if}
                {#if filters.activeCount >= 2}
                    <button
                        type="button"
                        class="text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer select-none ml-1 shrink-0"
                        onclick={() => filters.clear()}
                    >
                        Clear all
                    </button>
                {/if}
                <!-- Trailing spacer so last chip scrolls past the fade mask -->
                <div class="shrink-0 w-6 md:hidden" aria-hidden="true"></div>
            </div>

            <!-- View columns dropdown (moved from CourseTable) -->
            <div class="hidden md:block pb-1.5">
                <DropdownMenu.Root>
                    <DropdownMenu.Trigger
                        class="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer select-none shrink-0"
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
                                                    class="px-2 py-1.5 text-xs font-medium text-muted-foreground select-none"
                                                >
                                                    Toggle columns
                                                </DropdownMenu.GroupHeading>
                                                {#each columnDefs as col (col.id)}
                                                    <DropdownMenu.CheckboxItem
                                                        checked={columnVisibility[col.id] !== false}
                                                        closeOnSelect={false}
                                                        onCheckedChange={(checked) => {
                                                            userToggledColumns.add(col.id);
                                                            columnVisibility = {
                                                                ...columnVisibility,
                                                                [col.id]: checked,
                                                            };
                                                        }}
                                                        class="relative flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground"
                                                    >
                                                        {#snippet children({ checked })}
                                                            <span
                                                                class="flex size-4 items-center justify-center rounded-sm border border-border"
                                                            >
                                                                {#if checked}
                                                                    <Check class="size-3" />
                                                                {/if}
                                                            </span>
                                                            {col.label}
                                                        {/snippet}
                                                    </DropdownMenu.CheckboxItem>
                                                {/each}
                                            </DropdownMenu.Group>
                                            {#if hasCustomVisibility}
                                                <DropdownMenu.Separator class="mx-1 my-1 h-px bg-border" />
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
                bind:this={courseTableRef}
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
