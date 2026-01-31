<script lang="ts">
import type { CodeDescription } from "$lib/bindings";
import { ChevronDown } from "@lucide/svelte";
import { Popover } from "bits-ui";
import { fly } from "svelte/transition";

let {
  instructionalMethod = $bindable<string[]>([]),
  campus = $bindable<string[]>([]),
  partOfTerm = $bindable<string[]>([]),
  attributes = $bindable<string[]>([]),
  referenceData,
}: {
  instructionalMethod: string[];
  campus: string[];
  partOfTerm: string[];
  attributes: string[];
  referenceData: {
    instructionalMethods: CodeDescription[];
    campuses: CodeDescription[];
    partsOfTerm: CodeDescription[];
    attributes: CodeDescription[];
  };
} = $props();

const hasActiveFilters = $derived(
  instructionalMethod.length > 0 ||
    campus.length > 0 ||
    partOfTerm.length > 0 ||
    attributes.length > 0
);

function toggleValue(arr: string[], code: string): string[] {
  return arr.includes(code) ? arr.filter((v) => v !== code) : [...arr, code];
}

const sections: {
  label: string;
  key: "instructionalMethod" | "campus" | "partOfTerm" | "attributes";
  dataKey: "instructionalMethods" | "campuses" | "partsOfTerm" | "attributes";
}[] = [
  { label: "Instructional Method", key: "instructionalMethod", dataKey: "instructionalMethods" },
  { label: "Campus", key: "campus", dataKey: "campuses" },
  { label: "Part of Term", key: "partOfTerm", dataKey: "partsOfTerm" },
  { label: "Course Attributes", key: "attributes", dataKey: "attributes" },
];

function getSelected(
  key: "instructionalMethod" | "campus" | "partOfTerm" | "attributes"
): string[] {
  if (key === "instructionalMethod") return instructionalMethod;
  if (key === "campus") return campus;
  if (key === "partOfTerm") return partOfTerm;
  return attributes;
}

function toggle(key: "instructionalMethod" | "campus" | "partOfTerm" | "attributes", code: string) {
  if (key === "instructionalMethod") instructionalMethod = toggleValue(instructionalMethod, code);
  else if (key === "campus") campus = toggleValue(campus, code);
  else if (key === "partOfTerm") partOfTerm = toggleValue(partOfTerm, code);
  else attributes = toggleValue(attributes, code);
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
    Attributes
    <ChevronDown class="size-3" />
  </Popover.Trigger>
  <Popover.Content
    class="z-50 rounded-md border border-border bg-card p-3 text-card-foreground shadow-lg w-80 max-h-96 overflow-y-auto"
    sideOffset={4}
    forceMount
  >
    {#snippet child({ wrapperProps, props, open })}
      {#if open}
        <div {...wrapperProps} style:view-transition-name="filter-overlay">
          <div {...props} transition:fly={{ duration: 150, y: -4 }}>
            <div class="flex flex-col gap-3">
              {#each sections as { label, key, dataKey }, i (key)}
                {#if i > 0}
                  <div class="h-px bg-border"></div>
                {/if}
                <div class="flex flex-col gap-1.5">
                  <span class="text-xs font-medium text-muted-foreground">{label}</span>
                  <div class="flex flex-wrap gap-1">
                    {#each referenceData[dataKey] as item (item.code)}
                      {@const selected = getSelected(key)}
                      <button
                        type="button"
                        class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium transition-colors cursor-pointer
                               {selected.includes(item.code)
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                        onclick={() => toggle(key, item.code)}
                        title={item.description}
                      >
                        {item.description}
                      </button>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    {/snippet}
  </Popover.Content>
</Popover.Root>
