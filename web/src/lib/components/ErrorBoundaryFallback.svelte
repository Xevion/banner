<script lang="ts">
import { page } from "$app/state";
import { TriangleAlert, RotateCcw } from "@lucide/svelte";

interface Props {
  /** Heading shown in the error card */
  title?: string;
  /** The error value from svelte:boundary */
  error: unknown;
  /** Reset callback from svelte:boundary */
  reset: () => void;
}

let { title = "Something went wrong", error, reset }: Props = $props();

let errorName = $derived(error instanceof Error ? error.constructor.name : "Error");
let errorMessage = $derived(error instanceof Error ? error.message : String(error));
let errorStack = $derived(error instanceof Error ? error.stack : null);
</script>

<div class="flex items-center justify-center py-16 px-4">
  <div class="w-full max-w-lg rounded-lg border border-status-red/25 bg-status-red/5 overflow-hidden text-sm">
    <div class="px-4 py-2.5 border-b border-status-red/15 flex items-center justify-between gap-4">
      <div class="flex items-center gap-2 text-status-red">
        <TriangleAlert size={16} strokeWidth={2.25} />
        <span class="font-semibold">{title}</span>
      </div>
      <span class="text-xs text-muted-foreground font-mono">{page.url.pathname}</span>
    </div>

    <div class="px-4 py-3 border-b border-status-red/15">
      <span class="text-xs text-muted-foreground/70 font-mono">{errorName}</span>
      <pre class="mt-1 text-xs text-foreground/80 overflow-auto whitespace-pre-wrap break-words">{errorMessage}</pre>
    </div>

    {#if errorStack}
      <details class="border-b border-status-red/15">
        <summary class="px-4 py-2 text-xs text-muted-foreground/70 cursor-pointer hover:text-muted-foreground select-none">
          Stack trace
        </summary>
        <pre class="px-4 py-3 text-xs text-muted-foreground/60 overflow-auto whitespace-pre-wrap break-words max-h-48">{errorStack}</pre>
      </details>
    {/if}

    <div class="px-4 py-2.5 flex items-center justify-end gap-3">
      <span class="text-xs text-muted-foreground/60">Retries this section, not the full page</span>
      <button
        class="shrink-0 cursor-pointer inline-flex items-center gap-1.5 rounded-md bg-status-red px-3 py-1.5 text-sm font-medium text-white hover:brightness-110 transition-all"
        onclick={reset}
      >
        <RotateCcw size={14} strokeWidth={2.25} />
        Try again
      </button>
    </div>
  </div>
</div>
