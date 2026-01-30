<script lang="ts">
import { navigationStore } from "$lib/stores/navigation.svelte";
import type { Snippet } from "svelte";
import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

type Axis = "horizontal" | "vertical";

let {
  key,
  children,
  axis = "horizontal",
  inDelay = 0,
  outDelay = 0,
}: {
  key: string;
  children: Snippet;
  axis?: Axis;
  inDelay?: number;
  outDelay?: number;
} = $props();

const DURATION = 400;
const OFFSET = 40;

function translate(axis: Axis, value: number): string {
  return axis === "vertical" ? `translateY(${value}px)` : `translateX(${value}px)`;
}

function inTransition(_node: HTMLElement): TransitionConfig {
  const dir = navigationStore.direction;
  if (dir === "fade") {
    return {
      duration: DURATION,
      delay: inDelay,
      easing: cubicOut,
      css: (t: number) => `opacity: ${t}`,
    };
  }
  const offset = dir === "right" ? OFFSET : -OFFSET;
  return {
    duration: DURATION,
    delay: inDelay,
    easing: cubicOut,
    css: (t: number) => `opacity: ${t}; transform: ${translate(axis, (1 - t) * offset)}`,
  };
}

function outTransition(_node: HTMLElement): TransitionConfig {
  const dir = navigationStore.direction;
  const base = "position: absolute; top: 0; left: 0; width: 100%; height: 100%";
  if (dir === "fade") {
    return {
      duration: DURATION,
      delay: outDelay,
      easing: cubicOut,
      css: (t: number) => `${base}; opacity: ${t}`,
    };
  }
  const offset = dir === "right" ? -OFFSET : OFFSET;
  return {
    duration: DURATION,
    delay: outDelay,
    easing: cubicOut,
    css: (t: number) => `${base}; opacity: ${t}; transform: ${translate(axis, (1 - t) * offset)}`,
  };
}
</script>

<div class="relative flex flex-1 flex-col overflow-hidden">
  {#key key}
    <div in:inTransition out:outTransition class="flex flex-1 flex-col">
      {@render children()}
    </div>
  {/key}
</div>
