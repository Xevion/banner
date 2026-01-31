<script lang="ts">
import type { Term } from "$lib/api";
import { Check, ChevronsUpDown } from "@lucide/svelte";
import { Combobox } from "bits-ui";
import { fly } from "svelte/transition";

let {
  terms,
  value = $bindable(),
}: {
  terms: Term[];
  value: string;
} = $props();

let open = $state(false);
let searchValue = $state("");
let containerEl = $state<HTMLDivElement>(null!);

// The first term from the backend is the most current
const currentTermSlug = $derived(terms[0]?.slug ?? "");

const selectedLabel = $derived(
  terms.find((t) => t.slug === value)?.description ?? "Select term..."
);

const filteredTerms = $derived.by(() => {
  const query = searchValue.toLowerCase();
  const matched =
    query === "" ? terms : terms.filter((t) => t.description.toLowerCase().includes(query));

  const current = matched.find((t) => t.slug === currentTermSlug);
  const rest = matched.filter((t) => t.slug !== currentTermSlug);
  return current ? [current, ...rest] : rest;
});

// Manage DOM input text: clear when open for searching, restore label when closed
$effect(() => {
  const _open = open;
  void value; // track selection changes
  const _label = selectedLabel;
  const input = containerEl?.querySelector("input");
  if (!input) return;
  if (_open) {
    input.value = "";
    searchValue = "";
  } else {
    input.value = _label;
  }
});
</script>

<Combobox.Root
  type="single"
  bind:value={() => value, (v) => { if (v) value = v; }}
  bind:open
  onOpenChange={(o: boolean) => {
    if (!o) searchValue = "";
  }}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="relative h-9 rounded-md border border-border bg-card
           flex items-center w-40 cursor-pointer
           has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring has-[:focus-visible]:ring-offset-2 has-[:focus-visible]:ring-offset-background"
    role="presentation"
    bind:this={containerEl}
    onclick={() => { containerEl?.querySelector('input')?.focus(); }}
    onkeydown={() => { containerEl?.querySelector('input')?.focus(); }}
  >
    <Combobox.Input
      oninput={(e) => (searchValue = e.currentTarget.value)}
      onfocus={() => { open = true; }}
      class="h-full w-full bg-transparent text-muted-foreground text-sm
             placeholder:text-muted-foreground outline-none border-none
             pl-3 pr-9 truncate"
      placeholder="Select term..."
      aria-label="Select term"
      autocomplete="off"
      autocorrect="off"
      spellcheck={false}
    />
    <span class="absolute end-2 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none">
      <ChevronsUpDown class="size-4" />
    </span>
  </div>
  <Combobox.Portal>
    <Combobox.Content
      customAnchor={containerEl}
      class="border border-border bg-card shadow-md
             outline-hidden z-50
             max-h-72 min-w-[var(--bits-combobox-anchor-width)]
             select-none rounded-md p-1
             data-[side=bottom]:translate-y-1 data-[side=top]:-translate-y-1"
      sideOffset={4}
      forceMount
    >
      {#snippet child({ wrapperProps, props, open: isOpen })}
        {#if isOpen}
          <div {...wrapperProps} style:view-transition-name="filter-overlay">
            <div {...props} transition:fly={{ duration: 150, y: -4 }}>
              <Combobox.Viewport class="p-0.5">
                {#each filteredTerms as term, i (term.slug)}
                  {#if i === 1 && term.slug !== currentTermSlug && filteredTerms[0]?.slug === currentTermSlug}
                    <div class="mx-2 my-1 h-px bg-border"></div>
                  {/if}
                  <Combobox.Item
                    class="rounded-sm outline-hidden flex h-8 w-full select-none items-center px-2 text-sm
                           data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground
                           {term.slug === value ? 'cursor-default' : 'cursor-pointer'}
                           {term.slug === currentTermSlug ? 'font-medium text-foreground' : 'text-foreground'}"
                    value={term.slug}
                    label={term.description}
                  >
                    {#snippet children({ selected })}
                      <span class="flex-1 truncate">
                        {term.description}
                        {#if term.slug === currentTermSlug}
                          <span class="ml-1.5 text-xs text-muted-foreground font-normal">current</span>
                        {/if}
                      </span>
                      {#if selected}
                        <Check class="ml-2 size-4 shrink-0" />
                      {/if}
                    {/snippet}
                  </Combobox.Item>
                {:else}
                  <span class="block px-2 py-2 text-sm text-muted-foreground">
                    No terms found.
                  </span>
                {/each}
              </Combobox.Viewport>
            </div>
          </div>
        {/if}
      {/snippet}
    </Combobox.Content>
  </Combobox.Portal>
</Combobox.Root>
