<script lang="ts">
import { onMount, onDestroy } from "svelte";
import {
  client,
  type InstructorListItem,
  type InstructorStats,
  type InstructorDetailResponse,
  type CandidateResponse,
} from "$lib/api";
import { formatInstructorName, isRatingValid, ratingStyle, rmpUrl } from "$lib/course";
import { themeStore } from "$lib/stores/theme.svelte";
import {
  Check,
  ChevronLeft,
  ChevronRight,
  ExternalLink,
  RefreshCw,
  X,
  XCircle,
} from "@lucide/svelte";

// --- State ---
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
let rescoreResult = $state<string | null>(null);

// Search debounce
let searchTimeout: ReturnType<typeof setTimeout> | undefined;

const filterOptions = [
  { label: "All", value: undefined as string | undefined },
  { label: "Needs Review", value: "unmatched" },
  { label: "Auto-matched", value: "auto" },
  { label: "Confirmed", value: "confirmed" },
  { label: "Rejected", value: "rejected" },
];

let matchedLegacyIds = $derived(
  new Set(detail?.currentMatches.map((m: { legacyId: number }) => m.legacyId) ?? [])
);

let progressDenom = $derived(stats.total || 1);
let autoPct = $derived((stats.auto / progressDenom) * 100);
let confirmedPct = $derived((stats.confirmed / progressDenom) * 100);
let unmatchedPct = $derived((stats.unmatched / progressDenom) * 100);

let totalPages = $derived(Math.max(1, Math.ceil(totalCount / perPage)));

// --- Data fetching ---
async function fetchInstructors() {
  loading = true;
  error = null;
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
});

onDestroy(() => clearTimeout(searchTimeout));

