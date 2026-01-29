<script lang="ts">
import type { Term, Subject } from "$lib/api";

let {
  terms,
  subjects,
  selectedTerm = $bindable(),
  selectedSubject = $bindable(),
  query = $bindable(),
  openOnly = $bindable(),
}: {
  terms: Term[];
  subjects: Subject[];
  selectedTerm: string;
  selectedSubject: string;
  query: string;
  openOnly: boolean;
} = $props();
</script>

<div class="flex flex-wrap gap-3 items-center">
  <select
    bind:value={selectedTerm}
    class="border border-border bg-card text-foreground rounded-md px-3 py-1.5 text-sm"
  >
    {#each terms as term (term.code)}
      <option value={term.code}>{term.description}</option>
    {/each}
  </select>

  <select
    bind:value={selectedSubject}
    class="border border-border bg-card text-foreground rounded-md px-3 py-1.5 text-sm"
  >
    <option value="">All Subjects</option>
    {#each subjects as subject (subject.code)}
      <option value={subject.code}>{subject.description}</option>
    {/each}
  </select>

  <input
    type="text"
    placeholder="Search courses..."
    bind:value={query}
    class="border border-border bg-card text-foreground rounded-md px-3 py-1.5 text-sm flex-1 min-w-[200px]"
  />

  <label class="flex items-center gap-1.5 text-sm text-muted-foreground cursor-pointer">
    <input type="checkbox" bind:checked={openOnly} />
    Open only
  </label>
</div>
