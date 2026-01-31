<script lang="ts">
import { type ScrapeJobDto, client } from "$lib/api";
import { FlexRender, createSvelteTable } from "$lib/components/ui/data-table/index.js";
import { formatAbsoluteDate } from "$lib/date";
import { formatDuration } from "$lib/time";
import { type ConnectionState, ScrapeJobsStore } from "$lib/ws";
import { ArrowDown, ArrowUp, ArrowUpDown, TriangleAlert } from "@lucide/svelte";
import {
  type ColumnDef,
  type SortingState,
  type Updater,
  getCoreRowModel,
  getSortedRowModel,
} from "@tanstack/table-core";
import { onMount } from "svelte";

let jobs = $state<ScrapeJobDto[]>([]);
let connectionState = $state<ConnectionState>("disconnected");
let initialized = $state(false);
let error = $state<string | null>(null);
let sorting: SortingState = $state([]);
let tick = $state(0);
let subjectMap = $state(new Map<string, string>());

let store: ScrapeJobsStore | undefined;

// Shared tooltip state — single tooltip for all timing cells via event delegation
let tooltipText = $state<string | null>(null);
let tooltipX = $state(0);
let tooltipY = $state(0);

function showTooltip(event: MouseEvent) {
  const target = (event.target as HTMLElement).closest<HTMLElement>("[data-timing-tooltip]");
  if (!target) return;
  tooltipText = target.dataset.timingTooltip ?? null;
  tooltipX = event.clientX;
  tooltipY = event.clientY;
}

function moveTooltip(event: MouseEvent) {
  if (tooltipText === null) return;
  const target = (event.target as HTMLElement).closest<HTMLElement>("[data-timing-tooltip]");
  if (!target) {
    tooltipText = null;
    return;
  }
  tooltipText = target.dataset.timingTooltip ?? null;
  tooltipX = event.clientX;
  tooltipY = event.clientY;
}

function hideTooltip() {
  tooltipText = null;
}

onMount(() => {
  // Tick every second for live time displays
  const tickInterval = setInterval(() => {
    tick++;
  }, 1000);

  // Load subject reference data
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

  // Initialize WebSocket store
  store = new ScrapeJobsStore(() => {
    if (!store) return;
    connectionState = store.getConnectionState();
    initialized = store.isInitialized();
    // getJobs() returns a cached array when unchanged, so only reassign
    // when the reference differs to avoid triggering reactive table rebuilds.
    const next = store.getJobs();
    if (next !== jobs) jobs = next;
  });
  store.connect();

  return () => {
    clearInterval(tickInterval);
    store?.disconnect();
  };
});

function handleSortingChange(updater: Updater<SortingState>) {
  sorting = typeof updater === "function" ? updater(sorting) : updater;
}

// --- Helper functions ---

function formatJobDetails(job: ScrapeJobDto, subjects: Map<string, string>): string {
  const payload = job.targetPayload as Record<string, unknown>;
  switch (job.targetType) {
    case "Subject": {
      const code = payload.subject as string;
      const desc = subjects.get(code);
      return desc ? `${code} \u2014 ${desc}` : code;
    }
    case "CrnList": {
      const crns = payload.crns as string[];
      return `${crns.length} CRNs`;
    }
    case "SingleCrn":
      return `CRN ${payload.crn as string}`;
    case "CourseRange":
      return `${payload.subject as string} ${payload.low as number}\u2013${payload.high as number}`;
    default:
      return JSON.stringify(payload);
  }
}

function priorityColor(priority: string): string {
  const p = priority.toLowerCase();
  if (p === "urgent" || p === "critical") return "text-red-500";
  if (p === "high") return "text-orange-500";
  if (p === "low") return "text-muted-foreground";
  return "text-foreground";
}

function retryColor(retryCount: number, maxRetries: number): string {
  if (retryCount >= maxRetries && maxRetries > 0) return "text-red-500";
  if (retryCount > 0) return "text-amber-500";
  return "text-muted-foreground";
}

