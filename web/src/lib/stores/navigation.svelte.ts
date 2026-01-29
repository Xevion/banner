import { beforeNavigate } from "$app/navigation";

export type NavDirection = "left" | "right" | "fade";

/** Admin sidebar order — indexes determine slide direction for same-depth siblings */
const ADMIN_NAV_ORDER = ["/admin", "/admin/scrape-jobs", "/admin/audit-log", "/admin/users"];

function getDepth(path: string): number {
  return path.replace(/\/$/, "").split("/").filter(Boolean).length;
}

function getAdminIndex(path: string): number {
  return ADMIN_NAV_ORDER.indexOf(path);
}

function computeDirection(from: string, to: string): NavDirection {
  const fromDepth = getDepth(from);
  const toDepth = getDepth(to);

  if (toDepth > fromDepth) return "right";
  if (toDepth < fromDepth) return "left";

  // Same depth — use admin sidebar ordering if both are admin routes
  const fromIdx = getAdminIndex(from);
  const toIdx = getAdminIndex(to);
  if (fromIdx >= 0 && toIdx >= 0) {
    return toIdx > fromIdx ? "right" : "left";
  }

  return "fade";
}

class NavigationStore {
  direction: NavDirection = $state("fade");
}

export const navigationStore = new NavigationStore();

/** Call once from root layout to start tracking navigation direction */
export function initNavigation() {
  beforeNavigate(({ from, to }) => {
    if (!from?.url || !to?.url) return;
    navigationStore.direction = computeDirection(from.url.pathname, to.url.pathname);
  });
}
