<script lang="ts">
import { onMount } from "svelte";
import { client, type AdminStatus } from "$lib/api";

let status = $state<AdminStatus | null>(null);
let error = $state<string | null>(null);

onMount(async () => {
  try {
    status = await client.getAdminStatus();
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load status";
  }
});
</script>

<h1 class="mb-6 text-2xl font-bold">Dashboard</h1>

{#if error}
  <p class="text-destructive">{error}</p>
{:else if !status}
  <p class="text-muted-foreground">Loading...</p>
{:else}
  <div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
    <div class="bg-card border-border rounded-lg border p-4">
      <p class="text-muted-foreground text-sm">Users</p>
      <p class="text-3xl font-bold">{status.userCount}</p>
    </div>
    <div class="bg-card border-border rounded-lg border p-4">
      <p class="text-muted-foreground text-sm">Active Sessions</p>
      <p class="text-3xl font-bold">{status.sessionCount}</p>
    </div>
    <div class="bg-card border-border rounded-lg border p-4">
      <p class="text-muted-foreground text-sm">Courses</p>
      <p class="text-3xl font-bold">{status.courseCount}</p>
    </div>
    <div class="bg-card border-border rounded-lg border p-4">
      <p class="text-muted-foreground text-sm">Scrape Jobs</p>
      <p class="text-3xl font-bold">{status.scrapeJobCount}</p>
    </div>
  </div>

  <h2 class="mt-8 mb-4 text-lg font-semibold">Services</h2>
  <div class="bg-card border-border rounded-lg border">
    {#each status.services as service}
      <div class="border-border flex items-center justify-between border-b px-4 py-3 last:border-b-0">
        <span class="font-medium">{service.name}</span>
        <span class="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900 dark:text-green-200">
          {service.status}
        </span>
      </div>
    {/each}
  </div>
{/if}
