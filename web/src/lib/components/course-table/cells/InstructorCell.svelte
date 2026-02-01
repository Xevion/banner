<script lang="ts">
import type { CourseResponse } from "$lib/api";
import {
  RMP_CONFIDENCE_THRESHOLD,
  abbreviateInstructor,
  getPrimaryInstructor,
  ratingStyle,
  rmpUrl,
} from "$lib/course";
import { themeStore } from "$lib/stores/theme.svelte";
import { formatNumber } from "$lib/utils";
import { ExternalLink, Star, Triangle } from "@lucide/svelte";
import LazyRichTooltip from "$lib/components/LazyRichTooltip.svelte";

let { course }: { course: CourseResponse } = $props();

let primary = $derived(getPrimaryInstructor(course.instructors));
let display = $derived(primary ? abbreviateInstructor(primary.displayName) : "Staff");
let commaIdx = $derived(display.indexOf(", "));
let ratingData = $derived(
  primary?.rmpRating != null
    ? {
        rating: primary.rmpRating,
        count: primary.rmpNumRatings ?? 0,
        legacyId: primary.rmpLegacyId ?? null,
      }
    : null
);
</script>

<td class="py-2 px-2 whitespace-nowrap">
  {#if display === "Staff"}
    <span class="text-xs text-muted-foreground/60 uppercase select-none">Staff</span>
  {:else}
    <span
      data-tooltip={primary?.displayName ?? "Staff"}
      data-tooltip-side="bottom"
      data-tooltip-delay="200"
    >
      {#if commaIdx !== -1}
        <span
          >{display.slice(0, commaIdx)},
          <span class="text-muted-foreground"
            >{display.slice(commaIdx + 1)}</span
          ></span
        >
      {:else}
        <span>{display}</span>
      {/if}
    </span>
  {/if}
  {#if ratingData}
    {@const lowConfidence = ratingData.count < RMP_CONFIDENCE_THRESHOLD}
    <LazyRichTooltip side="bottom" sideOffset={6} contentClass="px-2.5 py-1.5">
      {#snippet children()}
        <span
          class="ml-1 text-xs font-medium inline-flex items-center gap-0.5 select-none"
          style={ratingStyle(ratingData.rating, themeStore.isDark)}
        >
          {ratingData.rating.toFixed(1)}
          {#if lowConfidence}
            <Triangle class="size-2 fill-current" />
          {:else}
            <Star class="size-2.5 fill-current" />
          {/if}
        </span>
      {/snippet}
      {#snippet content()}
        <span class="inline-flex items-center gap-1.5 text-xs">
          {ratingData.rating.toFixed(1)}/5 · {formatNumber(ratingData.count)}
          ratings
          {#if (ratingData.count ?? 0) < RMP_CONFIDENCE_THRESHOLD}
            (low)
          {/if}
          {#if ratingData.legacyId != null}
            ·
            <a
              href={rmpUrl(ratingData.legacyId)}
              target="_blank"
              rel="noopener"
              class="inline-flex items-center gap-0.5 text-muted-foreground hover:text-foreground transition-colors"
            >
              RMP
              <ExternalLink class="size-3" />
            </a>
          {/if}
        </span>
      {/snippet}
    </LazyRichTooltip>
  {/if}
</td>
