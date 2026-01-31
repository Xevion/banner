/**
 * Run project tests.
 *
 * Usage: bun scripts/test.ts [rust|web|<nextest filter args>]
 */

import { run } from "./lib/proc";

const input = process.argv.slice(2).join(" ").trim();

if (input === "web") {
  run(["bun", "run", "--cwd", "web", "test"]);
} else if (input === "rust") {
  run(["cargo", "nextest", "run", "-E", "not test(export_bindings)"]);
} else if (input === "") {
  run(["cargo", "nextest", "run", "-E", "not test(export_bindings)"]);
  run(["bun", "run", "--cwd", "web", "test"]);
} else {
  run(["cargo", "nextest", "run", ...input.split(/\s+/)]);
}
