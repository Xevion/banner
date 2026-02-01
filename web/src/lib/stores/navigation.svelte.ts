import { beforeNavigate, onNavigate } from "$app/navigation";

export type NavDirection = "left" | "right" | "fade";
export type NavAxis = "horizontal" | "vertical";

/**
 * Path used for navbar label expansion. Deferred during view transitions so
 * CSS transitions run on visible DOM instead of snapping while hidden.
 * The pill animation (JS/RAF-driven) uses page.url.pathname directly.
 */
class NavbarState {
  path = $state(typeof window !== "undefined" ? window.location.pathname : "/");
}

export const navbar = new NavbarState();

/** Sidebar nav order — indexes determine slide direction for same-depth siblings */
const SIDEBAR_NAV_ORDER = [
  "/profile",
  "/settings",
  "/admin",
  "/admin/jobs",
  "/admin/audit",
  "/admin/users",
];

const APP_PREFIXES = ["/profile", "/settings", "/admin"];

function getDepth(path: string): number {
  return path.replace(/\/$/, "").split("/").filter(Boolean).length;
}

function getSidebarIndex(path: string): number {
  return SIDEBAR_NAV_ORDER.indexOf(path);
}

function computeDirection(from: string, to: string): NavDirection {
  const fromDepth = getDepth(from);
  const toDepth = getDepth(to);

  if (toDepth > fromDepth) return "right";
  if (toDepth < fromDepth) return "left";

  // Same depth — use sidebar ordering if both are sidebar routes
  const fromIdx = getSidebarIndex(from);
  const toIdx = getSidebarIndex(to);
  if (fromIdx >= 0 && toIdx >= 0) {
    return toIdx > fromIdx ? "right" : "left";
  }

  return "fade";
}

function computeAxis(from: string, to: string): NavAxis {
  const fromIsApp = APP_PREFIXES.some((p) => from.startsWith(p));
  const toIsApp = APP_PREFIXES.some((p) => to.startsWith(p));
  return fromIsApp && toIsApp ? "vertical" : "horizontal";
}

/** Call once from root layout to start tracking navigation direction */
export function initNavigation() {
  navbar.path = window.location.pathname;

  beforeNavigate(({ from, to }) => {
    if (!from?.url || !to?.url) return;
    const fromPath = from.url.pathname;
    const toPath = to.url.pathname;
    document.documentElement.dataset.navDirection = computeDirection(fromPath, toPath);
    document.documentElement.dataset.navAxis = computeAxis(fromPath, toPath);
  });

  onNavigate((navigation) => {
    if (!document.startViewTransition) {
      // No view transitions — update path when navigation completes
      navigation.complete.then(() => {
        navbar.path = window.location.pathname;
      });
      return;
    }

    return new Promise((resolve) => {
      const vt = document.startViewTransition(async () => {
        resolve();
        await navigation.complete;
      });

      // Update navbar path only after the view transition finishes and the
      // real DOM is visible again, so CSS transitions can actually run.
      vt.finished.then(() => {
        navbar.path = window.location.pathname;
      });
    });
  });
}
