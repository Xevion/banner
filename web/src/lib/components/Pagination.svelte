<script lang="ts">
  let { totalCount, offset, limit, onPageChange }: {
    totalCount: number;
    offset: number;
    limit: number;
    onPageChange: (newOffset: number) => void;
  } = $props();

  const start = $derived(offset + 1);
  const end = $derived(Math.min(offset + limit, totalCount));
  const hasPrev = $derived(offset > 0);
  const hasNext = $derived(offset + limit < totalCount);
</script>

{#if totalCount > 0}
  <div class="flex items-center justify-between text-sm">
    <span class="text-muted-foreground">
      Showing {start}–{end} of {totalCount} courses
    </span>
    <div class="flex gap-2">
      <button
        disabled={!hasPrev}
        onclick={() => onPageChange(offset - limit)}
        class="border border-border bg-card text-foreground rounded-md px-3 py-1.5 text-sm disabled:opacity-40 disabled:cursor-not-allowed hover:bg-muted/50 transition-colors"
      >
        Previous
      </button>
      <button
        disabled={!hasNext}
        onclick={() => onPageChange(offset + limit)}
        class="border border-border bg-card text-foreground rounded-md px-3 py-1.5 text-sm disabled:opacity-40 disabled:cursor-not-allowed hover:bg-muted/50 transition-colors"
      >
        Next
      </button>
    </div>
  </div>
{/if}
