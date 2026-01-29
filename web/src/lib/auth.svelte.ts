import type { User } from "$lib/bindings";

type AuthState =
  | { mode: "loading" }
  | { mode: "authenticated"; user: User }
  | { mode: "unauthenticated" };

class AuthStore {
  state = $state<AuthState>({ mode: "loading" });

  get user(): User | null {
    return this.state.mode === "authenticated" ? this.state.user : null;
  }

  get isAdmin(): boolean {
    return this.user?.isAdmin ?? false;
  }

  get isLoading(): boolean {
    return this.state.mode === "loading";
  }

  get isAuthenticated(): boolean {
    return this.state.mode === "authenticated";
  }

  async init() {
    try {
      const response = await fetch("/api/auth/me");
      if (response.ok) {
        const user: User = await response.json();
        this.state = { mode: "authenticated", user };
      } else {
        this.state = { mode: "unauthenticated" };
      }
    } catch {
      this.state = { mode: "unauthenticated" };
    }
  }

  login() {
    window.location.href = "/api/auth/login";
  }

  async logout() {
    try {
      await fetch("/api/auth/logout", { method: "POST" });
    } finally {
      this.state = { mode: "unauthenticated" };
      window.location.href = "/";
    }
  }
}

export const authStore = new AuthStore();
