<script lang="ts">
import type { CourseResponse } from "$lib/api";
import { FlexRender, createSvelteTable } from "$lib/components/ui/data-table/index.js";
import { useClipboard } from "$lib/composables/useClipboard.svelte";
import { useOverlayScrollbars } from "$lib/composables/useOverlayScrollbars.svelte";
import {
  RMP_CONFIDENCE_THRESHOLD,
  abbreviateInstructor,
  concernAccentColor,
  formatLocationDisplay,
  formatLocationTooltip,
  formatMeetingDays,
  formatMeetingTimesTooltip,
  formatTimeRange,
  getDeliveryConcern,
  getPrimaryInstructor,
  isAsyncOnline,
  isMeetingTimeTBA,
  isTimeTBA,
  openSeats,
  ratingStyle,
  rmpUrl,
  seatsColor,
  seatsDotColor,
} from "$lib/course";
import { themeStore } from "$lib/stores/theme.svelte";
import { cn, formatNumber, tooltipContentClass } from "$lib/utils";
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  Check,
  ExternalLink,
  RotateCcw,
  Star,
  Triangle,
} from "@lucide/svelte";
import {
  type ColumnDef,
  type SortingState,
  type Updater,
  type VisibilityState,
  getCoreRowModel,
  getSortedRowModel,
} from "@tanstack/table-core";
import { ContextMenu, DropdownMenu, Tooltip } from "bits-ui";
import { flip } from "svelte/animate";
import { cubicOut } from "svelte/easing";
import { fade, slide } from "svelte/transition";
import CourseDetail from "./CourseDetail.svelte";
import SimpleTooltip from "./SimpleTooltip.svelte";

let {
  courses,
  loading,
  sorting = [],
  onSortingChange,
  manualSorting = false,
  subjectMap = {},
  limit = 25,
  columnVisibility = $bindable({}),
}: {
  courses: CourseResponse[];
  loading: boolean;
  sorting?: SortingState;
  onSortingChange?: (sorting: SortingState) => void;
  manualSorting?: boolean;
  subjectMap?: Record<string, string>;
  limit?: number;
  columnVisibility?: VisibilityState;
} = $props();

let expandedCrn: string | null = $state(null);
let tableWrapper: HTMLDivElement = undefined!;
let tableElement: HTMLTableElement = undefined!;
const clipboard = useClipboard(1000);

// Track previous row count so skeleton matches expected result size
let previousRowCount = $state(0);
$effect(() => {
  if (courses.length > 0) {
    previousRowCount = courses.length;
  }
});
let skeletonRowCount = $derived(previousRowCount > 0 ? previousRowCount : limit);

// Animate container height via ResizeObserver
let contentHeight = $state<number | null>(null);
$effect(() => {
  if (!tableElement) return;
  const observer = new ResizeObserver(([entry]) => {
    contentHeight = entry.contentRect.height;
  });
  observer.observe(tableElement);
  return () => observer.disconnect();
});

// Collapse expanded row when the dataset changes to avoid stale detail rows
// and FLIP position calculation glitches from lingering expanded content
$effect(() => {
  courses; // track dependency
  expandedCrn = null;
});

useOverlayScrollbars(() => tableWrapper, {
  overflow: { x: "scroll", y: "hidden" },
  scrollbars: { autoHide: "never" },
});

function resetColumnVisibility() {
  columnVisibility = {};
}

function handleVisibilityChange(updater: Updater<VisibilityState>) {
  const newVisibility = typeof updater === "function" ? updater(columnVisibility) : updater;
  columnVisibility = newVisibility;
}

// visibleColumnIds and hasCustomVisibility derived after column definitions below

function toggleRow(crn: string) {
  expandedCrn = expandedCrn === crn ? null : crn;
}

function primaryInstructorDisplay(course: CourseResponse): string {
  const primary = getPrimaryInstructor(course.instructors);
  if (!primary) return "Staff";
  return abbreviateInstructor(primary.displayName);
}

