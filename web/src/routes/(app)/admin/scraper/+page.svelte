<script module lang="ts">
import type {
  ScraperStatsResponse,
  SubjectDetailResponse,
  SubjectSummary,
  TimeseriesResponse,
} from "$lib/bindings";
import type { DbTerm } from "$lib/api";

// Module-level cache persisted across navigation
let statsCache = $state<ScraperStatsResponse | null>(null);
let timeseriesCache = $state<TimeseriesResponse | null>(null);
let subjectsCache = $state<SubjectSummary[]>([]);
let termsCache = $state<DbTerm[]>([]);
</script>

<script lang="ts">
import { client, type ScraperPeriod } from "$lib/api";
import { useAutoRefresh } from "$lib/composables/useAutoRefresh.svelte";
import SimpleTooltip from "$lib/components/SimpleTooltip.svelte";
import { FlexRender, createSvelteTable } from "$lib/components/ui/data-table/index.js";
import { formatAbsoluteDate } from "$lib/date";
import { formatDuration, formatDurationMs, relativeTime } from "$lib/time";
import { formatNumber } from "$lib/utils";
import { Chart, Svg, Area, Axis, Highlight, Tooltip } from "layerchart";
import { curveMonotoneX } from "d3-shape";
import { cubicOut } from "svelte/easing";
import { Tween } from "svelte/motion";
import { scaleTime, scaleLinear } from "d3-scale";
import {
  AlertCircle,
  ChevronDown,
  ChevronRight,
  ChevronUp,
  Info,
  LoaderCircle,
  ArrowUp,
  ArrowDown,
  ArrowUpDown,
} from "@lucide/svelte";
import {
  type ColumnDef,
  type SortingState,
  type Updater,
  getCoreRowModel,
  getSortedRowModel,
} from "@tanstack/table-core";
import { Select } from "bits-ui";
import { onDestroy } from "svelte";
import { fade, slide } from "svelte/transition";

const PERIODS: ScraperPeriod[] = ["1h", "6h", "24h", "7d", "30d"];
const REFRESH_INTERVAL = 5_000;
const MIN_SPIN_MS = 700;

let selectedPeriod = $state<ScraperPeriod>("24h");
let selectedTerm = $state<string | undefined>(undefined);

// --- Auto-refresh hooks ---

const stats = useAutoRefresh({
  fetcher: () => client.getScraperStats(selectedPeriod, selectedTerm),
  deps: () => [selectedPeriod, selectedTerm],
  interval: REFRESH_INTERVAL,
  minLoadingMs: MIN_SPIN_MS,
});

const timeseries = useAutoRefresh({
  fetcher: () => client.getScraperTimeseries(selectedPeriod, undefined, selectedTerm),
  deps: () => [selectedPeriod, selectedTerm],
  interval: REFRESH_INTERVAL,
});

const subjects = useAutoRefresh({
  fetcher: () => client.getScraperSubjects().then((r) => r.subjects),
  interval: REFRESH_INTERVAL,
});

const terms = useAutoRefresh({
  fetcher: () => client.getAdminTerms().then((r) => r.terms),
  interval: 0, // Fetch once, no auto-refresh
  paused: termsCache.length > 0, // Skip if already cached
});

// Sync fetched data to module-level cache for navigation persistence
$effect(() => {
  if (stats.data) statsCache = stats.data;
});
$effect(() => {
  if (timeseries.data) timeseriesCache = timeseries.data;
});
$effect(() => {
  if (subjects.data && subjects.data.length > 0) subjectsCache = subjects.data;
});
$effect(() => {
  if (terms.data && terms.data.length > 0) termsCache = terms.data;
});

// Use cached data as fallback, fresh data when available
let currentStats = $derived(stats.data ?? statsCache);
let currentTimeseries = $derived(timeseries.data ?? timeseriesCache);
let currentSubjects = $derived(subjects.data ?? subjectsCache);
let currentTerms = $derived(terms.data ?? termsCache);

