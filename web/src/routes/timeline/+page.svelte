<script lang="ts">
import { onMount } from "svelte";
import TimelineCanvas from "$lib/components/TimelineCanvas.svelte";

// Prevent body scroll while this page is mounted via a CSS class
// (avoids conflicting with other components that may manage overflow).
onMount(() => {
  document.documentElement.classList.add("overflow-hidden", "overscroll-none");
  document.body.classList.add("overflow-hidden", "overscroll-none");
  return () => {
    document.documentElement.classList.remove("overflow-hidden", "overscroll-none");
    document.body.classList.remove("overflow-hidden", "overscroll-none");
  };
});
</script>

<svelte:head>
  <title>Class Timeline</title>
</svelte:head>

<div class="fixed inset-0 z-0 overscroll-none overflow-hidden">
  <TimelineCanvas />
</div>