function primaryRating(
  course: CourseResponse
): { rating: number; count: number; legacyId: number | null } | null {
  const primary = getPrimaryInstructor(course.instructors);
  if (!primary?.rmpRating) return null;
  return {
    rating: primary.rmpRating,
    count: primary.rmpNumRatings ?? 0,
    legacyId: primary.rmpLegacyId ?? null,
  };
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
      return `${formatMeetingDays(mt)} ${formatTimeRange(mt.begin_time, mt.end_time)}`;
    },
    header: "Time",
    enableSorting: true,
  },
  {
    id: "location",
    accessorFn: (row) => formatLocationDisplay(row) ?? "",
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
  getRowId: (row) => String(row.crn),
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

{#snippet columnVisibilityGroup(
    Group: typeof DropdownMenu.Group,
    GroupHeading: typeof DropdownMenu.GroupHeading,
    CheckboxItem: typeof DropdownMenu.CheckboxItem,
    Separator: typeof DropdownMenu.Separator,
    Item: typeof DropdownMenu.Item,
)}
    <Group>
        <GroupHeading
            class="px-2 py-1.5 text-xs font-medium text-muted-foreground"
        >
            Toggle columns
        </GroupHeading>
        {#each columns as col}
            {@const id = col.id!}
            {@const label = typeof col.header === "string" ? col.header : id}
            <CheckboxItem
                checked={columnVisibility[id] !== false}
                closeOnSelect={false}
                onCheckedChange={(checked) => {
                    columnVisibility = {
                        ...columnVisibility,
                        [id]: checked,
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
                    {label}
                {/snippet}
            </CheckboxItem>
        {/each}
    </Group>
    {#if hasCustomVisibility}
        <Separator class="mx-1 my-1 h-px bg-border" />
        <Item
            class="flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm cursor-pointer select-none outline-none data-highlighted:bg-accent data-highlighted:text-accent-foreground"
            onSelect={resetColumnVisibility}
        >
            <RotateCcw class="size-3.5" />
            Reset to default
        </Item>
    {/if}
{/snippet}

<!-- Table with context menu on header -->
<div
    bind:this={tableWrapper}
    class="overflow-x-auto overflow-y-hidden transition-[height] duration-200"
    style:height={contentHeight != null ? `${contentHeight}px` : undefined}
    style:view-transition-name="search-results"
    style:contain="layout"
    data-search-results
>
    <ContextMenu.Root>
        <ContextMenu.Trigger class="contents">
            <table bind:this={tableElement} class="w-full min-w-160 border-collapse text-sm">
                <thead>
                    {#each table.getHeaderGroups() as headerGroup}
                        <tr
                            class="border-b border-border text-left text-muted-foreground"
                        >
                            {#each headerGroup.headers as header}
                                {#if header.column.getIsVisible()}
                                    <th
                                        class="py-2 px-2 font-medium {header.id ===
                                        'seats'
                                            ? 'text-right'
                                            : ''}"
                                        class:cursor-pointer={header.column.getCanSort()}
                                        class:select-none={header.column.getCanSort()}
                                        onclick={header.column.getToggleSortingHandler()}
                                    >
                                        {#if header.column.getCanSort()}
                                            <span
                                                class="inline-flex items-center gap-1"
                                            >
                                                {#if typeof header.column.columnDef.header === "string"}
                                                    {header.column.columnDef
                                                        .header}
                                                {:else}
                                                    <FlexRender
                                                        content={header.column
                                                            .columnDef.header}
                                                        context={header.getContext()}
                                                    />
                                                {/if}
                                                {#if header.column.getIsSorted() === "asc"}
                                                    <ArrowUp class="size-3.5" />
                                                {:else if header.column.getIsSorted() === "desc"}
                                                    <ArrowDown
                                                        class="size-3.5"
                                                    />
                                                {:else}
                                                    <ArrowUpDown
                                                        class="size-3.5 text-muted-foreground/40"
                                                    />
                                                {/if}
                                            </span>
                                        {:else if typeof header.column.columnDef.header === "string"}
                                            {header.column.columnDef.header}
                                        {:else}
                                            <FlexRender
                                                content={header.column.columnDef
                                                    .header}
                                                context={header.getContext()}
                                            />
                                        {/if}
                                    </th>
                                {/if}
                            {/each}
                        </tr>
                    {/each}
                </thead>
                {#if loading && courses.length === 0}
                    <tbody>
                        {#each Array(skeletonRowCount) as _}
                            <tr class="border-b border-border">
                                {#each table.getVisibleLeafColumns() as col}
                                    <td class="py-2.5 px-2">
                                        <div
                                            class="h-4 bg-muted rounded animate-pulse {col.id ===
                                            'seats'
                                                ? 'w-14 ml-auto'
                                                : col.id === 'title'
                                                  ? 'w-40'
                                                  : col.id === 'crn'
                                                    ? 'w-10'
                                                    : 'w-20'}"
                                        ></div>
                                    </td>
                                {/each}
                            </tr>
                        {/each}
                    </tbody>
                {:else if courses.length === 0 && !loading}
                    <tbody>
                        <tr>
                            <td
                                colspan={visibleColumnIds.length}
                                class="py-12 text-center text-muted-foreground"
                            >
                                No courses found. Try adjusting your filters.
                            </td>
                        </tr>
                    </tbody>
                {:else}
                    <!-- No out: transition — Svelte outros break table layout (tbody loses positioning and overlaps) -->
                    {#each table.getRowModel().rows as row, i (row.id)}
                        {@const course = row.original}
                        <tbody
                            class="transition-opacity duration-200 {loading ? 'opacity-45 pointer-events-none' : ''}"
                            animate:flip={{ duration: 300 }}
                            in:fade={{
                                duration: 200,
                                delay: Math.min(i * 25, 300),
                                easing: cubicOut,
                            }}
                        >
                            <tr
                                class="border-b border-border cursor-pointer hover:bg-muted/50 transition-colors whitespace-nowrap {expandedCrn ===
                                course.crn
                                    ? 'bg-muted/30'
                                    : ''}"
                                onclick={() => toggleRow(course.crn)}
                            >
                                {#each row.getVisibleCells() as cell (cell.id)}
                                    {@const colId = cell.column.id}
                                    {#if colId === "crn"}
                                        <td class="py-2 px-2 relative">
                                            <button
                                                class="relative inline-flex items-center rounded-full px-2 py-0.5 border border-border/50 bg-muted/20 hover:bg-muted/40 hover:border-foreground/30 transition-colors duration-150 cursor-copy focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring font-mono text-xs text-muted-foreground/70"
                                                onclick={(e) =>
                                                    clipboard.copy(
                                                        course.crn,
                                                        e,
                                                    )}
                                                onkeydown={(e) => {
                                                    if (
                                                        e.key === "Enter" ||
                                                        e.key === " "
                                                    ) {
                                                        e.preventDefault();
                                                        clipboard.copy(
                                                            course.crn,
                                                            e,
                                                        );
                                                    }
                                                }}
                                                aria-label="Copy CRN {course.crn} to clipboard"
                                            >
                                                {course.crn}
                                                {#if clipboard.copiedValue === course.crn}
                                                    <span
                                                        class="absolute -top-8 left-1/2 -translate-x-1/2 whitespace-nowrap text-xs px-2 py-1 rounded-md bg-green-500/10 border border-green-500/20 text-green-700 dark:text-green-300 pointer-events-none z-10"
                                                        in:fade={{
                                                            duration: 100,
                                                        }}
                                                        out:fade={{
                                                            duration: 200,
                                                        }}
                                                    >
                                                        Copied!
                                                    </span>
                                                {/if}
                                            </button>
                                        </td>
                                    {:else if colId === "course_code"}
                                        {@const subjectDesc =
                                            subjectMap[course.subject]}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            <SimpleTooltip
                                                text={subjectDesc
                                                    ? `${subjectDesc} ${course.courseNumber}`
                                                    : `${course.subject} ${course.courseNumber}`}
                                                delay={200}
                                                side="bottom"
                                                passthrough
                                            >
                                                <span class="font-semibold"
                                                    >{course.subject}
                                                    {course.courseNumber}</span
                                                >{#if course.sequenceNumber}<span
                                                        class="text-muted-foreground"
                                                        >-{course.sequenceNumber}</span
                                                    >{/if}
                                            </SimpleTooltip>
                                        </td>
                                    {:else if colId === "title"}
                                        <td
                                            class="py-2 px-2 font-medium max-w-50 truncate"
                                        >
                                            <SimpleTooltip
                                                text={course.title}
                                                delay={200}
                                                side="bottom"
                                                passthrough
                                            >
                                                <span class="block truncate"
                                                    >{course.title}</span
                                                >
                                            </SimpleTooltip>
                                        </td>
                                    {:else if colId === "instructor"}
                                        {@const primary = getPrimaryInstructor(
                                            course.instructors,
                                        )}
                                        {@const display =
                                            primaryInstructorDisplay(course)}
                                        {@const commaIdx =
                                            display.indexOf(", ")}
                                        {@const ratingData =
                                            primaryRating(course)}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            {#if display === "Staff"}
                                                <span
                                                    class="text-xs text-muted-foreground/60 uppercase"
                                                    >Staff</span
                                                >
                                            {:else}
                                                <SimpleTooltip
                                                    text={primary?.displayName ??
                                                        "Staff"}
                                                    delay={200}
                                                    side="bottom"
                                                    passthrough
                                                >
                                                    {#if commaIdx !== -1}
                                                        <span
                                                            >{display.slice(
                                                                0,
                                                                commaIdx,
                                                            )},
                                                            <span
                                                                class="text-muted-foreground"
                                                                >{display.slice(
                                                                    commaIdx +
                                                                        1,
                                                                )}</span
                                                            ></span
                                                        >
                                                    {:else}
                                                        <span>{display}</span>
                                                    {/if}
                                                </SimpleTooltip>
                                            {/if}
                                            {#if ratingData}
                                                {@const lowConfidence =
                                                    ratingData.count <
                                                    RMP_CONFIDENCE_THRESHOLD}
                                                <Tooltip.Root
                                                    delayDuration={150}
                                                >
                                                    <Tooltip.Trigger>
                                                        <span
                                                            class="ml-1 text-xs font-medium inline-flex items-center gap-0.5"
                                                            style={ratingStyle(
                                                                ratingData.rating,
                                                                themeStore.isDark,
                                                            )}
                                                        >
                                                            {ratingData.rating.toFixed(
                                                                1,
                                                            )}
                                                            {#if lowConfidence}
                                                                <Triangle
                                                                    class="size-2 fill-current"
                                                                />
                                                            {:else}
                                                                <Star
                                                                    class="size-2.5 fill-current"
                                                                />
                                                            {/if}
                                                        </span>
                                                    </Tooltip.Trigger>
                                                    <Tooltip.Content
                                                        side="bottom"
                                                        sideOffset={6}
                                                        class={cn(
                                                            tooltipContentClass,
                                                            "px-2.5 py-1.5",
                                                        )}
                                                    >
                                                        <span
                                                            class="inline-flex items-center gap-1.5 text-xs"
                                                        >
                                                            {ratingData.rating.toFixed(
                                                                1,
                                                            )}/5 · {formatNumber(ratingData.count)}
                                                            ratings
                                                            {#if (ratingData.count ?? 0) < RMP_CONFIDENCE_THRESHOLD}
                                                                (low)
                                                            {/if}
                                                            {#if ratingData.legacyId != null}
                                                                ·
                                                                <a
                                                                    href={rmpUrl(
                                                                        ratingData.legacyId,
                                                                    )}
                                                                    target="_blank"
                                                                    rel="noopener"
                                                                    class="inline-flex items-center gap-0.5 text-muted-foreground hover:text-foreground transition-colors"
                                                                >
                                                                    RMP
                                                                    <ExternalLink
                                                                        class="size-3"
                                                                    />
                                                                </a>
                                                            {/if}
                                                        </span>
                                                    </Tooltip.Content>
                                                </Tooltip.Root>
                                            {/if}
                                        </td>
                                    {:else if colId === "time"}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            <SimpleTooltip
                                                text={formatMeetingTimesTooltip(
                                                    course.meetingTimes,
                                                )}
                                                passthrough
                                            >
                                                {#if isAsyncOnline(course)}
                                                    <span
                                                        class="text-xs text-muted-foreground/60"
                                                        >Async</span
                                                    >
                                                {:else if timeIsTBA(course)}
                                                    <span
                                                        class="text-xs text-muted-foreground/60"
                                                        >TBA</span
                                                    >
                                                {:else}
                                                    {@const mt =
                                                        course.meetingTimes[0]}
                                                    <span>
                                                        {#if !isMeetingTimeTBA(mt)}
                                                            <span
                                                                class="font-mono font-medium"
                                                                >{formatMeetingDays(
                                                                    mt,
                                                                )}</span
                                                            >
                                                            {" "}
                                                        {/if}
                                                        {#if !isTimeTBA(mt)}
                                                            <span
                                                                class="text-muted-foreground"
                                                                >{formatTimeRange(
                                                                    mt.begin_time,
                                                                    mt.end_time,
                                                                )}</span
                                                            >
                                                        {:else}
                                                            <span
                                                                class="text-xs text-muted-foreground/60"
                                                                >TBA</span
                                                            >
                                                        {/if}
                                                        {#if course.meetingTimes.length > 1}
                                                            <span
                                                                class="ml-1 text-xs text-muted-foreground/70 font-medium"
                                                                >+{course
                                                                    .meetingTimes
                                                                    .length -
                                                                    1}</span
                                                            >
                                                        {/if}
                                                    </span>
                                                {/if}
                                            </SimpleTooltip>
                                        </td>
                                    {:else if colId === "location"}
                                        {@const concern =
                                            getDeliveryConcern(course)}
                                        {@const accentColor =
                                            concernAccentColor(concern)}
                                        {@const locTooltip =
                                            formatLocationTooltip(course)}
                                        {@const locDisplay =
                                            formatLocationDisplay(course)}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            {#if locTooltip}
                                                <SimpleTooltip
                                                    text={locTooltip}
                                                    delay={200}
                                                    passthrough
                                                >
                                                    <span
                                                        class="text-muted-foreground"
                                                        class:pl-2={accentColor !==
                                                            null}
                                                        style:border-left={accentColor
                                                            ? `2px solid ${accentColor}`
                                                            : undefined}
                                                    >
                                                        {locDisplay ?? "—"}
                                                    </span>
                                                </SimpleTooltip>
                                            {:else if locDisplay}
                                                <span
                                                    class="text-muted-foreground"
                                                >
                                                    {locDisplay}
                                                </span>
                                            {:else}
                                                <span
                                                    class="text-xs text-muted-foreground/50"
                                                    >—</span
                                                >
                                            {/if}
                                        </td>
                                    {:else if colId === "seats"}
                                        <td
                                            class="py-2 px-2 text-right whitespace-nowrap"
                                        >
                                            <SimpleTooltip
                                                text="{formatNumber(openSeats(
                                                    course,
                                                ))} of {formatNumber(course.maxEnrollment)} seats open, {formatNumber(course.enrollment)} enrolled{course.waitCount >
                                                0
                                                    ? `, ${formatNumber(course.waitCount)} waitlisted`
                                                    : ''}"
                                                delay={200}
                                                side="left"
                                                passthrough
                                            >
                                                <span
                                                    class="inline-flex items-center gap-1.5"
                                                >
                                                    <span
                                                        class="size-1.5 rounded-full {seatsDotColor(
                                                            course,
                                                        )} shrink-0"
                                                    ></span>
                                                    <span
                                                        class="{seatsColor(
                                                            course,
                                                        )} font-medium tabular-nums"
                                                        >{#if openSeats(course) === 0}Full{:else}{openSeats(
                                                                course,
                                                            )} open{/if}</span
                                                    >
                                                    <span
                                                        class="text-muted-foreground/60 tabular-nums"
                                                        >{formatNumber(course.enrollment)}/{formatNumber(course.maxEnrollment)}{#if course.waitCount > 0}
                                                            · WL {formatNumber(course.waitCount)}/{formatNumber(course.waitCapacity)}{/if}</span
                                                    >
                                                </span>
                                            </SimpleTooltip>
                                        </td>
                                    {/if}
                                {/each}
                            </tr>
                            {#if expandedCrn === course.crn}
                                <tr>
                                    <td
                                        colspan={visibleColumnIds.length}
                                        class="p-0"
                                    >
                                        <div
                                            transition:slide={{ duration: 200 }}
                                        >
                                            <CourseDetail {course} />
                                        </div>
                                    </td>
                                </tr>
                            {/if}
                        </tbody>
                    {/each}
                {/if}
            </table>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
            <ContextMenu.Content
                class="z-50 min-w-40 rounded-md border border-border bg-card p-1 text-card-foreground shadow-lg"
                forceMount
            >
                {#snippet child({ wrapperProps, props, open })}
                    {#if open}
                        <div {...wrapperProps}>
                            <div
                                {...props}
                                in:fade={{ duration: 100 }}
                                out:fade={{ duration: 100 }}
                            >
                                {@render columnVisibilityGroup(
                                    ContextMenu.Group,
                                    ContextMenu.GroupHeading,
                                    ContextMenu.CheckboxItem,
                                    ContextMenu.Separator,
                                    ContextMenu.Item,
                                )}
                            </div>
                        </div>
                    {/if}
                {/snippet}
            </ContextMenu.Content>
        </ContextMenu.Portal>
    </ContextMenu.Root>
</div>
