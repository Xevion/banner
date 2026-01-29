<script lang="ts">
import { navigationStore } from "$lib/stores/navigation.svelte";
import type { Snippet } from "svelte";
import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

let { key, children }: { key: string; children: Snippet } = $props();

const DURATION = 250;
const OFFSET = 40;

function inTransition(_node: HTMLElement): TransitionConfig {
  const dir = navigationStore.direction;
  if (dir === "fade") {
    return { duration: DURATION, easing: cubicOut, css: (t: number) => `opacity: ${t}` };
  }
  const x = dir === "right" ? OFFSET : -OFFSET;
  return {
    duration: DURATION,
    easing: cubicOut,
    css: (t: number) => `opacity: ${t}; transform: translateX(${(1 - t) * x}px)`,
  };
}

function outTransition(_node: HTMLElement): TransitionConfig {
  const dir = navigationStore.direction;
  // Outgoing element is positioned absolutely so incoming flows normally
  const base = "position: absolute; top: 0; left: 0; width: 100%";
  if (dir === "fade") {
    return { duration: DURATION, easing: cubicOut, css: (t: number) => `${base}; opacity: ${t}` };
  }
  const x = dir === "right" ? -OFFSET : OFFSET;
  return {
    duration: DURATION,
    easing: cubicOut,
    css: (t: number) => `${base}; opacity: ${t}; transform: translateX(${(1 - t) * x}px)`,
  };
}
</script>

<div class="relative overflow-hidden">
  {#key key}
    <div in:inTransition out:outTransition class="w-full">
      {@render children()}
    </div>
  {/key}
</div>