function statusColor(status: string): { text: string; dot: string } {
  switch (status) {
    case "processing":
      return { text: "text-blue-500", dot: "bg-blue-500" };
    case "pending":
      return { text: "text-green-500", dot: "bg-green-500" };
    case "scheduled":
      return { text: "text-muted-foreground", dot: "bg-muted-foreground" };
    case "staleLock":
      return { text: "text-red-500", dot: "bg-red-500" };
    case "exhausted":
      return { text: "text-red-500", dot: "bg-red-500" };
    default:
      return { text: "text-muted-foreground", dot: "bg-muted-foreground" };
  }
}

function formatStatusLabel(status: string): string {
  // Convert camelCase to separate words, capitalize first letter
  return status.replace(/([a-z])([A-Z])/g, "$1 $2").replace(/^\w/, (c) => c.toUpperCase());
}

function lockDurationColor(ms: number): string {
  const minutes = ms / 60_000;
  if (minutes >= 8) return "text-red-500";
  if (minutes >= 5) return "text-amber-500";
  return "text-foreground";
}

function overdueDurationColor(ms: number): string {
  const minutes = ms / 60_000;
  if (minutes >= 5) return "text-red-500";
  return "text-amber-500";
}

// --- Table columns ---

const columns: ColumnDef<ScrapeJobDto, unknown>[] = [
  {
    id: "id",
    accessorKey: "id",
    header: "ID",
    enableSorting: false,
  },
  {
    id: "status",
    accessorKey: "status",
    header: "Status",
    enableSorting: true,
    sortingFn: (rowA, rowB) => {
      const order: Record<string, number> = {
        processing: 0,
        staleLock: 1,
        pending: 2,
        scheduled: 3,
        exhausted: 4,
      };
      const a = order[rowA.original.status] ?? 3;
      const b = order[rowB.original.status] ?? 3;
      return a - b;
    },
  },
  {
    id: "targetType",
    accessorKey: "targetType",
    header: "Type",
    enableSorting: false,
  },
  {
    id: "details",
    accessorFn: () => "",
    header: "Details",
    enableSorting: false,
  },
  {
    id: "priority",
    accessorKey: "priority",
    header: "Priority",
    enableSorting: true,
    sortingFn: (rowA, rowB) => {
      const order: Record<string, number> = {
        critical: 0,
        urgent: 0,
        high: 1,
        normal: 2,
        medium: 2,
        low: 3,
      };
      const a = order[String(rowA.original.priority).toLowerCase()] ?? 2;
      const b = order[String(rowB.original.priority).toLowerCase()] ?? 2;
      return a - b;
    },
  },
  {
    id: "timing",
    accessorFn: (row) => {
      if (row.lockedAt) return Date.now() - new Date(row.lockedAt).getTime();
      return Date.now() - new Date(row.queuedAt).getTime();
    },
    header: "Timing",
    enableSorting: true,
  },
];

const table = createSvelteTable({
  get data() {
    return jobs;
  },
  getRowId: (row) => String(row.id),
  columns,
  state: {
    get sorting() {
      return sorting;
    },
  },
  onSortingChange: handleSortingChange,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  enableSortingRemoval: true,
});

const skeletonWidths: Record<string, string> = {
  id: "w-6",
  status: "w-20",
  targetType: "w-16",
  details: "w-32",
  priority: "w-16",
  timing: "w-32",
};

