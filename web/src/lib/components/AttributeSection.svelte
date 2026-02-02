<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import { toggleValue } from "$lib/filters";
import { getAttributeFilterLabel } from "$lib/labels";
import TogglePill from "./TogglePill.svelte";

let {
  title,
  items,
  selected = $bindable<string[]>([]),
}: {
  title: string;
  items: CodeDescription[];
  selected: string[];
} = $props();
</script>

{#if items.length > 0}
  <div class="flex flex-col gap-1.5">
    <span class="text-xs font-medium text-muted-foreground select-none">{title}</span>
    <div class="flex flex-wrap gap-1 whitespace-nowrap">
      {#each items as item (item.filterValue)}
        <TogglePill
          active={selected.includes(item.filterValue)}
          onclick={() => (selected = toggleValue(selected, item.filterValue))}
          label={getAttributeFilterLabel(item.filterValue)}
          title={item.description}
        />
      {/each}
    </div>
  </div>
{/if}
