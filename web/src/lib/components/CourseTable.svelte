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
import { formatNumber } from "$lib/utils";
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
import { ContextMenu, DropdownMenu } from "bits-ui";
import { flip } from "svelte/animate";
import { fade, slide } from "svelte/transition";
import { useTooltipDelegation } from "$lib/composables/useTooltipDelegation";
import CourseCard from "./CourseCard.svelte";
import CourseDetail from "./CourseDetail.svelte";
import LazyRichTooltip from "./LazyRichTooltip.svelte";

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

// Skip FLIP on initial load: all items are new so there's nothing to animate,
// but Svelte still measures every element's position. $effect runs AFTER the
// DOM update, so hadResults is still false during the first results render.
let hadResults = $state(false);
$effect(() => {
  if (courses.length > 0) hadResults = true;
});

useOverlayScrollbars(() => tableWrapper, {
  overflow: { x: "scroll", y: "hidden" },
  scrollbars: { autoHide: "never" },
});

// Singleton tooltip: one imperative tooltip element for all data-tooltip cells
$effect(() => {
  if (!tableElement) return;
  const { destroy } = useTooltipDelegation(tableElement);
  return destroy;
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

function timeIsTBA(course: CourseResponse): boolean {
  if (course.meetingTimes.length === 0) return true;
  const mt = course.meetingTimes[0];
  return isMeetingTimeTBA(mt) && isTimeTBA(mt);
}

// Skeleton widths per column ID (used by the raw-HTML skeleton builder)
const SKELETON_WIDTHS: Record<string, string> = {
  crn: "w-10",
  course_code: "w-20",
  title: "w-40",
  instructor: "w-20",
  time: "w-20",
  location: "w-20",
  seats: "w-14 ml-auto",
};

/** Build skeleton rows as a raw HTML string — one innerHTML instead of N×M reactive nodes. */
function buildSkeletonHtml(colIds: string[], rowCount: number): string {
  const cells = colIds
    .map((id) => {
      const w = SKELETON_WIDTHS[id] ?? "w-20";
      return `<td class="py-2.5 px-2"><div class="h-4 bg-muted rounded animate-pulse ${w}"></div></td>`;
    })
    .join("");
  const row = `<tr class="border-b border-border">${cells}</tr>`;
  return row.repeat(rowCount);
}

/** Build mobile card skeletons as raw HTML. */
function buildCardSkeletonHtml(count: number): string {
  const card = `<div class="rounded-lg border border-border bg-card p-3 animate-pulse"><div class="flex items-baseline justify-between gap-2"><div class="flex items-baseline gap-1.5"><div class="h-4 w-16 bg-muted rounded"></div><div class="h-4 w-32 bg-muted rounded"></div></div><div class="h-4 w-10 bg-muted rounded"></div></div><div class="flex items-center justify-between gap-2 mt-1"><div class="h-3 w-24 bg-muted rounded"></div><div class="h-3 w-20 bg-muted rounded"></div></div></div>`;
  return card.repeat(count);
}

// Calculate max subject code length for alignment
let maxSubjectLength = $derived(
  courses.length > 0 ? Math.max(...courses.map((c) => c.subject.length)) : 3
);

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

{#snippet emptyState()}
    <div class="py-8 text-center text-sm text-muted-foreground">
        No courses found. Try adjusting your filters.
    </div>
{/snippet}

{#snippet columnVisibilityGroup(
    Group: typeof DropdownMenu.Group,
    GroupHeading: typeof DropdownMenu.GroupHeading,
    CheckboxItem: typeof DropdownMenu.CheckboxItem,
    Separator: typeof DropdownMenu.Separator,
    Item: typeof DropdownMenu.Item,
)}
    <Group>
        <GroupHeading
            class="px-2 py-1.5 text-xs font-medium text-muted-foreground select-none"
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

<!-- Mobile cards -->
<div class="flex flex-col gap-2 sm:hidden">
    {#if loading && courses.length === 0}
        {@html buildCardSkeletonHtml(skeletonRowCount)}
    {:else if courses.length === 0 && !loading}
        {@render emptyState()}
    {:else}
        {#each courses as course (course.crn)}
            <div class="transition-opacity duration-200 {loading ? 'opacity-45 pointer-events-none' : ''}">
                <CourseCard
                    {course}
                    expanded={expandedCrn === course.crn}
                    onToggle={() => toggleRow(course.crn)}
                />
            </div>
        {/each}
    {/if}
</div>

<!-- CourseTable uses sm: (640px) for card/table switch intentionally.
     The table renders well at smaller widths than the full app layout (which uses md: 768px). -->

<!-- Desktop table
     IMPORTANT: !important flags on hidden/block are required because OverlayScrollbars
     applies inline styles (style="display: ...") to set up its custom scrollbar UI.
     Inline styles have higher CSS specificity than class utilities, so without !important,
     the table would remain visible at all viewport widths instead of hiding below 640px. -->
<div
    bind:this={tableWrapper}
    class="!hidden sm:!block overflow-x-auto overflow-y-hidden transition-[height] duration-200"
    style:height={contentHeight != null ? `${contentHeight}px` : undefined}
    style:view-transition-name="search-results"
    style:contain="layout"
    data-search-results
>
    <ContextMenu.Root>
        <ContextMenu.Trigger class="contents">
            <table bind:this={tableElement} class="w-full min-w-120 md:min-w-160 border-collapse text-sm">
                <thead>
                    {#each table.getHeaderGroups() as headerGroup}
                        <tr
                            class="border-b border-border text-left text-muted-foreground"
                        >
                            {#each headerGroup.headers as header}
                                {#if header.column.getIsVisible()}
                                    <th
                                        class="py-2 px-2 font-medium select-none {header.id ===
                                        'seats'
                                            ? 'text-right'
                                            : ''}"
                                        class:cursor-pointer={header.column.getCanSort()}
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
                        {@html buildSkeletonHtml(visibleColumnIds, skeletonRowCount)}
                    </tbody>
                {:else if courses.length === 0 && !loading}
                    <tbody>
                        <tr>
                            <td
                                colspan={visibleColumnIds.length}
                                class="py-12 text-center text-muted-foreground"
                            >
                                {@render emptyState()}
                            </td>
                        </tr>
                    </tbody>
                {:else}
                    <!-- No out: transition — Svelte outros break table layout (tbody loses positioning and overlaps) -->
                    {#each table.getRowModel().rows as row, i (row.id)}
                        {@const course = row.original}
                        <tbody
                            class="transition-opacity duration-200 animate-fade-in {loading ? 'opacity-45 pointer-events-none' : ''}"
                            animate:flip={{ duration: hadResults ? 300 : 0 }}
                            style:animation-delay="{Math.min(i * 25, 300)}ms"
                        >
                            <tr
                                class="border-b border-border cursor-pointer hover:bg-muted/50 transition-colors whitespace-nowrap {expandedCrn ===
                                course.crn
                                    ? 'bg-muted/30'
                                    : ''}"
                                onclick={() => toggleRow(course.crn)}
                            >
                                {#each visibleColumnIds as colId (colId)}
                                    {#if colId === "crn"}
                                        <td class="py-2 px-2 relative">
                                            <button
                                                class="relative inline-flex items-center rounded-full px-2 py-0.5 border border-border/50 bg-muted/20 hover:bg-muted/40 hover:border-foreground/30 transition-colors duration-150 cursor-copy select-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-ring font-mono text-xs text-muted-foreground/70"
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
                                        {@const paddedSubject =
                                            course.subject.padStart(
                                                maxSubjectLength,
                                                " ",
                                            )}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            <span
                                                data-tooltip={subjectDesc
                                                    ? `${subjectDesc} ${course.courseNumber}`
                                                    : `${course.subject} ${course.courseNumber}`}
                                                data-tooltip-side="bottom"
                                                data-tooltip-delay="200"
                                            >
                                                <span class="font-semibold font-mono tracking-tight whitespace-pre">{paddedSubject} {course.courseNumber}</span>{#if course.sequenceNumber}<span class="text-muted-foreground font-mono tracking-tight">-{course.sequenceNumber}</span>{/if}
                                            </span>
                                        </td>
                                    {:else if colId === "title"}
                                        <td
                                            class="py-2 px-2 font-medium max-w-50 truncate"
                                        >
                                            <span
                                                class="block truncate"
                                                data-tooltip={course.title}
                                                data-tooltip-side="bottom"
                                                data-tooltip-delay="200"
                                            >{course.title}</span>
                                        </td>
                                    {:else if colId === "instructor"}
                                        {@const primary = getPrimaryInstructor(
                                            course.instructors,
                                        )}
                                        {@const display = primary
                                            ? abbreviateInstructor(primary.displayName)
                                            : "Staff"}
                                        {@const commaIdx =
                                            display.indexOf(", ")}
                                        {@const ratingData = primary?.rmpRating != null
                                            ? {
                                                rating: primary.rmpRating,
                                                count: primary.rmpNumRatings ?? 0,
                                                legacyId: primary.rmpLegacyId ?? null,
                                            }
                                            : null}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            {#if display === "Staff"}
                                                <span
                                                    class="text-xs text-muted-foreground/60 uppercase select-none"
                                                    >Staff</span
                                                >
                                            {:else}
                                                <span
                                                    data-tooltip={primary?.displayName ?? "Staff"}
                                                    data-tooltip-side="bottom"
                                                    data-tooltip-delay="200"
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
                                                </span>
                                            {/if}
                                            {#if ratingData}
                                                {@const lowConfidence =
                                                    ratingData.count <
                                                    RMP_CONFIDENCE_THRESHOLD}
                                                <LazyRichTooltip
                                                    side="bottom"
                                                    sideOffset={6}
                                                    contentClass="px-2.5 py-1.5"
                                                >
                                                    {#snippet children()}
                                                        <span
                                                            class="ml-1 text-xs font-medium inline-flex items-center gap-0.5 select-none"
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
                                                    {/snippet}
                                                    {#snippet content()}
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
                                                    {/snippet}
                                                </LazyRichTooltip>
                                            {/if}
                                        </td>
                                    {:else if colId === "time"}
                                        <td
                                            class="py-2 px-2 whitespace-nowrap"
                                            data-tooltip={formatMeetingTimesTooltip(course.meetingTimes)}
                                        >
                                                {#if isAsyncOnline(course)}
                                                    <span
                                                        class="text-xs text-muted-foreground/60 select-none"
                                                        >Async</span
                                                    >
                                                {:else if timeIsTBA(course)}
                                                    <span
                                                        class="text-xs text-muted-foreground/60 select-none"
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
                                                                class="text-xs text-muted-foreground/60 select-none"
                                                                >TBA</span
                                                            >
                                                        {/if}
                                                        {#if course.meetingTimes.length > 1}
                                                            <span
                                                                class="ml-1 text-xs text-muted-foreground/70 font-medium select-none"
                                                                >+{course
                                                                    .meetingTimes
                                                                    .length -
                                                                    1}</span
                                                            >
                                                        {/if}
                                                    </span>
                                                {/if}
                                        </td>
                                    {:else if colId === "location"}
                                        {@const concern =
                                            getDeliveryConcern(course)}
                                        {@const accentColor =
                                            concernAccentColor(concern)}
                                        {@const locTooltip =
                                            formatLocationTooltip(course, concern)}
                                        {@const locDisplay =
                                            formatLocationDisplay(course, concern)}
                                        <td class="py-2 px-2 whitespace-nowrap">
                                            {#if locDisplay}
                                                <span
                                                    class="text-muted-foreground"
                                                    class:pl-2={accentColor !== null}
                                                    style:border-left={accentColor
                                                        ? `2px solid ${accentColor}`
                                                        : undefined}
                                                    data-tooltip={locTooltip}
                                                    data-tooltip-delay="200"
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
                                        {@const open = openSeats(course)}
                                        {@const seatsTip = `${formatNumber(open)} of ${formatNumber(course.maxEnrollment)} seats open, ${formatNumber(course.enrollment)} enrolled${course.waitCount > 0 ? `, ${formatNumber(course.waitCount)} waitlisted` : ""}`}
                                        <td
                                            class="py-2 px-2 text-right whitespace-nowrap"
                                        >
                                            <span
                                                class="inline-flex items-center gap-1.5 select-none"
                                                data-tooltip={seatsTip}
                                                data-tooltip-side="left"
                                                data-tooltip-delay="200"
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
                                                    >{#if open === 0}Full{:else}{open} open{/if}</span
                                                >
                                                <span
                                                    class="text-muted-foreground/60 tabular-nums"
                                                    >{formatNumber(course.enrollment)}/{formatNumber(course.maxEnrollment)}{#if course.waitCount > 0}
                                                        · WL {formatNumber(course.waitCount)}/{formatNumber(course.waitCapacity)}{/if}</span
                                                >
                                            </span>
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
