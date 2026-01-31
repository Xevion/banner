<script lang="ts">
import { goto } from "$app/navigation";
import {
  type SearchResponse,
  type SortColumn,
  type SortDirection,
  type Subject,
  client,
} from "$lib/api";
import CourseTable from "$lib/components/CourseTable.svelte";
import Footer from "$lib/components/Footer.svelte";
import Pagination from "$lib/components/Pagination.svelte";
import SearchFilters from "$lib/components/SearchFilters.svelte";
import SearchStatus, { type SearchMeta } from "$lib/components/SearchStatus.svelte";
import type { SortingState } from "@tanstack/table-core";
import { tick, untrack } from "svelte";

let { data } = $props();

// Read initial state from URL params (intentionally captured once)
const initialParams = untrack(() => new URLSearchParams(data.url.search));

// The default term is the first one returned by the backend (most current)
const defaultTermSlug = untrack(() => data.terms[0]?.slug ?? "");

// Default to the first term when no URL param is present
const urlTerm = initialParams.get("term");
let selectedTerm = $state(
  untrack(() => (urlTerm && data.terms.some((t) => t.slug === urlTerm) ? urlTerm : defaultTermSlug))
);
let selectedSubjects: string[] = $state(untrack(() => initialParams.getAll("subject")));
let query = $state(initialParams.get("q") ?? "");
let openOnly = $state(initialParams.get("open") === "true");
let offset = $state(Number(initialParams.get("offset")) || 0);
const limit = 25;

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
let subjects: Subject[] = $state([]);
let subjectMap: Record<string, string> = $derived(
  Object.fromEntries(subjects.map((s) => [s.code, s.description]))
);
let searchResult: SearchResponse | null = $state(null);
let searchMeta: SearchMeta | null = $state(null);
let loading = $state(false);
let error = $state<string | null>(null);

// Track if we're validating subjects to prevent cascading search
let validatingSubjects = false;

// Fetch subjects when term changes
$effect(() => {
  const term = selectedTerm;
  if (!term) return;
  client
    .getSubjects(term)
    .then((s) => {
      subjects = s;
      const validCodes = new Set(s.map((sub) => sub.code));
      const filtered = selectedSubjects.filter((code) => validCodes.has(code));
      if (filtered.length !== selectedSubjects.length) {
        validatingSubjects = true;
        selectedSubjects = filtered;
        validatingSubjects = false;
      }
    })
    .catch((e) => {
      console.error("Failed to fetch subjects:", e);
    });
});

// Centralized throttle configuration - maps trigger source to throttle delay (ms)
const THROTTLE_MS = {
  term: 0, // Immediate
  subjects: 100, // Short delay for combobox selection
  query: 300, // Standard input debounce
  openOnly: 0, // Immediate
  offset: 0, // Immediate (pagination)
  sorting: 0, // Immediate (column sort)
} as const;

let searchTimeout: ReturnType<typeof setTimeout> | undefined;
let lastSearchKey = "";

function searchKey(
  term: string,
  subjects: string[],
  q: string,
  open: boolean,
  off: number,
  sort: SortingState
): string {
  return `${term}|${subjects.join(",")}|${q}|${open}|${off}|${JSON.stringify(sort)}`;
}

function scheduleSearch(source: keyof typeof THROTTLE_MS) {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    const key = searchKey(selectedTerm, selectedSubjects, query, openOnly, offset, sorting);
    if (key === lastSearchKey) return;
    performSearch(selectedTerm, selectedSubjects, query, openOnly, offset, sorting);
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

// Reset offset when filters change (not offset itself)
let prevFilters = $state("");
$effect(() => {
  const key = `${selectedTerm}|${selectedSubjects.join(",")}|${query}|${openOnly}`;
  if (prevFilters && key !== prevFilters) {
    offset = 0;
  }
  prevFilters = key;
});

async function performSearch(
  term: string,
  subjects: string[],
  q: string,
  open: boolean,
  off: number,
  sort: SortingState
) {
  if (!term) return;
  const key = searchKey(term, subjects, q, open, off, sort);
  lastSearchKey = key;
  loading = true;
  error = null;

  const sortBy = sort.length > 0 ? SORT_COLUMN_MAP[sort[0].id] : undefined;
  const sortDir: SortDirection | undefined =
    sort.length > 0 ? (sort[0].desc ? "desc" : "asc") : undefined;

  const params = new URLSearchParams();
  for (const s of subjects) {
    params.append("subject", s);
  }
  if (q) params.set("q", q);
  if (open) params.set("open", "true");
  if (off > 0) params.set("offset", String(off));
  if (sortBy) params.set("sort_by", sortBy);
  if (sortDir && sortBy) params.set("sort_dir", sortDir);

  // Include term in URL only when it differs from the default or other params are active
  const hasOtherParams = params.size > 0;
  if (term !== defaultTermSlug || hasOtherParams) {
    params.set("term", term);
  }
  goto(`?${params.toString()}`, { replaceState: true, noScroll: true, keepFocus: true });

  const t0 = performance.now();
  try {
    const result = await client.searchCourses({
      term,
      subject: subjects.length > 0 ? subjects : [],
      q: q || undefined,
      openOnly: open || false,
      limit,
      offset: off,
      sortBy,
      sortDir,
    });

    const applyUpdate = () => {
      searchResult = result;
      searchMeta = {
        totalCount: result.totalCount,
        durationMs: performance.now() - t0,
        timestamp: new Date(),
      };
    };

    const tableEl = document.querySelector("[data-search-results]") as HTMLElement | null;
    const scopedSupport = tableEl && "startViewTransition" in tableEl;

    if (scopedSupport) {
      // Scoped transition — no top-layer issue, no need for filter-overlay workaround
      const transition = (tableEl as any).startViewTransition(async () => {
        applyUpdate();
        await tick();
      });
      await transition.finished;
    } else if (document.startViewTransition) {
      // Document-level fallback with z-index layering for filter overlays
      const transition = document.startViewTransition(async () => {
        applyUpdate();
        await tick();
      });
      await transition.finished;
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
</script>

<div class="min-h-screen flex flex-col items-center px-5 pb-5 pt-20">
  <div class="w-full max-w-6xl flex flex-col gap-6 pt-2">

    <!-- Search status + Filters -->
    <div class="flex flex-col gap-1.5">
      <SearchStatus meta={searchMeta} {loading} />
      <!-- Filters -->
    <SearchFilters
      terms={data.terms}
      {subjects}
      bind:selectedTerm
      bind:selectedSubjects
      bind:query
      bind:openOnly
    />
    </div>

    <!-- Results -->
    {#if error}
      <div class="text-center py-8">
        <p class="text-status-red">{error}</p>
        <button
          onclick={() => performSearch(selectedTerm, selectedSubjects, query, openOnly, offset, sorting)}
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
