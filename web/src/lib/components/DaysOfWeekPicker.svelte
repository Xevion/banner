<script lang="ts">
import { DAY_OPTIONS, toggleDay as _toggleDay } from "$lib/filters";
import TogglePill from "./TogglePill.svelte";

let {
  days = $bindable<string[]>([]),
  mobile = false,
}: {
  days: string[];
  mobile?: boolean;
} = $props();

function toggleDay(day: string) {
  days = _toggleDay(days, day);
}
</script>

<div class="flex flex-col gap-1.5">
  <span class="text-xs font-medium text-muted-foreground select-none">Days of week</span>
  <div class="flex gap-1 {mobile ? '' : 'whitespace-nowrap'}">
    {#each DAY_OPTIONS as { label, value } (value)}
      <TogglePill
        active={days.includes(value)}
        onclick={() => toggleDay(value)}
        {label}
        ariaLabel={value.charAt(0).toUpperCase() + value.slice(1)}
        variant="rounded-md"
        flex={!mobile}
      />
    {/each}
  </div>
</div>
