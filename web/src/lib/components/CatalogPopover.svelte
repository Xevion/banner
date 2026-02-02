<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import { toggleValue } from "$lib/filters";
import { AttributeValues as AV } from "$lib/filterValues";
import { getAttributeFilterLabel } from "$lib/labels";
import FilterPopover from "./FilterPopover.svelte";
import AvailabilityFilter from "./AvailabilityFilter.svelte";

let {
  campus = $bindable<string[]>([]),
  attributes = $bindable<string[]>([]),
  referenceData,
}: {
  campus: string[];
  attributes: string[];
  referenceData: {
    attributes: CodeDescription[];
  };
} = $props();

const hasActiveFilters = $derived(campus.length > 0 || attributes.length > 0);

// Known core curriculum attribute filter values
const CORE_CODES = new Set<string>([
  AV.CoreCommunication,
  AV.CoreMathematics,
  AV.CoreLifePhysicalSciences,
  AV.CoreLanguagePhilosophy,
  AV.CoreCreativeArts,
  AV.CoreAmericanHistory,
  AV.CoreGovernment,
  AV.CoreSocialBehavioral,
  AV.CoreComponentArea,
]);

// Known course level attribute filter values
const LEVEL_CODES = new Set<string>([
  AV.Developmental,
  AV.LowerDivision,
  AV.UpperDivision,
  AV.Graduate,
]);

// Split reference data into groups
const coreAttributes = $derived(
  referenceData.attributes.filter((a) => CORE_CODES.has(a.filterValue))
);
const levelAttributes = $derived(
  referenceData.attributes.filter((a) => LEVEL_CODES.has(a.filterValue))
);
const specialAttributes = $derived(
  referenceData.attributes.filter(
    (a) => !CORE_CODES.has(a.filterValue) && !LEVEL_CODES.has(a.filterValue)
  )
);

function toggleAttr(code: string) {
  attributes = toggleValue(attributes, code);
}
</script>

<FilterPopover label="Catalog" active={hasActiveFilters} width="w-[19rem] max-h-[28rem] overflow-y-auto">
  {#snippet content()}
    <AvailabilityFilter bind:campus />

    <div class="h-px bg-border"></div>

    <!-- Course Level -->
    {#if levelAttributes.length > 0}
      <div class="flex flex-col gap-1.5">
        <span class="text-xs font-medium text-muted-foreground select-none">Course Level</span>
        <div class="flex flex-wrap gap-1 whitespace-nowrap">
          {#each levelAttributes as item (item.filterValue)}
            <button
              type="button"
              aria-pressed={attributes.includes(item.filterValue)}
              class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                     {attributes.includes(item.filterValue)
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
              onclick={() => toggleAttr(item.filterValue)}
              title={item.description}
            >
              {getAttributeFilterLabel(item.filterValue)}
            </button>
          {/each}
        </div>
      </div>

      <div class="h-px bg-border"></div>
    {/if}

    <!-- Core Curriculum -->
    {#if coreAttributes.length > 0}
      <div class="flex flex-col gap-1.5">
        <span class="text-xs font-medium text-muted-foreground select-none">Core Curriculum</span>
        <div class="flex flex-wrap gap-1 whitespace-nowrap">
          {#each coreAttributes as item (item.filterValue)}
            <button
              type="button"
              aria-pressed={attributes.includes(item.filterValue)}
              class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                     {attributes.includes(item.filterValue)
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
              onclick={() => toggleAttr(item.filterValue)}
              title={item.description}
            >
              {getAttributeFilterLabel(item.filterValue)}
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Special Designations -->
    {#if specialAttributes.length > 0}
      <div class="h-px bg-border"></div>

      <div class="flex flex-col gap-1.5">
        <span class="text-xs font-medium text-muted-foreground select-none">Designations</span>
        <div class="flex flex-wrap gap-1 whitespace-nowrap">
          {#each specialAttributes as item (item.filterValue)}
            <button
              type="button"
              aria-pressed={attributes.includes(item.filterValue)}
              class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer select-none
                     {attributes.includes(item.filterValue)
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
              onclick={() => toggleAttr(item.filterValue)}
              title={item.description}
            >
              {getAttributeFilterLabel(item.filterValue)}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  {/snippet}
</FilterPopover>
