<script lang="ts">
import type { CourseResponse } from "$lib/api";
import { useClipboard } from "$lib/composables/useClipboard.svelte";
import {
  formatCreditHours,
  formatDate,
  formatMeetingDaysLong,
  formatTime,
  ratingStyle,
  rmpUrl,
} from "$lib/course";
import { themeStore } from "$lib/stores/theme.svelte";
import { formatNumber } from "$lib/utils";
import {
  Calendar,
  Check,
  Copy,
  Download,
  ExternalLink,
  Info,
  Star,
  Triangle,
} from "@lucide/svelte";
import RichTooltip from "./RichTooltip.svelte";
import SimpleTooltip from "./SimpleTooltip.svelte";

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
                        <RichTooltip delay={200} contentClass="px-3 py-2">
                            {#snippet children()}
                                <span
                                    class="inline-flex items-center gap-1.5 text-sm font-medium bg-card border border-border rounded-md px-2.5 py-1 text-foreground hover:border-foreground/20 hover:bg-card/80 transition-colors"
                                >
                                     {instructor.displayName}
                                     {#if instructor.rmp != null}
                                         {@const rating = instructor.rmp.avgRating}
                                         {@const lowConfidence = !instructor.rmp.isConfident}
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
                            {/snippet}
                            {#snippet content()}
                                <div class="flex flex-col gap-y-1.5">
                                    <div class="font-medium">
                                        {instructor.displayName}
                                    </div>
                                     {#if instructor.isPrimary}
                                         <div class="text-muted-foreground">
                                             Primary instructor
                                         </div>
                                     {/if}
                                     {#if instructor.rmp != null}
                                         <div class="text-muted-foreground">
                                             {instructor.rmp.avgRating.toFixed(1)}/5
                                             · {instructor.rmp.numRatings} ratings
                                             {#if !instructor.rmp.isConfident}
                                                 (low)
                                             {/if}
                                         </div>
                                     {/if}
                                     {#if instructor.rmp?.legacyId != null}
                                         <a
                                             href={rmpUrl(
                                                 instructor.rmp.legacyId,
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
                            {/snippet}
                        </RichTooltip>
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
                 <ul class="flex flex-col gap-y-2">
                     {#each course.meetingTimes as mt}
                         <li>
                             {#if mt.days.length === 0 && mt.timeRange === null}
                                 <span class="italic text-muted-foreground"
                                     >TBA</span
                                 >
                             {:else}
                                 <div class="flex items-baseline gap-1.5">
                                     {#if mt.days.length > 0}
                                         <span
                                             class="font-medium text-foreground"
                                         >
                                             {formatMeetingDaysLong(mt)}
                                         </span>
                                     {/if}
                                     {#if mt.timeRange !== null}
                                         <span class="text-muted-foreground">
                                             {formatTime(
                                                 mt.timeRange.start,
                                             )}&ndash;{formatTime(mt.timeRange.end)}
                                         </span>
                                     {:else}
                                         <span
                                             class="italic text-muted-foreground"
                                             >Time TBA</span
                                         >
                                     {/if}
                                 </div>
                             {/if}
                             {#if mt.location?.building || mt.location?.room}
                                 <div
                                     class="text-xs text-muted-foreground mt-0.5"
                                 >
                                     {mt.location.buildingDescription ??
                                         mt.location.building}{mt.location.room
                                         ? ` ${mt.location.room}`
                                         : ""}
                                 </div>
                             {/if}
                             <div
                                 class="text-xs text-muted-foreground/70 mt-0.5"
                             >
                                 {formatDate(mt.dateRange.start)} &ndash; {formatDate(
                                     mt.dateRange.end,
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
            {@const crossList = course.crossList}
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
                <RichTooltip passthrough>
                    {#snippet children()}
                        <span
                            class="inline-flex items-center gap-1.5 text-foreground font-mono"
                        >
                            <span
                                class="bg-card border border-border rounded-md px-2 py-0.5 text-xs font-medium"
                            >
                                {crossList.identifier}
                            </span>
                            {#if crossList.count != null && crossList.capacity != null}
                                <span class="text-muted-foreground text-xs">
                                    {formatNumber(crossList.count)}/{formatNumber(crossList.capacity)}
                                </span>
                            {/if}
                        </span>
                    {/snippet}
                    {#snippet content()}
                        Group <span class="font-mono font-medium"
                            >{crossList.identifier}</span
                        >
                        {#if crossList.count != null && crossList.capacity != null}
                            — {formatNumber(crossList.count)} enrolled across {formatNumber(crossList.capacity)}
                            shared seats
                        {/if}
                    {/snippet}
                </RichTooltip>
            </div>
        {/if}

         <!-- Waitlist -->
         {#if course.enrollment.waitCapacity > 0}
             <div>
                 <h4 class="text-sm text-foreground mb-2">Waitlist</h4>
                 <span class="text-2foreground"
                     >{formatNumber(course.enrollment.waitCount)} / {formatNumber(course.enrollment.waitCapacity)}</span
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
