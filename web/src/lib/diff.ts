export interface DiffEntry {
  path: string;
  oldVal: unknown;
  newVal: unknown;
}

function isObject(val: unknown): val is Record<string, unknown> {
  return val !== null && typeof val === "object" && !Array.isArray(val);
}

/**
 * Recursively compares two JSON-compatible values and returns a list of
 * structural differences with dot-notation paths.
 */
export function jsonDiff(oldVal: unknown, newVal: unknown): DiffEntry[] {
  return diffRecurse("", oldVal, newVal);
}

function diffRecurse(path: string, oldVal: unknown, newVal: unknown): DiffEntry[] {
  // Both arrays: compare by index up to max length
  if (Array.isArray(oldVal) && Array.isArray(newVal)) {
    const entries: DiffEntry[] = [];
    const maxLen = Math.max(oldVal.length, newVal.length);
    for (let i = 0; i < maxLen; i++) {
      const childPath = `${path}[${i}]`;
      const inOld = i < oldVal.length;
      const inNew = i < newVal.length;
      if (inOld && inNew) {
        entries.push(...diffRecurse(childPath, oldVal[i], newVal[i]));
      } else if (inNew) {
        entries.push({ path: childPath, oldVal: undefined, newVal: newVal[i] });
      } else {
        entries.push({ path: childPath, oldVal: oldVal[i], newVal: undefined });
      }
    }
    return entries;
  }

  // Both objects: iterate union of keys
  if (isObject(oldVal) && isObject(newVal)) {
    const entries: DiffEntry[] = [];
    const allKeys = new Set([...Object.keys(oldVal), ...Object.keys(newVal)]);
    for (const key of allKeys) {
      const childPath = `${path}.${key}`;
      const inOld = key in oldVal;
      const inNew = key in newVal;
      if (inOld && inNew) {
        entries.push(...diffRecurse(childPath, oldVal[key], newVal[key]));
      } else if (inNew) {
        entries.push({ path: childPath, oldVal: undefined, newVal: newVal[key] });
      } else {
        entries.push({ path: childPath, oldVal: oldVal[key], newVal: undefined });
      }
    }
    return entries;
  }

  // Leaf comparison (primitives, or type mismatch between object/array/primitive)
  if (oldVal !== newVal) {
    return [{ path, oldVal, newVal }];
  }

  return [];
}

/**
 * Cleans up a diff path for display. Strips the leading dot produced by
 * object-key paths, and returns "(root)" for the empty root path.
 */
export function formatDiffPath(path: string): string {
  if (path === "") return "(root)";
  if (path.startsWith(".")) return path.slice(1);
  return path;
}

/**
 * Attempts to parse a string as JSON. Returns the parsed value on success,
 * or null if parsing fails. Used by the audit log to detect JSON values.
 */
export function tryParseJson(value: string): unknown | null {
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}
