<script lang="ts">
import { authStore } from "$lib/auth.svelte";
import { LogIn } from "@lucide/svelte";
</script>

<div class="flex flex-1 items-center justify-center pb-14">
  <div class="w-full max-w-sm space-y-6 text-center">
    <h1 class="text-3xl font-bold">Sign In</h1>
    <p class="text-muted-foreground">Sign in with your Discord account to continue.</p>
    <button
      onclick={() => authStore.login()}
      class="inline-flex w-full items-center justify-center gap-2 rounded-lg bg-[#5865F2] px-6 py-3 text-lg font-semibold text-white transition-colors hover:bg-[#4752C4]"
    >
      <LogIn size={20} />
      Sign in with Discord
    </button>
  </div>
</div>