// Unified timing display: shows the most relevant duration for the job's current state.
// Uses _tick dependency so Svelte re-evaluates every second.
function getTimingDisplay(
  job: ScrapeJobDto,
  _tick: number
): { text: string; colorClass: string; icon: "warning" | "none"; tooltip: string } {
  const now = Date.now();
  const queuedTime = new Date(job.queuedAt).getTime();
  const executeTime = new Date(job.executeAt).getTime();

  if (job.status === "processing" || job.status === "staleLock") {
    const lockedTime = job.lockedAt ? new Date(job.lockedAt).getTime() : now;
    const processingMs = now - lockedTime;
    const waitedMs = lockedTime - queuedTime;

    const prefix = job.status === "staleLock" ? "stale" : "processing";
    const colorClass =
      job.status === "staleLock" ? "text-red-500" : lockDurationColor(processingMs);

    const tooltipLines = [
      `Queued: ${formatAbsoluteDate(job.queuedAt)}`,
      `Waited: ${formatDuration(Math.max(0, waitedMs))}`,
    ];
    if (job.lockedAt) {
      tooltipLines.push(`Locked: ${formatAbsoluteDate(job.lockedAt)}`);
    }
    tooltipLines.push(
      `${job.status === "staleLock" ? "Stale for" : "Processing"}: ${formatDuration(processingMs)}`
    );

    return {
      text: `${prefix} ${formatDuration(processingMs)}`,
      colorClass,
      icon: job.status === "staleLock" ? "warning" : "none",
      tooltip: tooltipLines.join("\n"),
    };
  }

  if (job.status === "exhausted") {
    const tooltipLines = [
      `Queued: ${formatAbsoluteDate(job.queuedAt)}`,
      `Retries: ${job.retryCount}/${job.maxRetries} exhausted`,
    ];
    return {
      text: "exhausted",
      colorClass: "text-red-500",
      icon: "warning",
      tooltip: tooltipLines.join("\n"),
    };
  }

  // Scheduled (future execute_at)
  const executeAtDiff = now - executeTime;
  if (job.status === "scheduled" || executeAtDiff < 0) {
    const tooltipLines = [
      `Queued: ${formatAbsoluteDate(job.queuedAt)}`,
      `Executes: ${formatAbsoluteDate(job.executeAt)}`,
    ];
    return {
      text: `in ${formatDuration(Math.abs(executeAtDiff))}`,
      colorClass: "text-muted-foreground",
      icon: "none",
      tooltip: tooltipLines.join("\n"),
    };
  }

  // Pending (overdue — execute_at is in the past, waiting to be picked up)
  const waitingMs = now - queuedTime;
  const tooltipLines = [
    `Queued: ${formatAbsoluteDate(job.queuedAt)}`,
    `Waiting: ${formatDuration(waitingMs)}`,
  ];
  return {
    text: `waiting ${formatDuration(waitingMs)}`,
    colorClass: overdueDurationColor(waitingMs),
    icon: "warning",
    tooltip: tooltipLines.join("\n"),
  };
}
</script>

