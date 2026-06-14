#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { performance } from "node:perf_hooks";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const aspBin = process.env.ASP_BIN || "asp";
const caseBudgetMs = Number(process.env.ASP_PROVIDER_KNOWLEDGE_AXIS_CASE_BUDGET_MS || 10000);
const totalBudgetMs = Number(process.env.ASP_PROVIDER_KNOWLEDGE_AXIS_TOTAL_BUDGET_MS || 90000);

const axes = ["env", "runtime-source", "lang", "std", "capability", "extension", "pattern", "compare"];

const providers = [
  {
    language: "gerbil-scheme",
    workspace: "languages/gerbil-scheme-language-project-harness",
    semanticPacketRequired: false,
    cases: [
      ["env", ["gxi"], "fact"],
      ["runtime-source", ["macro", "sugar"], "fact"],
      ["lang", ["module", "import"], "fact"],
      ["std", ["json"], "fact"],
      ["capability", ["macro", "poo"], "fact"],
      ["extension", ["poo", "syntax"], "fact"],
      ["pattern", ["poo", "object-system"], "fact"],
      ["compare", ["env", "active", "documented"], "fact"],
    ],
  },
  {
    language: "typescript",
    workspace: "languages/typescript-lang-project-harness",
    semanticPacketRequired: true,
    cases: [
      ["env", ["package"], "fact"],
      ["runtime-source", ["source"], "unknown"],
      ["lang", ["module", "import"], "fact"],
      ["std", ["Promise"], "fact"],
      ["capability", ["owner"], "fact"],
      ["extension", ["typescript"], "fact"],
      ["pattern", ["dependency"], "fact"],
      ["compare", ["esm", "cjs"], "fact"],
    ],
  },
  {
    language: "python",
    workspace: "languages/python-lang-project-harness",
    semanticPacketRequired: true,
    cases: [
      ["env", ["pyproject"], "fact"],
      ["runtime-source", ["source"], "unknown"],
      ["lang", ["import", "decorator"], "fact"],
      ["std", ["pathlib"], "fact"],
      ["capability", ["owner"], "fact"],
      ["extension", ["pytest"], "fact"],
      ["pattern", ["dependency"], "fact"],
      ["compare", ["ast", "tokenize"], "fact"],
    ],
  },
  {
    language: "julia",
    workspace: "languages/JuliaLangProjectHarness.jl",
    semanticPacketRequired: true,
    cases: [
      ["env", ["Project.toml"], "fact"],
      ["runtime-source", ["source"], "unknown"],
      ["lang", ["dispatch", "macro"], "fact"],
      ["std", ["Test"], "fact"],
      ["capability", ["owner"], "fact"],
      ["extension", ["JSON3"], "fact"],
      ["pattern", ["dispatch"], "fact"],
      ["compare", ["module", "package"], "fact"],
    ],
  },
];

const failures = [];
const records = [];

