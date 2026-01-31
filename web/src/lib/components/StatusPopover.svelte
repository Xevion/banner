<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import { fly } from "svelte/transition";
import RangeSlider from "./RangeSlider.svelte";

let {
  openOnly = $bindable(false),
  waitCountMax = $bindable<number | null>(null),
  waitCountMaxRange = 0,
}: {
  openOnly: boolean;
  waitCountMax: number | null;
  waitCountMaxRange: number;
} = $props();

let _dummyLow = $state<number | null>(null);

const hasActiveFilters = $derived(openOnly || waitCountMax !== null);
</script>

<Popover.Root>
  <Popover.Trigger
    class="inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium transition-colors cursor-pointer
           {hasActiveFilters
      ? 'border-primary/50 bg-primary/10 text-primary hover:bg-primary/20'
      : 'border-border bg-background text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
  >
    {#if hasActiveFilters}
      <span class="size-1.5 rounded-full bg-primary"></span>
    {/if}
    Status
    <ChevronDown class="size-3" />
  </Popover.Trigger>
  <Popover.Content
    class="z-50 rounded-md border border-border bg-card p-3 text-card-foreground shadow-lg w-64"
    sideOffset={4}
    forceMount
  >
    {#snippet child({ wrapperProps, props, open })}
      {#if open}
        <div {...wrapperProps}>
          <div {...props} transition:fly={{ duration: 150, y: -4 }}>
            <div class="flex flex-col gap-3">
              <div class="flex flex-col gap-1.5">
                <span class="text-xs font-medium text-muted-foreground">Availability</span>
                <button
                  type="button"
                  class="inline-flex items-center justify-center rounded-full px-3 py-1 text-xs font-medium transition-colors cursor-pointer
                         {openOnly
                    ? 'bg-primary text-primary-foreground'
                    : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                  onclick={() => (openOnly = !openOnly)}
                >
                  Open only
                </button>
              </div>

              <div class="h-px bg-border"></div>

              {#if waitCountMaxRange > 0}
                <RangeSlider
                  min={0}
                  max={waitCountMaxRange}
                  step={10}
                  bind:valueLow={_dummyLow}
                  bind:valueHigh={waitCountMax}
                  label="Max waitlist"
                  dual={false}
                  formatValue={(v) => v === 0 ? "Off" : String(v)}
                />
              {:else}
                <div class="flex flex-col gap-1.5">
                  <span class="text-xs font-medium text-muted-foreground">Max waitlist</span>
                  <span class="text-xs text-muted-foreground">No waitlisted courses</span>
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </Popover.Content>
</Popover.Root>
