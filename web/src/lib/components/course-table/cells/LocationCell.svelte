<script lang="ts">
import type { CourseResponse } from "$lib/api";
import { concernAccentColor, formatLocationTooltip } from "$lib/course";

let { course }: { course: CourseResponse } = $props();

let accentColor = $derived(concernAccentColor(course.deliveryMode));
let locTooltip = $derived(formatLocationTooltip(course));
let locDisplay = $derived(course.primaryLocation);
</script>

<td class="py-2 px-2 whitespace-nowrap">
  {#if locDisplay}
    <span
      class="text-muted-foreground"
      class:pl-2={accentColor !== null}
      style:border-left={accentColor ? `2px solid ${accentColor}` : undefined}
      data-tooltip={locTooltip}
      data-tooltip-delay="200"
    >
      {locDisplay}
    </span>
  {:else}
    <span class="text-xs text-muted-foreground/50">—</span>
  {/if}
</td>
