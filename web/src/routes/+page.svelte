<script lang="ts">
import { untrack } from "svelte";
import { goto } from "$app/navigation";
import {
  type Subject,
  type SearchResponse,
  type SortColumn,
  type SortDirection,
  client,
} from "$lib/api";
import type { SortingState } from "@tanstack/table-core";
import SearchFilters from "$lib/components/SearchFilters.svelte";
import CourseTable from "$lib/components/CourseTable.svelte";
import Pagination from "$lib/components/Pagination.svelte";

let { data } = $props();

// Read initial state from URL params (intentionally captured once)
const initialParams = untrack(() => new URLSearchParams(data.url.search));

// Filter state
let selectedTerm = $state(untrack(() => initialParams.get("term") ?? data.terms[0]?.code ?? ""));
let selectedSubject = $state(initialParams.get("subject") ?? "");
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
let loading = $state(false);
let error = $state<string | null>(null);

// Fetch subjects when term changes
$effect(() => {
  const term = selectedTerm;
  if (!term) return;
  client.getSubjects(term).then((s) => {
    subjects = s;
    if (selectedSubject && !s.some((sub) => sub.code === selectedSubject)) {
      selectedSubject = "";
    }
  });
});

// Debounced search
let searchTimeout: ReturnType<typeof setTimeout> | undefined;
$effect(() => {
  const term = selectedTerm;
  const subject = selectedSubject;
  const q = query;
  const open = openOnly;
  const off = offset;
  const sort = sorting;

  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    performSearch(term, subject, q, open, off, sort);
  }, 300);

  return () => clearTimeout(searchTimeout);
});

// Reset offset when filters change (not offset itself)
let prevFilters = $state("");
$effect(() => {
  const key = `${selectedTerm}|${selectedSubject}|${query}|${openOnly}`;
  if (prevFilters && key !== prevFilters) {
    offset = 0;
  }
  prevFilters = key;
});

async function performSearch(
  term: string,
  subject: string,
  q: string,
  open: boolean,
  off: number,
  sort: SortingState
) {
  if (!term) return;
  loading = true;
  error = null;

  // Derive server sort params from TanStack sorting state
  const sortBy = sort.length > 0 ? SORT_COLUMN_MAP[sort[0].id] : undefined;
  const sortDir: SortDirection | undefined =
    sort.length > 0 ? (sort[0].desc ? "desc" : "asc") : undefined;

  // Sync URL
  const params = new URLSearchParams();
  params.set("term", term);
  if (subject) params.set("subject", subject);
  if (q) params.set("q", q);
  if (open) params.set("open", "true");
  if (off > 0) params.set("offset", String(off));
  if (sortBy) params.set("sort_by", sortBy);
  if (sortDir && sortBy) params.set("sort_dir", sortDir);
  goto(`?${params.toString()}`, { replaceState: true, noScroll: true, keepFocus: true });

  try {
    searchResult = await client.searchCourses({
      term,
      subject: subject || undefined,
      q: q || undefined,
      open_only: open || undefined,
      limit,
      offset: off,
      sort_by: sortBy,
      sort_dir: sortDir,
    });
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

<div class="min-h-screen flex flex-col items-center p-5">
  <div class="w-full max-w-4xl flex flex-col gap-6">
    <!-- Title -->
    <div class="text-center pt-8 pb-2">
      <h1 class="text-2xl font-semibold text-foreground">UTSA Course Search</h1>
    </div>

    <!-- Filters -->
    <SearchFilters
      terms={data.terms}
      {subjects}
      bind:selectedTerm
      bind:selectedSubject
      bind:query
      bind:openOnly
    />

    <!-- Results -->
    {#if error}
      <div class="text-center py-8">
        <p class="text-status-red">{error}</p>
        <button
          onclick={() => performSearch(selectedTerm, selectedSubject, query, openOnly, offset, sorting)}
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
      />

      {#if searchResult}
        <Pagination
          totalCount={searchResult.totalCount}
          offset={searchResult.offset}
          {limit}
          onPageChange={handlePageChange}
        />
      {/if}
    {/if}

    <!-- Footer -->
    <div class="flex justify-center items-center gap-2 mt-auto pt-6 pb-4">
      {#if __APP_VERSION__}
        <span class="text-xs text-muted-foreground">v{__APP_VERSION__}</span>
        <div class="w-px h-3 bg-muted-foreground opacity-30"></div>
      {/if}
      <a
        href="https://github.com/Xevion/banner"
        target="_blank"
        rel="noopener noreferrer"
        class="text-xs text-muted-foreground no-underline hover:underline"
      >
        GitHub
      </a>
      <div class="w-px h-3 bg-muted-foreground opacity-30"></div>
      <a href="/health" class="text-xs text-muted-foreground no-underline hover:underline">
        Status
      </a>
    </div>
  </div>
</div>
