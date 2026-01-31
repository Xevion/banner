<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import { fly } from "svelte/transition";

let {
  creditHourMin = $bindable<number | null>(null),
  creditHourMax = $bindable<number | null>(null),
  instructor = $bindable(""),
  courseNumberLow = $bindable<number | null>(null),
  courseNumberHigh = $bindable<number | null>(null),
}: {
  creditHourMin: number | null;
  creditHourMax: number | null;
  instructor: string;
  courseNumberLow: number | null;
  courseNumberHigh: number | null;
} = $props();

const hasActiveFilters = $derived(
  creditHourMin !== null ||
    creditHourMax !== null ||
    instructor !== "" ||
    courseNumberLow !== null ||
    courseNumberHigh !== null
);

function parseNumberInput(value: string): number | null {
  if (value === "") return null;
  const n = Number(value);
  return Number.isNaN(n) ? null : n;
}
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
    More
    <ChevronDown class="size-3" />
  </Popover.Trigger>
  <Popover.Content
    class="z-50 rounded-md border border-border bg-card p-3 text-card-foreground shadow-lg w-72"
    sideOffset={4}
    forceMount
  >
    {#snippet child({ wrapperProps, props, open })}
      {#if open}
        <div {...wrapperProps} style:view-transition-name="filter-overlay">
          <div {...props} transition:fly={{ duration: 150, y: -4 }}>
            <div class="flex flex-col gap-3">
              <div class="flex flex-col gap-1.5">
                <span class="text-xs font-medium text-muted-foreground">Credit hours</span>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    min="0"
                    placeholder="Min"
                    value={creditHourMin ?? ""}
                    oninput={(e) => (creditHourMin = parseNumberInput(e.currentTarget.value))}
                    class="h-8 w-20 border border-border bg-card text-foreground rounded-md px-2 text-sm
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                  />
                  <span class="text-xs text-muted-foreground">to</span>
                  <input
                    type="number"
                    min="0"
                    placeholder="Max"
                    value={creditHourMax ?? ""}
                    oninput={(e) => (creditHourMax = parseNumberInput(e.currentTarget.value))}
                    class="h-8 w-20 border border-border bg-card text-foreground rounded-md px-2 text-sm
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                  />
                </div>
              </div>

              <div class="h-px bg-border"></div>

              <div class="flex flex-col gap-1.5">
                <label for="instructor-input" class="text-xs font-medium text-muted-foreground">
                  Instructor
                </label>
                <input
                  id="instructor-input"
                  type="text"
                  placeholder="Search by name..."
                  bind:value={instructor}
                  class="h-8 border border-border bg-card text-foreground rounded-md px-2 text-sm
                         focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                />
              </div>

              <div class="h-px bg-border"></div>

              <div class="flex flex-col gap-1.5">
                <span class="text-xs font-medium text-muted-foreground">Course number</span>
                <div class="flex items-center gap-2">
                  <input
                    type="number"
                    min="0"
                    placeholder="Min"
                    value={courseNumberLow ?? ""}
                    oninput={(e) => (courseNumberLow = parseNumberInput(e.currentTarget.value))}
                    class="h-8 w-20 border border-border bg-card text-foreground rounded-md px-2 text-sm
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                  />
                  <span class="text-xs text-muted-foreground">to</span>
                  <input
                    type="number"
                    min="0"
                    placeholder="Max"
                    value={courseNumberHigh ?? ""}
                    oninput={(e) => (courseNumberHigh = parseNumberInput(e.currentTarget.value))}
                    class="h-8 w-20 border border-border bg-card text-foreground rounded-md px-2 text-sm
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </Popover.Content>
</Popover.Root>
