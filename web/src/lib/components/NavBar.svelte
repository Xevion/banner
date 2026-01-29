<script lang="ts">
import { page } from "$app/state";
import { Search, User } from "@lucide/svelte";
import { authStore } from "$lib/auth.svelte";
import ThemeToggle from "./ThemeToggle.svelte";

const staticTabs = [{ href: "/", label: "Search", icon: Search }] as const;

const APP_PREFIXES = ["/profile", "/settings", "/admin"];

let profileTab = $derived({
  href: authStore.isAuthenticated ? "/profile" : "/login",
  label: authStore.isAuthenticated ? "Account" : "Login",
  icon: User,
});

function isActive(tabHref: string): boolean {
  if (tabHref === "/") return page.url.pathname === "/";
  if (tabHref === "/profile") {
    return APP_PREFIXES.some((p) => page.url.pathname.startsWith(p));
  }
  return page.url.pathname.startsWith(tabHref);
}
</script>

<nav class="w-full flex justify-center pt-5 px-5">
  <div class="w-full max-w-6xl flex items-center justify-between">
    <div class="flex items-center gap-1 rounded-lg bg-muted p-1">
      {#each staticTabs as tab}
        <a
          href={tab.href}
          class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors no-underline
            {isActive(tab.href)
              ? 'bg-background text-foreground shadow-sm'
              : 'text-muted-foreground hover:text-foreground hover:bg-background/50'}"
        >
          <tab.icon size={15} strokeWidth={2} />
          {tab.label}
        </a>
      {/each}
      <a
        href={profileTab.href}
        class="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-sm font-medium transition-colors no-underline
          {isActive(profileTab.href)
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground hover:bg-background/50'}"
      >
        <User size={15} strokeWidth={2} />
        {profileTab.label}
      </a>
    </div>

    <ThemeToggle />
  </div>
</nav>
