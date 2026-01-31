<script lang="ts">
import { onMount, onDestroy } from "svelte";
import { slide, fade } from "svelte/transition";
import {
  client,
  type InstructorListItem,
  type InstructorStats,
  type InstructorDetailResponse,
  type CandidateResponse,
} from "$lib/api";
import { formatInstructorName, isRatingValid, ratingStyle } from "$lib/course";
import { themeStore } from "$lib/stores/theme.svelte";
import {
  Check,
  ChevronLeft,
  ChevronRight,
  LoaderCircle,
  RefreshCw,
  Search,
  X,
} from "@lucide/svelte";
import SimpleTooltip from "$lib/components/SimpleTooltip.svelte";
import CandidateCard from "./CandidateCard.svelte";

// --- State ---
let subjectMap = $state(new Map<string, string>());
let instructors = $state<InstructorListItem[]>([]);
let stats = $state<InstructorStats>({
  total: 0,
  unmatched: 0,
  auto: 0,
  confirmed: 0,
  rejected: 0,
  withCandidates: 0,
});
let totalCount = $state(0);
let currentPage = $state(1);
let perPage = $state(25);
let activeFilter = $state<string | undefined>(undefined);
let searchQuery = $state("");
let searchInput = $state("");
let error = $state<string | null>(null);
let loading = $state(true);

// Expanded row detail
let expandedId = $state<number | null>(null);
let detail = $state<InstructorDetailResponse | null>(null);
let detailLoading = $state(false);
let detailError = $state<string | null>(null);

// Action states
let actionLoading = $state<string | null>(null);
let rescoreLoading = $state(false);
let rescoreResult = $state<{ message: string; isError: boolean } | null>(null);

// Stale row tracking: rows that changed status but haven't been refetched yet
let recentlyChanged = $state(new Set<number>());
let highlightTimeouts = new Map<number, ReturnType<typeof setTimeout>>();

// Reject-all inline confirmation
let showRejectConfirm = $state<number | null>(null);

// Search debounce
let searchTimeout: ReturnType<typeof setTimeout> | undefined;

// --- Constants ---
const filterCards: Array<{
  label: string;
  value: string | undefined;
  stat: keyof InstructorStats;
  textColor: string;
  ringColor: string;
}> = [
  {
    label: "Total",
    value: undefined,
    stat: "total",
    textColor: "text-muted-foreground",
    ringColor: "ring-primary",
  },
  {
    label: "Unmatched",
    value: "unmatched",
    stat: "unmatched",
    textColor: "text-amber-600 dark:text-amber-400",
    ringColor: "ring-amber-500",
  },
  {
    label: "Auto",
    value: "auto",
    stat: "auto",
    textColor: "text-blue-600 dark:text-blue-400",
    ringColor: "ring-blue-500",
  },
  {
    label: "Confirmed",
    value: "confirmed",
    stat: "confirmed",
    textColor: "text-green-600 dark:text-green-400",
    ringColor: "ring-green-500",
  },
  {
    label: "Rejected",
    value: "rejected",
    stat: "rejected",
    textColor: "text-red-600 dark:text-red-400",
    ringColor: "ring-red-500",
  },
];

const progressSegments = [
  { stat: "auto" as keyof InstructorStats, color: "bg-blue-500", label: "Auto" },
  { stat: "confirmed" as keyof InstructorStats, color: "bg-green-500", label: "Confirmed" },
  { stat: "unmatched" as keyof InstructorStats, color: "bg-amber-500", label: "Unmatched" },
  { stat: "rejected" as keyof InstructorStats, color: "bg-red-500", label: "Rejected" },
];

// --- Derived ---
let matchedLegacyIds = $derived(
  new Set(detail?.currentMatches.map((m: { legacyId: number }) => m.legacyId) ?? [])
);

let progressDenom = $derived(stats.total || 1);
let totalPages = $derived(Math.max(1, Math.ceil(totalCount / perPage)));

// --- Data fetching ---
async function fetchInstructors() {
  loading = true;
  error = null;
  recentlyChanged = new Set();
  clearHighlightTimeouts();
  try {
    const res = await client.getAdminInstructors({
      status: activeFilter,
      search: searchQuery || undefined,
      page: currentPage,
      per_page: perPage,
    });
    instructors = res.instructors;
    totalCount = res.total;
    stats = res.stats;
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load instructors";
  } finally {
    loading = false;
  }
}

