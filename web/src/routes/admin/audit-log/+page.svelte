<script lang="ts">
import { onMount } from "svelte";
import { client, type AuditLogResponse } from "$lib/api";

let data = $state<AuditLogResponse | null>(null);
let error = $state<string | null>(null);

onMount(async () => {
  try {
    data = await client.getAdminAuditLog();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load audit log";
  }
});
</script>

<h1 class="mb-6 text-2xl font-bold">Audit Log</h1>

{#if error}
  <p class="text-destructive">{error}</p>
{:else if !data}
  <p class="text-muted-foreground">Loading...</p>
{:else if data.entries.length === 0}
  <p class="text-muted-foreground">No audit log entries found.</p>
{:else}
  <div class="bg-card border-border overflow-hidden rounded-lg border">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-border border-b">
          <th class="px-4 py-3 text-left font-medium">Time</th>
          <th class="px-4 py-3 text-left font-medium">Course ID</th>
          <th class="px-4 py-3 text-left font-medium">Field</th>
          <th class="px-4 py-3 text-left font-medium">Old Value</th>
          <th class="px-4 py-3 text-left font-medium">New Value</th>
        </tr>
      </thead>
      <tbody>
        {#each data.entries as entry}
          <tr class="border-border border-b last:border-b-0">
            <td class="px-4 py-3">{new Date(entry.timestamp).toLocaleString()}</td>
            <td class="px-4 py-3">{entry.courseId}</td>
            <td class="px-4 py-3 font-mono text-xs">{entry.fieldChanged}</td>
            <td class="px-4 py-3">{entry.oldValue}</td>
            <td class="px-4 py-3">{entry.newValue}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
