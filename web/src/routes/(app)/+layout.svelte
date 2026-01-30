<script lang="ts">
import { goto } from "$app/navigation";
import { page } from "$app/state";
import { authStore } from "$lib/auth.svelte";
import PageTransition from "$lib/components/PageTransition.svelte";
import ErrorBoundaryFallback from "$lib/components/ErrorBoundaryFallback.svelte";
import {
  ClipboardList,
  FileText,
  LayoutDashboard,
  LogOut,
  Settings,
  User,
  Users,
} from "@lucide/svelte";
import { onMount, tick } from "svelte";

let { children } = $props();

// Track boundary reset function so navigation can auto-clear errors
let boundaryReset = $state<(() => void) | null>(null);
let errorPathname = $state<string | null>(null);

function onBoundaryError(e: unknown, reset: () => void) {
  console.error("[page boundary]", e);
  boundaryReset = reset;
  errorPathname = page.url.pathname;
}

// Auto-reset the boundary only when the user navigates away from the errored page
$effect(() => {
  const currentPath = page.url.pathname;

  if (boundaryReset && errorPathname && currentPath !== errorPathname) {
    const reset = boundaryReset;
    boundaryReset = null;
    errorPathname = null;
    tick().then(() => reset());
  }
});

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

const userItems = [
  { href: "/profile", label: "Profile", icon: User },
  { href: "/settings", label: "Settings", icon: Settings },
];

const adminItems = [
  { href: "/admin", label: "Dashboard", icon: LayoutDashboard },
  { href: "/admin/jobs", label: "Scrape Jobs", icon: ClipboardList },
  { href: "/admin/audit", label: "Audit Log", icon: FileText },
  { href: "/admin/users", label: "Users", icon: Users },
];

function isActive(href: string): boolean {
  if (href === "/admin") return page.url.pathname === "/admin";
  return page.url.pathname.startsWith(href);
}
</script>

{#if authStore.isLoading}
  <div class="flex flex-col items-center p-5 pt-2">
    <div class="w-full max-w-6xl">
      <p class="text-muted-foreground py-12 text-center text-sm">Loading...</p>
    </div>
  </div>
{:else if !authStore.isAuthenticated}
  <div class="flex flex-col items-center p-5 pt-2">
    <div class="w-full max-w-6xl">
      <p class="text-muted-foreground py-12 text-center text-sm">Redirecting to login...</p>
    </div>
  </div>
{:else}
  <div class="flex flex-col items-center p-5 pt-2">
    <div class="w-full max-w-6xl flex gap-8">
      <!-- Inline sidebar -->
      <aside class="w-48 shrink-0 pt-1">
        {#if authStore.user}
          <div class="mb-4 px-2">
            <p class="text-sm font-medium text-foreground">{authStore.user.discordUsername}</p>
          </div>
        {/if}

        <nav class="flex flex-col gap-0.5">
          <span class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5">User</span>
          {#each userItems as item}
            <a
              href={item.href}
              class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors
                {isActive(item.href)
                  ? 'text-foreground bg-muted font-medium'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
            >
              <item.icon size={15} strokeWidth={2} />
              {item.label}
            </a>
          {/each}

          {#if authStore.isAdmin}
            <div class="my-2 mx-2 border-t border-border"></div>
            <span class="px-2 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60 mb-0.5">Admin</span>
            {#each adminItems as item}
              <a
                href={item.href}
                class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm no-underline transition-colors
                  {isActive(item.href)
                    ? 'text-foreground bg-muted font-medium'
                    : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
              >
                <item.icon size={15} strokeWidth={2} />
                {item.label}
              </a>
            {/each}
          {/if}

          <div class="my-2 mx-2 border-t border-border"></div>
          <button
            onclick={() => authStore.logout()}
            class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer
              bg-transparent border-none text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
          >
            <LogOut size={15} strokeWidth={2} />
            Sign Out
          </button>
        </nav>
      </aside>

      <!-- Content -->
      <main class="flex-1 min-w-0">
        <svelte:boundary onerror={onBoundaryError}>
          <PageTransition key={page.url.pathname} axis="vertical">
            {@render children()}
          </PageTransition>

          {#snippet failed(error, reset)}
            <ErrorBoundaryFallback title="Page error" {error} {reset} />
          {/snippet}
        </svelte:boundary>
      </main>
    </div>
  </div>
{/if}
