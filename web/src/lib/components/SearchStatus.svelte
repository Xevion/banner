<script lang="ts">
export interface SearchMeta {
  totalCount: number;
  durationMs: number;
  timestamp: Date;
}

let { meta }: { meta: SearchMeta | null } = $props();

let formattedTime = $derived(
  meta
    ? meta.timestamp.toLocaleTimeString(undefined, {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      })
    : ""
);

let countLabel = $derived(meta ? meta.totalCount.toLocaleString() : "");
let resultNoun = $derived(meta ? (meta.totalCount !== 1 ? "results" : "result") : "");
let durationLabel = $derived(meta ? `${Math.round(meta.durationMs)}ms` : "");
</script>

{#if meta}
  <p
    class="pl-1 text-xs"
    title="Last searched at {formattedTime}"
  >
    <span class="text-muted-foreground/70">{countLabel}</span>
    <span class="text-muted-foreground/35">{resultNoun} in</span>
    <span class="text-muted-foreground/70">{durationLabel}</span>
  </p>
{/if}
