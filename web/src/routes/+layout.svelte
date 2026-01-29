<script lang="ts">
import "overlayscrollbars/overlayscrollbars.css";
import "./layout.css";
import { page } from "$app/state";
import PageTransition from "$lib/components/PageTransition.svelte";
import ThemeToggle from "$lib/components/ThemeToggle.svelte";
import { useOverlayScrollbars } from "$lib/composables/useOverlayScrollbars.svelte";
import { initNavigation } from "$lib/stores/navigation.svelte";
import { themeStore } from "$lib/stores/theme.svelte";
import { Tooltip } from "bits-ui";
import { onMount } from "svelte";

let { children } = $props();

initNavigation();

useOverlayScrollbars(() => document.body, {
  scrollbars: {
    autoHide: "leave",
    autoHideDelay: 800,
  },
});

onMount(() => {
  themeStore.init();
});
</script>

<Tooltip.Provider>
  <div class="fixed top-5 right-5 z-50">
    <ThemeToggle />
  </div>

  <PageTransition key={page.url.pathname}>
    {@render children()}
  </PageTransition>
</Tooltip.Provider>
