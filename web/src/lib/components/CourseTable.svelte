<script lang="ts">
import type { CourseResponse } from "$lib/api";
import {
  abbreviateInstructor,
  formatTime,
  formatMeetingDays,
  formatLocation,
  getPrimaryInstructor,
  isMeetingTimeTBA,
  isTimeTBA,
} from "$lib/course";
import CourseDetail from "./CourseDetail.svelte";
import { slide } from "svelte/transition";
import { onMount } from "svelte";
import { OverlayScrollbars } from "overlayscrollbars";
import { themeStore } from "$lib/stores/theme.svelte";
import { createSvelteTable, FlexRender } from "$lib/components/ui/data-table/index.js";
import {
  getCoreRowModel,
  getSortedRowModel,
  type ColumnDef,
  type SortingState,
  type VisibilityState,
  type Updater,
} from "@tanstack/table-core";
import { ArrowUp, ArrowDown, ArrowUpDown, Columns3, Check, RotateCcw } from "@lucide/svelte";
import { DropdownMenu, ContextMenu } from "bits-ui";
import { fade, fly } from "svelte/transition";

let {
  courses,
  loading,
  sorting = [],
  onSortingChange,
  manualSorting = false,
}: {
  courses: CourseResponse[];
  loading: boolean;
  sorting?: SortingState;
  onSortingChange?: (sorting: SortingState) => void;
  manualSorting?: boolean;
} = $props();

let expandedCrn: string | null = $state(null);
let tableWrapper: HTMLDivElement = undefined!;

onMount(() => {
  const osInstance = OverlayScrollbars(tableWrapper, {
    overflow: { x: "scroll", y: "hidden" },
    scrollbars: {
      autoHide: "never",
      theme: themeStore.isDark ? "os-theme-dark" : "os-theme-light",
    },
  });

  // React to theme changes
  const unwatch = $effect.root(() => {
    $effect(() => {
      osInstance.options({
        scrollbars: {
          theme: themeStore.isDark ? "os-theme-dark" : "os-theme-light",
        },
      });
    });
  });

  return () => {
    unwatch();
    osInstance.destroy();
  };
});

// Column visibility state
let columnVisibility: VisibilityState = $state({});

const DEFAULT_VISIBILITY: VisibilityState = {};

function resetColumnVisibility() {
  columnVisibility = { ...DEFAULT_VISIBILITY };
}

function handleVisibilityChange(updater: Updater<VisibilityState>) {
  const newVisibility = typeof updater === "function" ? updater(columnVisibility) : updater;
  columnVisibility = newVisibility;
}

// visibleColumnIds and hasCustomVisibility derived after column definitions below

function toggleRow(crn: string) {
  expandedCrn = expandedCrn === crn ? null : crn;
}

function openSeats(course: CourseResponse): number {
  return Math.max(0, course.maxEnrollment - course.enrollment);
}

function seatsColor(course: CourseResponse): string {
  const open = openSeats(course);
  if (open === 0) return "text-status-red";
  if (open <= 5) return "text-yellow-500";
  return "text-status-green";
}

function seatsDotColor(course: CourseResponse): string {
  const open = openSeats(course);
  if (open === 0) return "bg-red-500";
  if (open <= 5) return "bg-yellow-500";
  return "bg-green-500";
}

function primaryInstructorDisplay(course: CourseResponse): string {
  const primary = getPrimaryInstructor(course.instructors);
  if (!primary) return "Staff";
  return abbreviateInstructor(primary.displayName);
}

function ratingColor(rating: number): string {
  if (rating >= 4.0) return "text-status-green";
  if (rating >= 3.0) return "text-yellow-500";
  return "text-status-red";
}

function primaryRating(course: CourseResponse): { rating: number; count: number } | null {
  const primary = getPrimaryInstructor(course.instructors);
  if (!primary?.rmpRating) return null;
  return { rating: primary.rmpRating, count: primary.rmpNumRatings ?? 0 };
}

