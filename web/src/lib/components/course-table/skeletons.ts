export const SKELETON_WIDTHS: Record<string, string> = {
  crn: "w-10",
  course_code: "w-20",
  title: "w-40",
  instructor: "w-20",
  time: "w-20",
  location: "w-20",
  seats: "w-14 ml-auto",
};

export function buildSkeletonHtml(colIds: string[], rowCount: number): string {
  const cells = colIds
    .map((id) => {
      const w = SKELETON_WIDTHS[id] ?? "w-20";
      return `<td class="py-2.5 px-2"><div class="h-4 bg-muted rounded animate-pulse ${w}"></div></td>`;
    })
    .join("");
  const row = `<tr class="border-b border-border">${cells}</tr>`;
  return row.repeat(rowCount);
}

export function buildCardSkeletonHtml(count: number): string {
  const card = `<div class="rounded-lg border border-border bg-card p-3 animate-pulse"><div class="flex items-baseline justify-between gap-2"><div class="flex items-baseline gap-1.5"><div class="h-4 w-16 bg-muted rounded"></div><div class="h-4 w-32 bg-muted rounded"></div></div><div class="h-4 w-10 bg-muted rounded"></div></div><div class="flex items-center justify-between gap-2 mt-1"><div class="h-3 w-24 bg-muted rounded"></div><div class="h-3 w-20 bg-muted rounded"></div></div></div>`;
  return card.repeat(count);
}
