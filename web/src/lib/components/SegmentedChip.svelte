<script lang="ts">
import { X } from "@lucide/svelte";

let {
  segments,
  onRemoveSegment,
  onRemoveAll,
}: {
  segments: string[];
  onRemoveSegment: (segment: string) => void;
  onRemoveAll: () => void;
} = $props();
</script>

{#if segments.length > 0}
  <span
    class="inline-flex items-center rounded-full border border-border bg-muted/40 text-xs text-foreground"
  >
    {#each segments as segment, i}
      {#if i > 0}
        <span class="w-px self-stretch bg-border"></span>
      {/if}
      <button
        type="button"
        class="px-2 py-0.5 hover:bg-muted/60 transition-colors cursor-pointer first:rounded-l-full"
        onclick={() => onRemoveSegment(segment)}
        aria-label="Remove {segment} filter"
      >
        {segment}
      </button>
    {/each}
    <span class="w-px self-stretch bg-border"></span>
    <button
      type="button"
      class="inline-flex items-center justify-center px-1.5 py-0.5 rounded-r-full hover:bg-muted/60 transition-colors cursor-pointer"
      onclick={onRemoveAll}
      aria-label="Remove all subject filters"
    >
      <X class="size-3" />
    </button>
  </span>
{/if}
