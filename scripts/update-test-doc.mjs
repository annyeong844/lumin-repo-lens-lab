#!/usr/bin/env node
// Generate tests/README.md deterministically from CHANGELOG.md + the
// actual test file inventory.
//
// Motivation: four consecutive releases (1.8.2 → 1.8.5) drifted the
// README number or per-release retrospective section. Each time the
// reviewer caught it. Continuing to hand-edit that file is the wrong
// fix. Instead: the README is now a generated artifact, the generator
// is deterministic, and `npm run check:test-doc` fails CI if the file
// and the generator output diverge.
//
// Design choices:
//   - Top of file intentionally does NOT hardcode an assertion count.
//     `npm test` output is authoritative. (Reviewer suggestion from
//     the 1.8.3 review, finally taken.)
//   - Per-release retrospective is extracted from CHANGELOG subject
//     lines only. Assertion counts are not parsed from CHANGELOG at
//     all (dead since 1.9.1, parse removed in 1.9.3). `npm test`
//     is the only authoritative current count. Historical prose
//     copied from subjects may incidentally contain counts — that's
//     factual record from the release description, not drift.
//   - Suite list is enumerated from the actual `tests/test-*.mjs`
//     files on disk. New tests appear automatically next release.
//
// Modes:
//   update-test-doc.mjs              → write tests/README.md
//   update-test-doc.mjs --check      → exit 1 if generated != on-disk

import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const README = path.join(ROOT, "tests/README.md");
const CHANGELOG = path.join(ROOT, "CHANGELOG.md");

const args = process.argv.slice(2);
const CHECK_MODE = args.includes("--check");

function normalizeLineEndings(text) {
  return text.replace(/\r\n/g, "\n");
}

