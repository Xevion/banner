<script lang="ts">
import { cn, tooltipContentClass } from "$lib/utils";
import { Tooltip } from "bits-ui";
import type { Snippet } from "svelte";

let {
  delay = 150,
  side = "top" as "top" | "bottom" | "left" | "right",
  sideOffset = 6,
  triggerClass = "",
  contentClass = "",
  avoidCollisions = true,
  collisionPadding = 8,
  children,
  content,
}: {
  delay?: number;
  side?: "top" | "bottom" | "left" | "right";
  sideOffset?: number;
  triggerClass?: string;
  contentClass?: string;
  avoidCollisions?: boolean;
  collisionPadding?: number;
  children: Snippet;
  content: Snippet;
} = $props();

let hovered = $state(false);
let focused = $state(false);
let active = $derived(hovered || focused);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span
  class={triggerClass}
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onfocusin={() => (focused = true)}
  onfocusout={() => (focused = false)}
>
  {#if active}
    <Tooltip.Root delayDuration={delay} disableHoverableContent={false}>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <span {...props}>
            {@render children()}
          </span>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content
          {side}
          {sideOffset}
          {avoidCollisions}
          {collisionPadding}
          class={cn(tooltipContentClass, contentClass)}
        >
          {@render content()}
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  {:else}
    {@render children()}
  {/if}
</span>
