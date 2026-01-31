<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import { fly } from "svelte/transition";

let {
  openOnly = $bindable(false),
  waitCountMax = $bindable<number | null>(null),
}: {
  openOnly: boolean;
  waitCountMax: number | null;
} = $props();

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
        <div {...wrapperProps} style:view-transition-name="filter-overlay">
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

              <div class="flex flex-col gap-1.5">
                <label for="wait-count-max" class="text-xs font-medium text-muted-foreground">
                  Max waitlist
                </label>
                <input
                  id="wait-count-max"
                  type="number"
                  min="0"
                  placeholder="Any"
                  value={waitCountMax ?? ""}
                  oninput={(e) => {
                    const v = e.currentTarget.value;
                    if (v === "") { waitCountMax = null; return; }
                    const n = Number(v);
                    waitCountMax = Number.isNaN(n) ? null : n;
                  }}
                  class="h-8 w-20 border border-border bg-card text-foreground rounded-md px-2 text-sm
                         focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                />
              </div>
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </Popover.Content>
</Popover.Root>