function runCommand(args, input = undefined) {
  const started = performance.now();
  const result = spawnSync(aspBin, args, {
    cwd: repoRoot,
    encoding: "utf8",
    input,
    maxBuffer: 16 * 1024 * 1024,
  });
  const elapsedMs = performance.now() - started;
  if (result.error) {
    throw new Error(`failed to execute ${aspBin}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(
      [
        `command failed (${result.status}): ${aspBin} ${args.join(" ")}`,
        result.stderr.trim(),
        result.stdout.trim(),
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return { stdout: result.stdout, stderr: result.stderr, elapsedMs };
}

function parseJson(stdout, label) {
  const trimmed = stdout.trim();
  try {
    return JSON.parse(trimmed);
  } catch (error) {
    const jsonStart = trimmed.indexOf("{");
    if (jsonStart >= 0) {
      try {
        return JSON.parse(trimmed.slice(jsonStart));
      } catch {
        // Fall through to the clearer error below.
      }
    }
    throw new Error(`invalid JSON for ${label}: ${error.message}\n${trimmed.slice(0, 600)}`);
  }
}

function numberField(value) {
  return typeof value === "number" && Number.isFinite(value) ? value : 0;
}

function evidenceSummary(packet, axis) {
  const headerFields = packet.header?.fields ?? {};
  const semanticPacket = packet.schemaId === "agent.semantic-protocols.semantic-search-packet";
  const schemaId = packet.schemaId || "provider-local";
  const grade = headerFields.evidenceGrade || packet.evidenceGrade || "missing";
  const view = packet.view || packet.namespace || axis;
  const method = packet.method || `search/${axis}`;
  const hits = Array.isArray(packet.hits) ? packet.hits.length : numberField(headerFields.hit);
  const facts =
    numberField(headerFields.fact) ||
    (Array.isArray(packet.facts) ? packet.facts.length : 0) ||
    (Array.isArray(packet.matches) ? packet.matches.length : 0) ||
    (Array.isArray(packet.comparisons) ? packet.comparisons.length : 0) ||
    (packet.patternMapping ? 1 : 0) ||
    (Array.isArray(packet.packages) ? packet.packages.length : 0) ||
    (Array.isArray(packet.nodes) ? packet.nodes.length : 0);
  const notes = Array.isArray(packet.notes) ? packet.notes : [];
  return { semanticPacket, schemaId, grade, view, method, facts, hits, notes };
}

function check(condition, label) {
  if (!condition) {
    failures.push(label);
  }
}

function expectedGuideNeedles(language) {
  return axes.map((axis) => `search ${axis}`);
}

const totalStarted = performance.now();

for (const provider of providers) {
  const guide = runCommand([provider.language, "guide", provider.workspace]);
  for (const needle of expectedGuideNeedles(provider.language)) {
    check(
      guide.stdout.includes(needle),
      `${provider.language} guide does not advertise ${needle}`,
    );
  }
  check(
    guide.elapsedMs <= caseBudgetMs,
    `${provider.language} guide exceeded budget: ${guide.elapsedMs.toFixed(1)}ms > ${caseBudgetMs}ms`,
  );
  records.push({
    language: provider.language,
    axis: "guide",
    grade: "advertised",
    facts: axes.length,
    hits: 0,
    schemaId: "text-guide",
    elapsedMs: guide.elapsedMs,
  });

  for (const [axis, terms, expectedGrade] of provider.cases) {
    const args = [
      provider.language,
      "search",
      axis,
      ...terms,
      "--json",
      "--workspace",
      provider.workspace,
    ];
    const result = runCommand(args);
    const packet = parseJson(result.stdout, `${provider.language} ${axis}`);
    const summary = evidenceSummary(packet, axis);
    const label = `${provider.language} ${axis}`;

    check(summary.view === axis, `${label} returned view ${summary.view}`);
    check(summary.method === `search/${axis}`, `${label} returned method ${summary.method}`);
    check(summary.grade === expectedGrade, `${label} grade ${summary.grade} != ${expectedGrade}`);
    if (provider.semanticPacketRequired) {
      check(summary.semanticPacket, `${label} did not return semantic-search-packet`);
      check(packet.languageId === provider.language, `${label} languageId ${packet.languageId}`);
      check(
        path.resolve(packet.projectRoot || "") === path.resolve(repoRoot, provider.workspace),
        `${label} projectRoot ${packet.projectRoot}`,
      );
    }
    if (expectedGrade === "fact") {
      check(summary.facts > 0, `${label} returned no facts`);
    } else {
      check(summary.facts === 0, `${label} unknown response carried facts=${summary.facts}`);
      check(summary.hits === 0, `${label} unknown response carried hits=${summary.hits}`);
      check(summary.notes.length > 0, `${label} unknown response had no frontier notes`);
    }
    check(
      result.elapsedMs <= caseBudgetMs,
      `${label} exceeded budget: ${result.elapsedMs.toFixed(1)}ms > ${caseBudgetMs}ms`,
    );
    records.push({
      language: provider.language,
      axis,
      grade: summary.grade,
      facts: summary.facts,
      hits: summary.hits,
      schemaId: summary.schemaId,
      elapsedMs: result.elapsedMs,
    });
    console.log(
      `[provider-knowledge-axis] ${label} grade=${summary.grade} facts=${summary.facts} hits=${summary.hits} schema=${summary.schemaId} ms=${result.elapsedMs.toFixed(1)}`,
    );
  }
}

const totalMs = performance.now() - totalStarted;
check(totalMs <= totalBudgetMs, `total exceeded budget: ${totalMs.toFixed(1)}ms > ${totalBudgetMs}ms`);

const slowest = records
  .slice()
  .sort((left, right) => right.elapsedMs - left.elapsedMs)
  .slice(0, 5)
  .map((record) => `${record.language}/${record.axis}=${record.elapsedMs.toFixed(1)}ms`)
  .join(", ");

console.log(
  `[provider-knowledge-axis] ok=${failures.length === 0} cases=${records.length} totalMs=${totalMs.toFixed(1)} slowest=${slowest}`,
);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`[provider-knowledge-axis] FAIL ${failure}`);
  }
  process.exit(1);
}
