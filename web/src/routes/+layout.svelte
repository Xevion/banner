<script lang="ts">
import "overlayscrollbars/overlayscrollbars.css";
import "./layout.css";
import { onMount } from "svelte";
import { OverlayScrollbars } from "overlayscrollbars";
import { Tooltip } from "bits-ui";
import ThemeToggle from "$lib/components/ThemeToggle.svelte";
import { themeStore } from "$lib/stores/theme.svelte";

let { children } = $props();

onMount(() => {
  themeStore.init();

  const osInstance = OverlayScrollbars(document.body, {
    scrollbars: {
      autoHide: "leave",
      autoHideDelay: 800,
      theme: themeStore.isDark ? "os-theme-dark" : "os-theme-light",
    },
  });

  return () => {
    osInstance?.destroy();
  };
});
</script>

<Tooltip.Provider>
  <div class="fixed top-5 right-5 z-50">
    <ThemeToggle />
  </div>

  {@render children()}
</Tooltip.Provider>
