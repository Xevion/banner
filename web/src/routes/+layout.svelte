<script lang="ts">
import "./layout.css";
import { Tooltip } from "bits-ui";
import ThemeToggle from "$lib/components/ThemeToggle.svelte";

let { children } = $props();
</script>

<svelte:head>
  {@html `<script>
    (function() {
      const stored = localStorage.getItem("theme");
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      if (stored === "dark" || (!stored && prefersDark) || (stored === "system" && prefersDark)) {
        document.documentElement.classList.add("dark");
      }
    })();
  </script>`}
</svelte:head>

<Tooltip.Provider>
  <div class="fixed top-5 right-5 z-50">
    <ThemeToggle />
  </div>

  {@render children()}
</Tooltip.Provider>
