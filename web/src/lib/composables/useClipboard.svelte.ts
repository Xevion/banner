/**
 * Reactive clipboard copy with automatic "copied" state reset.
 *
 * Returns a `copiedValue` that is non-null while the copied feedback
 * should be displayed, and a `copy()` function to trigger a copy.
 */
export function useClipboard(resetMs = 2000) {
  let copiedValue = $state<string | null>(null);
  let timeoutId: number | undefined;

  async function copy(text: string, event?: MouseEvent | KeyboardEvent) {
    event?.stopPropagation();
    try {
      await navigator.clipboard.writeText(text);
      clearTimeout(timeoutId);
      copiedValue = text;
      timeoutId = window.setTimeout(() => {
        copiedValue = null;
        timeoutId = undefined;
      }, resetMs);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  }

  return {
    get copiedValue() {
      return copiedValue;
    },
    copy,
  };
}
