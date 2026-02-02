<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import type { Snippet } from "svelte";
import { fly } from "svelte/transition";

let {
  label,
  active = false,
  width = "min-w-72",
  content,
}: {
  label: string;
  active?: boolean;
  width?: string;
  content: Snippet;
} = $props();

let open = $state(false);

// Show chevron when popover is open OR when filter is not active
let showChevron = $derived(open || !active);
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    aria-label="{label} filters"
    class="inline-flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-xs font-medium transition-colors cursor-pointer select-none
           {active
      ? 'border-primary/50 bg-primary/10 text-primary hover:bg-primary/20'
      : 'border-border bg-background text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
  >
    {label}
    <div class="relative size-3 flex items-center justify-center">
      <ChevronDown
        class="size-3 absolute transition-all duration-150 {showChevron
          ? 'opacity-100 scale-100'
          : 'opacity-0 scale-75'} {open ? 'rotate-180' : ''}"
      />
      <span
        class="size-1.5 rounded-full bg-primary absolute transition-all duration-150 {showChevron
          ? 'opacity-0 scale-75'
          : 'opacity-100 scale-100'}"
      ></span>
    </div>
  </Popover.Trigger>
  <Popover.Content
    class="z-50 rounded-md border border-border bg-card p-3 text-card-foreground shadow-lg {width}"
    sideOffset={4}
    forceMount
  >
    {#snippet child({ wrapperProps, props, open })}
      {#if open}
        <div {...wrapperProps}>
          <div {...props} transition:fly={{ duration: 150, y: -4 }}>
            <div class="flex flex-col gap-3">
              {@render content()}
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </Popover.Content>
</Popover.Root>
