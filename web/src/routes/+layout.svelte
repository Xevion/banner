<script lang="ts">
import "overlayscrollbars/overlayscrollbars.css";
import "./layout.css";
import { page } from "$app/state";
import PageTransition from "$lib/components/PageTransition.svelte";
import NavBar from "$lib/components/NavBar.svelte";
import { useOverlayScrollbars } from "$lib/composables/useOverlayScrollbars.svelte";
import { initNavigation } from "$lib/stores/navigation.svelte";
import { themeStore } from "$lib/stores/theme.svelte";
import { Tooltip } from "bits-ui";
import ErrorBoundaryFallback from "$lib/components/ErrorBoundaryFallback.svelte";
import { onMount } from "svelte";

let { children } = $props();

const APP_PREFIXES = ["/profile", "/settings", "/admin"];

/**
 * Coarsened key so sub-route navigation within the (app) layout group
 * doesn't re-trigger the root page transition — the shared layout handles its own.
 */
let transitionKey = $derived(
  APP_PREFIXES.some((p) => page.url.pathname.startsWith(p)) ? "/app" : page.url.pathname
);

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
  <div class="flex min-h-screen flex-col">
    <NavBar />

    <svelte:boundary onerror={(e) => console.error("[root boundary]", e)}>
      <PageTransition key={transitionKey}>
        {@render children()}
      </PageTransition>

      {#snippet failed(error, reset)}
        <ErrorBoundaryFallback {error} {reset} />
      {/snippet}
    </svelte:boundary>
  </div>
</Tooltip.Provider>
