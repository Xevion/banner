<script lang="ts">
import { browser } from "$app/environment";
import { Monitor, Moon, Sun } from "@lucide/svelte";

type Theme = "light" | "dark" | "system";

let theme = $state<Theme>("system");

if (browser) {
  theme = (localStorage.getItem("theme") as Theme) ?? "system";
}

const nextTheme = $derived<Theme>(
  theme === "light" ? "dark" : theme === "dark" ? "system" : "light"
);

function applyTheme(t: Theme) {
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const isDark = t === "dark" || (t === "system" && prefersDark);
  document.documentElement.classList.toggle("dark", isDark);
}

function toggle() {
  const next = nextTheme;

  const update = () => {
    theme = next;
    localStorage.setItem("theme", next);
    applyTheme(next);
  };

  if (document.startViewTransition) {
    document.startViewTransition(update);
  } else {
    update();
  }
}
</script>

<button
  onclick={toggle}
  class="cursor-pointer border-none rounded-md flex items-center justify-center p-2 scale-125
    text-muted-foreground hover:bg-muted bg-transparent transition-colors"
  aria-label="Toggle theme"
>
  {#if nextTheme === "dark"}
    <Moon size={18} />
  {:else if nextTheme === "system"}
    <Monitor size={18} />
  {:else}
    <Sun size={18} />
  {/if}
</button>
