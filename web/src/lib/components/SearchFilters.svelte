<script lang="ts">
import type { Term, Subject } from "$lib/api";
import SimpleTooltip from "./SimpleTooltip.svelte";
import TermCombobox from "./TermCombobox.svelte";
import SubjectCombobox from "./SubjectCombobox.svelte";

let {
  terms,
  subjects,
  selectedTerm = $bindable(),
  selectedSubjects = $bindable(),
  query = $bindable(),
  openOnly = $bindable(),
}: {
  terms: Term[];
  subjects: Subject[];
  selectedTerm: string;
  selectedSubjects: string[];
  query: string;
  openOnly: boolean;
} = $props();
</script>

<div class="flex flex-wrap gap-3 items-start">
  <TermCombobox {terms} bind:value={selectedTerm} />

  <SubjectCombobox {subjects} bind:value={selectedSubjects} />

  <input
    type="text"
    placeholder="Search courses..."
    bind:value={query}
    class="h-9 border border-border bg-card text-foreground rounded-md px-3 text-sm flex-1 min-w-[200px]
           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background
           transition-colors"
  />

  <SimpleTooltip text="Show only courses with available seats" delay={200} passthrough>
    <label class="flex items-center gap-1.5 h-9 text-sm text-muted-foreground cursor-pointer">
      <input type="checkbox" bind:checked={openOnly} />
      Open only
    </label>
  </SimpleTooltip>
</div>