// ─── Parse CHANGELOG ──────────────────────────────────────
function parseChangelog() {
  const src = readFileSync(CHANGELOG, "utf8");
  // Split on `## X.Y.Z — DATE` headings.
  const sections = src.split(/^## (\d+\.\d+\.\d+) /m);
  // sections[0] is the pre-header preamble; then [version, body, version, body, ...]
  const entries = [];
  for (let i = 1; i < sections.length; i += 2) {
    const version = sections[i];
    const body = sections[i + 1] ?? "";
    // First non-blank paragraph after the date line is the subject.
    const lines = body.split("\n");
    let subject = "";
    const startIdx =
      lines[0]?.startsWith("— ") || lines[0]?.includes("—") ? 1 : 0;
    for (let j = startIdx; j < lines.length; j++) {
      const L = lines[j].trim();
      if (!L) {
        if (subject) break;
        continue;
      }
      if (L.startsWith("#")) break;
      subject = subject ? `${subject} ${L}` : L;
    }
    // v1.9.3: previously parsed `### Tests (N total)` into entries
    // but never rendered them. Dead since the 1.9.1 render-path
    // change. Removed to keep the data shape honest — README gets
    // what it shows.
    entries.push({ version, subject });
  }
  return entries;
}

// ─── Enumerate suites on disk ─────────────────────────────
function listSuites() {
  return readdirSync(path.join(ROOT, "tests"))
    .filter((f) => f.startsWith("test-") && f.endsWith(".mjs"))
    .sort();
}

// Short descriptions keyed by filename. Keeping this as a small local
// dictionary rather than parsing suite headers keeps the output stable;
// new suites show up without a description and the release author is
// prompted to add one.
const SUITE_DESCRIPTIONS = {
  "test-alias.mjs": "export alias misclassification",
  "test-audit-manifest-export-surface.mjs":
    "audit-manifest export surface hides living-audit internals and mirrors review-only evidence summaries",
  "test-audit-repo-pre-write.mjs":
    "audit-repo --pre-write lifecycle wiring and missing-baseline evidence availability",
  "test-audit-repo-symbol-incremental.mjs":
    "audit-repo forwards strict incremental flags to build-symbol-graph",
  "test-audit-repo.mjs": "orchestrator profiles + blindZones detection",
  "test-behavior-corpus-verifier.mjs":
    "saved-answer behavior verifier: offline no-jargon, caveat, and summary-order checks",
  "test-checklist-facts.mjs": "checklist-facts.mjs pre-compute layer (v1.10.2)",
  "test-cjs-classification.mjs":
    "PCEF P0 CJS consumer classification through symbol graph + dead-export pipeline",
  "test-cjs-integration.mjs":
    "CJS export surface, alias destructuring, and dynamic require opacity integration guard",
  "test-cjs-export-surface-artifact.mjs":
    "CJS export surface facts survive into symbols.json for downstream blind-zone handling",
  "test-class-method-prewrite-surface.mjs":
    "WT-15 class method index remains separate from defIndex and feeds pre-write review cues",
  "test-class-method-index-prototype-names.mjs":
    "class method index handles prototype method names as plain dictionary keys",
  "test-call-graph-bounded.mjs":
    "PCEF P3 bounded member-call resolution for exported object member calls",
  "test-call-graph-parse-errors.mjs":
    "call-graph artifact marks parse-error scans incomplete with file-level diagnostics",
  "test-call-graph-truncation-defense.mjs":
    "PCEF P3 call graph full fan-in maps remain complete beyond topCallees display truncation",
  "test-calibration-corpora.mjs":
    "calibration corpus registry anchors threshold policy corpus references",
  "test-classify-facts-ast.mjs": "AST identifier ref counting (v1.10.0 P0)",
  "test-classify-policies-export-surface.mjs":
    "classify-policies export surface stays limited to active policy APIs",
  "test-classify-performance-metadata.mjs":
    "classify performance metadata + safe text-zero shortcut",
  "test-cli.mjs": "CLI flag parsing",
  "test-collect.mjs": "collectFiles language filter",
  "test-corpus.mjs": "precision corpus + FP budget gate (v1.10.0 P2)",
  "test-dynamic-import.mjs": "topology dynamic imports",
  "test-entry-surface-artifact.mjs":
    "PCEF P2b entry-surface artifact and audit pipeline hook",
  "test-evidence-honesty.mjs": "compare-repos + doc-script-refs guards",
  "test-export-action-safety.mjs":
    "PCEF P1 export-action-safety producer: demote/delete proof, blockers, and module marker patch",
  "test-definition-id-canonical.mjs":
    "PCEF P3 canonical definitionId shared by symbols, action-safety, and call-graph alias fan-in",
  "test-definition-id-export.mjs":
    "definition-id export surface hides raw id builder",
  "test-extract-cjs-consumer.mjs":
    "PCEF P0 direct CJS require consumer extraction: exact, side-effect-only, and broad escape",
  "test-extract-cjs-export-surface.mjs":
    "Direct CJS export surface extraction: exact exports plus opaque export forms",
  "test-finding-local-provenance.mjs":
    "per-finding taintedBy/supportedBy (v1.10.0 P1)",
  "test-framework-policy-facts.mjs":
    "framework policy fact extraction for config/framework sentinel evidence",
  "test-framework-policy-matrix.mjs":
    "framework policy matrix contract for config and framework sentinel muting",
  "test-framework-resource-surfaces.mjs":
    "framework/resource surface classifier lanes for stories, Strapi paths, generated declarations, bundles, templates, and codemod resources",
  "test-hardcoding.mjs": "workspace labels, focus-class",
  "test-hash-imports.mjs":
    "Node `#imports` subpath — exact, wildcard, and suffix wildcard alias resolution",
  "test-incremental-cache-store.mjs":
    "strict incremental cache store schema, current-hash reuse, malformed-cache fallback",
  "test-incremental-snapshot.mjs":
    "strict incremental repo snapshot identity, content hashes, unreadable file visibility",
  "test-import-meta-glob-diagnostics.mjs":
    "import.meta.glob unsupported dynamic-module diagnostics",
  "test-js-module-edge-scanner.mjs":
    "tokenizer-state JS module edge scanner shadow/equivalence fixtures",
  "test-jsonc-edge-cases.mjs":
    "JSONC tsconfig parser edge cases: schema URLs, comments, trailing commas, string comment markers, BOM, and unresolved extends",
  "test-lang-matrix.mjs": "per-extension parser dispatch",
  "test-maintainer-scripts.mjs":
    "maintainer script hardening: child process spawn errors and optional public package reads",
  "test-mode-dispatch.mjs":
    "mode dispatch contract: write triggers, non-trigger reasons, repo context, prose rewrites, comment typo fixes, and inspection guards",
  "test-module-reachability.mjs":
    "PCEF P2c module-reachability artifact: runtime/type BFS, bounded-out cap, and audit pipeline hook",
  "test-namespace-reexport-deadness.mjs":
    "namespace re-export member fan-in: exact/chained used members stay live, unused members remain dead, opaque escapes are diagnosed",
  "test-node-imports-unsupported.mjs":
    "Node #imports unsupported-family diagnostics: no external fallback, no graph edge, dedicated unsupported lane",
  "test-output-source-layout-diagnostics.mjs":
    "package exports output-to-source layout unsupported diagnostics: no fake edge, dedicated family, candidate-scoped blind zone",
  "test-python-conventions.mjs": "Python __all__, decorators, dunders",
  "test-rank-fixes.mjs": "4-tier fix-plan ranking predicates + merge",
  "test-resolved-edges.mjs":
    "PCEF P2a resolvedInternalEdges file-level graph artifact",
  "test-resolver-diagnostics-artifacts.mjs":
    "resolver capabilities and per-run diagnostics artifact contract",
  "test-resolver-blind-zone-relevance.mjs":
    "resolver blind-zone relevance scoping for per-finding SAFE_FIX taint",
  "test-resolver-paths.mjs": "resolver edge cases (FP-16 etc.)",
  "test-sarif-fix-plan.mjs": "emit-sarif fix-plan branch: tier → SARIF level",
  "test-shell-safety.mjs": "shell injection + triage refactor",
  "test-shape-index-incremental.mjs":
    "strict incremental build-shape-index cold/warm equivalence + changed/deleted file behavior",
  "test-skill-surface.mjs":
    "product surface contract: shared audit engine + 3 skill surfaces + stable validation modes + internal-vs-public doc split",
  "test-skill-package.mjs":
    "deployable skill package builder: plugin wrapper, 5 public scripts, slash commands, _engine internals, canonical/templates/references, no lab payload",
  "test-symbol-graph-incremental.mjs":
    "strict incremental build-symbol-graph cold/warm equivalence + changed/deleted file behavior",
  "test-plugin-package.mjs":
    "Claude Code plugin-root package builder: plugin metadata, slash commands, generated skill surfaces, Codex wrapper opt-in",
  "test-publish-public-plugin.mjs":
    "public plugin repo publisher: generated package allowlist, changelog prepend, dry-run, and push flow",
  "test-public-deep-import-risk.mjs":
    "PCEF P2 public package exports risk gate for entry-unreachable confidence support",
  "test-refactor-plan-verifier.mjs":
    "refactor-plan output verifier: SHORT/FULL shape, tone guard, evidence anchor, pre-write handoff",
  "test-run-tests-grouped.mjs":
    "grouped Node test runner: deterministic groups, bounded jobs, compact logs, and replay commands",
  "test-smoke-uncovered.mjs": "scripts without dedicated suites",
  "test-symlink-aliasing.mjs": "symlink canonicalization",
  "test-temp-repo-fixture-helper.mjs":
    "shared temporary repo fixture helper safety contract",
  "test-tsconfig-paths-scoped.mjs":
    "FP-36: scope-aware tsconfig paths in monorepos",
  "test-topology-mermaid.mjs":
    "topology.mermaid.md renderer contract: diagrams, hub files, caps, and citation guardrails",
  "test-threshold-policies.mjs":
    "threshold policy metadata: policy ids, versions, hashes, and compact artifact summaries",
  "test-threshold-policy-drift-guard.mjs":
    "threshold policy numeric drift requires an explicit snapshot review",
  "test-type-only-reexport.mjs": "type-only re-export runtime-lens filter",
  "test-unused-deps-producer.mjs":
    "unused-deps.json producer: review-only dependency hygiene, package script tools, and audit artifact visibility",
  "test-update-test-doc.mjs": "tests/README.md generator drift guard",
  "test-vocab.mjs":
    "locks _lib/vocab.mjs constant values + forwarder (v1.10.1)",
  "test-workspace-no-exports.mjs":
    "FP-38: workspace packages without `exports` field",
  "test-wildcard.mjs": "exports wildcard subpath",
  // ─── P2-0 (any-inventory producer + pre-write snapshot hook) ───
  "test-canonical-fact-model-drift.mjs":
    "canonical §3.9 escapeKind drift guard (P2-0)",
  "test-citation-verifier.mjs":
    "Rule 1 grounded citation verifier: artifact path/value checks for saved model output",
  "test-classification-label-emission-corpus.mjs":
    "synthetic TS corpus proving canonical type classification labels emit through build-symbol-graph → canon-draft",
  "test-extract-ts-escapes.mjs":
    "11 escapeKind extractor + occurrenceKey stability (P2-0)",
  "test-any-inventory.mjs": "any-inventory.json producer + meta shape (P2-0)",
  "test-any-inventory-incremental.mjs":
    "strict incremental any-inventory cold/warm equivalence + changed/deleted file behavior",
  "test-pre-write-advisory-artifact.mjs":
    "pre-write advisory artifact shape, lifecycle metadata, and evidence availability contract",
  "test-pre-write-bootstrap.mjs":
    "pre-write first-run bootstrap keeps missing baseline evidence explicitly unavailable",
  "test-pre-write-canonical-parser.mjs":
    "pre-write canonical parser keeps owner claims deterministic before lookup rendering",
  "test-pre-write-cli.mjs":
    "pre-write CLI intent parsing, baseline evidence routing, and advisory output contract",
  "test-pre-write-drift.mjs":
    "pre-write canonical/AST drift states stay structured and scoped",
  "test-pre-write-integration.mjs":
    "pre-write end-to-end lookup, evidence availability, and advisory rendering integration",
  "test-pre-write-intent.mjs":
    "pre-write intent parser extracts names, files, shapes, and refactor sources without overclaiming",
  "test-pre-write-inventory-hook.mjs":
    "pre-write P2-0 snapshot hook (preWrite.anyInventoryPath)",
  "test-pre-write-cue-tiers.mjs":
    "pre-write cue tier artifact contract and weak-token suppression classification",
  "test-pre-write-inline-patterns.mjs":
    "pre-write inline extraction cues from explicit refactorSources and inline-patterns.json",
  "test-pre-write-lookup-dep.mjs":
    "pre-write dependency lookup distinguishes observed package evidence from unavailable scan evidence",
  "test-pre-write-lookup-file.mjs":
    "pre-write file lookup surfaces exact, near, missing, and evidence-unavailable targets",
  "test-pre-write-lookup-name.mjs":
    "pre-write name lookup exact identities, suppressed diagnostics, and service-operation sibling policy evidence",
  "test-pre-write-local-operation-index.mjs":
    "pre-write nested local operation index stays review-only and out of export lookup lanes",
  "test-pre-write-lookup-shape.mjs":
    "P4 pre-write shape lookup: exact hash/typeLiteral, schema validation, no heuristic fallback",
  "test-pre-write-render.mjs":
    "pre-write Markdown renderer keeps advisory evidence review-only and avoids stronger action wording",
  // ─── P6-0 (measurement harness + readiness gates) ───
  "test-p6-measurement.mjs":
    "P6-0 measurement artifact contract: candidate counts, FP denominator, schema round-trip, dirty corpus, readiness gates",
  "test-p6-member-precision.mjs":
    "P6-3 namespace and dynamic import member precision: direct member calls protect only the called export; degraded aliases stay conservative",
  "test-p6-safe-fix-calibration.mjs":
    "P6 SAFE_FIX calibration corpus: real mini git repo + runtime/staleness convergence + P6 measurement",
  "test-public-surface.mjs":
    "P6-1 package/public surface collector: root workspace package entries, declaration targets, script-driven tsup/rollup/esbuild entrypoints, HTML module entrypoints",
  "test-mdx-consumers.mjs":
    "P6-1 MDX import consumers: docs-driven component imports contribute symbol fan-in without file-level overprotection",
  "test-sfc-consumers.mjs":
    "SFC consumers: script imports, script-src reachability, style assets, template refs, and global registration evidence stay in separate lanes",
  // --- P4-1 (shape-hash pure core) ---
  "test-shape-hash.mjs":
    "P4-1 shape-hash pure core: field normalization, stable hashes, unsupported-shape diagnostics",
  "test-build-shape-index.mjs":
    "P4-2 build-shape-index.mjs producer: shape-index artifact, grouping, diagnostics, scan scope",
  "test-build-block-clone-index.mjs":
    "build-block-clone-index.mjs producer: repeated token/block clone review-only artifact",
  "test-build-function-clone-index.mjs":
    "build-function-clone-index.mjs producer: exported helper/function clone cue artifact",
  "test-build-framework-resource-surfaces.mjs":
    "build-framework-resource-surfaces.mjs producer and audit-repo artifact visibility",
  "test-inline-pattern-index.mjs":
    "build-inline-pattern-index.mjs producer: repeated inline catch-block review cue artifact",
  "test-function-clone-export-surface.mjs":
    "function-clone artifact export surface hides version internals",
  "test-function-clone-incremental.mjs":
    "strict incremental build-function-clone-index cold/warm equivalence + changed/deleted file behavior",
  "test-function-clone-audit-forwarding.mjs":
    "audit-repo incremental flag forwarding for function clone producer",
  "test-pre-write-shape-index.mjs":
    "P4-3 pre-write shape lookup consumes shape-index.json by exact hash",
  // ─── P2-1 (post-write delta engine) ───
  "test-post-write-delta.mjs":
    "computeDelta: 6-label classification + purity (P2-1)",
  "test-file-delta-export.mjs":
    "post-write file-delta export surface hides path normalizer internals",
  "test-post-write-render.mjs": "post-write Markdown + JSON render (P2-1)",
  "test-post-write-artifact.mjs": "post-write-delta dual-write + atomic (P2-1)",
  "test-post-write-cli.mjs":
    "post-write.mjs CLI smoke + scan-range flag forwarding (P2-1)",
  "test-post-write-incremental.mjs":
    "post-write after-snapshot incremental forwarding + immutable pre-write baseline",
  // ─── P2-2 (audit-repo orchestrator integration + release-blocking integration test) ───
  "test-audit-repo-post-write.mjs":
    "audit-repo --post-write wiring + manifest summary fields (P2-2)",
  "test-post-write-integration.mjs":
    "release-blocking end-to-end: multi-label + baseline-missing fixtures (P2-2)",
  // ─── E-6 incremental cache + stat-first-cut ───
  "test-incremental.mjs": "file-hash cache + stat-first-cut fast path (E-6)",
  // ─── P3-1 (canon draft generator: type-ownership source) + P3-2 (helper-registry source) ───
  "test-classification-gates.mjs":
    "canonical §3/§9/§10.3/§10.4/§11.4/§12.3 label-set + LOW_INFO + TOPOLOGY + NAMING mirrors drift-lock (P3-1..P3-4) + canon-drift.md §3 category/family mirror (P5-0)",
  "test-canon-draft.mjs":
    "type classifier rules (group + single-identity) + markdown helpers (P3-1)",
  "test-canon-draft-type-ownership.mjs":
    "identity aggregation + renderer scenarios (P3-1)",
  "test-generate-canon-draft-cli.mjs":
    "generate-canon-draft.mjs CLI flags + versioning + scope (P3-1)",
  "test-github-actions-ci-policy.mjs":
    "GitHub Actions CI policy guard: draft PRs skip runner jobs while ready/manual/push still run",
  "test-hook-ack-observer.mjs": "auto-hook Phase 1E Stop ACK observer core",
  "test-hook-event-drain-renderer.mjs":
    "auto-hook Phase 1D event drainer and reminder renderer core",
  "test-hook-event-store.mjs": "auto-hook Phase 1C session event store core",
  "test-hook-doctor.mjs":
    "auto-hook Phase 1A hook manifest and doctor smoke test",
  "test-hook-id-safety.mjs":
    "auto-hook Phase 1A session/tool id safety helpers",
  "test-hook-path-safety.mjs": "auto-hook Phase 1A path/root safety helpers",
  "test-hook-post-write-lite.mjs":
    "auto-hook Phase 1F post-write-lite silent-new event generation core",
  "test-hook-preimage-store.mjs": "auto-hook Phase 1B session preimage store",
  "test-hook-runner-scripts.mjs":
    "auto-hook Phase 1G hook runner scripts and manifest activation",
  "test-canon-draft-integration.mjs":
    "end-to-end symbols→canon-draft via fixture repos (P3-1)",
  "test-canon-draft-helpers.mjs":
    "helper classifier rules (group + single-identity) + precedence pins (P3-2)",
  "test-canon-draft-helper-registry.mjs":
    "helper aggregator + renderer via DI; PF-3/PF-4 fan-in pins (P3-2)",
  "test-generate-canon-draft-cli-helpers.mjs":
    "CLI --source helper-registry + versioning + stale call-graph (P3-2)",
  "test-canon-draft-integration-helpers.mjs":
    "end-to-end helper-registry via real extractor + resolver (P3-2)",
  // ─── P3-3 (canon draft generator: topology source) ───
  "test-topology-producer-cross-edges.mjs":
    "measure-topology.mjs crossSubmoduleEdges full-list producer shape pin (P3-3-pre)",
  "test-canon-draft-topology.mjs":
    "topology classifier rules (§11.1 submodule + §11.2 SCC + §11.3 oversize) (P3-3)",
  "test-canon-draft-topology-structure.mjs":
    "topology aggregator + renderer via DI; §5.3.1 inventory source order + degraded-mode guard (P3-3)",
  "test-generate-canon-draft-cli-topology.mjs":
    "CLI --source topology + exit 2 on missing topology.json (P3-3)",
  "test-canon-draft-integration-topology.mjs":
    "end-to-end topology draft via measure-topology + triage-repo + canon CLI (P3-3)",
  // ─── P3-4 (canon draft generator: naming source + audit-repo orchestrator) ───
  "test-canon-draft-naming.mjs":
    "naming classifier + detectConvention + normalizeFileBasename + low-info Rule 0 (P3-4)",
  "test-canon-draft-naming-structure.mjs":
    "naming aggregator + renderer via DI; cohort inventory always from collectFiles (P3-4)",
  "test-generate-canon-draft-cli-naming.mjs":
    "CLI --source naming + versioning + scope + regression (P3-4)",
  "test-audit-repo-canon-draft.mjs":
    "orchestrator --canon-draft + --sources CANON_DRAFT_SOURCES + thin-wrapper pin (P3-4)",
  // ─── P5-0 (canon-drift canonical + parser contract round-trip) ───
  "test-canon-drift-parser-contract.mjs":
    "round-trip: each P3 renderer header matches canon-drift.md §5 column contract (P5-0)",
  // ─── P5 drift detector suite (P5-1 types / P5-2 helpers / P5-3 topology / P5-4 naming + orchestrator) ───
  "test-check-canon-utils.mjs":
    "PURE parser (3-tier strictness) for all 4 sources + multi-section topology + multi-section naming + makeDriftRecord + buildCanonDriftJsonObject + 4 LABEL_SETs (P5-1..P5-4)",
  "test-check-canon-artifact.mjs":
    "I/O layer — load{TypeOwnership,HelperRegistry,Topology,Naming}Canon + writeCanonDriftArtifacts no-merge policy (P5-1..P5-4)",
  "test-check-canon-types.mjs":
    "type-ownership drift engine + 1:1 owner-change upgrade + label-preserving renderer (P5-1)",
  "test-check-canon-helpers.mjs":
    "helper-registry drift engine + evidence-gated contamination dispatch (per-identity) + fan-in-tier-changed + extractor-throw → parse-error promotion (P5-2)",
  "test-check-canon-topology.mjs":
    "topology drift engine + 3 sub-diffs (submodules/oversize/cross-edges) + §1/§3 SCC agreement + top-30 sort-before-slice (P5-3)",
  "test-check-canon-naming.mjs":
    "naming drift engine + cohort + outlier sub-diffs + PF-4 identity format per category (file/symbol cohort / file/symbol outlier) (P5-4)",
  "test-generate-check-canon-cli.mjs":
    "check-canon.mjs CLI flags + exit matrix + per-source artifact strictness asymmetry + --source all aggregation by checked-source rule (P5-1..P5-4)",
  "test-generated-artifact-evidence.mjs":
    "generated artifact evidence policy: strong build/static output quorum plus supporting path-segment hints",
  "test-generated-blind-zone-relevance.mjs":
    "generated artifact blind-zone relevance scoping for SAFE_FIX taint",
  "test-generated-consumer-blind-zones.mjs":
    "generated consumer blind-zone inventory in symbols.json for missing or excluded generated surfaces",
  "test-generated-virtual-surface.mjs":
    "generated virtual surface contract for Prisma enum imports without generator execution",
  "test-audit-repo-check-canon.mjs":
    "audit-repo.mjs --check-canon orchestrator + manifest.checkCanon shape + advisory vs --strict-check-canon + child exit 1/2 are per-source outcomes (P5-4)",
  "test-check-canon-integration.mjs":
    "end-to-end drift via 5 type + 6 helper + 7 topology fixtures; canonical bytes immutable; stale canonical-draft ignored (P5-1..P5-3)",
};

// ─── Render ───────────────────────────────────────────────
function render() {
  const entries = parseChangelog();
  const suites = listSuites();
  const missingDescriptions = suites.filter((s) => !SUITE_DESCRIPTIONS[s]);

  const suiteWidth = Math.max(...suites.map((s) => s.length));
  const suiteLines = suites.map((s) => {
    const desc =
      SUITE_DESCRIPTIONS[s] ??
      "(no description — add one to scripts/update-test-doc.mjs)";
    return `node tests/${s.padEnd(suiteWidth)} # ${desc}`;
  });

  const releaseBullets = entries
    .filter((e) => e.subject) // skip sections without a clear subject
    .map((e) => {
      // v1.9.1: counts REMOVED from bullets. Previously rendered as
      // `**v1.9.0** (171): ...`. Reviewer caught the hole — those
      // numbers came from CHANGELOG's `### Tests (N total` line, which
      // we can't verify, so a wrong CHANGELOG count would propagate
      // into README without CI catching it. The 1.9.0 release claimed
      // "mechanically impossible to ship drift" but this was a
      // remaining vector. Now: bullets carry version + subject only;
      // `npm test` is the only source of truth for counts.
      return `- **v${e.version}**: ${e.subject}`;
    });

  return [
    "<!--",
    "  GENERATED FILE — do not edit by hand.",
    "  Source: CHANGELOG.md + tests/test-*.mjs files.",
    "  Regenerate with: npm run update-test-doc",
    "  CI guard: npm run check:test-doc (exits non-zero if stale)",
    "-->",
    "",
    "# Tests",
    "",
    "Regression guards built up across releases. Each change that could",
    "have broken a correctness property got a corresponding assertion so",
    "the next regression fails fast.",
    "",
    "The authoritative assertion count is the output of `npm test`. This",
    "README intentionally avoids hardcoding a total — four consecutive",
    "releases (1.8.2 → 1.8.5) drifted the number in this file, so the",
    "number was removed and the file became generated.",
    "",
    "## Run",
    "",
    "```bash",
    "cd <skill-dir>",
    "npm install        # first run only",
    "npm test           # all suites, stops at first failing assertion",
    "```",
    "",
    "## Suite Map",
    "",
    "- **Smoke:** start with `test-skill-package.mjs` and",
    "  `test-skill-surface.mjs` when packaging or prompt surface changes.",
    "- **Contract:** suites named `test-pre-write-*`, `test-post-write-*`,",
    "  `test-canon-*`, `test-check-canon-*`,",
    "  `test-generate-canon-draft-*`, and",
    "  `test-generate-check-canon-*` guard lifecycle artifacts and CLI",
    "  contracts.",
    "- **Regression:** the remaining suites pin resolver, parser, ranking,",
    "  false-positive, drift, and fixture-specific behavior. Run them when",
    "  touching shared engine logic or before release.",
    "",
    "Or run individual suites:",
    "",
    "```bash",
    ...suiteLines,
    "```",
    "",
    "## Fixtures",
    "",
    "Tests build their own fixtures under `/tmp/fx-*` on each run.",
    "Fixtures are disposable — every suite clears its own working dirs",
    "at start. No shared state between runs.",
    "",
    "Each test script exits non-zero on any failure. `npm test` stops",
    "at the first failing suite.",
    "",
    "## What the tests cover by release",
    "",
    ...releaseBullets,
    "",
    "## What's NOT covered",
    "",
    "Documented honestly so future maintainers know where the guard",
    "rails stop:",
    "",
    "- Cross-process cache sharing between scripts (each of",
    "  `measure-topology`, `build-call-graph`, `check-barrel-discipline`",
    "  currently re-parses from scratch). No suite exercises shared",
    "  cache.",
    "- Rust source trees are owned by `lumin-rust-analyzer`; this JS test suite",
    "  covers routing, manifest, and blind-zone behavior only, not a JS",
    "  Rust parser fallback.",
    "- `__getattr__`-based lazy export maps in Python `__init__.py`",
    "  files. Known residual FP source; no fixture.",
    "- Interactive `--focus-class` output beyond the smoke check that",
    "  the block appears.",
    "",
    ...(missingDescriptions.length > 0
      ? [
          "## Maintainer note",
          "",
          `New suite(s) without a description in scripts/update-test-doc.mjs:`,
          ...missingDescriptions.map((s) => `- ${s}`),
          "",
        ]
      : []),
  ].join("\n");
}

// ─── Main ─────────────────────────────────────────────────
const generated = render();

if (CHECK_MODE) {
  const current = (() => {
    try {
      return readFileSync(README, "utf8");
    } catch {
      return "";
    }
  })();
  const normalizedCurrent = normalizeLineEndings(current);
  const normalizedGenerated = normalizeLineEndings(generated);
  if (normalizedCurrent === normalizedGenerated) {
    console.log("[update-test-doc] tests/README.md is up to date");
    process.exit(0);
  }
  console.error(
    "[update-test-doc] DRIFT: tests/README.md differs from generated output.",
  );
  console.error(
    "                   Run `npm run update-test-doc` to regenerate.",
  );
  // Show a diff-ish preview: first line that differs.
  const curLines = normalizedCurrent.split("\n");
  const genLines = normalizedGenerated.split("\n");
  const maxLines = Math.max(curLines.length, genLines.length);
  let shown = 0;
  for (let i = 0; i < maxLines && shown < 5; i++) {
    if (curLines[i] !== genLines[i]) {
      console.error(`  line ${i + 1}:`);
      console.error(
        `    on-disk:   ${JSON.stringify(curLines[i] ?? "(end)").slice(0, 100)}`,
      );
      console.error(
        `    generated: ${JSON.stringify(genLines[i] ?? "(end)").slice(0, 100)}`,
      );
      shown++;
    }
  }
  process.exit(1);
}

writeFileSync(README, generated);
console.log(`[update-test-doc] wrote ${README} (${generated.length} bytes)`);
