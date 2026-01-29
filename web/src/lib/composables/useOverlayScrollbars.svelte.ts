import { onMount } from "svelte";
import { OverlayScrollbars, type PartialOptions } from "overlayscrollbars";
import { themeStore } from "$lib/stores/theme.svelte";

/**
 * Set up OverlayScrollbars on an element with automatic theme reactivity.
 *
 * Must be called during component initialization (uses `onMount` internally).
 * The scrollbar theme automatically syncs with `themeStore.isDark`.
 */
export function useOverlayScrollbars(getElement: () => HTMLElement, options: PartialOptions = {}) {
  onMount(() => {
    const element = getElement();
    const osInstance = OverlayScrollbars(element, {
      ...options,
      scrollbars: {
        ...options.scrollbars,
        theme: themeStore.isDark ? "os-theme-dark" : "os-theme-light",
      },
    });

    const unwatch = $effect.root(() => {
      $effect(() => {
        osInstance.options({
          scrollbars: {
            theme: themeStore.isDark ? "os-theme-dark" : "os-theme-light",
          },
        });
      });
    });

    return () => {
      unwatch();
      osInstance.destroy();
    };
  });
}
