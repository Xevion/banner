<script lang="ts">
import type { CourseResponse } from "$lib/api";
import {
  abbreviateInstructor,
  formatMeetingTimeSummary,
  getPrimaryInstructor,
  openSeats,
  seatsColor,
  seatsDotColor,
} from "$lib/course";
import { formatNumber } from "$lib/utils";
import { slide } from "svelte/transition";
import CourseDetail from "./CourseDetail.svelte";

let {
  course,
  expanded,
  onToggle,
}: {
  course: CourseResponse;
  expanded: boolean;
  onToggle: () => void;
} = $props();
</script>

<div
  class="rounded-lg border border-border bg-card overflow-hidden transition-colors
    {expanded ? 'border-border/80' : 'hover:bg-muted/30'}"
>
  <button
    class="w-full text-left p-3 cursor-pointer"
    aria-expanded={expanded}
    onclick={onToggle}
  >
    <!-- Line 1: Course code + title + seats -->
    <div class="flex items-baseline justify-between gap-2">
      <div class="flex items-baseline gap-1.5 min-w-0">
        <span class="font-mono font-semibold text-sm tracking-tight shrink-0">
          {course.subject} {course.courseNumber}
        </span>
        <span class="text-sm text-muted-foreground truncate">{course.title}</span>
      </div>
      <span class="inline-flex items-center gap-1 shrink-0 text-xs select-none">
        <span class="size-1.5 rounded-full {seatsDotColor(course)} shrink-0"></span>
        <span class="{seatsColor(course)} font-medium tabular-nums">
          {#if openSeats(course) === 0}Full{:else}{openSeats(course)}/{formatNumber(course.maxEnrollment)}{/if}
        </span>
      </span>
    </div>

    <!-- Line 2: Instructor + time -->
    <div class="flex items-center justify-between gap-2 mt-1">
      <span class="text-xs text-muted-foreground truncate">
        {abbreviateInstructor(getPrimaryInstructor(course.instructors)?.displayName ?? "Staff")}
      </span>
      <span class="text-xs text-muted-foreground shrink-0">
        {formatMeetingTimeSummary(course)}
      </span>
    </div>
  </button>

  {#if expanded}
    <div transition:slide={{ duration: 200 }}>
      <CourseDetail {course} />
    </div>
  {/if}
</div>