function timeIsTBA(course: CourseResponse): boolean {
  if (course.meetingTimes.length === 0) return true;
  const mt = course.meetingTimes[0];
  return isMeetingTimeTBA(mt) && isTimeTBA(mt);
}

// Column definitions
const columns: ColumnDef<CourseResponse, unknown>[] = [
  {
    id: "crn",
    accessorKey: "crn",
    header: "CRN",
    enableSorting: false,
  },
  {
    id: "course_code",
    accessorFn: (row) => `${row.subject} ${row.courseNumber}`,
    header: "Course",
    enableSorting: true,
  },
  {
    id: "title",
    accessorKey: "title",
    header: "Title",
    enableSorting: true,
  },
  {
    id: "instructor",
    accessorFn: (row) => primaryInstructorDisplay(row),
    header: "Instructor",
    enableSorting: true,
  },
  {
    id: "time",
    accessorFn: (row) => {
      if (row.meetingTimes.length === 0) return "";
      const mt = row.meetingTimes[0];
      return `${formatMeetingDays(mt)} ${formatTime(mt.begin_time)}`;
    },
    header: "Time",
    enableSorting: true,
  },
  {
    id: "location",
    accessorFn: (row) => formatLocation(row) ?? "",
    header: "Location",
    enableSorting: false,
  },
  {
    id: "seats",
    accessorFn: (row) => openSeats(row),
    header: "Seats",
    enableSorting: true,
  },
];

/** Column IDs that are currently visible */
let visibleColumnIds = $derived(
  columns.map((c) => c.id!).filter((id) => columnVisibility[id] !== false)
);

let hasCustomVisibility = $derived(Object.values(columnVisibility).some((v) => v === false));

function handleSortingChange(updater: Updater<SortingState>) {
  const newSorting = typeof updater === "function" ? updater(sorting) : updater;
  onSortingChange?.(newSorting);
}

const table = createSvelteTable({
  get data() {
    return courses;
  },
  columns,
  state: {
    get sorting() {
      return sorting;
    },
    get columnVisibility() {
      return columnVisibility;
    },
  },
  onSortingChange: handleSortingChange,
  onColumnVisibilityChange: handleVisibilityChange,
  getCoreRowModel: getCoreRowModel(),
  get getSortedRowModel() {
    return manualSorting ? undefined : getSortedRowModel<CourseResponse>();
  },
  get manualSorting() {
    return manualSorting;
  },
  enableSortingRemoval: true,
});
</script>

