/**
 * PostgreSQL Docker container management.
 *
 * Usage: bun scripts/db.ts [start|reset|rm]
 */

import { readFile, writeFile } from "fs/promises";
import { spawnSync } from "child_process";

const NAME = "banner-postgres";
const USER = "banner";
const PASS = "banner";
const DB = "banner";
const PORT = "59489";
const ENV_FILE = ".env";

const cmd = process.argv[2] || "start";

function docker(...args: string[]) {
  return spawnSync("docker", args, { encoding: "utf8" });
}

function getContainer() {
  const res = docker("ps", "-a", "--filter", `name=^${NAME}$`, "--format", "json");
  return res.stdout.trim() ? JSON.parse(res.stdout) : null;
}

async function updateEnv() {
  const url = `postgresql://${USER}:${PASS}@localhost:${PORT}/${DB}`;
  try {
    let content = await readFile(ENV_FILE, "utf8");
    content = content.includes("DATABASE_URL=")
      ? content.replace(/DATABASE_URL=.*$/m, `DATABASE_URL=${url}`)
      : content.trim() + `\nDATABASE_URL=${url}\n`;
    await writeFile(ENV_FILE, content);
  } catch {
    await writeFile(ENV_FILE, `DATABASE_URL=${url}\n`);
  }
}

function create() {
  docker(
    "run", "-d", "--name", NAME,
    "-e", `POSTGRES_USER=${USER}`,
    "-e", `POSTGRES_PASSWORD=${PASS}`,
    "-e", `POSTGRES_DB=${DB}`,
    "-p", `${PORT}:5432`,
    "postgres:17-alpine",
  );
  console.log("created");
}

const container = getContainer();

if (cmd === "rm") {
  if (!container) process.exit(0);
  docker("stop", NAME);
  docker("rm", NAME);
  console.log("removed");
} else if (cmd === "reset") {
  if (!container) {
    create();
  } else {
    docker("exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `DROP DATABASE IF EXISTS ${DB}`);
    docker("exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `CREATE DATABASE ${DB}`);
    console.log("reset");
  }
  await updateEnv();
} else {
  if (!container) {
    create();
  } else if (container.State !== "running") {
    docker("start", NAME);
    console.log("started");
  } else {
    console.log("running");
  }
  await updateEnv();
}
