import { it } from "vitest";
// Tests for `audit-repo.mjs --canon-draft` orchestrator — P3-4 Step 4.
//
// Pinning rules from docs/history/phases/p3/p3-4.md v2 §5.5:
//   - Fixture A (topology.json present) → 4 drafts emitted.
//   - Fixture B (topology.json absent) → 3 drafts + topology soft-fail exit 2.
//   - --sources <csv> scopes; unknown values exit 1.
//   - CANON_DRAFT_SOURCES single source-of-truth validation.
//   - Not in default profiles (source-grep pin).
//   - perSource requested-only.
//   - Coexistence with --pre-write / --post-write.
//   - Thin spawn wrapper (no source-logic imports in audit-repo.mjs).
//   - Shell safety.

import { execFileSync, spawnSync } from "node:child_process";
import {
  writeFileSync,
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
const AUDIT_CLI = path.join(DIR, "audit-repo.mjs");
const TOPO_CLI = path.join(DIR, "measure-topology.mjs");

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

function buildFixture(fx) {
  write(fx, "package.json", JSON.stringify({ name: "ar-fx", type: "module" }));
  write(fx, "_lib/util.mjs", `export function helper() { return 1 }\n`);
  write(
    fx,
    "src/app.mjs",
    `import { helper } from '../_lib/util.mjs';\n` +
      `export const x = helper();\n`,
  );
}

function readManifest(out) {
  return JSON.parse(readFileSync(path.join(out, "manifest.json"), "utf8"));
}

// ═══ Fixture A: topology.json present → 4 drafts ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-a-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-a-out-"));
  try {
    buildFixture(fx);
    // Pre-produce topology.json for the topology source.
    execFileSync(NODE, [TOPO_CLI, "--root", fx, "--output", out], {
      stdio: "ignore",
    });

    const res = spawnSync(
      NODE,
      [AUDIT_CLI, "--root", fx, "--output", out, "--canon-draft"],
      { encoding: "utf8" },
    );
    assert(
      "FA-1. exit 0 with topology.json present",
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );

    const manifest = readManifest(out);
    assert(
      "FA-2. manifest.canonDraft.requested === true",
      manifest.canonDraft?.requested === true,
    );
    assert(
      "FA-3. manifest.canonDraft.ran === true",
      manifest.canonDraft?.ran === true,
    );
    assert(
      "FA-4. perSource has all 4 keys",
      manifest.canonDraft?.perSource &&
        Object.keys(manifest.canonDraft.perSource).length === 4,
    );
    assert(
      "FA-5. all 4 sources ran + exitCode 0",
      ["type-ownership", "helper-registry", "topology", "naming"].every(
        (s) =>
          manifest.canonDraft.perSource[s]?.ran === true &&
          manifest.canonDraft.perSource[s]?.exitCode === 0,
      ),
    );
    assert(
      "FA-6. draftPaths has 4 entries",
      manifest.canonDraft?.draftPaths?.length === 4,
    );
    // Check drafts exist on disk.
    assert(
      "FA-7. naming.md exists",
      existsSync(path.join(fx, "canonical-draft", "naming.md")),
    );
    assert(
      "FA-8. topology.md exists",
      existsSync(path.join(fx, "canonical-draft", "topology.md")),
    );
    const summaryMd = readFileSync(
      path.join(out, "audit-summary.latest.md"),
      "utf8",
    );
    assert(
      "FA-9. first-read summary surfaces canon-draft command result",
      summaryMd.includes("## Command Result") &&
        summaryMd.includes("Canon draft wrote 4 proposal files"),
      summaryMd,
    );
    assert(
      "FA-10. console preview surfaces canon-draft result before JSON spelunking",
      res.stdout.includes("Command Result") &&
        res.stdout.includes("Canon draft wrote 4 proposal files"),
      res.stdout.slice(-1000),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ Fixture B: orchestrator's pipeline guarantees topology.json ═══
//
// SPEC nuance (p3-4 v2 §5.6): "Fixture B topology absent → 3 drafts" describes
// the STANDALONE CLI scenario. Through `audit-repo.mjs --canon-draft` the
// pipeline runs `measure-topology.mjs` unconditionally before canon-draft
// executes, so topology.json is guaranteed present. Fixture B is therefore
// NOT orchestrator-reachable in normal flow — its behavior is tested instead
// in `test-generate-canon-draft-cli-topology.mjs::T3a` (direct CLI exit 2).
//
// This block documents the guarantee as a positive test.

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-b-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-b-out-"));
  try {
    buildFixture(fx);
    // Caller does NOT pre-produce topology.json. Orchestrator's own pipeline
    // will produce it as part of its normal run.

    const res = spawnSync(
      NODE,
      [AUDIT_CLI, "--root", fx, "--output", out, "--canon-draft"],
      { encoding: "utf8" },
    );
    assert(
      "FB-1. exit 0 (advisory)",
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );

    const manifest = readManifest(out);
    assert(
      "FB-2. manifest.canonDraft.ran === true",
      manifest.canonDraft?.ran === true,
    );
    assert(
      "FB-3. topology source SUCCEEDED — pipeline produced topology.json before canon-draft",
      manifest.canonDraft?.perSource?.topology?.ran === true &&
        manifest.canonDraft?.perSource?.topology?.exitCode === 0,
      `topology=${JSON.stringify(manifest.canonDraft?.perSource?.topology)}`,
    );
    assert(
      "FB-4. all 4 sources succeeded",
      ["type-ownership", "helper-registry", "topology", "naming"].every(
        (s) => manifest.canonDraft?.perSource?.[s]?.ran === true,
      ),
    );
    assert(
      "FB-5. topology.json was produced by audit-repo pipeline",
      existsSync(path.join(out, "topology.json")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ --sources scoping ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-src-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-src-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--sources",
        "type-ownership,naming",
      ],
      { encoding: "utf8" },
    );
    assert(
      "S1. exit 0 with scoped --sources",
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );
    const manifest = readManifest(out);
    assert(
      "S2. requestedSources = [type-ownership, naming]",
      Array.isArray(manifest.canonDraft?.requestedSources) &&
        manifest.canonDraft.requestedSources.length === 2 &&
        manifest.canonDraft.requestedSources.includes("type-ownership") &&
        manifest.canonDraft.requestedSources.includes("naming"),
    );
    assert(
      "S3. perSource has EXACTLY those 2 keys (requested-only per P1-9)",
      Object.keys(manifest.canonDraft.perSource).length === 2 &&
        "type-ownership" in manifest.canonDraft.perSource &&
        "naming" in manifest.canonDraft.perSource,
    );
    assert(
      "S4. topology + helper-registry NOT in perSource (not requested)",
      !("topology" in manifest.canonDraft.perSource) &&
        !("helper-registry" in manifest.canonDraft.perSource),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ --sources all expands to the four named sources ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-source-alias-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-source-alias-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--source",
        "naming",
      ],
      { encoding: "utf8" },
    );
    assert(
      "SAL-1. --source alias scopes canon-draft",
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );
    const manifest = readManifest(out);
    assert(
      "SAL-2. requestedSources = [naming]",
      Array.isArray(manifest.canonDraft?.requestedSources) &&
        manifest.canonDraft.requestedSources.length === 1 &&
        manifest.canonDraft.requestedSources[0] === "naming",
      JSON.stringify(manifest.canonDraft?.requestedSources),
    );
    assert(
      "SAL-3. perSource has only naming",
      Object.keys(manifest.canonDraft?.perSource ?? {}).join(",") === "naming",
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-all-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-all-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--sources",
        "all",
      ],
      { encoding: "utf8" },
    );
    assert(
      "SA-1. --sources all → exit 0",
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );
    const manifest = readManifest(out);
    assert(
      "SA-2. requestedSources expanded to 4 named sources",
      Array.isArray(manifest.canonDraft?.requestedSources) &&
        manifest.canonDraft.requestedSources.length === 4 &&
        ["type-ownership", "helper-registry", "topology", "naming"].every((s) =>
          manifest.canonDraft.requestedSources.includes(s),
        ),
    );
    assert(
      "SA-3. perSource has all 4 entries",
      Object.keys(manifest.canonDraft?.perSource ?? {}).length === 4,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ --sources all mixed with a named source dedupes ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-all-dedupe-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-all-dedupe-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--sources",
        "all,naming",
      ],
      { encoding: "utf8" },
    );
    assert(
      'SD-1. --sources "all,naming" → exit 0',
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );
    const manifest = readManifest(out);
    assert(
      "SD-2. requestedSources deduped to 4",
      Array.isArray(manifest.canonDraft?.requestedSources) &&
        manifest.canonDraft.requestedSources.length === 4,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ Manifest draftPath records the actual versioned file emitted ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-versioned-manifest-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-versioned-manifest-out-"));
  try {
    buildFixture(fx);
    write(fx, "canonical-draft/type-ownership.md", "# existing draft\n");
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--sources",
        "type-ownership",
      ],
      { encoding: "utf8" },
    );
    assert(
      "SV-1. versioned draft run exits 0",
      res.status === 0,
      `stderr=${res.stderr.slice(0, 400)}`,
    );
    const manifest = readManifest(out);
    const draftPath =
      manifest.canonDraft?.perSource?.["type-ownership"]?.draftPath;
    assert(
      "SV-2. manifest draftPath points at type-ownership.v2.md",
      typeof draftPath === "string" &&
        path.basename(draftPath) === "type-ownership.v2.md",
      `draftPath=${draftPath}`,
    );
    assert(
      "SV-3. manifest draftPath exists on disk",
      typeof draftPath === "string" && existsSync(draftPath),
    );
    assert(
      "SV-4. draftPaths uses the same versioned path",
      Array.isArray(manifest.canonDraft?.draftPaths) &&
        manifest.canonDraft.draftPaths.length === 1 &&
        manifest.canonDraft.draftPaths[0] === draftPath,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ --sources with unknown value → exit 1 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-unk-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-unk-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--sources",
        "type-ownership,foobar,naming",
      ],
      { encoding: "utf8" },
    );
    assert(
      "U1. unknown --sources value → exit 1",
      res.status === 1,
      `got=${res.status}`,
    );
    const manifest = readManifest(out);
    assert(
      "U2. manifest.canonDraft.ran === false with reason",
      manifest.canonDraft?.ran === false &&
        /unknown --sources/.test(manifest.canonDraft?.reason ?? ""),
    );
    assert(
      "U3. No partial perSource keys (validation before execution)",
      !manifest.canonDraft?.perSource ||
        Object.keys(manifest.canonDraft.perSource).length === 0,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ --sources topology alone through orchestrator — pipeline still produces topology.json ═══
//
// Same nuance as Fixture B: audit-repo's pipeline runs unconditionally.
// "All requested sources failed" path is tested directly via
// test-generate-canon-draft-cli-topology.mjs (standalone CLI exit 2) — not
// reachable through orchestrator in v1 because the pipeline always produces
// topology.json first.
//
// This block documents that `--sources topology` via orchestrator SUCCEEDS
// because the prerequisite is guaranteed.

{
  const fx = mkdtempSync(path.join(tmpdir(), "arcd-tonly-"));
  const out = mkdtempSync(path.join(tmpdir(), "arcd-tonly-out-"));
  try {
    buildFixture(fx);
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--sources",
        "topology",
      ],
      { encoding: "utf8" },
    );
    assert(
      "T1. --sources topology through orchestrator → exit 0 (pipeline guaranteed prerequisite)",
      res.status === 0,
      `got=${res.status}; stderr=${res.stderr.slice(0, 400)}`,
    );
    const manifest = readManifest(out);
    assert(
      "T2. ran=true with perSource = {topology: {ran:true, exitCode:0}}",
      manifest.canonDraft?.ran === true &&
        manifest.canonDraft?.perSource?.topology?.ran === true &&
        Object.keys(manifest.canonDraft.perSource).length === 1,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ Public orchestrator forwards --canon-output ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), "arcd-canon-output-"));
  const fx = path.join(parent, "root");
  const out = path.join(parent, "out");
  const canonOut = path.join(parent, "drafts");
  try {
    buildFixture(fx);
    mkdirSync(out, { recursive: true });
    mkdirSync(canonOut, { recursive: true });
    const res = spawnSync(
      NODE,
      [
        AUDIT_CLI,
        "--root",
        fx,
        "--output",
        out,
        "--canon-draft",
        "--source",
        "naming",
        "--canon-output",
        canonOut,
      ],
      { encoding: "utf8" },
    );
    assert(
      "CO-1. --canon-output through orchestrator → exit 0",
      res.status === 0,
      `got=${res.status}; stderr=${res.stderr.slice(0, 400)}`,
    );
    assert(
      "CO-2. custom canon-output receives naming draft",
      existsSync(path.join(canonOut, "naming.md")),
    );
    assert(
      "CO-3. custom canon-output avoids default root/canonical-draft write",
      !existsSync(path.join(fx, "canonical-draft", "naming.md")),
    );
    const manifest = readManifest(out);
    assert(
      "CO-4. manifest draftPath points at custom canon-output",
      manifest.canonDraft?.draftPaths?.length === 1 &&
        path.resolve(manifest.canonDraft.draftPaths[0]) ===
          path.join(canonOut, "naming.md"),
    );
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ Default profiles DO NOT enable --canon-draft ═══

{
  // Source-grep: the profile block in audit-repo.mjs must not mention canon-draft.
  const src = readFileSync(path.join(DIR, "audit-repo.mjs"), "utf8");
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Profile-list line: validates quick|full|ci.
  const profileSectionRe = /\[.quick.,\s*.full.,\s*.ci.\]/;
  assert(
    "P1. PROFILE validation keeps quick|full|ci only (no canon-draft profile)",
    profileSectionRe.test(stripped),
  );

  // --canon-draft must never appear as DEFAULT for profile.
  const suspect =
    /canon-?draft.*['"]quick['"]|canon-?draft.*['"]full['"]|canon-?draft.*['"]ci['"]/;
  assert(
    "P2. no profile-default includes --canon-draft",
    !suspect.test(stripped),
  );
}

// ═══ Thin-wrapper source-grep: audit-repo.mjs does NOT import source-logic functions ═══

{
  const src = readFileSync(path.join(DIR, "audit-repo.mjs"), "utf8");
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // Thin wrapper means NO direct imports of classifier / aggregator / renderer
  // functions from _lib/canon-draft.mjs. Only CANON_DRAFT_SOURCES constant is
  // OK (validation helper). Everything else must be subprocess.
  const forbidden = [
    /\bcollectTypeIdentities\b/,
    /\bcollectHelperIdentities\b/,
    /\bcollectTopologyStructure\b/,
    /\bcollectNamingCohorts\b/,
    /\bclassifyTypeNameGroup\b/,
    /\bclassifyHelperIdentity\b/,
    /\bclassifyTopologySubmodule\b/,
    /\bclassifyNamingCohort\b/,
    /\brenderTypeOwnership\b/,
    /\brenderHelperRegistry\b/,
    /\brenderTopology\b/,
    /\brenderNaming\b/,
  ];
  const flagged = forbidden.filter((re) => re.test(stripped));
  assert(
    "W1. audit-repo.mjs does NOT import source-logic (thin wrapper)",
    flagged.length === 0,
    `flagged: ${flagged.map((r) => r.toString()).join(", ")}`,
  );
}

// ═══ CANON_DRAFT_SOURCES single source-of-truth — helper imports, audit-repo delegates ═══

{
  const src = readFileSync(path.join(DIR, "audit-repo.mjs"), "utf8");
  const helper = readFileSync(
    path.join(DIR, "_lib", "audit-canon-draft.mjs"),
    "utf8",
  );
  const stripped = src
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");
  const strippedHelper = helper
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n]*/g, "");

  // The lifecycle helper must import from _lib/canon-draft-utils.mjs.
  const helperHasImport =
    /CANON_DRAFT_SOURCES/.test(strippedHelper) &&
    /canon-draft-utils\.mjs/.test(strippedHelper);
  assert(
    "C1. audit-canon-draft.mjs references CANON_DRAFT_SOURCES + imports from canon-draft-utils.mjs",
    helperHasImport,
  );

  assert(
    "C1b. audit-repo.mjs delegates canon-draft lifecycle to Rust audit-core",
    /executeCanonDraftLifecycle/.test(stripped) &&
      !/runCanonDraftLifecycle/.test(stripped) &&
      !/audit-canon-draft\.mjs/.test(stripped),
  );

  // Must NOT locally define the 4-element source array.
  const suspect = /CANON_DRAFT_SOURCES\s*=\s*\[/;
  assert(
    "C2. audit-repo.mjs and helper do NOT locally declare CANON_DRAFT_SOURCES",
    !suspect.test(stripped) && !suspect.test(strippedHelper),
  );
}

// ═══ Shell safety ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), "arcd-shell-"));
  const fx = path.join(parent, "my $root");
  const out = path.join(parent, "my $out");
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    execFileSync(NODE, [TOPO_CLI, "--root", fx, "--output", out], {
      stdio: "ignore",
    });
    const res = spawnSync(
      NODE,
      [AUDIT_CLI, "--root", fx, "--output", out, "--canon-draft"],
      { encoding: "utf8" },
    );
    assert(
      "SH1. path with spaces + $ survives orchestrator",
      res.status === 0 &&
        existsSync(path.join(fx, "canonical-draft", "naming.md")),
      `stderr=${res.stderr.slice(0, 400)}`,
    );
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}