{#snippet columnVisibilityItems(variant: "dropdown" | "context")}
  {#if variant === "dropdown"}
    <DropdownMenu.Group>
      <DropdownMenu.GroupHeading class="px-2 py-1.5 text-xs font-medium text-muted-foreground">
        Toggle columns
      </DropdownMenu.GroupHeading>
      {#each columns as col}
        {@const id = col.id!}
        {@const label = typeof col.header === "string" ? col.header : id}
        <DropdownMenu.CheckboxItem
          checked={columnVisibility[id] !== false}
          closeOnSelect={false}
          onCheckedChange={(checked) => {
            columnVisibility = { ...columnVisibility, [id]: checked };
          }}
          class="relative flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground"
        >
          {#snippet children({ checked })}
            <span class="flex size-4 items-center justify-center rounded-sm border border-border">
              {#if checked}
                <Check class="size-3" />
              {/if}
            </span>
            {label}
          {/snippet}
        </DropdownMenu.CheckboxItem>
      {/each}
    </DropdownMenu.Group>
    {#if hasCustomVisibility}
      <DropdownMenu.Separator class="mx-1 my-1 h-px bg-border" />
      <DropdownMenu.Item
        class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground"
        onSelect={resetColumnVisibility}
      >
        <RotateCcw class="size-3.5" />
        Reset to default
      </DropdownMenu.Item>
    {/if}
  {:else}
    <ContextMenu.Group>
      <ContextMenu.GroupHeading class="px-2 py-1.5 text-xs font-medium text-muted-foreground">
        Toggle columns
      </ContextMenu.GroupHeading>
      {#each columns as col}
        {@const id = col.id!}
        {@const label = typeof col.header === "string" ? col.header : id}
        <ContextMenu.CheckboxItem
          checked={columnVisibility[id] !== false}
          closeOnSelect={false}
          onCheckedChange={(checked) => {
            columnVisibility = { ...columnVisibility, [id]: checked };
          }}
          class="relative flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground"
        >
          {#snippet children({ checked })}
            <span class="flex size-4 items-center justify-center rounded-sm border border-border">
              {#if checked}
                <Check class="size-3" />
              {/if}
            </span>
            {label}
          {/snippet}
        </ContextMenu.CheckboxItem>
      {/each}
    </ContextMenu.Group>
    {#if hasCustomVisibility}
      <ContextMenu.Separator class="mx-1 my-1 h-px bg-border" />
      <ContextMenu.Item
        class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground"
        onSelect={resetColumnVisibility}
      >
        <RotateCcw class="size-3.5" />
        Reset to default
      </ContextMenu.Item>
    {/if}
  {/if}
{/snippet}

<!-- Toolbar: View columns button -->
<div class="flex items-center justify-end pb-2">
  <DropdownMenu.Root>
    <DropdownMenu.Trigger
      class="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
    >
      <Columns3 class="size-3.5" />
      View
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <DropdownMenu.Content
        class="z-50 min-w-[160px] rounded-md border border-border bg-card p-1 text-card-foreground shadow-lg"
        align="end"
        sideOffset={4}
        forceMount
      >
        {#snippet child({ wrapperProps, props, open })}
          {#if open}
            <div {...wrapperProps}>
              <div {...props} transition:fly={{ duration: 150, y: -10 }}>
                {@render columnVisibilityItems("dropdown")}
              </div>
            </div>
          {/if}
        {/snippet}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
</div>

<!-- Table with context menu on header -->
<div bind:this={tableWrapper} class="overflow-x-auto">
  <ContextMenu.Root>
    <ContextMenu.Trigger class="contents">
      <table class="w-full min-w-[640px] border-collapse text-sm">
        <thead>
          {#each table.getHeaderGroups() as headerGroup}
            <tr class="border-b border-border text-left text-muted-foreground">
              {#each headerGroup.headers as header}
                {#if header.column.getIsVisible()}
                  <th
                    class="py-2 px-2 font-medium {header.id === 'seats' ? 'text-right' : ''}"
                    class:cursor-pointer={header.column.getCanSort()}
                    class:select-none={header.column.getCanSort()}
                    onclick={header.column.getToggleSortingHandler()}
                  >
                    {#if header.column.getCanSort()}
                      <span class="inline-flex items-center gap-1">
                        {#if typeof header.column.columnDef.header === "string"}
                          {header.column.columnDef.header}
                        {:else}
                          <FlexRender content={header.column.columnDef.header} context={header.getContext()} />
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
                      <FlexRender content={header.column.columnDef.header} context={header.getContext()} />
                    {/if}
                  </th>
                {/if}
              {/each}
            </tr>
          {/each}
        </thead>
        <tbody>
          {#if loading && courses.length === 0}
            {#each Array(5) as _}
              <tr class="border-b border-border">
                {#each table.getVisibleLeafColumns() as col}
                  <td class="py-2.5 px-2">
                    <div class="h-4 bg-muted rounded animate-pulse {col.id === 'seats' ? 'w-14 ml-auto' : col.id === 'title' ? 'w-40' : col.id === 'crn' ? 'w-10' : 'w-20'}"></div>
                  </td>
                {/each}
              </tr>
            {/each}
          {:else if courses.length === 0}
            <tr>
              <td colspan={visibleColumnIds.length} class="py-12 text-center text-muted-foreground">
                No courses found. Try adjusting your filters.
              </td>
            </tr>
          {:else}
            {#each table.getRowModel().rows as row (row.id)}
              {@const course = row.original}
              <tr
                class="border-b border-border cursor-pointer hover:bg-muted/50 transition-colors whitespace-nowrap {expandedCrn === course.crn ? 'bg-muted/30' : ''}"
                onclick={() => toggleRow(course.crn)}
              >
                {#each row.getVisibleCells() as cell (cell.id)}
                  {@const colId = cell.column.id}
                  {#if colId === "crn"}
                    <td class="py-2 px-2 font-mono text-xs text-muted-foreground/70">{course.crn}</td>
                  {:else if colId === "course_code"}
                    <td class="py-2 px-2 whitespace-nowrap">
                      <span class="font-semibold">{course.subject} {course.courseNumber}</span>{#if course.sequenceNumber}<span class="text-muted-foreground">-{course.sequenceNumber}</span>{/if}
                    </td>
                  {:else if colId === "title"}
                    <td class="py-2 px-2 font-medium max-w-[200px] truncate" title={course.title}>{course.title}</td>
                  {:else if colId === "instructor"}
                    <td class="py-2 px-2 whitespace-nowrap">
                      {primaryInstructorDisplay(course)}
                      {#if primaryRating(course)}
                        {@const r = primaryRating(course)!}
                        <span
                          class="ml-1 text-xs font-medium {ratingColor(r.rating)}"
                          title="{r.rating.toFixed(1)}/5 ({r.count} ratings)"
                        >{r.rating.toFixed(1)}★</span>
                      {/if}
                    </td>
                  {:else if colId === "time"}
                    <td class="py-2 px-2 whitespace-nowrap">
                      {#if timeIsTBA(course)}
                        <span class="text-xs text-muted-foreground/60">TBA</span>
                      {:else}
                        {@const mt = course.meetingTimes[0]}
                        {#if !isMeetingTimeTBA(mt)}
                          <span class="font-mono font-medium">{formatMeetingDays(mt)}</span>
                          {" "}
                        {/if}
                        {#if !isTimeTBA(mt)}
                          <span class="text-muted-foreground">{formatTime(mt.begin_time)}&ndash;{formatTime(mt.end_time)}</span>
                        {:else}
                          <span class="text-xs text-muted-foreground/60">TBA</span>
                        {/if}
                      {/if}
                    </td>
                  {:else if colId === "location"}
                    <td class="py-2 px-2 whitespace-nowrap">
                      {#if formatLocation(course)}
                        <span class="text-muted-foreground">{formatLocation(course)}</span>
                      {:else}
                        <span class="text-xs text-muted-foreground/50">—</span>
                      {/if}
                    </td>
                  {:else if colId === "seats"}
                    <td class="py-2 px-2 text-right whitespace-nowrap">
                      <span class="inline-flex items-center gap-1.5">
                        <span class="size-1.5 rounded-full {seatsDotColor(course)} shrink-0"></span>
                        <span class="{seatsColor(course)} font-medium tabular-nums">{#if openSeats(course) === 0}Full{:else}{openSeats(course)} open{/if}</span>
                        <span class="text-muted-foreground/60 tabular-nums">{course.enrollment}/{course.maxEnrollment}{#if course.waitCount > 0} · WL {course.waitCount}/{course.waitCapacity}{/if}</span>
                      </span>
                    </td>
                  {/if}
                {/each}
              </tr>
              {#if expandedCrn === course.crn}
                <tr>
                  <td colspan={visibleColumnIds.length} class="p-0">
                    <div transition:slide={{ duration: 200 }}>
                      <CourseDetail {course} />
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          {/if}
        </tbody>
      </table>
    </ContextMenu.Trigger>
    <ContextMenu.Portal>
      <ContextMenu.Content
        class="z-50 min-w-[160px] rounded-md border border-border bg-card p-1 text-card-foreground shadow-lg"
        forceMount
      >
        {#snippet child({ wrapperProps, props, open })}
          {#if open}
            <div {...wrapperProps}>
              <div {...props} in:fade={{ duration: 100 }} out:fade={{ duration: 100 }}>
                {@render columnVisibilityItems("context")}
              </div>
            </div>
          {/if}
        {/snippet}
      </ContextMenu.Content>
    </ContextMenu.Portal>
  </ContextMenu.Root>
</div>