async function fetchDetail(id: number) {
  detailLoading = true;
  detailError = null;
  detail = null;
  try {
    detail = await client.getAdminInstructor(id);
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Failed to load details";
  } finally {
    detailLoading = false;
  }
}

onMount(() => {
  fetchInstructors();
  client
    .getReference("subject")
    .then((entries) => {
      const map = new Map<string, string>();
      for (const entry of entries) {
        map.set(entry.code, entry.description);
      }
      subjectMap = map;
    })
    .catch(() => {
      // Subject lookup is best-effort
    });
});

onDestroy(() => {
  clearTimeout(searchTimeout);
  clearHighlightTimeouts();
});

// --- Navigation & filters ---
function setFilter(value: string | undefined) {
  activeFilter = value;
  currentPage = 1;
  expandedId = null;
  showRejectConfirm = null;
  fetchInstructors();
}

function handleSearch() {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    searchQuery = searchInput;
    currentPage = 1;
    expandedId = null;
    fetchInstructors();
  }, 300);
}

function clearSearch() {
  searchInput = "";
  searchQuery = "";
  currentPage = 1;
  expandedId = null;
  fetchInstructors();
}

function clearAllFilters() {
  searchInput = "";
  searchQuery = "";
  activeFilter = undefined;
  currentPage = 1;
  expandedId = null;
  fetchInstructors();
}

function goToPage(page: number) {
  if (page < 1 || page > totalPages) return;
  currentPage = page;
  expandedId = null;
  showRejectConfirm = null;
  fetchInstructors();
}

async function toggleExpand(id: number) {
  if (expandedId === id) {
    expandedId = null;
    detail = null;
    showRejectConfirm = null;
    return;
  }
  expandedId = id;
  showRejectConfirm = null;
  await fetchDetail(id);
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && expandedId !== null) {
    expandedId = null;
    detail = null;
    showRejectConfirm = null;
  }
}

// --- Local state updates (no refetch) ---
function updateLocalStatus(instructorId: number, newStatus: string) {
  instructors = instructors.map((i) =>
    i.id === instructorId ? { ...i, rmpMatchStatus: newStatus } : i
  );
  markChanged(instructorId);
}

function markChanged(id: number) {
  // Clear any existing timeout for this ID
  const existing = highlightTimeouts.get(id);
  if (existing) clearTimeout(existing);

  recentlyChanged = new Set([...recentlyChanged, id]);

  const timeout = setTimeout(() => {
    const next = new Set(recentlyChanged);
    next.delete(id);
    recentlyChanged = next;
    highlightTimeouts.delete(id);
  }, 2000);
  highlightTimeouts.set(id, timeout);
}

function clearHighlightTimeouts() {
  for (const timeout of highlightTimeouts.values()) {
    clearTimeout(timeout);
  }
  highlightTimeouts.clear();
}

function matchesFilter(status: string): boolean {
  if (!activeFilter) return true;
  return status === activeFilter;
}

// --- Actions ---
async function handleMatch(instructorId: number, rmpLegacyId: number) {
  actionLoading = `match-${rmpLegacyId}`;
  try {
    detail = await client.matchInstructor(instructorId, rmpLegacyId);
    updateLocalStatus(instructorId, "confirmed");
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Match failed";
  } finally {
    actionLoading = null;
  }
}

async function handleReject(instructorId: number, rmpLegacyId: number) {
  actionLoading = `reject-${rmpLegacyId}`;
  try {
    await client.rejectCandidate(instructorId, rmpLegacyId);
    await fetchDetail(instructorId);
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Reject failed";
  } finally {
    actionLoading = null;
  }
}

function requestRejectAll(instructorId: number) {
  showRejectConfirm = instructorId;
}

async function confirmRejectAll(instructorId: number) {
  showRejectConfirm = null;
  actionLoading = "reject-all";
  try {
    await client.rejectAllCandidates(instructorId);
    await fetchDetail(instructorId);
    updateLocalStatus(instructorId, "rejected");
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Reject all failed";
  } finally {
    actionLoading = null;
  }
}

function cancelRejectAll() {
  showRejectConfirm = null;
}

async function handleUnmatch(instructorId: number, rmpLegacyId: number) {
  actionLoading = `unmatch-${rmpLegacyId}`;
  try {
    await client.unmatchInstructor(instructorId, rmpLegacyId);
    await fetchDetail(instructorId);
    updateLocalStatus(instructorId, "unmatched");
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Unmatch failed";
  } finally {
    actionLoading = null;
  }
}

