<script lang="ts">
import "overlayscrollbars/overlayscrollbars.css";
import "./layout.css";
import { onMount } from "svelte";
import { Tooltip } from "bits-ui";
import ThemeToggle from "$lib/components/ThemeToggle.svelte";
import { themeStore } from "$lib/stores/theme.svelte";
import { useOverlayScrollbars } from "$lib/composables/useOverlayScrollbars.svelte";

let { children } = $props();

useOverlayScrollbars(() => document.body, {
  scrollbars: {
    autoHide: "leave",
    autoHideDelay: 800,
  },
});

onMount(() => {
  themeStore.init();

  requestAnimationFrame(() => {
    document.documentElement.classList.remove("no-transition");
  });
});
</script>

<Tooltip.Provider>
  <div class="fixed top-5 right-5 z-50">
    <ThemeToggle />
  </div>

  {@render children()}
</Tooltip.Provider>
