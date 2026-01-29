<script lang="ts">
import { Tooltip } from "bits-ui";
import type { Snippet } from "svelte";

let {
  text,
  delay = 150,
  side = "top",
  passthrough = false,
  children,
}: {
  text: string;
  delay?: number;
  side?: "top" | "bottom" | "left" | "right";
  passthrough?: boolean;
  children: Snippet;
} = $props();
</script>

<Tooltip.Root delayDuration={delay} disableHoverableContent={passthrough}>
  <Tooltip.Trigger>
    {@render children()}
  </Tooltip.Trigger>
  <Tooltip.Content
    {side}
    sideOffset={6}
    class="z-50 bg-card text-card-foreground text-xs border border-border rounded-md px-2.5 py-1.5 shadow-md whitespace-pre-line max-w-max"
  >
    {text}
  </Tooltip.Content>
</Tooltip.Root>