async function handleRescore() {
  rescoreLoading = true;
  rescoreResult = null;
  try {
    const res = await client.rescoreInstructors();
    rescoreResult = {
      message: `Rescored: ${res.totalUnmatched} unmatched, ${res.candidatesCreated} candidates created, ${res.autoMatched} auto-matched`,
      isError: false,
    };
    await fetchInstructors();
  } catch (e) {
    rescoreResult = {
      message: e instanceof Error ? e.message : "Rescore failed",
      isError: true,
    };
  } finally {
    rescoreLoading = false;
  }
}

// --- Helpers ---
function statusBadge(status: string): { label: string; classes: string } {
  switch (status) {
    case "unmatched":
      return {
        label: "Unmatched",
        classes: "bg-amber-100 text-amber-800 dark:bg-amber-900 dark:text-amber-200",
      };
    case "auto":
      return {
        label: "Auto",
        classes: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
      };
    case "confirmed":
      return {
        label: "Confirmed",
        classes: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
      };
    case "rejected":
      return {
        label: "Rejected",
        classes: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
      };
    default:
      return { label: status, classes: "bg-muted text-muted-foreground" };
  }
}

function formatScore(score: number): string {
  return (score * 100).toFixed(0);
}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Header -->
<div class="flex items-center gap-3 mb-4">
  <h1 class="text-lg font-semibold text-foreground">Instructors</h1>
  <div class="flex-1"></div>

  <!-- Search -->
  <div class="relative">
    <Search
      size={14}
      class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none"
    />
    <input
      type="text"
      placeholder="Search name or email..."
      bind:value={searchInput}
      oninput={handleSearch}
      class="bg-card border-border rounded-md border pl-8 pr-8 py-1.5 text-sm text-foreground
             placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-ring w-64 transition-shadow"
    />
    {#if searchInput}
      <button
        onclick={clearSearch}
        class="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
        aria-label="Clear search"
      >
        <X size={14} />
      </button>
    {/if}
  </div>

  <!-- Rescore -->
  <button
    onclick={handleRescore}
    disabled={rescoreLoading}
    class="inline-flex items-center gap-1.5 rounded-md bg-muted px-3 py-1.5 text-sm font-medium
           text-foreground hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
  >
    <RefreshCw size={14} class={rescoreLoading ? "animate-spin" : ""} />
    Rescore
  </button>
</div>

<!-- Rescore result (dismissable) -->
{#if rescoreResult}
  <div
    class="mb-4 rounded-md px-3 py-2 text-sm flex items-center justify-between gap-2
           {rescoreResult.isError
      ? 'bg-destructive/10 text-destructive'
      : 'bg-muted text-muted-foreground'}"
    transition:fade={{ duration: 150 }}
  >
    <span>{rescoreResult.message}</span>
    <button
      onclick={() => (rescoreResult = null)}
      class="text-muted-foreground hover:text-foreground transition-colors cursor-pointer shrink-0"
      aria-label="Dismiss"
    >
      <X size={14} />
    </button>
  </div>
{/if}

<!-- Error -->
{#if error}
  <div
    class="mb-4 rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive"
    transition:fade={{ duration: 150 }}
  >
    {error}
  </div>
{/if}

{#if loading && instructors.length === 0}
  <!-- Skeleton stats cards -->
  <div class="mb-4 grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
    {#each Array(5) as _}
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="h-3 w-16 animate-pulse rounded bg-muted mb-2"></div>
        <div class="h-6 w-12 animate-pulse rounded bg-muted"></div>
      </div>
    {/each}
  </div>
  <div class="bg-muted mb-6 h-2 rounded-full overflow-hidden"></div>
  <!-- Skeleton table rows -->
  <div class="bg-card border-border overflow-hidden rounded-lg border">
    <div class="divide-y divide-border">
      {#each Array(8) as _}
        <div class="flex items-center gap-4 px-4 py-3">
          <div class="space-y-1.5 flex-1">
            <div class="h-4 w-40 animate-pulse rounded bg-muted"></div>
            <div class="h-3 w-28 animate-pulse rounded bg-muted"></div>
          </div>
          <div class="h-5 w-20 animate-pulse rounded-full bg-muted"></div>
          <div class="h-4 w-32 animate-pulse rounded bg-muted"></div>
          <div class="h-4 w-8 animate-pulse rounded bg-muted"></div>
          <div class="h-6 w-16 animate-pulse rounded bg-muted"></div>
        </div>
      {/each}
    </div>
  </div>
{:else}
  <div class="relative">
    <!-- Loading overlay for refetching -->
    {#if loading}
      <div
        class="absolute inset-0 z-10 flex items-center justify-center bg-background/60 rounded-lg"
        in:fade={{ duration: 100, delay: 150 }}
        out:fade={{ duration: 100 }}
      >
        <LoaderCircle size={24} class="animate-spin text-muted-foreground" />
      </div>
    {/if}

    <!-- Stats / Filter Cards -->
    <div class="mb-4 grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
      {#each filterCards as card}
        <button
          onclick={() => setFilter(card.value)}
          class="bg-card border-border rounded-lg border p-3 text-left transition-all duration-200
                 cursor-pointer hover:shadow-sm hover:border-border/80
                 {activeFilter === card.value ? `ring-2 ${card.ringColor} shadow-sm` : ''}"
        >
          <div class="text-xs {card.textColor}">{card.label}</div>
          <div class="text-lg font-semibold tabular-nums">{stats[card.stat]}</div>
        </button>
      {/each}
    </div>

    <!-- Progress Bar -->
    <div class="mb-6">
      <div class="bg-muted h-2 rounded-full overflow-hidden flex">
        {#each progressSegments as seg}
          {@const pct = (stats[seg.stat] / progressDenom) * 100}
          <div
            class="{seg.color} h-full transition-all duration-500"
            style="width: {pct}%"
            title="{seg.label}: {stats[seg.stat]}"
          ></div>
        {/each}
      </div>
    </div>

    {#if instructors.length === 0}
      <div class="py-12 text-center">
        {#if searchQuery || activeFilter}
          <p class="text-muted-foreground text-sm">No instructors match your filters.</p>
          <button
            onclick={clearAllFilters}
            class="mt-2 text-sm text-primary hover:underline cursor-pointer"
          >
            Clear all filters
          </button>
        {:else}
          <p class="text-muted-foreground text-sm">No instructors found.</p>
        {/if}
      </div>
    {:else}
      <div class="bg-card border-border overflow-hidden rounded-lg border">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-border border-b text-left text-muted-foreground">
              <th class="px-4 py-2.5 font-medium">Name</th>
              <th class="px-4 py-2.5 font-medium">Status</th>
              <th class="px-4 py-2.5 font-medium">Top Candidate</th>
              <th class="px-4 py-2.5 font-medium text-center">Candidates</th>
              <th class="px-4 py-2.5 font-medium text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each instructors as instructor (instructor.id)}
              {@const badge = statusBadge(instructor.rmpMatchStatus)}
              {@const isExpanded = expandedId === instructor.id}
              {@const isStale = !matchesFilter(instructor.rmpMatchStatus)}
              {@const isHighlighted = recentlyChanged.has(instructor.id)}
              <tr
                class="border-border border-b cursor-pointer transition-colors duration-700
                       {isExpanded ? 'bg-muted/30' : 'hover:bg-muted/50'}
                       {isHighlighted ? 'bg-primary/10' : ''}
                       {isStale && !isHighlighted ? 'opacity-60' : ''}"
                onclick={() => toggleExpand(instructor.id)}
              >
                <td class="px-4 py-2.5">
                  <div class="font-medium text-foreground">
                    {formatInstructorName(instructor.displayName)}
                  </div>
                  <div class="text-xs text-muted-foreground">{instructor.email}</div>
                </td>
                <td class="px-4 py-2.5">
                  <span
                    class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors duration-300 {badge.classes}"
                  >
                    {badge.label}
                  </span>
                </td>
                <td class="px-4 py-2.5">
                  {#if instructor.topCandidate}
                    {@const tc = instructor.topCandidate}
                    <div class="flex items-center gap-2">
                      <span class="text-foreground">{tc.firstName} {tc.lastName}</span>
                      {#if isRatingValid(tc.avgRating, tc.numRatings ?? 0)}
                        <span
                          class="font-semibold tabular-nums"
                          style={ratingStyle(tc.avgRating!, themeStore.isDark)}
                        >
                          {tc.avgRating!.toFixed(1)}
                        </span>
                      {:else}
                        <span class="text-xs text-muted-foreground">N/A</span>
                      {/if}
                      <span class="text-xs text-muted-foreground tabular-nums">
                        ({formatScore(tc.score ?? 0)}%)
                      </span>
                    </div>
                  {:else}
                    <span class="text-muted-foreground text-xs">No candidates</span>
                  {/if}
                </td>
                <td class="px-4 py-2.5 text-center tabular-nums text-muted-foreground">
                  {instructor.candidateCount}
                </td>
                <td class="px-4 py-2.5 text-right">
                  <div class="inline-flex items-center gap-1">
                    {#if instructor.topCandidate && instructor.rmpMatchStatus === "unmatched"}
                      <button
                        onclick={(e) => {
                          e.stopPropagation();
                          handleMatch(instructor.id, instructor.topCandidate!.rmpLegacyId);
                        }}
                        disabled={actionLoading !== null}
                        class="rounded p-1 text-green-600 hover:bg-green-100 dark:hover:bg-green-900/30
                               transition-colors disabled:opacity-50 cursor-pointer"
                        title="Accept top candidate"
                      >
                        {#if actionLoading === `match-${instructor.topCandidate.rmpLegacyId}`}
                          <LoaderCircle size={16} class="animate-spin" />
                        {:else}
                          <Check size={16} />
                        {/if}
                      </button>
                    {/if}
                    <button
                      onclick={(e) => {
                        e.stopPropagation();
                        toggleExpand(instructor.id);
                      }}
                      class="rounded p-1 text-muted-foreground hover:bg-muted transition-colors cursor-pointer"
                      title={isExpanded ? "Collapse" : "Expand details"}
                      aria-expanded={isExpanded}
                    >
                      <ChevronRight
                        size={16}
                        class="transition-transform duration-200 {isExpanded ? 'rotate-90' : ''}"
                      />
                    </button>
                  </div>
                </td>
              </tr>

              <!-- Expanded detail panel -->
              {#if isExpanded}
                <tr class="border-border border-b bg-muted/20">
                  <td colspan="5" class="p-0 overflow-hidden">
                    <div transition:slide={{ duration: 200 }} class="p-4">
                      {#if detailLoading}
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                          <div class="space-y-3 animate-pulse">
                            <div class="h-4 w-20 rounded bg-muted"></div>
                            <div class="space-y-2">
                              <div class="h-3 w-36 rounded bg-muted"></div>
                              <div class="h-3 w-44 rounded bg-muted"></div>
                              <div class="h-3 w-28 rounded bg-muted"></div>
                            </div>
                          </div>
                          <div class="lg:col-span-2 space-y-3 animate-pulse">
                            <div class="h-4 w-32 rounded bg-muted"></div>
                            <div class="space-y-2">
                              <div class="h-20 rounded bg-muted"></div>
                              <div class="h-20 rounded bg-muted"></div>
                            </div>
                          </div>
                        </div>
                      {:else if detailError}
                        <div class="rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
                          {detailError}
                        </div>
                      {:else if detail}
                        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                          <!-- Instructor info -->
                          <div class="space-y-3">
                            <h3 class="font-medium text-foreground text-sm">Instructor</h3>
                            <dl
                              class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-sm"
                            >
                              <dt class="text-muted-foreground">Name</dt>
                              <dd class="text-foreground">
                                {formatInstructorName(detail.instructor.displayName)}
                              </dd>

                              <dt class="text-muted-foreground">Email</dt>
                              <dd class="text-foreground break-all">{detail.instructor.email}</dd>

                              <dt class="text-muted-foreground">Courses</dt>
                              <dd class="text-foreground tabular-nums">
                                {detail.instructor.courseCount}
                              </dd>

                              {#if detail.instructor.subjectsTaught.length > 0}
                                <dt class="text-muted-foreground">Subjects</dt>
                                <dd class="flex flex-wrap gap-1">
                                  {#each detail.instructor.subjectsTaught as subj}
                                    {#if subjectMap.has(subj)}
                                      <SimpleTooltip text={subjectMap.get(subj)!} delay={75}>
                                        <span class="rounded bg-muted px-1.5 py-0.5 text-xs font-medium"
                                          >{subj}</span
                                        >
                                      </SimpleTooltip>
                                    {:else}
                                      <span class="rounded bg-muted px-1.5 py-0.5 text-xs font-medium"
                                        >{subj}</span
                                      >
                                    {/if}
                                  {/each}
                                </dd>
                              {/if}
                            </dl>
                          </div>

                          <!-- Candidates -->
                          <div class="lg:col-span-2 space-y-3">
                            <div class="flex items-center justify-between gap-2">
                              <h3 class="font-medium text-foreground text-sm">
                                Candidates
                                <span class="text-muted-foreground font-normal"
                                  >({detail.candidates.length})</span
                                >
                              </h3>
                              {#if detail.candidates.some((c: CandidateResponse) => c.status !== "rejected" && !matchedLegacyIds.has(c.rmpLegacyId))}
                                {#if showRejectConfirm === detail.instructor.id}
                                  <div
                                    class="inline-flex items-center gap-2 text-xs"
                                    in:fade={{ duration: 100 }}
                                  >
                                    <span class="text-muted-foreground"
                                      >Reject all candidates?</span
                                    >
                                    <button
                                      onclick={(e) => {
                                        e.stopPropagation();
                                        confirmRejectAll(detail!.instructor.id);
                                      }}
                                      disabled={actionLoading !== null}
                                      class="font-medium text-red-600 hover:text-red-700
                                             dark:text-red-400 dark:hover:text-red-300
                                             cursor-pointer disabled:opacity-50"
                                    >
                                      Confirm
                                    </button>
                                    <button
                                      onclick={(e) => {
                                        e.stopPropagation();
                                        cancelRejectAll();
                                      }}
                                      class="text-muted-foreground hover:text-foreground cursor-pointer"
                                    >
                                      Cancel
                                    </button>
                                  </div>
                                {:else}
                                  <button
                                    onclick={(e) => {
                                      e.stopPropagation();
                                      requestRejectAll(detail!.instructor.id);
                                    }}
                                    disabled={actionLoading !== null}
                                    class="inline-flex items-center gap-1 rounded-md bg-red-100 px-2 py-1
                                           text-xs font-medium text-red-700 hover:bg-red-200
                                           dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50
                                           transition-colors disabled:opacity-50 cursor-pointer"
                                  >
                                    <X size={12} /> Reject All
                                  </button>
                                {/if}
                              {/if}
                            </div>

                            {#if detail.candidates.length === 0}
                              <p class="text-muted-foreground text-sm py-2">
                                No candidates available.
                              </p>
                            {:else}
                              <div class="max-h-80 overflow-y-auto space-y-2 pr-1">
                                {#each detail.candidates as candidate (candidate.id)}
                                  <CandidateCard
                                    {candidate}
                                    isMatched={candidate.status === "matched" ||
                                      matchedLegacyIds.has(candidate.rmpLegacyId)}
                                    isRejected={candidate.status === "rejected"}
                                    disabled={actionLoading !== null}
                                    {actionLoading}
                                    isDark={themeStore.isDark}
                                    onmatch={() =>
                                      handleMatch(
                                        detail!.instructor.id,
                                        candidate.rmpLegacyId
                                      )}
                                    onreject={() =>
                                      handleReject(
                                        detail!.instructor.id,
                                        candidate.rmpLegacyId
                                      )}
                                    onunmatch={() =>
                                      handleUnmatch(
                                        detail!.instructor.id,
                                        candidate.rmpLegacyId
                                      )}
                                  />
                                {/each}
                              </div>
                            {/if}
                          </div>
                        </div>
                      {/if}
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      <div class="mt-4 flex items-center justify-between text-sm">
        <span class="text-muted-foreground">
          Showing {(currentPage - 1) * perPage + 1}&ndash;{Math.min(
            currentPage * perPage,
            totalCount
          )} of {totalCount}
        </span>
        <div class="flex items-center gap-1">
          <button
            onclick={() => goToPage(currentPage - 1)}
            disabled={currentPage <= 1}
            class="inline-flex items-center justify-center size-8 rounded-md bg-muted text-foreground
                   hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer disabled:cursor-default"
            aria-label="Previous page"
          >
            <ChevronLeft size={16} />
          </button>
          <span class="text-muted-foreground tabular-nums px-2">
            {currentPage} / {totalPages}
          </span>
          <button
            onclick={() => goToPage(currentPage + 1)}
            disabled={currentPage >= totalPages}
            class="inline-flex items-center justify-center size-8 rounded-md bg-muted text-foreground
                   hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer disabled:cursor-default"
            aria-label="Next page"
          >
            <ChevronRight size={16} />
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
