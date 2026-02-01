<script lang="ts">
let {
  segments,
  onRemoveSegment,
}: {
  segments: string[];
  onRemoveSegment: (segment: string) => void;
} = $props();
</script>

{#if segments.length > 0}
  <span
    class="inline-flex items-center rounded-full border border-border bg-muted/40 text-xs text-muted-foreground"
  >
    {#each segments as segment, i}
      {#if i > 0}
        <span class="w-px self-stretch bg-border"></span>
      {/if}
      <button
        type="button"
        class="px-2 py-0.5 hover:bg-muted/60 transition-colors cursor-pointer select-none first:rounded-l-full last:rounded-r-full"
        onclick={() => onRemoveSegment(segment)}
        aria-label="Remove {segment} filter"
      >
        {segment}
      </button>
    {/each}
  </span>
{/if}
