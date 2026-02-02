<script lang="ts">
import { InstructionalMethodValues as IM } from "$lib/filterValues";
import CompoundFilterButton from "./CompoundFilterButton.svelte";
import FilterPopover from "./FilterPopover.svelte";

let {
  instructionalMethod = $bindable<string[]>([]),
}: {
  instructionalMethod: string[];
} = $props();

const hasActiveFilters = $derived(instructionalMethod.length > 0);

const buttons = [
  { label: "In Person", codes: [IM.InPerson] },
  {
    label: "Online",
    codes: [IM.Online.Async, IM.Online.Sync, IM.Online.Mixed],
    variants: [
      { code: IM.Online.Async, label: "Async" },
      { code: IM.Online.Sync, label: "Sync" },
    ],
  },
  {
    label: "Hybrid",
    codes: [IM.Hybrid.Half, IM.Hybrid.OneThird, IM.Hybrid.TwoThirds],
    variants: [
      { code: IM.Hybrid.Half, label: "Half" },
      { code: IM.Hybrid.OneThird, label: "One Third" },
      { code: IM.Hybrid.TwoThirds, label: "Two Thirds" },
    ],
  },
  { label: "Independent", codes: [IM.Independent] },
];
</script>

<FilterPopover label="Format" active={hasActiveFilters} width="min-w-72">
  {#snippet content()}
    <div class="flex flex-col gap-1.5">
      {#each buttons as btn (btn.label)}
        <CompoundFilterButton
          label={btn.label}
          codes={btn.codes}
          variants={btn.variants}
          bind:selected={instructionalMethod}
        />
      {/each}
    </div>
  {/snippet}
</FilterPopover>
