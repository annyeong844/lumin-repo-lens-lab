import { it } from "vitest";
// Tests for audit-repo.mjs --pre-write integration — P1-3 step 5.5.
//
// Exit-code contract (docs/history/phases/p1/p1-3.md §4.4):
//   0 — audit succeeded; pre-write either ran or was not requested.
//   2 — --pre-write requested but --intent missing.
//
// Additional pinning:
//   - Default audit (no --pre-write) does NOT create advisory artifacts.
//   - --pre-write --intent works under a path with spaces and $.
//   - --pre-write --intent - preserves stdin dispatch through the recommended path.
//   - manifest.json.preWrite reflects what happened.

import { execFileSync, spawnSync } from "node:child_process";
import {
  writeFileSync,
  chmodSync,
  readFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  existsSync,
} from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");
const NODE = process.execPath;
const AUDIT_REPO = path.join(DIR, "audit-repo.mjs");

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}
function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildFixture(fxDir) {
  write(
    fxDir,
    "package.json",
    JSON.stringify({ name: "au-fx", type: "module" }),
  );
  write(fxDir, "src/a.ts", "export const formatDate = (d) => d.toString();\n");
  write(
    fxDir,
    "src/b.ts",
    "import { formatDate } from './a';\nexport const useFmt = () => formatDate(new Date());\n",
  );
  write(
    fxDir,
    "src/utils/mime.ts",
    "export function getMimeType(path) { return path.endsWith('.json') ? 'application/json' : 'text/plain'; }\n",
  );
}

function writeFakeRustAnalyzer(dir) {
  const script = path.join(dir, "fake-rust-analyzer.mjs");
  writeFileSync(
    script,
    `#!/usr/bin/env node
import { readFileSync, writeFileSync } from 'node:fs';
const outIndex = process.argv.indexOf('--output');
const output = outIndex >= 0 ? process.argv[outIndex + 1] : null;
if (process.argv.includes('--intent')) readFileSync(0, 'utf8');
if (!output) process.exit(2);
writeFileSync(output, JSON.stringify({
  schemaVersion: 'rust-pre-write.v1',
  policyVersion: 'rust-pre-write-policy.v1',
  intent: { files: ['src/lib.rs'] },
  meta: { producer: 'lumin-rust-analyzer' },
  coverage: { files: 'ran' },
  lookups: [],
  cueCards: []
}, null, 2));
console.log('## rust pre-write');
`,
  );
  if (process.platform === "win32") {
    const cmd = path.join(dir, "fake-rust-analyzer.cmd");
    writeFileSync(cmd, `@echo off\r\n"${NODE}" "${script}" %*\r\n`);
    return cmd;
  }
  chmodSync(script, 0o755);
  return script;
}

