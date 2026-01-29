<script lang="ts">
import { cn } from "$lib/utils";

let {
  commitHash,
  showStatusLink = true,
  class: className,
}: {
  commitHash?: string | null;
  showStatusLink?: boolean;
  class?: string;
} = $props();
</script>

<div class={cn("flex justify-center items-center gap-2 mt-auto pt-6 pb-4", className)}>
  {#if __APP_VERSION__}
    <span class="text-xs text-muted-foreground">v{__APP_VERSION__}</span>
    <div class="w-px h-3 bg-muted-foreground opacity-30"></div>
  {/if}
  <a
    href={commitHash
      ? `https://github.com/Xevion/banner/commit/${commitHash}`
      : "https://github.com/Xevion/banner"}
    target="_blank"
    rel="noopener noreferrer"
    class="text-xs text-muted-foreground no-underline hover:underline"
  >
    GitHub
  </a>
  {#if showStatusLink}
    <div class="w-px h-3 bg-muted-foreground opacity-30"></div>
    <a href="/health" class="text-xs text-muted-foreground no-underline hover:underline">
      Status
    </a>
  {/if}
</div>