// Combined loading state (show spinner if main stats are loading)
let isLoading = $derived(stats.isLoading);
let hasError = $derived(stats.hasError || timeseries.hasError || subjects.hasError);
let errorMessage = $derived(stats.error ?? timeseries.error ?? subjects.error);

// --- Expanded subject detail ---
let expandedSubject = $state<string | null>(null);
let subjectDetail = $state<SubjectDetailResponse | null>(null);
let detailLoading = $state(false);

async function toggleSubjectDetail(subject: string) {
  if (expandedSubject === subject) {
    expandedSubject = null;
    subjectDetail = null;
    return;
  }
  expandedSubject = subject;
  detailLoading = true;
  try {
    subjectDetail = await client.getScraperSubjectDetail(subject);
  } catch {
    subjectDetail = null;
  } finally {
    detailLoading = false;
  }
}

// --- Live-updating clock for relative timestamps ---
let now = $state(new Date());
let tickTimer: ReturnType<typeof setTimeout> | undefined;

function scheduleTick() {
  tickTimer = setTimeout(() => {
    now = new Date();
    scheduleTick();
  }, 1000);
}

// Start the clock
scheduleTick();

onDestroy(() => {
  clearTimeout(tickTimer);
});

// --- Chart data ---

interface ChartPoint {
  date: Date;
  success: number;
  errors: number;
  coursesChanged: number;
}

let chartData = $derived(
  (currentTimeseries?.points ?? []).map((p) => ({
    date: new Date(p.timestamp),
    success: p.successCount,
    errors: p.errorCount,
    coursesChanged: p.coursesChanged,
  })),
);

// Tween the data array so stacked areas stay aligned
const tweenedChart = new Tween<ChartPoint[]>([], {
  duration: 600,
  easing: cubicOut,
  interpolate(from, to) {
    if (from.length !== to.length) return () => to;
    return (t) =>
      to.map((dest, i) => ({
        date: dest.date,
        success: from[i].success + (dest.success - from[i].success) * t,
        errors: from[i].errors + (dest.errors - from[i].errors) * t,
        coursesChanged: from[i].coursesChanged + (dest.coursesChanged - from[i].coursesChanged) * t,
      }));
  },
});

$effect(() => {
  void tweenedChart.set(chartData);
});

let scrapeYMax = $derived(Math.max(1, ...chartData.map((d) => d.success + d.errors)));
let changesYMax = $derived(Math.max(1, ...chartData.map((d) => d.coursesChanged)));

// --- Term Select Items ---
let termItems = $derived([
  { value: "", label: "All Terms" },
  ...currentTerms.map((t) => ({ value: t.code, label: t.description })),
]);

let termSelectValue = $derived(selectedTerm ?? "");

// --- Helpers ---