function setFilter(value: string | undefined) {
  activeFilter = value;
  currentPage = 1;
  expandedId = null;
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

function goToPage(page: number) {
  if (page < 1 || page > totalPages) return;
  currentPage = page;
  expandedId = null;
  fetchInstructors();
}

async function toggleExpand(id: number) {
  if (expandedId === id) {
    expandedId = null;
    detail = null;
    return;
  }
  expandedId = id;
  await fetchDetail(id);
}

// --- Actions ---
async function handleMatch(instructorId: number, rmpLegacyId: number) {
  actionLoading = `match-${rmpLegacyId}`;
  try {
    detail = await client.matchInstructor(instructorId, rmpLegacyId);
    await fetchInstructors();
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
    await Promise.all([fetchDetail(instructorId), fetchInstructors()]);
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Reject failed";
  } finally {
    actionLoading = null;
  }
}

async function handleRejectAll(instructorId: number) {
  actionLoading = "reject-all";
  try {
    await client.rejectAllCandidates(instructorId);
    await Promise.all([fetchDetail(instructorId), fetchInstructors()]);
  } catch (e) {
    detailError = e instanceof Error ? e.message : "Reject all failed";
  } finally {
    actionLoading = null;
  }
}

async function handleUnmatch(instructorId: number, rmpLegacyId: number) {
  actionLoading = `unmatch-${rmpLegacyId}`;
  try {
    await client.unmatchInstructor(instructorId, rmpLegacyId);
    await Promise.all([fetchDetail(instructorId), fetchInstructors()]);
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
    rescoreResult = `Rescored: ${res.totalUnmatched} unmatched, ${res.candidatesCreated} candidates created, ${res.autoMatched} auto-matched`;
    await fetchInstructors();
  } catch (e) {
    rescoreResult = e instanceof Error ? e.message : "Rescore failed";
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

const scoreWeights: Record<string, number> = {
  name: 0.5,
  department: 0.25,
  uniqueness: 0.15,
  volume: 0.1,
};
const scoreColors: Record<string, string> = {
  name: "bg-blue-500",
  department: "bg-purple-500",
  uniqueness: "bg-amber-500",
  volume: "bg-emerald-500",
};
const scoreLabels: Record<string, string> = {
  name: "Name",
  department: "Dept",
  uniqueness: "Unique",
  volume: "Volume",
};
</script>

<div class="flex items-center justify-between mb-4">
  <h1 class="text-lg font-semibold text-foreground">Instructors</h1>
  <button
    onclick={handleRescore}
    disabled={rescoreLoading}
    class="inline-flex items-center gap-1.5 rounded-md bg-muted px-3 py-1.5 text-sm font-medium text-foreground hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
  >
    <RefreshCw size={14} class={rescoreLoading ? "animate-spin" : ""} />
    Rescore
  </button>
</div>

{#if rescoreResult}
  <div class="mb-4 rounded-md bg-muted px-3 py-2 text-sm text-muted-foreground">
    {rescoreResult}
  </div>
{/if}

{#if error}
  <p class="text-destructive mb-4">{error}</p>
{/if}

{#if loading && instructors.length === 0}
  <!-- Skeleton stats cards -->
  <div class="mb-4 grid grid-cols-5 gap-3">
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

<!-- Stats Bar -->
<div class="mb-4 grid grid-cols-5 gap-3">
  <div class="bg-card border-border rounded-lg border p-3">
    <div class="text-muted-foreground text-xs">Total</div>
    <div class="text-lg font-semibold">{stats.total}</div>
  </div>
  <div class="bg-card border-border rounded-lg border p-3">
    <div class="text-xs text-amber-600 dark:text-amber-400">Unmatched</div>
    <div class="text-lg font-semibold">{stats.unmatched}</div>
  </div>
  <div class="bg-card border-border rounded-lg border p-3">
    <div class="text-xs text-blue-600 dark:text-blue-400">Auto</div>
    <div class="text-lg font-semibold">{stats.auto}</div>
  </div>
  <div class="bg-card border-border rounded-lg border p-3">
    <div class="text-xs text-green-600 dark:text-green-400">Confirmed</div>
    <div class="text-lg font-semibold">{stats.confirmed}</div>
  </div>
  <div class="bg-card border-border rounded-lg border p-3">
    <div class="text-xs text-red-600 dark:text-red-400">Rejected</div>
    <div class="text-lg font-semibold">{stats.rejected}</div>
  </div>
</div>

<!-- Progress Bar -->
<div class="bg-muted mb-6 h-2 rounded-full overflow-hidden flex">
  <div class="bg-blue-500 h-full transition-all duration-300"
       style="width: {autoPct}%" title="Auto: {stats.auto}"></div>
  <div class="bg-green-500 h-full transition-all duration-300"
       style="width: {confirmedPct}%" title="Confirmed: {stats.confirmed}"></div>
  <div class="bg-amber-500 h-full transition-all duration-300"
       style="width: {unmatchedPct}%" title="Unmatched: {stats.unmatched}"></div>
</div>

<!-- Filters -->
<div class="mb-4 flex items-center gap-4">
  <div class="flex gap-2">
    {#each filterOptions as opt}
      <button
        onclick={() => setFilter(opt.value)}
        class="rounded-md px-3 py-1.5 text-sm transition-colors cursor-pointer
          {activeFilter === opt.value
            ? 'bg-primary text-primary-foreground'
            : 'bg-muted text-muted-foreground hover:bg-accent'}"
      >
        {opt.label}
      </button>
    {/each}
  </div>

  <input
    type="text"
    placeholder="Search by name or email..."
    bind:value={searchInput}
    oninput={handleSearch}
    class="bg-card border-border rounded-md border px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground outline-none focus:ring-1 focus:ring-ring w-64"
  />
</div>

{#if instructors.length === 0}
  <p class="text-muted-foreground py-8 text-center text-sm">No instructors found.</p>
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
          <tr
            class="border-border border-b cursor-pointer hover:bg-muted/50 transition-colors {isExpanded ? 'bg-muted/30' : ''}"
            onclick={() => toggleExpand(instructor.id)}
          >
            <td class="px-4 py-2.5">
              <div class="font-medium text-foreground">{formatInstructorName(instructor.displayName)}</div>
              <div class="text-xs text-muted-foreground">{instructor.email}</div>
            </td>
            <td class="px-4 py-2.5">
              <span class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {badge.classes}">
                {badge.label}
              </span>
            </td>
            <td class="px-4 py-2.5">
              {#if instructor.topCandidate}
                {@const tc = instructor.topCandidate}
                <div class="flex items-center gap-2">
                  <span class="text-foreground">{tc.firstName} {tc.lastName}</span>
                  {#if isRatingValid(tc.avgRating, tc.numRatings ?? 0)}
                    <span class="font-semibold tabular-nums" style={ratingStyle(tc.avgRating!, themeStore.isDark)}>
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
                    onclick={(e) => { e.stopPropagation(); handleMatch(instructor.id, instructor.topCandidate!.rmpLegacyId); }}
                    disabled={actionLoading !== null}
                    class="rounded p-1 text-green-600 hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors disabled:opacity-50 cursor-pointer"
                    title="Accept top candidate"
                  >
                    <Check size={16} />
                  </button>
                {/if}
                <button
                  onclick={(e) => { e.stopPropagation(); toggleExpand(instructor.id); }}
                  class="rounded p-1 text-muted-foreground hover:bg-muted transition-colors cursor-pointer"
                  title={isExpanded ? "Collapse" : "Expand"}
                >
                  <ChevronRight size={16} class="transition-transform duration-200 {isExpanded ? 'rotate-90' : ''}" />
                </button>
              </div>
            </td>
          </tr>

          <!-- Expanded detail panel -->
          {#if isExpanded}
            <tr class="border-border border-b bg-muted/20">
              <td colspan="5" class="p-0">
                <div class="p-4">
                  {#if detailLoading}
                    <p class="text-muted-foreground text-sm py-4 text-center">Loading details...</p>
                  {:else if detailError}
                    <p class="text-destructive text-sm py-2">{detailError}</p>
                  {:else if detail}
                    <div class="grid grid-cols-3 gap-6">
                      <!-- Left: Instructor info -->
                      <div class="space-y-3">
                        <h3 class="font-medium text-foreground">Instructor</h3>
                        <dl class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-sm">
                          <dt class="text-muted-foreground">Name</dt>
                          <dd class="text-foreground">{formatInstructorName(detail.instructor.displayName)}</dd>

                          <dt class="text-muted-foreground">Email</dt>
                          <dd class="text-foreground">{detail.instructor.email}</dd>

                          <dt class="text-muted-foreground">Courses</dt>
                          <dd class="text-foreground tabular-nums">{detail.instructor.courseCount}</dd>

                          {#if detail.instructor.subjectsTaught.length > 0}
                            <dt class="text-muted-foreground">Subjects</dt>
                            <dd class="flex flex-wrap gap-1">
                              {#each detail.instructor.subjectsTaught as subj}
                                <span class="rounded bg-muted px-1.5 py-0.5 text-xs">{subj}</span>
                              {/each}
                            </dd>
                          {/if}
                        </dl>
                      </div>

                      <!-- Right: Candidates (spans 2 cols) -->
                      <div class="col-span-2 space-y-3">
                        <div class="flex items-center justify-between">
                          <h3 class="font-medium text-foreground">
                            Candidates ({detail.candidates.length})
                          </h3>
                          {#if detail.candidates.some((c: CandidateResponse) => c.status !== "rejected" && !matchedLegacyIds.has(c.rmpLegacyId))}
                            <button
                              onclick={(e) => { e.stopPropagation(); handleRejectAll(detail!.instructor.id); }}
                              disabled={actionLoading !== null}
                              class="inline-flex items-center gap-1 rounded-md bg-red-100 px-2 py-1 text-xs font-medium text-red-700 hover:bg-red-200 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-900/50 transition-colors disabled:opacity-50 cursor-pointer"
                            >
                              <X size={12} /> Reject All
                            </button>
                          {/if}
                        </div>

                        {#if detail.candidates.length === 0}
                          <p class="text-muted-foreground text-sm">No candidates available.</p>
                        {:else}
                          <div class="max-h-80 overflow-y-auto space-y-2 pr-1">
                            {#each detail.candidates as candidate (candidate.id)}
                              {@const isMatched = candidate.status === "matched" || matchedLegacyIds.has(candidate.rmpLegacyId)}
                              {@const isRejected = candidate.status === "rejected"}
                              {@const isPending = !isMatched && !isRejected}
                              <div class="rounded-md border p-3 {isMatched ? 'border-l-4 border-l-green-500 bg-green-500/5 border-border' : isRejected ? 'border-border bg-card opacity-50' : 'border-border bg-card'}">
                                <div class="flex items-start justify-between gap-2">
                                  <div>
                                    <div class="flex items-center gap-2">
                                      <span class="font-medium text-foreground text-sm">
                                        {candidate.firstName} {candidate.lastName}
                                      </span>
                                      {#if isMatched}
                                        <span class="text-[10px] rounded px-1 py-0.5 bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400">
                                          Matched
                                        </span>
                                      {:else if isRejected}
                                        <span class="text-[10px] rounded px-1 py-0.5 bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400">
                                          Rejected
                                        </span>
                                      {/if}
                                    </div>
                                    {#if candidate.department}
                                      <div class="text-xs text-muted-foreground">{candidate.department}</div>
                                    {/if}
                                  </div>
                                  <div class="flex items-center gap-1 shrink-0">
                                    {#if isMatched}
                                      <button
                                        onclick={(e) => { e.stopPropagation(); handleUnmatch(detail!.instructor.id, candidate.rmpLegacyId); }}
                                        disabled={actionLoading !== null}
                                        class="inline-flex items-center gap-1 rounded p-1 text-xs text-red-500 hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors disabled:opacity-50 cursor-pointer"
                                        title="Unmatch"
                                      >
                                        <XCircle size={14} /> Unmatch
                                      </button>
                                    {:else if isPending}
                                      <button
                                        onclick={(e) => { e.stopPropagation(); handleMatch(detail!.instructor.id, candidate.rmpLegacyId); }}
                                        disabled={actionLoading !== null}
                                        class="rounded p-1 text-green-600 hover:bg-green-100 dark:hover:bg-green-900/30 transition-colors disabled:opacity-50 cursor-pointer"
                                        title="Accept"
                                      >
                                        <Check size={14} />
                                      </button>
                                      <button
                                        onclick={(e) => { e.stopPropagation(); handleReject(detail!.instructor.id, candidate.rmpLegacyId); }}
                                        disabled={actionLoading !== null}
                                        class="rounded p-1 text-red-500 hover:bg-red-100 dark:hover:bg-red-900/30 transition-colors disabled:opacity-50 cursor-pointer"
                                        title="Reject"
                                      >
                                        <X size={14} />
                                      </button>
                                    {/if}
                                    <a
                                      href={rmpUrl(candidate.rmpLegacyId)}
                                      target="_blank"
                                      rel="noopener noreferrer"
                                      onclick={(e) => e.stopPropagation()}
                                      class="rounded p-1 text-muted-foreground hover:bg-muted transition-colors cursor-pointer"
                                      title="View on RMP"
                                    >
                                      <ExternalLink size={14} />
                                    </a>
                                  </div>
                                </div>

                                <!-- Stats row -->
                                <div class="mt-2 flex items-center gap-3 text-xs">
                                  {#if isRatingValid(candidate.avgRating, candidate.numRatings ?? 0)}
                                    <span class="font-semibold tabular-nums" style={ratingStyle(candidate.avgRating!, themeStore.isDark)}>
                                      {candidate.avgRating!.toFixed(1)}
                                    </span>
                                  {:else}
                                    <span class="text-xs text-muted-foreground">N/A</span>
                                  {/if}
                                  {#if candidate.avgDifficulty !== null}
                                    <span class="text-muted-foreground tabular-nums">{candidate.avgDifficulty.toFixed(1)} diff</span>
                                  {/if}
                                  <span class="text-muted-foreground">{candidate.numRatings} ratings</span>
                                  {#if candidate.wouldTakeAgainPct !== null}
                                    <span class="text-muted-foreground">{candidate.wouldTakeAgainPct.toFixed(0)}% again</span>
                                  {/if}
                                </div>

                                <!-- Score bar (weighted segments) -->
                                <div class="mt-2">
                                  <div class="flex items-center gap-2 text-xs">
                                    <span class="text-muted-foreground shrink-0">Score:</span>
                                    <div class="bg-muted h-2 flex-1 rounded-full overflow-hidden flex">
                                      {#each Object.entries(candidate.scoreBreakdown ?? {}) as [key, value]}
                                        {@const w = scoreWeights[key] ?? 0}
                                        {@const widthPct = (value as number) * w * 100}
                                        <div
                                          class="{scoreColors[key] ?? 'bg-primary'} h-full transition-all"
                                          style="width: {widthPct}%"
                                          title="{scoreLabels[key] ?? key}: {((value as number) * 100).toFixed(0)}% (×{w})"
                                        ></div>
                                      {/each}
                                    </div>
                                    <span class="tabular-nums font-medium text-foreground shrink-0">{formatScore(candidate.score ?? 0)}%</span>
                                  </div>
                                </div>
                              </div>
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
      Showing {(currentPage - 1) * perPage + 1}–{Math.min(currentPage * perPage, totalCount)} of {totalCount}
    </span>
    <div class="flex items-center gap-2">
      <button
        onclick={() => goToPage(currentPage - 1)}
        disabled={currentPage <= 1}
        class="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-1 text-sm text-foreground hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
      >
        <ChevronLeft size={14} /> Prev
      </button>
      <span class="text-muted-foreground tabular-nums">
        {currentPage} / {totalPages}
      </span>
      <button
        onclick={() => goToPage(currentPage + 1)}
        disabled={currentPage >= totalPages}
        class="inline-flex items-center gap-1 rounded-md bg-muted px-2 py-1 text-sm text-foreground hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
      >
        Next <ChevronRight size={14} />
      </button>
    </div>
  </div>
{/if}
{/if}
