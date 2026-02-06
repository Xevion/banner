/**
 * Pure reducer utilities for stream state management.
 *
 * These functions are designed to be testable and composable.
 */

/**
 * Update an item in an array by its ID, merging in partial updates.
 */
export function updateById<T extends { id: number }>(
  items: T[],
  id: number,
  updates: Partial<T>
): T[] {
  return items.map((item) => (item.id === id ? { ...item, ...updates } : item));
}

/**
 * Remove an item from an array by its ID.
 */
export function removeById<T extends { id: number }>(items: T[], id: number): T[] {
  return items.filter((item) => item.id !== id);
}

/**
 * Add an item to an array and optionally sort it.
 */
export function addItem<T>(items: T[], item: T, sortFn?: (a: T, b: T) => number): T[] {
  const result = [...items, item];
  return sortFn ? result.sort(sortFn) : result;
}

/**
 * Merge updates into an array by key, handling additions, updates, and removals.
 * Used for computed stream delta handling.
 */
export function mergeByKey<T, K>(
  items: T[],
  updates: T[],
  keyFn: (item: T) => K,
  removed?: K[]
): T[] {
  const result = new Map<K, T>();

  // Start with existing items
  for (const item of items) {
    result.set(keyFn(item), item);
  }

  // Remove items
  if (removed) {
    for (const key of removed) {
      result.delete(key);
    }
  }

  // Upsert updates (adds new, updates existing)
  for (const item of updates) {
    result.set(keyFn(item), item);
  }

  return Array.from(result.values());
}
