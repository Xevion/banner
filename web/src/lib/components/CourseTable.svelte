<script lang="ts">
  import type { CourseResponse } from "$lib/api";
  import { abbreviateInstructor, formatMeetingTime, getPrimaryInstructor } from "$lib/course";
  import CourseDetail from "./CourseDetail.svelte";

  let { courses, loading }: { courses: CourseResponse[]; loading: boolean } = $props();

  let expandedCrn: string | null = $state(null);

  function toggleRow(crn: string) {
    expandedCrn = expandedCrn === crn ? null : crn;
  }

  function seatsColor(course: CourseResponse): string {
    return course.enrollment < course.maxEnrollment ? "text-status-green" : "text-status-red";
  }

  function primaryInstructorDisplay(course: CourseResponse): string {
    const primary = getPrimaryInstructor(course.instructors);
    if (!primary) return "Staff";
    return abbreviateInstructor(primary.displayName);
  }

  function timeDisplay(course: CourseResponse): string {
    if (course.meetingTimes.length === 0) return "TBA";
    return formatMeetingTime(course.meetingTimes[0]);
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
        <th class="py-2 px-2 font-medium text-right">Seats</th>
      </tr>
    </thead>
    <tbody>
      {#if loading && courses.length === 0}
        {#each Array(5) as _}
          <tr class="border-b border-border">
            <td class="py-2.5 px-2"><div class="h-4 w-12 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-24 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-40 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-20 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-28 bg-muted rounded animate-pulse"></div></td>
            <td class="py-2.5 px-2"><div class="h-4 w-12 bg-muted rounded animate-pulse ml-auto"></div></td>
          </tr>
        {/each}
      {:else if courses.length === 0}
        <tr>
          <td colspan="6" class="py-12 text-center text-muted-foreground">
            No courses found. Try adjusting your filters.
          </td>
        </tr>
      {:else}
        {#each courses as course (course.crn)}
          <tr
            class="border-b border-border cursor-pointer hover:bg-muted/50 transition-colors {expandedCrn === course.crn ? 'bg-muted/30' : ''}"
            onclick={() => toggleRow(course.crn)}
          >
            <td class="py-2 px-2 font-mono">{course.crn}</td>
            <td class="py-2 px-2 whitespace-nowrap">
              {course.subject} {course.courseNumber}-{course.sequenceNumber ?? ""}
            </td>
            <td class="py-2 px-2">{course.title}</td>
            <td class="py-2 px-2 whitespace-nowrap">{primaryInstructorDisplay(course)}</td>
            <td class="py-2 px-2 whitespace-nowrap">{timeDisplay(course)}</td>
            <td class="py-2 px-2 text-right whitespace-nowrap {seatsColor(course)}">
              {course.enrollment}/{course.maxEnrollment}
              {#if course.waitCount > 0}
                <div class="text-xs text-muted-foreground">WL: {course.waitCount}/{course.waitCapacity}</div>
              {/if}
            </td>
          </tr>
          {#if expandedCrn === course.crn}
            <tr>
              <td colspan="6" class="p-0">
                <CourseDetail {course} />
              </td>
            </tr>
          {/if}
        {/each}
      {/if}
    </tbody>
  </table>
</div>
