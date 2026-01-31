/**
 * Shared process spawning utilities for project scripts.
 */

import { elapsed } from "./fmt";

export interface CollectResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  elapsed: string;
}

/** Sync spawn with inherited stdio. Exits process on failure. */
export function run(cmd: string[]): void {
  const proc = Bun.spawnSync(cmd, { stdio: ["inherit", "inherit", "inherit"] });
  if (proc.exitCode !== 0) process.exit(proc.exitCode);
}

/** Sync spawn with piped stdio. Returns captured output. */
export function runPiped(cmd: string[]): { exitCode: number; stdout: string; stderr: string } {
  const proc = Bun.spawnSync(cmd, { stdout: "pipe", stderr: "pipe" });
  return {
    exitCode: proc.exitCode,
    stdout: proc.stdout?.toString() ?? "",
    stderr: proc.stderr?.toString() ?? "",
  };
}

/**
 * Async spawn that collects stdout/stderr. Returns a result object.
 * Catches spawn failures (e.g. missing binary) instead of throwing.
 */
export async function spawnCollect(cmd: string[], startTime: number): Promise<CollectResult> {
  try {
    const proc = Bun.spawn(cmd, {
      env: { ...process.env, FORCE_COLOR: "1" },
      stdout: "pipe",
      stderr: "pipe",
    });
    const [stdout, stderr] = await Promise.all([
      new Response(proc.stdout).text(),
      new Response(proc.stderr).text(),
    ]);
    await proc.exited;
    return { stdout, stderr, exitCode: proc.exitCode, elapsed: elapsed(startTime) };
  } catch (err) {
    return { stdout: "", stderr: String(err), exitCode: 1, elapsed: elapsed(startTime) };
  }
}

/**
 * Race all promises, yielding results in completion order via callback.
 * Spawn failures become results, not unhandled rejections.
 */
export async function raceInOrder<T extends { name: string }>(
  promises: Promise<T & CollectResult>[],
  fallbacks: T[],
  onResult: (r: T & CollectResult) => void,
): Promise<void> {
  const tagged = promises.map((p, i) =>
    p
      .then((r) => ({ i, r }))
      .catch((err) => ({
        i,
        r: {
          ...fallbacks[i],
          exitCode: 1,
          stdout: "",
          stderr: String(err),
          elapsed: "?",
        } as T & CollectResult,
      })),
  );
  for (let n = 0; n < promises.length; n++) {
    const { i, r } = await Promise.race(tagged);
    tagged[i] = new Promise(() => {}); // sentinel: never resolves
    onResult(r);
  }
}

/** Spawn managed processes with coordinated cleanup on exit. */
export class ProcessGroup {
  private procs: ReturnType<typeof Bun.spawn>[] = [];

  constructor() {
    const cleanup = async () => {
      await this.killAll();
      process.exit(0);
    };
    process.on("SIGINT", cleanup);
    process.on("SIGTERM", cleanup);
  }

  spawn(cmd: string[]): ReturnType<typeof Bun.spawn> {
    const proc = Bun.spawn(cmd, { stdio: ["inherit", "inherit", "inherit"] });
    this.procs.push(proc);
    return proc;
  }

  async killAll(): Promise<void> {
    for (const p of this.procs) p.kill();
    await Promise.all(this.procs.map((p) => p.exited));
  }

  /** Wait for any process to exit, kill the rest, return exit code. */
  async waitForFirst(): Promise<number> {
    const results = this.procs.map((p, i) => p.exited.then((code) => ({ i, code })));
    const first = await Promise.race(results);
    await this.killAll();
    return first.code;
  }
}
