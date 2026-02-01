<script lang="ts">
let {
  label,
  onRemove,
}: {
  label: string;
  onRemove: () => void;
} = $props();
</script>

<button
  type="button"
  class="inline-flex items-center rounded-full border border-border bg-muted/40 px-2.5 py-0.5 text-xs text-muted-foreground hover:bg-muted/60 transition-colors cursor-pointer"
  onclick={onRemove}
  aria-label="Remove {label} filter"
>
  {label}
</button>
