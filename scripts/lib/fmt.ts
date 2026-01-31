/**
 * Shared formatting, color, and CLI argument parsing utilities.
 */

const isTTY = process.stdout.isTTY ?? false;
const isStderrTTY = process.stderr.isTTY ?? false;

/** ANSI color wrapper — no-op when stdout is not a TTY. */
export function c(code: string, text: string): string {
  return isTTY ? `\x1b[${code}m${text}\x1b[0m` : text;
}

/** Elapsed seconds since `start` as a formatted string. */
export function elapsed(start: number): string {
  return ((Date.now() - start) / 1000).toFixed(1);
}

/** Whether stderr is a TTY (for progress spinners). */
export { isStderrTTY };

/**
 * Parse short and long CLI flags from a flat argument array.
 *
 * `spec` maps flag names to their type:
 * - `"bool"` — presence sets the value to `true`
 * - `"string"` — consumes the next argument as the value
 *
 * Short flags can be combined: `-fbW` expands to `-f -b -W`.
 * Long flags: `--frontend-only`, `--tracing pretty`.
 * `--` terminates flag parsing; remaining args go to `passthrough`.
 *
 * Returns `{ flags, passthrough }`.
 */
export function parseFlags<T extends Record<string, "bool" | "string">>(
  argv: string[],
  spec: T,
  shortMap: Record<string, keyof T>,
  defaults: { [K in keyof T]: T[K] extends "bool" ? boolean : string },
): { flags: typeof defaults; passthrough: string[] } {
  const flags = { ...defaults };
  const passthrough: string[] = [];
  let i = 0;

  while (i < argv.length) {
    const arg = argv[i];

    if (arg === "--") {
      passthrough.push(...argv.slice(i + 1));
      break;
    }

    if (arg.startsWith("--")) {
      const name = arg.slice(2);
      if (!(name in spec)) {
        console.error(`Unknown flag: ${arg}`);
        process.exit(1);
      }
      if (spec[name] === "string") {
        (flags as Record<string, unknown>)[name] = argv[++i] || "";
      } else {
        (flags as Record<string, unknown>)[name] = true;
      }
    } else if (arg.startsWith("-") && arg.length > 1) {
      for (const ch of arg.slice(1)) {
        const mapped = shortMap[ch];
        if (!mapped) {
          console.error(`Unknown flag: -${ch}`);
          process.exit(1);
        }
        if (spec[mapped as string] === "string") {
          (flags as Record<string, unknown>)[mapped as string] = argv[++i] || "";
        } else {
          (flags as Record<string, unknown>)[mapped as string] = true;
        }
      }
    } else {
      console.error(`Unknown argument: ${arg}`);
      process.exit(1);
    }

    i++;
  }

  return { flags, passthrough };
}

/**
 * Simple positional-or-keyword argument parser.
 * Returns the first positional arg, or empty string.
 */
export function parseArgs(raw: string): string[] {
  return raw
    .trim()
    .split(/\s+/)
    .filter(Boolean);
}
