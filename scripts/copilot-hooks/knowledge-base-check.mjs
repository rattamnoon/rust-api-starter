#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import { dirname, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { execFile as execFileCallback } from "node:child_process";
import { promisify } from "node:util";

const execFile = promisify(execFileCallback);

const scriptPath = fileURLToPath(import.meta.url);
const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(dirname(__dirname));

const CODE_PATH_PREFIXES = ["src/", "migrations/", "scripts/"];
const CONFIG_PATHS = new Set([
  "Cargo.toml",
  "Cargo.lock",
  "docker-compose.yml",
  ".env",
  ".env.example",
  "README.md",
  "AGENTS.md",
]);
const DOC_PATH_PREFIXES = ["docs/"];
const ARCHITECTURE_KEYWORDS = [
  "architecture",
  "data flow",
  "module",
  "boundary",
  "ownership",
  "dependency",
  "auth flow",
  "service layout",
];
const INCIDENT_KEYWORDS = [
  "incident",
  "outage",
  "production",
  "sev",
  "degraded",
  "downtime",
  "hotfix",
];
const BUG_KEYWORDS = [
  "bug",
  "fix",
  "root cause",
  "regression",
  "failure",
  "error",
  "prevention",
];
const ADR_KEYWORDS = [
  "adr",
  "decision",
  "tradeoff",
  "why",
  "standardize",
  "adopt",
  "choose",
];

async function main() {
  const input = await readStdin();
  const payload = safeJsonParse(input);

  const changedPaths = await getChangedPaths(payload?.cwd ?? repoRoot);
  if (changedPaths.length === 0 || !hasEngineeringChanges(changedPaths)) {
    return printJson({ continue: true });
  }

  const docsChanged = changedPaths.some((path) => startsWithAny(path, DOC_PATH_PREFIXES));
  if (docsChanged) {
    return printJson({ continue: true });
  }

  const transcriptText = await readTranscript(payload);
  const recommendation = recommendTarget(changedPaths, transcriptText);

  return printJson({
    continue: true,
    systemMessage: buildMessage(recommendation),
  });
}

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      data += chunk;
    });
    process.stdin.on("end", () => resolve(data));
    process.stdin.on("error", reject);
  });
}

function safeJsonParse(value) {
  if (!value.trim()) {
    return {};
  }

  try {
    return JSON.parse(value);
  } catch {
    return {};
  }
}

async function getChangedPaths(cwd) {
  try {
    const { stdout } = await execFile("git", ["status", "--short"], {
      cwd,
      maxBuffer: 1024 * 1024,
    });

    return stdout
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .map(parsePorcelainPath)
      .filter(Boolean);
  } catch {
    return [];
  }
}

function parsePorcelainPath(line) {
  const normalized = line.replace(/^([A-Z?]| )([A-Z?]| )\s+/, "");
  const renameIndex = normalized.lastIndexOf(" -> ");
  return renameIndex >= 0 ? normalized.slice(renameIndex + 4) : normalized;
}

function hasEngineeringChanges(paths) {
  return paths.some((path) => {
    if (startsWithAny(path, DOC_PATH_PREFIXES)) {
      return false;
    }

    return startsWithAny(path, CODE_PATH_PREFIXES) || CONFIG_PATHS.has(path);
  });
}

async function readTranscript(payload) {
  const transcriptPath = payload?.transcript_path ?? payload?.transcriptPath;
  if (!transcriptPath) {
    return "";
  }

  try {
    const content = await readFile(transcriptPath, "utf8");
    return content.toLowerCase();
  } catch {
    return "";
  }
}

function recommendTarget(changedPaths, transcriptText) {
  const suggestions = [];

  if (containsAny(transcriptText, INCIDENT_KEYWORDS)) {
    suggestions.push({
      label: "incident record",
      target: "docs/incidents/",
      template: "docs/templates/incident.md",
      reason: "the session looks production-impacting or incident-oriented",
    });
  }

  if (containsAny(transcriptText, ADR_KEYWORDS)) {
    suggestions.push({
      label: "ADR",
      target: "docs/adr/",
      template: "docs/templates/adr.md",
      reason: "the session includes decision or tradeoff language",
    });
  }

  if (
    containsAny(transcriptText, BUG_KEYWORDS) ||
    changedPaths.some((path) => path.startsWith("src/modules/") || path.startsWith("src/errors/"))
  ) {
    suggestions.push({
      label: "bug record",
      target: "docs/bugs/",
      template: "docs/templates/bug.md",
      reason: "the session looks like a bugfix or defect investigation",
    });
  }

  if (
    containsAny(transcriptText, ARCHITECTURE_KEYWORDS) ||
    changedPaths.some((path) => path.startsWith("src/app.rs") || path.startsWith("src/shared/") || path.startsWith("migrations/"))
  ) {
    suggestions.push({
      label: "architecture update",
      target: "docs/architecture/",
      template: "docs/architecture/data-flow.md",
      reason: "the changes may affect data flow, ownership, or system boundaries",
    });
  }

  suggestions.push({
    label: "change note",
    target: "docs/changes/",
    template: "docs/templates/change.md",
    reason: "code or config changed without a matching knowledge-base update",
  });

  return dedupeSuggestions(suggestions);
}

function dedupeSuggestions(suggestions) {
  const seen = new Set();
  return suggestions.filter((item) => {
    if (seen.has(item.target)) {
      return false;
    }

    seen.add(item.target);
    return true;
  });
}

function buildMessage(suggestions) {
  const formatted = suggestions
    .slice(0, 3)
    .map((item) => `- ${item.target} using ${item.template}: ${item.reason}`)
    .join("\n");

  return [
    "Knowledge base reminder: repo-tracked engineering files changed but `docs/` did not.",
    "Use `docs/README.md` and the `knowledge-base-engineering` skill before closing the loop.",
    "Recommended updates:",
    formatted,
  ].join("\n");
}

function startsWithAny(value, prefixes) {
  return prefixes.some((prefix) => value.startsWith(prefix));
}

function containsAny(text, keywords) {
  return keywords.some((keyword) => text.includes(keyword));
}

function printJson(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

main().catch((error) => {
  const fallback = {
    continue: true,
    systemMessage: `Knowledge base hook failed in ${relative(repoRoot, scriptPath)}: ${error.message}`,
  };
  printJson(fallback);
});
