<script lang="ts">
import { cn } from "$lib/utils";
import { Tooltip } from "bits-ui";
import type { Snippet } from "svelte";

let {
  text,
  delay = 150,
  side = "top",
  passthrough = false,
  triggerClass = "",
  contentClass = "",
  sideOffset = 6,
  children,
}: {
  text: string;
  delay?: number;
  side?: "top" | "bottom" | "left" | "right";
  passthrough?: boolean;
  triggerClass?: string;
  contentClass?: string;
  sideOffset?: number;
  children: Snippet;
} = $props();
</script>

<Tooltip.Root delayDuration={delay} disableHoverableContent={passthrough}>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      <span class={triggerClass} {...props}>
        {@render children()}
      </span>
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content
    {side}
    {sideOffset}
    class={cn("z-50 bg-card text-card-foreground text-xs border border-border rounded-md px-2.5 py-1.5 shadow-sm whitespace-pre-line max-w-max text-left", contentClass)}
  >
    {text}
  </Tooltip.Content>
</Tooltip.Root>
