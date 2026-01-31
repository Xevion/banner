<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import { fly } from "svelte/transition";

let {
  days = $bindable<string[]>([]),
  timeStart = $bindable<string | null>(null),
  timeEnd = $bindable<string | null>(null),
}: {
  days: string[];
  timeStart: string | null;
  timeEnd: string | null;
} = $props();

const DAY_OPTIONS: { label: string; value: string }[] = [
  { label: "M", value: "monday" },
  { label: "T", value: "tuesday" },
  { label: "W", value: "wednesday" },
  { label: "Th", value: "thursday" },
  { label: "F", value: "friday" },
  { label: "Sa", value: "saturday" },
  { label: "Su", value: "sunday" },
];

const hasActiveFilters = $derived(days.length > 0 || timeStart !== null || timeEnd !== null);

function toggleDay(day: string) {
  if (days.includes(day)) {
    days = days.filter((d) => d !== day);
  } else {
    days = [...days, day];
  }
}

/** Convert "10:00 AM" or "14:30" input to 24h string like "1000" or "1430" */
function parseTimeInput(input: string): string | null {
  const trimmed = input.trim();
  if (trimmed === "") return null;

  // Try HH:MM AM/PM format
  const ampmMatch = trimmed.match(/^(\d{1,2}):(\d{2})\s*(AM|PM)$/i);
  if (ampmMatch) {
    let hours = parseInt(ampmMatch[1], 10);
    const minutes = parseInt(ampmMatch[2], 10);
    const period = ampmMatch[3].toUpperCase();
    if (period === "PM" && hours !== 12) hours += 12;
    if (period === "AM" && hours === 12) hours = 0;
    return String(hours).padStart(2, "0") + String(minutes).padStart(2, "0");
  }

  // Try HH:MM 24h format
  const militaryMatch = trimmed.match(/^(\d{1,2}):(\d{2})$/);
  if (militaryMatch) {
    const hours = parseInt(militaryMatch[1], 10);
    const minutes = parseInt(militaryMatch[2], 10);
    return String(hours).padStart(2, "0") + String(minutes).padStart(2, "0");
  }

  return null;
}

/** Convert 24h string like "1000" to "10:00 AM" for display */
function formatTime(time: string | null): string {
  if (time === null || time.length !== 4) return "";
  const hours = parseInt(time.slice(0, 2), 10);
  const minutes = time.slice(2);
  const period = hours >= 12 ? "PM" : "AM";
  const displayHours = hours === 0 ? 12 : hours > 12 ? hours - 12 : hours;
  return `${displayHours}:${minutes} ${period}`;
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
    Schedule
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
                <span class="text-xs font-medium text-muted-foreground">Days of week</span>
                <div class="flex gap-1">
                  {#each DAY_OPTIONS as { label, value } (value)}
                    <button
                      type="button"
                      class="flex items-center justify-center rounded-md px-2 py-1 text-xs font-medium transition-colors cursor-pointer min-w-[2rem]
                             {days.includes(value)
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                      onclick={() => toggleDay(value)}
                      aria-label={value.charAt(0).toUpperCase() + value.slice(1)}
                      aria-pressed={days.includes(value)}
                    >
                      {label}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="h-px bg-border"></div>

              <div class="flex flex-col gap-1.5">
                <span class="text-xs font-medium text-muted-foreground">Time range</span>
                <div class="flex items-center gap-2">
                  <input
                    type="text"
                    placeholder="10:00 AM"
                    value={formatTime(timeStart)}
                    onchange={(e) => {
                      timeStart = parseTimeInput(e.currentTarget.value);
                      e.currentTarget.value = formatTime(timeStart);
                    }}
                    class="h-8 w-24 border border-border bg-card text-foreground rounded-md px-2 text-sm
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                  />
                  <span class="text-xs text-muted-foreground">to</span>
                  <input
                    type="text"
                    placeholder="3:00 PM"
                    value={formatTime(timeEnd)}
                    onchange={(e) => {
                      timeEnd = parseTimeInput(e.currentTarget.value);
                      e.currentTarget.value = formatTime(timeEnd);
                    }}
                    class="h-8 w-24 border border-border bg-card text-foreground rounded-md px-2 text-sm
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
