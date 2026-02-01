import type { useClipboard } from "$lib/composables/useClipboard.svelte";
import { getContext } from "svelte";

export const TABLE_CONTEXT_KEY = Symbol("table-context");

export type TableContext = {
  clipboard: ReturnType<typeof useClipboard>;
  subjectMap: Record<string, string>;
  maxSubjectLength: number;
};

/** Type-safe utility for accessing table context in cell components */
export function getTableContext(): TableContext {
  return getContext<TableContext>(TABLE_CONTEXT_KEY);
}
