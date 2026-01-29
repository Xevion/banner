<script lang="ts">
  import type { CourseResponse } from "$lib/api";
  import {
    formatTime,
    formatMeetingDays,
    formatCreditHours,
  } from "$lib/course";

  let { course }: { course: CourseResponse } = $props();
</script>

<div class="bg-muted p-4 text-sm border-b border-border">
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
    <!-- Instructors -->
    <div>
      <h4 class="font-medium text-foreground mb-1">Instructors</h4>
      {#if course.instructors.length > 0}
        <ul class="space-y-0.5">
          {#each course.instructors as instructor}
            <li class="text-muted-foreground">
              {instructor.displayName}
              {#if instructor.isPrimary}
                <span class="text-xs bg-card border border-border rounded px-1 py-0.5 ml-1">primary</span>
              {/if}
              {#if instructor.email}
                <span class="text-xs"> — {instructor.email}</span>
              {/if}
            </li>
          {/each}
        </ul>
      {:else}
        <span class="text-muted-foreground">Staff</span>
      {/if}
    </div>

    <!-- Meeting Times -->
    <div>
      <h4 class="font-medium text-foreground mb-1">Meeting Times</h4>
      {#if course.meetingTimes.length > 0}
        <ul class="space-y-1">
          {#each course.meetingTimes as mt}
            <li class="text-muted-foreground">
              <span class="font-mono">{formatMeetingDays(mt) || "TBA"}</span>
              {formatTime(mt.begin_time)}–{formatTime(mt.end_time)}
              {#if mt.building || mt.room}
                <span class="text-xs">
                  ({mt.building_description ?? mt.building}{mt.room ? ` ${mt.room}` : ""})
                </span>
              {/if}
              <div class="text-xs opacity-70">{mt.start_date} – {mt.end_date}</div>
            </li>
          {/each}
        </ul>
      {:else}
        <span class="text-muted-foreground">TBA</span>
      {/if}
    </div>

    <!-- Delivery -->
    <div>
      <h4 class="font-medium text-foreground mb-1">Delivery</h4>
      <span class="text-muted-foreground">
        {course.instructionalMethod ?? "—"}
        {#if course.campus}
          · {course.campus}
        {/if}
      </span>
    </div>

    <!-- Credits -->
    <div>
      <h4 class="font-medium text-foreground mb-1">Credits</h4>
      <span class="text-muted-foreground">{formatCreditHours(course)}</span>
    </div>

    <!-- Attributes -->
    {#if course.attributes.length > 0}
      <div>
        <h4 class="font-medium text-foreground mb-1">Attributes</h4>
        <div class="flex flex-wrap gap-1">
          {#each course.attributes as attr}
            <span class="text-xs bg-card border border-border rounded px-1.5 py-0.5 text-muted-foreground">
              {attr}
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Cross-list -->
    {#if course.crossList}
      <div>
        <h4 class="font-medium text-foreground mb-1">Cross-list</h4>
        <span class="text-muted-foreground">
          {course.crossList}
          {#if course.crossListCount != null && course.crossListCapacity != null}
            ({course.crossListCount}/{course.crossListCapacity})
          {/if}
        </span>
      </div>
    {/if}

    <!-- Waitlist -->
    {#if course.waitCapacity > 0}
      <div>
        <h4 class="font-medium text-foreground mb-1">Waitlist</h4>
        <span class="text-muted-foreground">{course.waitCount} / {course.waitCapacity}</span>
      </div>
    {/if}
  </div>
</div>
