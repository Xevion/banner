<script lang="ts">
import { X } from "@lucide/svelte";

let {
  label,
  onRemove,
  onclick,
}: {
  label: string;
  onRemove: () => void;
  onclick?: () => void;
} = $props();
</script>

<span
  class="inline-flex items-center gap-1 rounded-full border border-border bg-muted/40 px-2.5 py-0.5 text-xs text-foreground"
>
  <button
    type="button"
    class="hover:text-foreground/80 transition-colors cursor-pointer"
    onclick={onclick}
  >
    {label}
  </button>
  <button
    type="button"
    class="inline-flex items-center justify-center rounded-full hover:bg-muted transition-colors cursor-pointer"
    onclick={(e) => {
      e.stopPropagation();
      onRemove();
    }}
    aria-label="Remove {label} filter"
  >
    <X class="size-3" />
  </button>
</span>
