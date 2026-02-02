<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import { toggleValue } from "$lib/filters";
import { getPartOfTermFilterLabel } from "$lib/labels";
import TogglePill from "./TogglePill.svelte";

let {
  partOfTerm = $bindable<string[]>([]),
  partsOfTerm,
  mobile = false,
}: {
  partOfTerm: string[];
  partsOfTerm: CodeDescription[];
  mobile?: boolean;
} = $props();
</script>

{#if partsOfTerm.length > 0}
  <div class="flex flex-col gap-1.5">
    <span class="text-xs font-medium text-muted-foreground select-none">Part of Term</span>
    <div class="flex {mobile ? 'flex-wrap' : ''} gap-1 whitespace-nowrap">
      {#each partsOfTerm as item (item.filterValue)}
        <TogglePill
          active={partOfTerm.includes(item.filterValue)}
          onclick={() => (partOfTerm = toggleValue(partOfTerm, item.filterValue))}
          label={getPartOfTermFilterLabel(item.filterValue)}
          title={item.description}
          variant={mobile ? "rounded-full" : "rounded-md"}
          flex={!mobile}
        />
      {/each}
    </div>
  </div>
{/if}
