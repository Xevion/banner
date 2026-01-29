class ThemeStore {
  isDark = $state<boolean>(false);
  private initialized = false;

  init() {
    if (this.initialized || typeof window === "undefined") return;
    this.initialized = true;

    const stored = localStorage.getItem("theme");
    if (stored === "light" || stored === "dark") {
      this.isDark = stored === "dark";
    } else {
      this.isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    }

    this.updateDOMClass();
  }

  toggle() {
    this.isDark = !this.isDark;
    localStorage.setItem("theme", this.isDark ? "dark" : "light");
    this.updateDOMClass();
  }

  private updateDOMClass() {
    if (typeof document === "undefined") return;

    if (this.isDark) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }
}

export const themeStore = new ThemeStore();