// ═══ T1. --pre-write without --intent → exit 2 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-noint-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-noint-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--profile",
        "quick",
      ],
      { encoding: "utf8" },
    );

    assert(
      "T1. --pre-write without --intent → exit code 2",
      res.status === 2,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`,
    );

    assert(
      "T1b. stderr contains helpful message",
      /--pre-write requested but skipped.*--intent/.test(res.stderr),
      `stderr=${res.stderr.slice(0, 300)}`,
    );

    const manifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    assert(
      "T1c. manifest.preWrite.requested === true",
      manifest.preWrite?.requested === true,
    );
    assert(
      "T1d. manifest.preWrite.ran === false",
      manifest.preWrite?.ran === false,
    );
    assert(
      "T1e. manifest.preWrite.reason mentions intent",
      /intent/i.test(manifest.preWrite?.reason ?? ""),
      `reason=${manifest.preWrite?.reason}`,
    );
    assert(
      "T1e2. pre-write-only missing intent does not run the base quick audit",
      !(manifest.commandsRun ?? []).some((cmd) =>
        [
          "triage-repo.mjs",
          "measure-topology.mjs",
          "build-symbol-graph.mjs",
        ].includes(cmd.step),
      ),
      JSON.stringify(manifest.commandsRun),
    );

    // No advisory should have been written.
    assert(
      "T1f. pre-write-advisory.latest.json NOT created",
      !existsSync(path.join(out, "pre-write-advisory.latest.json")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. --pre-write --intent <file> → exit 0 + advisory written ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-ok-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-ok-out-"));
  try {
    buildFixture(fx);
    const intent = {
      names: ["formatDate"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, "intent.json");
    mkdirSync(out, { recursive: true });
    writeFileSync(intentPath, JSON.stringify(intent));

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--intent",
        intentPath,
        "--profile",
        "quick",
      ],
      { encoding: "utf8" },
    );

    assert(
      "T2. --pre-write + --intent → exit 0",
      res.status === 0,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`,
    );

    const manifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    assert(
      "T2b. manifest.preWrite.ran === true",
      manifest.preWrite?.ran === true,
      `preWrite=${JSON.stringify(manifest.preWrite)}`,
    );

    assert(
      "T2c. pre-write-advisory.latest.json written",
      existsSync(path.join(out, "pre-write-advisory.latest.json")),
    );
    assert(
      "T2c2. manifest.preWrite.advisoryPath points at invocation-specific advisory",
      path
        .basename(manifest.preWrite?.advisoryPath ?? "")
        .startsWith("pre-write-advisory.") &&
        path.basename(manifest.preWrite?.advisoryPath ?? "") !==
          "pre-write-advisory.latest.json" &&
        existsSync(manifest.preWrite.advisoryPath),
      JSON.stringify(manifest.preWrite),
    );
    assert(
      "T2c3. manifest.preWrite.latestAdvisoryPath keeps latest as convenience pointer only",
      path.basename(manifest.preWrite?.latestAdvisoryPath ?? "") ===
        "pre-write-advisory.latest.json" &&
        existsSync(manifest.preWrite.latestAdvisoryPath),
      JSON.stringify(manifest.preWrite),
    );
    assert(
      "T2d. audit-repo --pre-write skips the base quick-audit producer chain",
      !(manifest.commandsRun ?? []).some((cmd) =>
        [
          "triage-repo.mjs",
          "measure-topology.mjs",
          "measure-discipline.mjs",
          "classify-dead-exports.mjs",
          "rank-fixes.mjs",
          "checklist-facts.mjs",
        ].includes(cmd.step),
      ),
      JSON.stringify(manifest.commandsRun),
    );
    assert(
      "T2e. names-only pre-write path creates symbols but not triage/topology/fix-plan",
      existsSync(path.join(out, "symbols.json")) &&
        !existsSync(path.join(out, "triage.json")) &&
        !existsSync(path.join(out, "topology.json")) &&
        !existsSync(path.join(out, "fix-plan.json")),
      `artifacts=${JSON.stringify(manifest.artifactsProduced)}`,
    );
    assert(
      "T2e2. manifest.preWrite mirrors advisory evidence availability",
      manifest.preWrite?.evidenceAvailability?.status === "available" &&
        manifest.preWrite.evidenceAvailability.artifacts?.some(
          (entry) =>
            entry.artifact === "symbols.json" && entry.status === "available",
        ),
      JSON.stringify(manifest.preWrite?.evidenceAvailability),
    );
    const summaryMd = readFileSync(
      path.join(out, "audit-summary.latest.md"),
      "utf8",
    );
    assert(
      "T2f. pre-write-only path emits a command-result summary, not a repo-wide audit summary",
      existsSync(path.join(out, "audit-summary.latest.md")) &&
        manifest.artifactsProduced?.includes("audit-summary.latest.md") &&
        summaryMd.includes("## Command Result") &&
        summaryMd.includes("Pre-write ran and wrote an advisory") &&
        summaryMd.includes(path.basename(manifest.preWrite.advisoryPath)) &&
        summaryMd.includes("# Audit Artifact Brief") &&
        !summaryMd.includes("## Already Stable"),
      summaryMd,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2b. structured name owner locality reaches service-operation cue policy ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-owner-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-owner-out-"));
  try {
    buildFixture(fx);
    const intent = {
      names: [
        {
          name: "searchMime",
          kind: "function",
          why: "search MIME helpers before adding another helper",
          ownerFile: "src/utils/mime.ts",
        },
      ],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, "intent.json");
    mkdirSync(out, { recursive: true });
    writeFileSync(intentPath, JSON.stringify(intent));

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--intent",
        intentPath,
        "--profile",
        "quick",
      ],
      { encoding: "utf8" },
    );

    assert(
      "T2g. structured ownerFile pre-write intent exits 0",
      res.status === 0,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`,
    );

    const advisory = JSON.parse(
      readFileSync(path.join(out, "pre-write-advisory.latest.json"), "utf8"),
    );
    const lookup = advisory.lookups?.find(
      (entry) => entry.kind === "name" && entry.intentName === "searchMime",
    );
    const promoted = lookup?.serviceOperationSiblingPolicy?.promoted?.find(
      (entry) => entry.name === "getMimeType",
    );
    assert(
      "T2h. structured ownerFile survives CLI intent normalization into nameDeclarations",
      advisory.intent?.nameDeclarations?.[0]?.ownerFile === "src/utils/mime.ts",
      JSON.stringify(advisory.intent?.nameDeclarations),
    );
    assert(
      "T2i. owner locality lets CLI route promote same-file read-query sibling as review evidence",
      promoted?.locality?.sameFile === true &&
        promoted?.operationFamily === "read-query" &&
        promoted?.sharedDomainTokens?.includes("mime"),
      JSON.stringify(lookup?.serviceOperationSiblingPolicy),
    );
    assert(
      "T2j. service-operation sibling remains a review cue, not a strong action",
      advisory.cueCards?.some((card) =>
        card.cues?.some(
          (cue) =>
            cue.evidenceLane === "service-operation-sibling" &&
            cue.cueTier === "AGENT_REVIEW_CUE" &&
            cue.confidence === "heuristic-review",
        ),
      ),
      JSON.stringify(advisory.cueCards),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. Default (no --pre-write) → exit 0, no advisory ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-def-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-def-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [AUDIT_REPO, "--root", fx, "--output", out, "--profile", "quick"],
      { encoding: "utf8" },
    );

    assert("T3. default audit exits 0", res.status === 0);

    const manifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    assert(
      "T3b. manifest.preWrite is absent or requested:false",
      !manifest.preWrite || manifest.preWrite.requested !== true,
      `preWrite=${JSON.stringify(manifest.preWrite)}`,
    );

    assert(
      "T3c. no pre-write advisory artifact",
      !existsSync(path.join(out, "pre-write-advisory.latest.json")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3b. --pre-write --intent - reads stdin through the orchestrator ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-stdin-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-stdin-out-"));
  try {
    buildFixture(fx);
    const intent = {
      names: ["formatDate"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--intent",
        "-",
        "--profile",
        "quick",
      ],
      {
        encoding: "utf8",
        input: JSON.stringify(intent),
      },
    );

    assert(
      "T3s1. --pre-write --intent - → exit 0",
      res.status === 0,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`,
    );

    const manifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    assert(
      "T3s2. stdin intent path records preWrite ran=true",
      manifest.preWrite?.ran === true,
      `preWrite=${JSON.stringify(manifest.preWrite)}`,
    );
    assert(
      "T3s3. stdin intent writes advisory",
      existsSync(path.join(out, "pre-write-advisory.latest.json")),
    );
    assert(
      "T3s4. stdin pre-write-only path still skips base quick audit",
      !(manifest.commandsRun ?? []).some((cmd) =>
        [
          "triage-repo.mjs",
          "measure-topology.mjs",
          "build-symbol-graph.mjs",
        ].includes(cmd.step),
      ),
      JSON.stringify(manifest.commandsRun),
    );
    const summaryMd = readFileSync(
      path.join(out, "audit-summary.latest.md"),
      "utf8",
    );
    assert(
      "T3s5. stdin pre-write path surfaces command result in the first-read summary",
      summaryMd.includes("## Command Result") &&
        summaryMd.includes("Pre-write ran and wrote an advisory"),
      summaryMd,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3c. pre-write child failure propagates exit code + manifest reason ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-stdin-bad-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-stdin-bad-out-"));
  try {
    buildFixture(fx);

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--intent",
        "-",
        "--profile",
        "quick",
      ],
      {
        encoding: "utf8",
        input: JSON.stringify({ names: "formatDate" }),
      },
    );

    assert(
      "T3f1. malformed stdin intent propagates non-zero exit",
      res.status === 1,
      `status=${res.status}, stderr=${res.stderr.slice(0, 500)}`,
    );

    const manifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    assert(
      "T3f2. failed stdin intent records preWrite ran=false",
      manifest.preWrite?.requested === true && manifest.preWrite?.ran === false,
      `preWrite=${JSON.stringify(manifest.preWrite)}`,
    );
    assert(
      "T3f3. failed stdin intent manifest names pre-write child failure",
      /pre-write\.mjs exited non-zero/.test(manifest.preWrite?.reason ?? ""),
      `reason=${manifest.preWrite?.reason}`,
    );
    assert(
      "T3f4. failed stdin intent does not create advisory",
      !existsSync(path.join(out, "pre-write-advisory.latest.json")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. Shell safety — --pre-write --intent under space+$ path ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), "au-pre-shell-"));
  const fx = path.join(parent, "my $fixture");
  const out = path.join(parent, "my $output");
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    const intent = {
      names: ["formatDate"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, "intent.json");
    writeFileSync(intentPath, JSON.stringify(intent));

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--intent",
        intentPath,
        "--profile",
        "quick",
      ],
      { encoding: "utf8" },
    );

    assert(
      "T4. space + $ path: audit-repo --pre-write exits 0",
      res.status === 0,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`,
    );
    assert(
      "T4b. advisory written under space+$ path",
      existsSync(path.join(out, "pre-write-advisory.latest.json")),
    );
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T5. scan-scope flags are forwarded into the pre-write child ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-scope-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-scope-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "au-pre-scope", type: "module" }),
    );
    write(fx, "src/prod.ts", "export const prodOnly = 1;\n");
    write(fx, "src/prod.test.ts", "export const testOnly = 1;\n");
    const intent = {
      names: ["prodOnly"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, "intent.json");
    writeFileSync(intentPath, JSON.stringify(intent));

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--intent",
        intentPath,
        "--profile",
        "quick",
        "--production",
      ],
      { encoding: "utf8" },
    );

    assert(
      "T5. audit-repo --pre-write --production exits 0",
      res.status === 0,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`,
    );

    const manifest = JSON.parse(
      readFileSync(path.join(out, "manifest.json"), "utf8"),
    );
    const advisory = JSON.parse(
      readFileSync(path.join(out, "pre-write-advisory.latest.json"), "utf8"),
    );
    const symbols = JSON.parse(
      readFileSync(path.join(out, "symbols.json"), "utf8"),
    );
    const defFiles = Object.keys(symbols.defIndex ?? {});

    assert(
      "T5b. orchestrator manifest records production scan range",
      manifest.scanRange?.includeTests === false &&
        manifest.scanRange?.production === true,
      JSON.stringify(manifest.scanRange),
    );
    assert(
      "T5c. pre-write advisory records the same production scan range",
      advisory.scanRange?.includeTests === false,
      JSON.stringify(advisory.scanRange),
    );
    assert(
      "T5d. pre-write child/cold-cache graph excludes test files",
      !defFiles.some((f) => f.includes(".test.")),
      JSON.stringify(defFiles),
    );
    assert(
      "T5e. production names-only pre-write still avoids base triage/topology",
      !existsSync(path.join(out, "triage.json")) &&
        !existsSync(path.join(out, "topology.json")),
      JSON.stringify(manifest.artifactsProduced),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. Rust pre-write advisory inventory includes Rust files ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "au-pre-rust-inventory-"));
  const out = mkdtempSync(path.join(tmpdir(), "au-pre-rust-inventory-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "au-pre-rust-inventory", type: "module" }),
    );
    write(fx, "src/lib.rs", "pub fn existing_rust() {}\n");
    write(fx, "src/app.ts", "export const existingTs = 1;\n");
    const analyzer = writeFakeRustAnalyzer(out);
    const intent = {
      language: "rust",
      files: ["src/lib.rs"],
      names: [],
      shapes: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, "intent.json");
    writeFileSync(intentPath, JSON.stringify(intent));

    const res = spawnSync(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fx,
        "--output",
        out,
        "--pre-write",
        "--pre-write-engine",
        "rust",
        "--intent",
        intentPath,
        "--profile",
        "quick",
      ],
      {
        encoding: "utf8",
        env: { ...process.env, LUMIN_RUST_ANALYZER_BIN: analyzer },
      },
    );

    assert(
      "T6. rust pre-write exits 0 with fake analyzer",
      res.status === 0,
      `status=${res.status}, stderr=${res.stderr.slice(0, 500)}`,
    );
    const advisory = JSON.parse(
      readFileSync(path.join(out, "pre-write-advisory.latest.json"), "utf8"),
    );
    assert(
      "T6b. rust pre-write inventory includes existing Rust file",
      advisory.preWrite?.fileInventory?.files?.includes("src/lib.rs"),
      JSON.stringify(advisory.preWrite?.fileInventory),
    );
    assert(
      "T6c. rust pre-write inventory keeps JS/TS files for mixed repo deltas",
      advisory.preWrite?.fileInventory?.files?.includes("src/app.ts"),
      JSON.stringify(advisory.preWrite?.fileInventory),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}
