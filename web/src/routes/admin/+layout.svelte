<script lang="ts">
import { onMount } from "svelte";
import { goto } from "$app/navigation";
import { authStore } from "$lib/auth.svelte";
import { LayoutDashboard, Users, ClipboardList, FileText, LogOut } from "@lucide/svelte";

let { children } = $props();

onMount(async () => {
  if (authStore.isLoading) {
    await authStore.init();
  }
});

$effect(() => {
  if (authStore.state.mode === "unauthenticated") {
    goto("/login");
  }
});

const navItems = [
  { href: "/admin", label: "Dashboard", icon: LayoutDashboard },
  { href: "/admin/scrape-jobs", label: "Scrape Jobs", icon: ClipboardList },
  { href: "/admin/audit-log", label: "Audit Log", icon: FileText },
  { href: "/admin/users", label: "Users", icon: Users },
];
</script>

{#if authStore.isLoading}
  <div class="flex min-h-screen items-center justify-center">
    <p class="text-muted-foreground">Loading...</p>
  </div>
{:else if !authStore.isAdmin}
  <div class="flex min-h-screen items-center justify-center">
    <div class="text-center">
      <h1 class="text-2xl font-bold">Access Denied</h1>
      <p class="text-muted-foreground mt-2">You do not have admin access.</p>
    </div>
  </div>
{:else}
  <div class="flex min-h-screen">
    <aside class="border-border bg-card flex w-64 flex-col border-r">
      <div class="border-border border-b p-4">
        <h2 class="text-lg font-semibold">Admin</h2>
        {#if authStore.user}
          <p class="text-muted-foreground text-sm">{authStore.user.username}</p>
        {/if}
      </div>
      <nav class="flex-1 space-y-1 p-2">
        {#each navItems as item}
          <a
            href={item.href}
            class="hover:bg-accent flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors"
          >
            <item.icon size={18} />
            {item.label}
          </a>
        {/each}
      </nav>
      <div class="border-border border-t p-2">
        <button
          onclick={() => authStore.logout()}
          class="hover:bg-destructive/10 text-destructive flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors"
        >
          <LogOut size={18} />
          Sign Out
        </button>
      </div>
    </aside>
    <main class="flex-1 overflow-auto p-6">
      {@render children()}
    </main>
  </div>
{/if}
