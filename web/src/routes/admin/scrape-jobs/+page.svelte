<script lang="ts">
import { onMount } from "svelte";
import { client, type ScrapeJobsResponse } from "$lib/api";

let data = $state<ScrapeJobsResponse | null>(null);
let error = $state<string | null>(null);

onMount(async () => {
  try {
    data = await client.getAdminScrapeJobs();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load scrape jobs";
  }
});
</script>

<h1 class="mb-6 text-2xl font-bold">Scrape Jobs</h1>

{#if error}
  <p class="text-destructive">{error}</p>
{:else if !data}
  <p class="text-muted-foreground">Loading...</p>
{:else if data.jobs.length === 0}
  <p class="text-muted-foreground">No scrape jobs found.</p>
{:else}
  <div class="bg-card border-border overflow-hidden rounded-lg border">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-border border-b">
          <th class="px-4 py-3 text-left font-medium">ID</th>
          <th class="px-4 py-3 text-left font-medium">Type</th>
          <th class="px-4 py-3 text-left font-medium">Priority</th>
          <th class="px-4 py-3 text-left font-medium">Execute At</th>
          <th class="px-4 py-3 text-left font-medium">Retries</th>
          <th class="px-4 py-3 text-left font-medium">Status</th>
        </tr>
      </thead>
      <tbody>
        {#each data.jobs as job}
          <tr class="border-border border-b last:border-b-0">
            <td class="px-4 py-3">{job.id}</td>
            <td class="px-4 py-3">{job.targetType}</td>
            <td class="px-4 py-3">{job.priority}</td>
            <td class="px-4 py-3">{new Date(job.executeAt).toLocaleString()}</td>
            <td class="px-4 py-3">{job.retryCount}/{job.maxRetries}</td>
            <td class="px-4 py-3">{job.lockedAt ? "Locked" : "Pending"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
