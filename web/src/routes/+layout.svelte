<script lang="ts">
import "./layout.css";
import { onMount } from "svelte";
import { Tooltip } from "bits-ui";
import ThemeToggle from "$lib/components/ThemeToggle.svelte";
import { themeStore } from "$lib/stores/theme.svelte";

let { children } = $props();

onMount(() => {
  themeStore.init();
});
</script>

<Tooltip.Provider>
  <div class="fixed top-5 right-5 z-50">
    <ThemeToggle />
  </div>

  {@render children()}
</Tooltip.Provider>
