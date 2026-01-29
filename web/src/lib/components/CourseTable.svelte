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

let { courses, loading }: { courses: CourseResponse[]; loading: boolean } = $props();

let expandedCrn: string | null = $state(null);

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
</script>

<div class="overflow-x-auto">
  <table class="w-full border-collapse text-sm">
    <thead>
      <tr class="border-b border-border text-left text-muted-foreground">
        <th class="py-2 px-2 font-medium">CRN</th>
        <th class="py-2 px-2 font-medium">Course</th>
        <th class="py-2 px-2 font-medium">Title</th>
        <th class="py-2 px-2 font-medium">Instructor</th>
        <th class="py-2 px-2 font-medium">Time</th>
        <th class="py-2 px-2 font-medium">Location</th>
        <th class="py-2 px-2 font-medium text-right">Seats</th>
      </tr>
    </thead>
    <tbody>
      {#if loading && courses.length === 0}
        {#each Array(5) as _}
          <tr class="border-b border-border">
            <td class="py-2.5 px-2"><div class="h-4 w-10 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-24 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-40 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-20 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-28 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-16 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-14 bg-muted rounded animate-pulse ml-auto"></div></td>
          </tr>
        {/each}
      {:else if courses.length === 0}
        <tr>
          <td colspan="7" class="py-12 text-center text-muted-foreground">
            No courses found. Try adjusting your filters.
          </td>
        </tr>
      {:else}
        {#each courses as course (course.crn)}
          <tr
            class="border-b border-border cursor-pointer hover:bg-muted/50 transition-colors whitespace-nowrap {expandedCrn === course.crn ? 'bg-muted/30' : ''}"
            onclick={() => toggleRow(course.crn)}
          >
            <td class="py-2 px-2 font-mono text-xs text-muted-foreground/70">{course.crn}</td>
            <td class="py-2 px-2 whitespace-nowrap">
              <span class="font-semibold">{course.subject} {course.courseNumber}</span>{#if course.sequenceNumber}<span class="text-muted-foreground">-{course.sequenceNumber}</span>{/if}
            </td>
            <td class="py-2 px-2 font-medium">{course.title}</td>
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
            <td class="py-2 px-2 whitespace-nowrap">
              {#if formatLocation(course)}
                <span class="text-muted-foreground">{formatLocation(course)}</span>
              {:else}
                <span class="text-xs text-muted-foreground/50">—</span>
              {/if}
            </td>
            <td class="py-2 px-2 text-right whitespace-nowrap">
              <span class="inline-flex items-center gap-1.5">
                <span class="size-1.5 rounded-full {seatsDotColor(course)} shrink-0"></span>
                <span class="{seatsColor(course)} font-medium tabular-nums">{#if openSeats(course) === 0}Full{:else}{openSeats(course)} open{/if}</span>
                <span class="text-muted-foreground/60 tabular-nums">{course.enrollment}/{course.maxEnrollment}{#if course.waitCount > 0} · WL {course.waitCount}/{course.waitCapacity}{/if}</span>
              </span>
            </td>
          </tr>
          {#if expandedCrn === course.crn}
            <tr>
              <td colspan="7" class="p-0">
                <CourseDetail {course} />
              </td>
            </tr>
          {/if}
        {/each}
      {/if}
    </tbody>
  </table>
</div>