function formatInterval(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.round(secs / 60)}m`;
  return `${(secs / 3600).toFixed(1)}h`;
}

function successRateColor(rate: number): string {
  if (rate >= 0.95) return "text-green-600 dark:text-green-400";
  if (rate >= 0.8) return "text-yellow-600 dark:text-yellow-400";
  return "text-red-600 dark:text-red-400";
}

function emphasisClass(value: number): string {
  return value === 0 ? "text-muted-foreground" : "text-foreground";
}

function xAxisFormat(period: ScraperPeriod) {
  return (v: Date) => {
    if (period === "1h" || period === "6h") {
      return v.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
    }
    if (period === "24h") {
      return v.toLocaleTimeString("en-US", { hour: "numeric" });
    }
    return v.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  };
}

// --- TanStack Table ---

let sorting: SortingState = $state([{ id: "subject", desc: false }]);

function handleSortingChange(updater: Updater<SortingState>) {
  sorting = typeof updater === "function" ? updater(sorting) : updater;
}

const columns: ColumnDef<SubjectSummary, unknown>[] = [
  {
    id: "subject",
    accessorKey: "subject",
    header: "Subject",
    enableSorting: true,
    sortingFn: (a, b) => a.original.subject.localeCompare(b.original.subject),
  },
  {
    id: "status",
    accessorFn: (row) => row.scheduleState,
    header: "Scrape in",
    enableSorting: true,
    sortingFn: (a, b) => {
      const order: Record<string, number> = { eligible: 0, cooldown: 1, paused: 2, read_only: 3 };
      const sa = order[a.original.scheduleState] ?? 4;
      const sb = order[b.original.scheduleState] ?? 4;
      if (sa !== sb) return sa - sb;
      return (a.original.cooldownRemainingSecs ?? Infinity) - (b.original.cooldownRemainingSecs ?? Infinity);
    },
  },
  {
    id: "interval",
    accessorFn: (row) => row.currentIntervalSecs * row.timeMultiplier,
    header: "Interval",
    enableSorting: true,
  },
  {
    id: "lastScraped",
    accessorKey: "lastScraped",
    header: "Last Scraped",
    enableSorting: true,
  },
  {
    id: "changeRate",
    accessorKey: "avgChangeRatio",
    header: "Change %",
    enableSorting: true,
  },
  {
    id: "zeros",
    accessorKey: "consecutiveZeroChanges",
    header: "Zeros",
    enableSorting: true,
  },
  {
    id: "runs",
    accessorKey: "recentRuns",
    header: "Runs",
    enableSorting: true,
  },
  {
    id: "fails",
    accessorKey: "recentFailures",
    header: "Fails",
    enableSorting: true,
  },
];

const table = createSvelteTable({
  get data() {
    return currentSubjects;
  },
  getRowId: (row) => row.subject,
  columns,
  state: {
    get sorting() {
      return sorting;
    },
  },
  onSortingChange: handleSortingChange,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel<SubjectSummary>(),
  enableSortingRemoval: true,
});

const skeletonWidths: Record<string, string> = {
  subject: "w-24",
  status: "w-20",
  interval: "w-14",
  lastScraped: "w-20",
  changeRate: "w-12",
  zeros: "w-8",
  runs: "w-8",
  fails: "w-8",
};

const columnCount = columns.length;
const detailGridCols = "grid-cols-[7fr_5fr_3fr_4fr_4fr_3fr_4fr_minmax(6rem,1fr)]";
</script>

<div class="flex flex-col gap-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <h1 class="text-base font-semibold text-foreground">Scraper</h1>
      {#if isLoading}
        <span in:fade={{ duration: 150 }} out:fade={{ duration: 200 }}>
          <LoaderCircle class="size-4 animate-spin text-muted-foreground" />
        </span>
      {:else if hasError}
        <span in:fade={{ duration: 150 }} out:fade={{ duration: 200 }}>
          <SimpleTooltip text={errorMessage ?? "Refresh failed"} side="right" passthrough>
            <AlertCircle class="size-4 text-destructive" />
          </SimpleTooltip>
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <!-- Term Dropdown -->
      <Select.Root
        type="single"
        value={termSelectValue}
        onValueChange={(v: string) => {
          selectedTerm = v === "" ? undefined : v;
        }}
        items={termItems}
      >
        <Select.Trigger
          class="inline-flex items-center justify-between gap-1.5 h-[30px] px-2.5
                 rounded-md text-xs font-medium
                 bg-muted text-muted-foreground
                 hover:text-foreground transition-colors
                 cursor-pointer select-none outline-none
                 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
        >
          <span class="truncate max-w-[120px]">
            {termItems.find((t) => t.value === termSelectValue)?.label ?? "All Terms"}
          </span>
          <ChevronDown class="size-3.5 shrink-0 opacity-60" />
        </Select.Trigger>
        <Select.Portal>
          <Select.Content
            class="border border-border bg-card shadow-md outline-hidden z-50
                   max-h-72 min-w-[140px] w-auto
                   select-none rounded-md p-1
                   data-[state=open]:animate-in data-[state=closed]:animate-out
                   data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0
                   data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95
                   data-[side=top]:slide-in-from-bottom-2
                   data-[side=bottom]:slide-in-from-top-2"
            side="bottom"
            sideOffset={4}
          >
            <Select.ScrollUpButton class="flex w-full items-center justify-center py-0.5">
              <ChevronUp class="size-3.5 text-muted-foreground" />
            </Select.ScrollUpButton>
            <Select.Viewport class="p-0.5">
              {#each termItems as item (item.value)}
                <Select.Item
                  class="rounded-sm outline-hidden flex h-8 w-full select-none items-center
                         px-2.5 text-xs
                         data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground
                         data-[selected]:font-semibold"
                  value={item.value}
                  label={item.label}
                >
                  {item.label}
                </Select.Item>
              {/each}
            </Select.Viewport>
            <Select.ScrollDownButton class="flex w-full items-center justify-center py-0.5">
              <ChevronDown class="size-3.5 text-muted-foreground" />
            </Select.ScrollDownButton>
          </Select.Content>
        </Select.Portal>
      </Select.Root>

      <!-- Period Selector -->
      <div class="bg-muted flex rounded-md p-0.5">
        {#each PERIODS as period (period)}
          <button
            class="rounded px-2.5 py-1 text-xs font-medium transition-colors
              {selectedPeriod === period
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground'}"
            onclick={() => (selectedPeriod = period)}
          >
            {period}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if errorMessage && !currentStats}
    <p class="text-destructive">{errorMessage}</p>
  {:else if currentStats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
      <div class="bg-card border-border rounded-lg border p-3">
        <p class="text-muted-foreground text-xs">Total Scrapes</p>
        <p class="text-2xl font-bold">{formatNumber(currentStats.totalScrapes)}</p>
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <p class="text-muted-foreground text-xs">Success Rate</p>
        {#if currentStats.successRate != null}
          <p class="text-2xl font-bold {successRateColor(currentStats.successRate)}">
            {(currentStats.successRate * 100).toFixed(1)}%
          </p>
        {:else}
          <p class="text-2xl font-bold text-muted-foreground">N/A</p>
        {/if}
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="flex items-center gap-1">
          <p class="text-muted-foreground text-xs">Avg Duration</p>
          <SimpleTooltip text="Average time per successful subject scrape (API fetch + database update)" side="top" passthrough>
            <Info class="size-3 text-muted-foreground/60" />
          </SimpleTooltip>
        </div>
        {#if currentStats.avgDurationMs != null}
          <p class="text-2xl font-bold">{formatDurationMs(currentStats.avgDurationMs)}</p>
        {:else}
          <p class="text-2xl font-bold text-muted-foreground">N/A</p>
        {/if}
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="flex items-center gap-1">
          <p class="text-muted-foreground text-xs">Courses Changed</p>
          <SimpleTooltip text="Total courses that had enrollment or schedule updates detected" side="top" passthrough>
            <Info class="size-3 text-muted-foreground/60" />
          </SimpleTooltip>
        </div>
        <p class="text-2xl font-bold">{formatNumber(currentStats.totalCoursesChanged)}</p>
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="flex items-center gap-1">
          <p class="text-muted-foreground text-xs">Pending Jobs</p>
          <SimpleTooltip text="Scrape jobs queued but not yet started (unlocked jobs waiting for a worker)" side="top" passthrough>
            <Info class="size-3 text-muted-foreground/60" />
          </SimpleTooltip>
        </div>
        <p class="text-2xl font-bold">{formatNumber(currentStats.pendingJobs)}</p>
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="flex items-center gap-1">
          <p class="text-muted-foreground text-xs">Locked Jobs</p>
          <SimpleTooltip text="Scrape jobs currently being processed by a worker" side="top" passthrough>
            <Info class="size-3 text-muted-foreground/60" />
          </SimpleTooltip>
        </div>
        <p class="text-2xl font-bold">{formatNumber(currentStats.lockedJobs)}</p>
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="flex items-center gap-1">
          <p class="text-muted-foreground text-xs">Courses Fetched</p>
          <SimpleTooltip text="Total courses retrieved from Banner API across all successful scrapes" side="top" passthrough>
            <Info class="size-3 text-muted-foreground/60" />
          </SimpleTooltip>
        </div>
        <p class="text-2xl font-bold">{formatNumber(currentStats.totalCoursesFetched)}</p>
      </div>
      <div class="bg-card border-border rounded-lg border p-3">
        <div class="flex items-center gap-1">
          <p class="text-muted-foreground text-xs">Audits Generated</p>
          <SimpleTooltip text="Change records created when course enrollment or schedule data changes" side="top" passthrough>
            <Info class="size-3 text-muted-foreground/60" />
          </SimpleTooltip>
        </div>
        <p class="text-2xl font-bold">{formatNumber(currentStats.totalAuditsGenerated)}</p>
      </div>
    </div>

    <!-- Time-Series Charts -->
    {#if chartData.length > 0}
      <div class="bg-card border-border rounded-lg border p-4">
        <h2 class="mb-3 text-xs font-semibold text-foreground">Scrape Activity</h2>
        <div class="h-[250px]">
          <Chart
            data={tweenedChart.current}
            x="date"
            xScale={scaleTime()}
            y={(d: ChartPoint) => d.success + d.errors}
            yScale={scaleLinear()}
            yDomain={[0, scrapeYMax]}
            yNice
            padding={{ top: 10, bottom: 30, left: 45, right: 10 }}
            tooltip={{ mode: "bisect-x" }}
          >
            <Svg>
              <Axis
                placement="left"
                grid={{ class: "stroke-muted-foreground/15" }}
                rule={false}
                classes={{ tickLabel: "fill-muted-foreground" }}
              />
              <Axis
                placement="bottom"
                format={xAxisFormat(selectedPeriod)}
                grid={{ class: "stroke-muted-foreground/10" }}
                rule={false}
                classes={{ tickLabel: "fill-muted-foreground" }}
              />
              <Area
                y1="success"
                fill="var(--status-green)"
                fillOpacity={0.4}
                curve={curveMonotoneX}
              />
              <Area
                y0="success"
                y1={(d: ChartPoint) => d.success + d.errors}
                fill="var(--status-red)"
                fillOpacity={0.4}
                curve={curveMonotoneX}
              />
              <Highlight lines />
            </Svg>
            <Tooltip.Root
              let:data
              classes={{ root: "text-xs" }}
              variant="none"
            >
              {@const d = data as ChartPoint}
              <div class="bg-card text-card-foreground shadow-md rounded-md px-2.5 py-1.5 flex flex-col gap-y-1">
                <p class="text-muted-foreground font-medium">{d.date.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" })}</p>
                <div class="flex items-center justify-between gap-4">
                  <span class="flex items-center gap-1.5"><span class="inline-block size-2 rounded-full bg-status-green"></span>Successful</span>
                  <span class="tabular-nums font-medium">{d.success}</span>
                </div>
                <div class="flex items-center justify-between gap-4">
                  <span class="flex items-center gap-1.5"><span class="inline-block size-2 rounded-full bg-status-red"></span>Errors</span>
                  <span class="tabular-nums font-medium">{d.errors}</span>
                </div>
              </div>
            </Tooltip.Root>
          </Chart>
        </div>

        <h2 class="mt-4 mb-3 text-xs font-semibold text-foreground">Courses Changed</h2>
        <div class="h-[150px]">
          <Chart
            data={tweenedChart.current}
            x="date"
            xScale={scaleTime()}
            y="coursesChanged"
            yScale={scaleLinear()}
            yDomain={[0, changesYMax]}
            yNice
            padding={{ top: 10, bottom: 30, left: 45, right: 10 }}
            tooltip={{ mode: "bisect-x" }}
          >
            <Svg>
              <Axis
                placement="left"
                grid={{ class: "stroke-muted-foreground/15" }}
                rule={false}
                classes={{ tickLabel: "fill-muted-foreground" }}
              />
              <Axis
                placement="bottom"
                format={xAxisFormat(selectedPeriod)}
                grid={{ class: "stroke-muted-foreground/10" }}
                rule={false}
                classes={{ tickLabel: "fill-muted-foreground" }}
              />
              <Area
                fill="var(--status-blue)"
                fillOpacity={0.3}
                curve={curveMonotoneX}
              />
              <Highlight lines />
            </Svg>
            <Tooltip.Root
              let:data
              classes={{ root: "text-xs" }}
              variant="none"
            >
              {@const d = data as ChartPoint}
              <div class="bg-card text-card-foreground shadow-md rounded-md px-2.5 py-1.5 flex flex-col gap-y-1">
                <p class="text-muted-foreground font-medium">{d.date.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" })}</p>
                <div class="flex items-center justify-between gap-4">
                  <span class="flex items-center gap-1.5"><span class="inline-block size-2 rounded-full bg-status-blue"></span>Changed</span>
                  <span class="tabular-nums font-medium">{d.coursesChanged}</span>
                </div>
              </div>
            </Tooltip.Root>
          </Chart>
        </div>
      </div>
    {/if}

    <!-- Subjects Table -->
    <div class="bg-card border-border rounded-lg border">
      <h2 class="border-border border-b px-3 py-2.5 text-xs font-semibold text-foreground">
        Subjects ({currentSubjects.length})
      </h2>
      <div class="overflow-x-auto">
        <table class="w-full min-w-160 border-collapse text-xs">
          <thead>
            {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
              <tr class="border-border border-b text-left text-muted-foreground">
                {#each headerGroup.headers as header (header.id)}
                  <th
                    class="px-3 py-1.5 text-[10px] font-medium uppercase tracking-wider whitespace-nowrap"
                    class:cursor-pointer={header.column.getCanSort()}
                    class:select-none={header.column.getCanSort()}
                    onclick={header.column.getToggleSortingHandler()}
                  >
                    {#if header.column.getCanSort()}
                      <span class="inline-flex items-center gap-1 hover:text-foreground">
                        {#if typeof header.column.columnDef.header === "string"}
                          {header.column.columnDef.header}
                        {:else}
                          <FlexRender
                            content={header.column.columnDef.header}
                            context={header.getContext()}
                          />
                        {/if}
                        {#if header.column.getIsSorted() === "asc"}
                          <ArrowUp class="size-3.5" />
                        {:else if header.column.getIsSorted() === "desc"}
                          <ArrowDown class="size-3.5" />
                        {:else}
                          <ArrowUpDown class="size-3.5 text-muted-foreground/40" />
                        {/if}
                      </span>
                    {:else if typeof header.column.columnDef.header === "string"}
                      {header.column.columnDef.header}
                    {:else}
                      <FlexRender
                        content={header.column.columnDef.header}
                        context={header.getContext()}
                      />
                    {/if}
                  </th>
                {/each}
              </tr>
            {/each}
          </thead>
          <tbody>
            {#if !currentSubjects.length && !errorMessage}
              <!-- Skeleton loading -->
              {#each Array(12) as _, i (i)}
                <tr class="border-border border-b">
                  {#each columns as col (col.id)}
                    <td class="px-3 py-2">
                      <div
                        class="h-3.5 rounded bg-muted animate-pulse {skeletonWidths[col.id ?? ''] ?? 'w-16'}"
                      ></div>
                    </td>
                  {/each}
                </tr>
              {/each}
            {:else}
              {#each table.getRowModel().rows as row (row.id)}
                {@const subject = row.original}
                {@const isExpanded = expandedSubject === subject.subject}
                {@const rel = relativeTime(new Date(subject.lastScraped), now)}
                <tr
                  class="border-border cursor-pointer border-b transition-colors hover:bg-muted/50
                    {isExpanded ? 'bg-muted/30' : ''}"
                  onclick={() => toggleSubjectDetail(subject.subject)}
                >
                  {#each row.getVisibleCells() as cell (cell.id)}
                    {@const colId = cell.column.id}
                    {#if colId === "subject"}
                      <td class="px-3 py-1.5 font-medium">
                        <div class="flex items-center gap-1.5">
                          {#if isExpanded}
                            <ChevronDown size={12} class="shrink-0" />
                          {:else}
                            <ChevronRight size={12} class="shrink-0" />
                          {/if}
                          <span>{subject.subject}</span>
                          {#if subject.subjectDescription}
                            <span
                              class="text-muted-foreground font-normal text-[10px] max-w-[140px] truncate inline-block align-middle"
                              title={subject.subjectDescription}
                            >{subject.subjectDescription}</span>
                          {/if}
                          {#if subject.trackedCourseCount > 0}
                            <span class="text-muted-foreground/60 font-normal text-[10px]">({subject.trackedCourseCount})</span>
                          {/if}
                        </div>
                      </td>
                    {:else if colId === "status"}
                      <td class="px-3 py-1.5">
                        {#if subject.scheduleState === "paused"}
                          <span class="text-orange-600 dark:text-orange-400">paused</span>
                        {:else if subject.scheduleState === "read_only"}
                          <span class="text-muted-foreground">read only</span>
                        {:else if subject.nextEligibleAt}
                          {@const remainingMs = new Date(subject.nextEligibleAt).getTime() - now.getTime()}
                          {#if remainingMs >= 1000}
                            <span class="text-muted-foreground">{formatDuration(remainingMs)}</span>
                          {:else}
                            <span class="text-green-600 dark:text-green-400 font-medium">ready</span>
                          {/if}
                        {:else}
                          <span class="text-green-600 dark:text-green-400 font-medium">ready</span>
                        {/if}
                      </td>
                    {:else if colId === "interval"}
                      <td class="px-3 py-1.5">
                        <span>{formatInterval(subject.currentIntervalSecs)}</span>
                        {#if subject.timeMultiplier !== 1}
                          <span class="text-muted-foreground ml-0.5">&times;{subject.timeMultiplier}</span>
                        {/if}
                      </td>
                    {:else if colId === "lastScraped"}
                      <td class="px-3 py-1.5">
                        <SimpleTooltip text={formatAbsoluteDate(subject.lastScraped)} side="top" passthrough>
                          <span class="text-muted-foreground">{rel.text === "now" ? "just now" : rel.text}</span>
                        </SimpleTooltip>
                      </td>
                    {:else if colId === "changeRate"}
                      <td class="px-3 py-1.5">
                        <span class={emphasisClass(subject.avgChangeRatio)}>{(subject.avgChangeRatio * 100).toFixed(2)}%</span>
                      </td>
                    {:else if colId === "zeros"}
                      <td class="px-3 py-1.5">
                        <span class={emphasisClass(subject.consecutiveZeroChanges)}>{subject.consecutiveZeroChanges}</span>
                      </td>
                    {:else if colId === "runs"}
                      <td class="px-3 py-1.5">
                        <span class={emphasisClass(subject.recentRuns)}>{subject.recentRuns}</span>
                      </td>
                    {:else if colId === "fails"}
                      <td class="px-3 py-1.5">
                        {#if subject.recentFailures > 0}
                          <span class="text-red-600 dark:text-red-400">{subject.recentFailures}</span>
                        {:else}
                          <span class="text-muted-foreground">{subject.recentFailures}</span>
                        {/if}
                      </td>
                    {/if}
                  {/each}
                </tr>
                <!-- Expanded Detail -->
                {#if isExpanded}
                  <tr class="border-border border-b last:border-b-0">
                    <td colspan={columnCount} class="p-0">
                      <div transition:slide={{ duration: 200 }}>
                        <div class="bg-muted/40 px-4 py-3">
                            <div class="text-xs overflow-x-auto">
                              <div class="min-w-fit">
                              <!-- Header (outside scroll region) -->
                              <div class="grid {detailGridCols} text-muted-foreground border-border/50 border-b">
                                <div class="px-3 py-1.5 font-medium">Time</div>
                                <div class="px-3 py-1.5 font-medium">Duration</div>
                                <div class="px-3 py-1.5 font-medium">Status</div>
                                <div class="px-3 py-1.5 font-medium">Fetched</div>
                                <div class="px-3 py-1.5 font-medium">Changed</div>
                                <div class="px-3 py-1.5 font-medium">%</div>
                                <div class="px-3 py-1.5 font-medium">Audits</div>
                                <div class="px-3 py-1.5 font-medium">Error</div>
                              </div>
                              <!-- Body (scrollable vertically, horizontal clipped to match header) -->
                              <div class="max-h-[280px] overflow-y-auto overflow-x-hidden">
                                {#if detailLoading}
                                  {#each Array(8) as _, i (i)}
                                    <div class="grid {detailGridCols} border-border/50 border-t">
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-16 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-12 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-8 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-10 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-10 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-8 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-8 rounded bg-muted animate-pulse"></div></div>
                                      <div class="px-3 py-1.5"><div class="h-3.5 w-16 rounded bg-muted animate-pulse"></div></div>
                                    </div>
                                  {/each}
                                {:else if subjectDetail && subjectDetail.results.length > 0}
                                  {#each subjectDetail.results as result (result.id)}
                                    {@const detailRel = relativeTime(new Date(result.completedAt), now)}
                                    <div class="grid {detailGridCols} border-border/50 border-t">
                                      <div class="px-3 py-1.5">
                                        <SimpleTooltip text={formatAbsoluteDate(result.completedAt)} side="top" passthrough>
                                          <span class="inline-block min-w-[4.5rem] text-muted-foreground">{detailRel.text === "now" ? "just now" : detailRel.text}</span>
                                        </SimpleTooltip>
                                      </div>
                                      <div class="px-3 py-1.5">{formatDurationMs(result.durationMs)}</div>
                                      <div class="px-3 py-1.5">
                                        {#if result.success}
                                          <span class="text-green-600 dark:text-green-400">ok</span>
                                        {:else}
                                          <span class="text-red-600 dark:text-red-400">fail</span>
                                        {/if}
                                      </div>
                                      <div class="px-3 py-1.5">
                                        <span class={emphasisClass(result.coursesFetched ?? 0)}>{result.coursesFetched ?? "\u2014"}</span>
                                      </div>
                                      <div class="px-3 py-1.5">
                                        <span class={emphasisClass(result.coursesChanged ?? 0)}>{result.coursesChanged ?? "\u2014"}</span>
                                      </div>
                                      <div class="px-3 py-1.5">
                                        {#if result.coursesFetched != null && result.coursesFetched > 0 && result.coursesChanged != null}
                                          <span class={emphasisClass(result.coursesChanged)}>{(result.coursesChanged / result.coursesFetched * 100).toFixed(1)}%</span>
                                        {:else}
                                          <span class="text-muted-foreground">—</span>
                                        {/if}
                                      </div>
                                      <div class="px-3 py-1.5">
                                        <span class={emphasisClass(result.auditsGenerated ?? 0)}>{result.auditsGenerated ?? "\u2014"}</span>
                                      </div>
                                      <div class="px-3 py-1.5">
                                        {#if !result.success && result.errorMessage}
                                          <SimpleTooltip text={result.errorMessage} side="top" passthrough>
                                            <span class="text-red-600 dark:text-red-400 max-w-[12rem] truncate inline-block align-middle">{result.errorMessage}</span>
                                          </SimpleTooltip>
                                        {:else}
                                          <span class="text-muted-foreground">—</span>
                                        {/if}
                                      </div>
                                    </div>
                                  {/each}
                                {:else}
                                  <div class="px-3 py-4 text-center text-muted-foreground text-sm">No recent results.</div>
                                {/if}
                              </div>
                              </div>
                            </div>
                        </div>
                      </div>
                    </td>
                  </tr>
                {/if}
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <!-- Initial loading skeleton -->
    <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
      {#each Array(8) as _, i (i)}
        <div class="bg-card border-border rounded-lg border p-4">
          <div class="h-4 w-24 rounded bg-muted animate-pulse"></div>
          <div class="mt-2 h-8 w-16 rounded bg-muted animate-pulse"></div>
        </div>
      {/each}
    </div>
  {/if}
</div>
