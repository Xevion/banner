<script lang="ts">
import type { CourseResponse } from "$lib/api";
import {
  formatTime,
  formatCreditHours,
  formatDate,
  formatMeetingDaysLong,
  isMeetingTimeTBA,
  isTimeTBA,
  ratingStyle,
  rmpUrl,
  RMP_CONFIDENCE_THRESHOLD,
} from "$lib/course";
import { themeStore } from "$lib/stores/theme.svelte";
import { useClipboard } from "$lib/composables/useClipboard.svelte";
import { cn, tooltipContentClass, formatNumber } from "$lib/utils";
import { Tooltip } from "bits-ui";
import SimpleTooltip from "./SimpleTooltip.svelte";
import {
  Info,
  Copy,
  Check,
  Star,
  Triangle,
  ExternalLink,
  Calendar,
  Download,
} from "@lucide/svelte";

let { course }: { course: CourseResponse } = $props();

const clipboard = useClipboard();
</script>

<div class="bg-muted/60 p-5 text-sm border-b border-border">
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
        <!-- Instructors -->
        <div>
            <h4 class="text-sm text-foreground mb-2">Instructors</h4>
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
                                        {@const lowConfidence =
                                            (instructor.rmpNumRatings ?? 0) <
                                            RMP_CONFIDENCE_THRESHOLD}
                                        <span
                                            class="text-[10px] font-semibold inline-flex items-center gap-0.5"
                                            style={ratingStyle(
                                                rating,
                                                themeStore.isDark,
                                            )}
                                        >
                                            {rating.toFixed(1)}
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
                                    {/if}
                                </span>
                            </Tooltip.Trigger>
                            <Tooltip.Content
                                sideOffset={6}
                                class={cn(tooltipContentClass, "px-3 py-2")}
                            >
                                <div class="space-y-1.5">
                                    <div class="font-medium">
                                        {instructor.displayName}
                                    </div>
                                    {#if instructor.isPrimary}
                                        <div class="text-muted-foreground">
                                            Primary instructor
                                        </div>
                                    {/if}
                                    {#if instructor.rmpRating != null}
                                        <div class="text-muted-foreground">
                                            {instructor.rmpRating.toFixed(1)}/5
                                            · {instructor.rmpNumRatings ?? 0} ratings
                                            {#if (instructor.rmpNumRatings ?? 0) < RMP_CONFIDENCE_THRESHOLD}
                                                (low)
                                            {/if}
                                        </div>
                                    {/if}
                                    {#if instructor.rmpLegacyId != null}
                                        <a
                                            href={rmpUrl(
                                                instructor.rmpLegacyId,
                                            )}
                                            target="_blank"
                                            rel="noopener"
                                            class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
                                        >
                                            <ExternalLink class="size-3" />
                                            <span>View on RMP</span>
                                        </a>
                                    {/if}
                                    {#if instructor.email}
                                        <button
                                            onclick={(e) =>
                                                clipboard.copy(
                                                    instructor.email!,
                                                    e,
                                                )}
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
            <h4 class="text-sm text-foreground mb-2">Meeting Times</h4>
            {#if course.meetingTimes.length > 0}
                <ul class="space-y-2">
                    {#each course.meetingTimes as mt}
                        <li>
                            {#if isMeetingTimeTBA(mt) && isTimeTBA(mt)}
                                <span class="italic text-muted-foreground"
                                    >TBA</span
                                >
                            {:else}
                                <div class="flex items-baseline gap-1.5">
                                    {#if !isMeetingTimeTBA(mt)}
                                        <span
                                            class="font-medium text-foreground"
                                        >
                                            {formatMeetingDaysLong(mt)}
                                        </span>
                                    {/if}
                                    {#if !isTimeTBA(mt)}
                                        <span class="text-muted-foreground">
                                            {formatTime(
                                                mt.begin_time,
                                            )}&ndash;{formatTime(mt.end_time)}
                                        </span>
                                    {:else}
                                        <span
                                            class="italic text-muted-foreground"
                                            >Time TBA</span
                                        >
                                    {/if}
                                </div>
                            {/if}
                            {#if mt.building || mt.room}
                                <div
                                    class="text-xs text-muted-foreground mt-0.5"
                                >
                                    {mt.building_description ??
                                        mt.building}{mt.room
                                        ? ` ${mt.room}`
                                        : ""}
                                </div>
                            {/if}
                            <div
                                class="text-xs text-muted-foreground/70 mt-0.5"
                            >
                                {formatDate(mt.start_date)} &ndash; {formatDate(
                                    mt.end_date,
                                )}
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
                    <SimpleTooltip
                        text="How the course is taught: in-person, online, hybrid, etc."
                        delay={150}
                        passthrough
                    >
                        <Info class="size-3 text-muted-foreground/50" />
                    </SimpleTooltip>
                </span>
            </h4>
            <span class="text-foreground">
                {course.instructionalMethod ?? "—"}
                {#if course.campus}
                    <span class="text-muted-foreground">
                        · {course.campus}
                    </span>
                {/if}
            </span>
        </div>

        <!-- Credits -->
        <div>
            <h4 class="text-sm text-foreground mb-2">Credits</h4>
            <span class="text-foreground">{formatCreditHours(course)}</span>
        </div>

        <!-- Attributes -->
        {#if course.attributes.length > 0}
            <div>
                <h4 class="text-sm text-foreground mb-2">
                    <span class="inline-flex items-center gap-1">
                        Attributes
                        <SimpleTooltip
                            text="Course flags for degree requirements, core curriculum, or special designations"
                            delay={150}
                            passthrough
                        >
                            <Info class="size-3 text-muted-foreground/50" />
                        </SimpleTooltip>
                    </span>
                </h4>
                <div class="flex flex-wrap gap-1.5">
                    {#each course.attributes as attr}
                        <SimpleTooltip
                            text="Course attribute code"
                            delay={150}
                            passthrough
                        >
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
                        <SimpleTooltip
                            text="Cross-listed sections share enrollment across multiple course numbers. Students in any linked section attend the same class."
                            delay={150}
                            passthrough
                        >
                            <Info class="size-3 text-muted-foreground/50" />
                        </SimpleTooltip>
                    </span>
                </h4>
                <Tooltip.Root delayDuration={150} disableHoverableContent>
                    <Tooltip.Trigger>
                        <span
                            class="inline-flex items-center gap-1.5 text-foreground font-mono"
                        >
                            <span
                                class="bg-card border border-border rounded-md px-2 py-0.5 text-xs font-medium"
                            >
                                {course.crossList}
                            </span>
                            {#if course.crossListCount != null && course.crossListCapacity != null}
                                <span class="text-muted-foreground text-xs">
                                    {formatNumber(course.crossListCount)}/{formatNumber(course.crossListCapacity)}
                                </span>
                            {/if}
                        </span>
                    </Tooltip.Trigger>
                    <Tooltip.Content sideOffset={6} class={tooltipContentClass}>
                        Group <span class="font-mono font-medium"
                            >{course.crossList}</span
                        >
                        {#if course.crossListCount != null && course.crossListCapacity != null}
                            — {formatNumber(course.crossListCount)} enrolled across {formatNumber(course.crossListCapacity)}
                            shared seats
                        {/if}
                    </Tooltip.Content>
                </Tooltip.Root>
            </div>
        {/if}

        <!-- Waitlist -->
        {#if course.waitCapacity > 0}
            <div>
                <h4 class="text-sm text-foreground mb-2">Waitlist</h4>
                <span class="text-2foreground"
                    >{formatNumber(course.waitCount)} / {formatNumber(course.waitCapacity)}</span
                >
            </div>
        {/if}

        <!-- Calendar Export -->
        {#if course.meetingTimes.length > 0}
            <div>
                <h4 class="text-sm text-foreground mb-2">
                    <span class="inline-flex items-center gap-1">
                        Calendar
                        <SimpleTooltip
                            text="Export this course schedule to your calendar app"
                            delay={150}
                            passthrough
                        >
                            <Info class="size-3 text-muted-foreground/50" />
                        </SimpleTooltip>
                    </span>
                </h4>
                <div class="flex flex-wrap gap-1.5">
                    <a
                        href="/api/courses/{course.termCode}/{course.crn}/calendar.ics"
                        download
                        class="inline-flex items-center gap-1.5 text-sm font-medium bg-card border border-border rounded-md px-2.5 py-1 text-foreground hover:border-foreground/20 hover:bg-card/80 transition-colors"
                    >
                        <Download class="size-3.5" />
                        ICS File
                    </a>
                    <a
                        href="/api/courses/{course.termCode}/{course.crn}/gcal"
                        target="_blank"
                        rel="noopener"
                        class="inline-flex items-center gap-1.5 text-sm font-medium bg-card border border-border rounded-md px-2.5 py-1 text-foreground hover:border-foreground/20 hover:bg-card/80 transition-colors"
                    >
                        <Calendar class="size-3.5" />
                        Google Calendar
                    </a>
                </div>
            </div>
        {/if}
    </div>
</div>