<div class="flex items-center justify-between mb-4">
  <h1 class="text-lg font-semibold text-foreground">Scrape Jobs</h1>
  <div class="flex items-center gap-2 text-sm">
    {#if connectionState === "connected"}
      <span class="inline-flex items-center gap-1.5">
        <span class="size-2 shrink-0 rounded-full bg-green-500"></span>
        <span class="text-green-500">Live</span>
      </span>
    {:else if connectionState === "reconnecting"}
      <span class="inline-flex items-center gap-1.5">
        <span class="size-2 shrink-0 rounded-full bg-amber-500"></span>
        <span class="text-amber-500">Reconnecting...</span>
      </span>
    {:else}
      <span class="inline-flex items-center gap-2">
        <span class="inline-flex items-center gap-1.5">
          <span class="size-2 shrink-0 rounded-full bg-red-500"></span>
          <span class="text-red-500">Disconnected</span>
        </span>
        <button
          class="rounded-md bg-muted px-2 py-0.5 text-xs font-medium text-foreground hover:bg-muted/80 transition-colors"
          onclick={() => store?.retry()}
        >
          Retry
        </button>
      </span>
    {/if}
  </div>
</div>

{#if error}
  <p class="text-destructive">{error}</p>
{:else}
  <div class="bg-card border-border overflow-hidden rounded-lg border">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <table
      class="w-full border-collapse text-xs"
      onmouseenter={showTooltip}
      onmousemove={moveTooltip}
      onmouseleave={hideTooltip}
    >
      <thead>
        {#each table.getHeaderGroups() as headerGroup}
          <tr class="border-b border-border text-left text-muted-foreground">
            {#each headerGroup.headers as header}
              <th
                class="px-3 py-2.5 font-medium whitespace-nowrap"
                class:cursor-pointer={header.column.getCanSort()}
                class:select-none={header.column.getCanSort()}
                onclick={header.column.getToggleSortingHandler()}
              >
                {#if header.column.getCanSort()}
                  <span class="inline-flex items-center gap-1">
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
      {#if !initialized}
        <tbody>
          {#each Array(5) as _}
            <tr class="border-b border-border">
              {#each columns as col}
                <td class="px-3 py-2.5">
                  <div
                    class="h-3.5 rounded bg-muted animate-pulse {skeletonWidths[col.id ?? ''] ?? 'w-20'}"
                  ></div>
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      {:else if jobs.length === 0}
        <tbody>
          <tr>
            <td colspan={columns.length} class="py-12 text-center text-muted-foreground">
              No scrape jobs found.
            </td>
          </tr>
        </tbody>
      {:else}
        <tbody>
          {#each table.getRowModel().rows as row (row.id)}
            {@const job = row.original}
            {@const sc = statusColor(job.status)}
            {@const timingDisplay = getTimingDisplay(job, tick)}
            <tr
              class="border-b border-border last:border-b-0 hover:bg-muted/50 transition-colors"
            >
              {#each row.getVisibleCells() as cell (cell.id)}
                {@const colId = cell.column.id}
                {#if colId === "id"}
                  <td class="px-3 py-2.5 tabular-nums text-muted-foreground/70 w-12">{job.id}</td>
                {:else if colId === "status"}
                  <td class="px-3 py-2.5 whitespace-nowrap">
                    <span class="inline-flex items-center gap-1.5">
                      <span class="size-1.5 shrink-0 rounded-full {sc.dot}"></span>
                      <span class="flex flex-col leading-tight">
                        <span class={sc.text}>{formatStatusLabel(job.status)}</span>
                        {#if job.maxRetries > 0}
                          <span class="text-[10px] {retryColor(job.retryCount, job.maxRetries)}">
                            {job.retryCount}/{job.maxRetries} retries
                          </span>
                        {/if}
                      </span>
                    </span>
                  </td>
                {:else if colId === "targetType"}
                  <td class="px-3 py-2.5 whitespace-nowrap">
                    <span
                      class="inline-flex items-center rounded-md bg-muted/60 px-1.5 py-0.5 font-mono text-[11px] text-muted-foreground"
                    >
                      {job.targetType}
                    </span>
                  </td>
                {:else if colId === "details"}
                  <td class="px-3 py-2.5 max-w-48 truncate text-muted-foreground" title={formatJobDetails(job, subjectMap)}>
                    {formatJobDetails(job, subjectMap)}
                  </td>
                {:else if colId === "priority"}
                  <td class="px-3 py-2.5 whitespace-nowrap">
                    <span class="font-medium capitalize {priorityColor(job.priority)}">
                      {job.priority}
                    </span>
                  </td>
                {:else if colId === "timing"}
                  <td class="px-3 py-2.5 whitespace-nowrap">
                    <span
                      class="inline-flex items-center gap-1.5 tabular-nums text-foreground"
                      data-timing-tooltip={timingDisplay.tooltip}
                    >
                      <span class="size-3.5 shrink-0 inline-flex items-center justify-center {timingDisplay.colorClass}">
                        {#if timingDisplay.icon === "warning"}
                          <TriangleAlert class="size-3.5" />
                        {/if}
                      </span>
                      {timingDisplay.text}
                    </span>
                  </td>
                {/if}
              {/each}
            </tr>
          {/each}
        </tbody>
      {/if}
    </table>
  </div>

  {#if tooltipText !== null}
    <div
      class="pointer-events-none fixed z-50 bg-card text-card-foreground text-xs border border-border rounded-md px-2.5 py-1.5 shadow-sm whitespace-pre-line max-w-max text-left"
      style="left: {tooltipX + 12}px; top: {tooltipY + 12}px;"
    >
      {tooltipText}
    </div>
  {/if}
{/if}
