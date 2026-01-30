<script lang="ts">
import { onMount } from "svelte";
import TimelineCanvas from "$lib/components/TimelineCanvas.svelte";

// Prevent body scroll while this page is mounted via a CSS class
// (avoids conflicting with other components that may manage overflow).
onMount(() => {
  document.documentElement.classList.add("overflow-hidden");
  document.body.classList.add("overflow-hidden");
  return () => {
    document.documentElement.classList.remove("overflow-hidden");
    document.body.classList.remove("overflow-hidden");
  };
});
</script>

<svelte:head>
  <title>Class Timeline</title>
</svelte:head>

<div class="fixed inset-0 z-0">
  <TimelineCanvas />
</div>
