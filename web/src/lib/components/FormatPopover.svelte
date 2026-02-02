<script lang="ts">
import { FORMAT_BUTTONS } from "$lib/labels";
import { getFiltersContext } from "$lib/stores/search-filters.svelte";
import CompoundFilterButton from "./CompoundFilterButton.svelte";
import FilterPopover from "./FilterPopover.svelte";

const filters = getFiltersContext();
const hasActiveFilters = $derived(filters.instructionalMethod.length > 0);
</script>

<FilterPopover label="Format" active={hasActiveFilters} width="min-w-72">
  {#snippet content()}
    <div class="flex flex-col gap-1.5">
      {#each FORMAT_BUTTONS as btn (btn.label)}
        <CompoundFilterButton
          label={btn.label}
          codes={btn.codes}
          variants={btn.variants}
          bind:selected={filters.instructionalMethod}
        />
      {/each}
    </div>
  {/snippet}
</FilterPopover>
