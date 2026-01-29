<script lang="ts">
import { type ScrapeJob, type ScrapeJobsResponse, client } from "$lib/api";
import SimpleTooltip from "$lib/components/SimpleTooltip.svelte";
import { FlexRender, createSvelteTable } from "$lib/components/ui/data-table/index.js";
import { formatAbsoluteDate, formatRelativeDate } from "$lib/date";
import { ArrowDown, ArrowUp, ArrowUpDown } from "@lucide/svelte";
import {
  type ColumnDef,
  type SortingState,
  type Updater,
  getCoreRowModel,
  getSortedRowModel,
} from "@tanstack/table-core";
import { onMount } from "svelte";

let data = $state<ScrapeJobsResponse | null>(null);
let error = $state<string | null>(null);
let sorting: SortingState = $state([]);

onMount(async () => {
  try {
    data = await client.getAdminScrapeJobs();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load scrape jobs";
  }
});

function handleSortingChange(updater: Updater<SortingState>) {
  sorting = typeof updater === "function" ? updater(sorting) : updater;
}

function priorityColor(priority: string): string {
  const p = priority.toLowerCase();
  if (p === "urgent") return "text-red-500";
  if (p === "low") return "text-muted-foreground";
  return "text-foreground";
}

function retryColor(retryCount: number, maxRetries: number): string {
  if (retryCount >= maxRetries && maxRetries > 0) return "text-red-500";
  if (retryCount > 0) return "text-amber-500";
  return "text-muted-foreground";
}

const columns: ColumnDef<ScrapeJob, unknown>[] = [
  {
    id: "id",
    accessorKey: "id",
    header: "ID",
    enableSorting: false,
  },
  {
    id: "targetType",
    accessorKey: "targetType",
    header: "Type",
    enableSorting: false,
  },
  {
    id: "priority",
    accessorKey: "priority",
    header: "Priority",
    enableSorting: true,
    sortingFn: (rowA, rowB) => {
      const order: Record<string, number> = { urgent: 0, high: 1, normal: 2, low: 3 };
      const a = order[String(rowA.original.priority).toLowerCase()] ?? 2;
      const b = order[String(rowB.original.priority).toLowerCase()] ?? 2;
      return a - b;
    },
  },
  {
    id: "executeAt",
    accessorKey: "executeAt",
    header: "Execute At",
    enableSorting: true,
  },
  {
    id: "createdAt",
    accessorKey: "createdAt",
    header: "Created At",
    enableSorting: false,
  },
  {
    id: "retries",
    accessorFn: (row) => row.retryCount,
    header: "Retries",
    enableSorting: false,
  },
  {
    id: "status",
    accessorFn: (row) => (row.lockedAt ? "Locked" : "Pending"),
    header: "Status",
    enableSorting: true,
  },
];

const table = createSvelteTable({
  get data() {
    return data?.jobs ?? [];
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
  id: "w-8",
  targetType: "w-16",
  priority: "w-16",
  executeAt: "w-28",
  createdAt: "w-28",
  retries: "w-12",
  status: "w-20",
};
</script>

<h1 class="mb-4 text-lg font-semibold text-foreground">Scrape Jobs</h1>

{#if error}
  <p class="text-destructive">{error}</p>
{:else}
  <div class="bg-card border-border overflow-hidden rounded-lg border">
    <table class="w-full border-collapse text-sm">
      <thead>
        {#each table.getHeaderGroups() as headerGroup}
          <tr class="border-b border-border text-left text-muted-foreground">
            {#each headerGroup.headers as header}
              <th
                class="px-4 py-3 font-medium"
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
      {#if !data}
        <tbody>
          {#each Array(5) as _}
            <tr class="border-b border-border">
              {#each columns as col}
                <td class="px-4 py-3">
                  <div
                    class="h-4 rounded bg-muted animate-pulse {skeletonWidths[col.id ?? ''] ?? 'w-20'}"
                  ></div>
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      {:else if data.jobs.length === 0}
        <tbody>
          <tr>
            <td colspan={columns.length} class="py-12 text-center text-muted-foreground">
              No scrape jobs found.
            </td>
          </tr>
        </tbody>
      {:else}
        <tbody>
          {#each table.getRowModel().rows as row}
            {@const job = row.original}
            <tr class="border-b border-border last:border-b-0 hover:bg-muted/50 transition-colors">
              {#each row.getVisibleCells() as cell (cell.id)}
                {@const colId = cell.column.id}
                {#if colId === "id"}
                  <td class="px-4 py-3 tabular-nums text-muted-foreground">{job.id}</td>
                {:else if colId === "targetType"}
                  <td class="px-4 py-3">
                    <span
                      class="inline-flex items-center rounded-md bg-muted/60 px-2 py-0.5 font-mono text-xs text-muted-foreground"
                    >
                      {job.targetType}
                    </span>
                  </td>
                {:else if colId === "priority"}
                  <td class="px-4 py-3">
                    <span class="font-medium capitalize {priorityColor(job.priority)}">
                      {job.priority}
                    </span>
                  </td>
                {:else if colId === "executeAt"}
                  <td class="px-4 py-3">
                    <SimpleTooltip text={formatAbsoluteDate(job.executeAt)} passthrough>
                      <span class="text-muted-foreground">
                        {formatRelativeDate(job.executeAt)}
                      </span>
                    </SimpleTooltip>
                  </td>
                {:else if colId === "createdAt"}
                  <td class="px-4 py-3">
                    <SimpleTooltip text={formatAbsoluteDate(job.createdAt)} passthrough>
                      <span class="text-muted-foreground">
                        {formatRelativeDate(job.createdAt)}
                      </span>
                    </SimpleTooltip>
                  </td>
                {:else if colId === "retries"}
                  <td class="px-4 py-3">
                    <span class="tabular-nums {retryColor(job.retryCount, job.maxRetries)}">
                      {job.retryCount}/{job.maxRetries}
                    </span>
                  </td>
                {:else if colId === "status"}
                  <td class="px-4 py-3">
                    {#if job.lockedAt}
                      <SimpleTooltip
                        text="Locked since {formatAbsoluteDate(job.lockedAt)}"
                        passthrough
                      >
                        <span class="inline-flex items-center gap-1.5">
                          <span class="size-2 shrink-0 rounded-full bg-amber-500"></span>
                          <span class="text-amber-500">Locked</span>
                        </span>
                      </SimpleTooltip>
                    {:else}
                      <span class="inline-flex items-center gap-1.5">
                        <span class="size-2 shrink-0 rounded-full bg-green-500"></span>
                        <span class="text-green-500">Pending</span>
                      </span>
                    {/if}
                  </td>
                {/if}
              {/each}
            </tr>
          {/each}
        </tbody>
      {/if}
    </table>
  </div>
{/if}
