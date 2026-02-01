<script lang="ts">
import type { CourseResponse } from "$lib/api";
import { openSeats, seatsColor, seatsDotColor } from "$lib/course";
import { formatNumber } from "$lib/utils";

let { course }: { course: CourseResponse } = $props();

let open = $derived(openSeats(course));
let seatsTip = $derived(
  `${formatNumber(open)} of ${formatNumber(course.maxEnrollment)} seats open, ${formatNumber(course.enrollment)} enrolled${course.waitCount > 0 ? `, ${formatNumber(course.waitCount)} waitlisted` : ""}`
);
</script>

<td class="py-2 px-2 text-right whitespace-nowrap">
  <span
    class="inline-flex items-center gap-1.5 select-none"
    data-tooltip={seatsTip}
    data-tooltip-side="left"
    data-tooltip-delay="200"
  >
    <span class="size-1.5 rounded-full {seatsDotColor(course)} shrink-0"></span>
    <span class="{seatsColor(course)} font-medium tabular-nums"
      >{#if open === 0}Full{:else}{open} open{/if}</span
    >
    <span class="text-muted-foreground/60 tabular-nums"
      >{formatNumber(course.enrollment)}/{formatNumber(course.maxEnrollment)}{#if course.waitCount > 0}
        · WL {formatNumber(course.waitCount)}/{formatNumber(course.waitCapacity)}{/if}</span
    >
  </span>
</td>
