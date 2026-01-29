<script lang="ts">
import type { CourseResponse } from "$lib/api";
import {
  formatTime,
  formatCreditHours,
  formatDate,
  formatMeetingDaysLong,
  isMeetingTimeTBA,
  isTimeTBA,
  ratingColor,
} from "$lib/course";
import { useClipboard } from "$lib/composables/useClipboard.svelte";
import { cn, tooltipContentClass } from "$lib/utils";
import { Tooltip } from "bits-ui";
import SimpleTooltip from "./SimpleTooltip.svelte";
import { Info, Copy, Check } from "@lucide/svelte";

let { course }: { course: CourseResponse } = $props();

const clipboard = useClipboard();
</script>

<div class="bg-muted/60 p-5 text-sm border-b border-border">
  <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
    <!-- Instructors -->
    <div>
      <h4 class="text-sm text-foreground mb-2">
        Instructors
      </h4>
      {#if course.instructors.length > 0}
        <div class="flex flex-wrap gap-1.5">
          {#each course.instructors as instructor}
            <Tooltip.Root delayDuration={200}>
              <Tooltip.Trigger>
                <span
                  class="inline-flex items-center gap-1.5 text-sm font-medium bg-card border border-border rounded-md px-2.5 py-1 text-foreground hover:border-foreground/20 hover:bg-card/80 transition-colors"
                >
                  {instructor.displayName}
                  {#if instructor.rmpRating != null}
                    {@const rating = instructor.rmpRating}
                    <span
                      class="text-[10px] font-semibold {ratingColor(rating)}"
                    >{rating.toFixed(1)}★</span>
                  {/if}
                </span>
              </Tooltip.Trigger>
              <Tooltip.Content
                sideOffset={6}
                class={cn(tooltipContentClass, "px-3 py-2")}
              >
                <div class="space-y-1.5">
                  <div class="font-medium">{instructor.displayName}</div>
                  {#if instructor.isPrimary}
                    <div class="text-muted-foreground">Primary instructor</div>
                  {/if}
                  {#if instructor.rmpRating != null}
                    <div class="text-muted-foreground">
                      {instructor.rmpRating.toFixed(1)}/5 ({instructor.rmpNumRatings ?? 0} ratings)
                    </div>
                  {/if}
                  {#if instructor.email}
                    <button
                      onclick={(e) => clipboard.copy(instructor.email!, e)}
                      class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                    >
                      {#if clipboard.copiedValue === instructor.email}
                        <Check class="size-3" />
                        <span>Copied!</span>
                      {:else}
                        <Copy class="size-3" />
                        <span>{instructor.email}</span>
                      {/if}
                    </button>
                  {/if}
                </div>
              </Tooltip.Content>
            </Tooltip.Root>
          {/each}
        </div>
      {:else}
        <span class="text-muted-foreground italic">Staff</span>
      {/if}
    </div>

    <!-- Meeting Times -->
    <div>
      <h4 class="text-sm text-foreground mb-2">
        Meeting Times
      </h4>
      {#if course.meetingTimes.length > 0}
        <ul class="space-y-2">
          {#each course.meetingTimes as mt}
            <li>
              {#if isMeetingTimeTBA(mt) && isTimeTBA(mt)}
                <span class="italic text-muted-foreground">TBA</span>
              {:else}
                <div class="flex items-baseline gap-1.5">
                  {#if !isMeetingTimeTBA(mt)}
                    <span class="font-medium text-foreground">
                      {formatMeetingDaysLong(mt)}
                    </span>
                  {/if}
                  {#if !isTimeTBA(mt)}
                    <span class="text-muted-foreground">
                      {formatTime(mt.begin_time)}&ndash;{formatTime(mt.end_time)}
                    </span>
                  {:else}
                    <span class="italic text-muted-foreground">Time TBA</span>
                  {/if}
                </div>
              {/if}
              {#if mt.building || mt.room}
                <div class="text-xs text-muted-foreground mt-0.5">
                  {mt.building_description ?? mt.building}{mt.room ? ` ${mt.room}` : ""}
                </div>
              {/if}
              <div class="text-xs text-muted-foreground/70 mt-0.5">
                {formatDate(mt.start_date)} &ndash; {formatDate(mt.end_date)}
              </div>
            </li>
          {/each}
        </ul>
      {:else}
        <span class="italic text-muted-foreground">TBA</span>
      {/if}
    </div>

    <!-- Delivery -->
    <div>
      <h4 class="text-sm text-foreground mb-2">
        <span class="inline-flex items-center gap-1">
          Delivery
          <SimpleTooltip text="How the course is taught: in-person, online, hybrid, etc." delay={150} passthrough>
            <Info class="size-3 text-muted-foreground/50" />
          </SimpleTooltip>
        </span>
      </h4>
      <span class="text-foreground">
        {course.instructionalMethod ?? "—"}
        {#if course.campus}
          <span class="text-muted-foreground"> · {course.campus}</span>
        {/if}
      </span>
    </div>

    <!-- Credits -->
    <div>
      <h4 class="text-sm text-foreground mb-2">
        Credits
      </h4>
      <span class="text-foreground">{formatCreditHours(course)}</span>
    </div>

    <!-- Attributes -->
    {#if course.attributes.length > 0}
      <div>
        <h4 class="text-sm text-foreground mb-2">
          <span class="inline-flex items-center gap-1">
            Attributes
            <SimpleTooltip text="Course flags for degree requirements, core curriculum, or special designations" delay={150} passthrough>
              <Info class="size-3 text-muted-foreground/50" />
            </SimpleTooltip>
          </span>
        </h4>
        <div class="flex flex-wrap gap-1.5">
          {#each course.attributes as attr}
            <SimpleTooltip text="Course attribute code" delay={150} passthrough>
              <span
                class="inline-flex text-xs font-medium bg-card border border-border rounded-md px-2 py-0.5 text-muted-foreground hover:text-foreground hover:border-foreground/20 transition-colors"
              >
                {attr}
              </span>
            </SimpleTooltip>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Cross-list -->
    {#if course.crossList}
      <div>
        <h4 class="text-sm text-foreground mb-2">
          <span class="inline-flex items-center gap-1">
            Cross-list
            <SimpleTooltip text="Cross-listed sections share enrollment across multiple course numbers. Students in any linked section attend the same class." delay={150} passthrough>
              <Info class="size-3 text-muted-foreground/50" />
            </SimpleTooltip>
          </span>
        </h4>
        <Tooltip.Root delayDuration={150} disableHoverableContent>
          <Tooltip.Trigger>
            <span class="inline-flex items-center gap-1.5 text-foreground font-mono">
              <span class="bg-card border border-border rounded-md px-2 py-0.5 text-xs font-medium">
                {course.crossList}
              </span>
              {#if course.crossListCount != null && course.crossListCapacity != null}
                <span class="text-muted-foreground text-xs">
                  {course.crossListCount}/{course.crossListCapacity}
                </span>
              {/if}
            </span>
          </Tooltip.Trigger>
          <Tooltip.Content
            sideOffset={6}
            class={tooltipContentClass}
          >
            Group <span class="font-mono font-medium">{course.crossList}</span>
            {#if course.crossListCount != null && course.crossListCapacity != null}
              — {course.crossListCount} enrolled across {course.crossListCapacity} shared seats
            {/if}
          </Tooltip.Content>
        </Tooltip.Root>
      </div>
    {/if}

    <!-- Waitlist -->
    {#if course.waitCapacity > 0}
      <div>
        <h4 class="text-sm text-foreground mb-2">
          Waitlist
        </h4>
        <span class="text-foreground">{course.waitCount} / {course.waitCapacity}</span>
      </div>
    {/if}
  </div>
</div>
