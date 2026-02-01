<script lang="ts">
import { themeStore } from "$lib/stores/theme.svelte";
import { Moon, Sun } from "@lucide/svelte";
import { tick } from "svelte";
import SimpleTooltip from "./SimpleTooltip.svelte";

/**
 * Theme toggle with View Transitions API circular reveal animation.
 * The clip-path circle expands from the click point to cover the viewport.
 */
async function handleToggle(event: MouseEvent) {
  const supportsViewTransition =
    typeof document !== "undefined" &&
    "startViewTransition" in document &&
    !window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  if (!supportsViewTransition) {
    themeStore.toggle();
    return;
  }

  const x = event.clientX;
  const y = event.clientY;
  const endRadius = Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y));

  // Suppress named view-transition elements during theme change so they don't
  // get their own transition group and snap to the new theme ahead of the mask.
  document.documentElement.classList.add("theme-transitioning");

  const transition = document.startViewTransition(async () => {
    themeStore.toggle();
    await tick();
  });

  transition.finished.finally(() => {
    document.documentElement.classList.remove("theme-transitioning");
  });

  transition.ready.then(() => {
    document.documentElement.animate(
      {
        clipPath: [`circle(0px at ${x}px ${y}px)`, `circle(${endRadius}px at ${x}px ${y}px)`],
      },
      {
        duration: 500,
        easing: "cubic-bezier(0.4, 0, 0.2, 1)",
        pseudoElement: "::view-transition-new(root)",
      }
    );
  });
}
</script>

<SimpleTooltip text={themeStore.isDark ? "Switch to light mode" : "Switch to dark mode"} delay={200} side="bottom" passthrough>
  <button
    type="button"
    onclick={(e) => handleToggle(e)}
    aria-label={themeStore.isDark ? "Switch to light mode" : "Switch to dark mode"}
    class="cursor-pointer border-none rounded-md flex items-center justify-center p-1.5 select-none
      text-muted-foreground hover:text-foreground hover:bg-background/50 bg-transparent transition-colors"
  >
    <div class="relative size-[18px]">
      <Sun
        size={18}
        class="absolute inset-0 transition-all duration-300 {themeStore.isDark
          ? 'rotate-90 scale-0 opacity-0'
          : 'rotate-0 scale-100 opacity-100'}"
      />
      <Moon
        size={18}
        class="absolute inset-0 transition-all duration-300 {themeStore.isDark
          ? 'rotate-0 scale-100 opacity-100'
          : '-rotate-90 scale-0 opacity-0'}"
      />
    </div>
  </button>
</SimpleTooltip>
