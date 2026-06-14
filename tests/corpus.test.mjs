// Release-blocking precision corpus + false-positive budget (v1.10.0 P2).
//
// Each CASE builds a small repo fixture in tmpdir, runs the audit
// pipeline (symbols → classify → rank-fixes), and asserts tier counts
// against the labeled truth. The corpus is designed to REGRESS if any
// of the core precision invariants break:
//
//   CASE-AST     — regex text counting would FP here (comment +
//                  string literal mentioning the symbol). AST counting
//                  must keep the symbol in Class C (truly dead).
//                  Guards v1.10.0 P0.
//
//   CASE-P1     — the repo has a known alias scope mismatch + one clean file.
//                  Global `unresolvedRatio >= 15%` would demote EVERY
//                  finding; local-scope taint must keep both unaffected
//                  findings un-degraded. Guards v1.10.0 P1 + PCEF P0.
//
//   CASE-FP40  — package.json `exports: { ".": "./dist/index.mjs" }`
//                  with src/index.ts as the real source. The barrel
//                  must be detected so its symbols are NOT in the
//                  dead-export candidate list. Guards R-8 / FP-40.
//
//   CASE-FP41  — TSX compound-component pattern: one export (parent)
//                  is live externally; a sibling export is used only
//                  via JSX inside the parent's render. The AST
//                  counter must see JSX identifiers as references;
//                  otherwise the sibling over-escalates from Tier A
//                  to Tier C. Guards FP-41 (JSX blindness in v1.10.0
//                  `countFileReferencesAst`).
//
// FP budget: zero. Any violation means a real precision regression,
// not a flaky fixture. If a fixture is flaky, fix the fixture — never
// raise the budget to paper over a drift.

import { execSync } from "node:child_process";
import { expect, it } from "vitest";
import {
  writeFileSync,
  readFileSync,
  mkdirSync,
  rmSync,
  mkdtempSync,
  existsSync,
} from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");
const NODE = process.execPath;

let precisionFailures = 0;
function assert(label, ok, detail = "") {
  if (!ok) precisionFailures++;
  it(label, () => {
    expect(ok, detail).toBeTruthy();
  });
}

// Small helper: write a nested file, creating parent dirs as needed.
function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

// Run a script from the repo root with the given args. Surface stderr
// on failure so a regression diagnostic is visible.
function run(script, args) {
  try {
    execSync(
      `"${NODE}" "${path.join(DIR, script)}" ${args.map((a) => `"${a}"`).join(" ")}`,
      {
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
  } catch (e) {
    const out = (e.stderr?.toString?.() ?? "") + (e.stdout?.toString?.() ?? "");
    throw new Error(`[${script}] ${e.message}\n${out.slice(0, 800)}`);
  }
}

// Pipeline: symbols → classify → action-safety → rank-fixes. Minimum
// needed for tier-count assertions under the PCEF contract. Staleness
// and runtime coverage are skipped (not required for the precision
// invariants under test).
function runPipeline(fx, out) {
  run("build-symbol-graph.mjs", ["--root", fx, "--output", out]);
  run("classify-dead-exports.mjs", ["--root", fx, "--output", out]);
  run("export-action-safety.mjs", ["--root", fx, "--output", out]);
  run("rank-fixes.mjs", ["--root", fx, "--output", out]);
}

function runProductionPipeline(fx, out) {
  run("build-symbol-graph.mjs", [
    "--root",
    fx,
    "--output",
    out,
    "--production",
  ]);
  run("classify-dead-exports.mjs", [
    "--root",
    fx,
    "--output",
    out,
    "--production",
  ]);
  run("export-action-safety.mjs", ["--root", fx, "--output", out]);
  run("rank-fixes.mjs", ["--root", fx, "--output", out]);
}

function readFixPlan(out) {
  return JSON.parse(readFileSync(path.join(out, "fix-plan.json"), "utf8"));
}
function readSymbols(out) {
  return JSON.parse(readFileSync(path.join(out, "symbols.json"), "utf8"));
}
function readClassify(out) {
  return JSON.parse(readFileSync(path.join(out, "dead-classify.json"), "utf8"));
}
function cleanupEntries(fixPlan) {
  return [...(fixPlan.safeFixes ?? []), ...(fixPlan.reviewFixes ?? [])];
}
function cleanupSymbols(fixPlan) {
  return new Set(cleanupEntries(fixPlan).map((s) => s.finding.symbol));
}
function cleanupIdentities(fixPlan) {
  return new Set(
    cleanupEntries(fixPlan).map(
      (s) => `${s.finding.file}::${s.finding.symbol}`,
    ),
  );
}
function findCleanup(fixPlan, predicate) {
  return cleanupEntries(fixPlan).find(predicate);
}

// ═════════════════════════════════════════════════════════════
// CASE-FP18B — non-literal dynamic import with static directory prefix
// ═════════════════════════════════════════════════════════════
// Command/plugin loaders often use `import(`./commands/${name}.js`)`.
// The scanner cannot resolve the exact file, but it can name the target
// directory family. Exports inside that directory must not remain
// review-visible cleanup candidates.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-fp18b-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-fp18b-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "corpus-fp18b", type: "module" }),
    );
    write(
      fx,
      "src/loader.ts",
      "export async function loadCommand(name: string) {\n" +
        "  return import(`./commands/${name}.js`);\n" +
        "}\n",
    );
    write(
      fx,
      "src/commands/build.ts",
      'export function runBuild() { return "build"; }\n',
    );
    write(fx, "src/private.ts", "export const trulyDead = 1;\n");

    runPipeline(fx, out);
    const symbols = readSymbols(out);
    const classify = readClassify(out);
    const fixPlan = readFixPlan(out);
    const visibleSymbols = cleanupSymbols(fixPlan);
    const mutedBuild = fixPlan.muted.find(
      (s) => s.finding.symbol === "runBuild",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) => s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-FP18B.1. symbols emits dynamic import opacity target directory",
      symbols.dynamicImportOpacity?.some(
        (e) =>
          e.consumerFile === "src/loader.ts" &&
          e.targetDir === "src/commands/" &&
          e.kind === "template-prefix",
      ),
      JSON.stringify(symbols.dynamicImportOpacity),
    );
    assert(
      "CASE-FP18B.2. dynamic command export is not review-visible cleanup",
      !visibleSymbols.has("runBuild"),
      `cleanup candidates: ${[...visibleSymbols].join(", ")}`,
    );
    assert(
      "CASE-FP18B.3. dynamic command export is MUTED with FP-18 evidence",
      mutedBuild &&
        mutedBuild.evidence?.policy?.reason === "dynamicImportOpacity_FP18" &&
        mutedBuild.evidence.policy.evidence?.some(
          (e) =>
            e.consumerFile === "src/loader.ts" &&
            e.targetDir === "src/commands/",
        ),
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FP18B.4. classifier summary counts dynamicImportOpacity_FP18",
      classify.summary.excluded.dynamicImportOpacity_FP18 === 1,
      JSON.stringify(classify.summary.excluded),
    );
    assert(
      "CASE-FP18B.5. unrelated private export remains review-visible",
      cleanupDead && cleanupDead.finding.file === "src/private.ts",
      `cleanup candidates: ${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-AST — v1.10.0 P0 regression guard
// ═════════════════════════════════════════════════════════════
// `deadOnly` is truly dead (0 real refs). Comment + string mention
// would have inflated the regex counter. AST counter must give 0.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-ast-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-ast-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "corpus-ast", type: "module" }),
    );
    write(
      fx,
      "src/entry.ts",
      `// This is the package entry — not actually consuming deadOnly\n` +
        `export const live = 1;\n`,
    );
    write(
      fx,
      "src/mod.ts",
      `// deadOnly is a legacy export (would be counted by old regex)\n` +
        `export const deadOnly = 1;\n` +
        `const msg = "deadOnly is a string mention";\n` +
        `export const live = () => msg;\n`,
    );

    runPipeline(fx, out);
    const classify = readClassify(out);
    // Locate deadOnly in the emitted proposals.
    const buckets = [
      ...(classify.proposal_C_remove_symbol ?? []),
      ...(classify.proposal_A_demote_to_internal ?? []),
      ...(classify.proposal_B_review ?? []),
    ];
    const deadOnly = buckets.find((p) => p.symbol === "deadOnly");

    assert(
      "CASE-AST.1. deadOnly is in the classified list",
      !!deadOnly,
      `proposals: ${JSON.stringify(buckets.map((b) => b.symbol))}`,
    );
    assert(
      "CASE-AST.2. deadOnly is classified as Class C (0 refs, not inflated by comment/string)",
      deadOnly && deadOnly.fileInternalUses === 0,
      `fileInternalUses=${deadOnly?.fileInternalUses} (old regex would have said 2)`,
    );
    assert(
      "CASE-AST.3. deadOnly evidence label is AST, not regex",
      deadOnly && deadOnly.fileInternalUsesEvidence === "ast-ident-ref-count",
      `evidence=${deadOnly?.fileInternalUsesEvidence}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P1 — v1.10.0 P1 regression guard
// ═════════════════════════════════════════════════════════════
// Two dead symbols: one outside a known alias target scope, one in a file
// that doesn't match anything. With PCEF P0 local-scope taint, neither should
// land in DEGRADED. Without it (old global gate), BOTH would be DEGRADED as
// soon as unresolvedRatio >= 15%.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p1-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p1-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "corpus-p1", type: "module" }),
    );
    // tsconfig registers `@/*` as a LOCAL alias rooted at `src/`. This
    // distinguishes unresolved-internal (alias recognized, target
    // absent) from external (npm package). Without this tsconfig the
    // `@/foo` imports would be classified as EXTERNAL and never
    // populate unresolvedInternalRatio.
    write(
      fx,
      "tsconfig.json",
      JSON.stringify({
        compilerOptions: { baseUrl: ".", paths: { "@/*": ["./src/*"] } },
      }),
    );

    // Dead symbol in a file OUTSIDE the known `@/* -> src/*` alias target
    // scope. The consumer's import of `@/components/authControl` will be
    // UNRESOLVED because no such file exists under src/. PCEF P0 should
    // classify this as a scoped no-match, not taint this unrelated file.
    write(
      fx,
      "apps/other/components/authControl.tsx",
      `export const AuthControl = () => null;\n`,
    );

    // Dead symbol in a CLEAN file — no unresolved specifier shape
    // could plausibly point here. Must stay un-degraded even with
    // the repo-wide unresolved ratio high.
    write(fx, "src/utils/logger.ts", `export const log = () => null;\n`);

    // Consumer with five fully-unresolved `@/*` imports (push global
    // ratio above 15%) plus one whose shape matches authControl.tsx.
    write(
      fx,
      "src/consumer.ts",
      `import { a } from '@/alpha';\n` +
        `import { b } from '@/beta';\n` +
        `import { c } from '@/gamma';\n` +
        `import { d } from '@/delta';\n` +
        `import { e } from '@/epsilon';\n` +
        `// Unresolvable, but path shape aligns with the dead authControl file:\n` +
        `import { x } from '@/components/authControl';\n` +
        `export const live = () => [a, b, c, d, e, x];\n`,
    );

    runPipeline(fx, out);
    const symbols = readSymbols(out);
    const fixPlan = readFixPlan(out);

    const ratio = symbols.uses?.unresolvedInternalRatio ?? 0;
    assert(
      "CASE-P1.1. fixture has high resolver blindness (forces the scenario)",
      ratio >= 0.15,
      `unresolvedInternalRatio=${ratio}, uses=${JSON.stringify(symbols.uses)}`,
    );

    // Find each dead candidate by symbol name
    const allTiers = [
      ...fixPlan.safeFixes.map((s) => ({ ...s, _tier: "SAFE_FIX" })),
      ...fixPlan.reviewFixes.map((s) => ({ ...s, _tier: "REVIEW_FIX" })),
      ...fixPlan.degraded.map((s) => ({ ...s, _tier: "DEGRADED" })),
    ];
    const authCtl = allTiers.find((s) => s.finding.symbol === "AuthControl");
    const loggerLog = allTiers.find((s) => s.finding.symbol === "log");

    assert(
      "CASE-P1.2. AuthControl outside known alias target is NOT DEGRADED",
      authCtl && authCtl._tier !== "DEGRADED",
      `tier=${authCtl?._tier}, reason=${authCtl?.reason}`,
    );
    assert(
      "CASE-P1.3. AuthControl reason does not cite unresolved spec taint",
      authCtl &&
        (typeof authCtl.reason !== "string" ||
          !authCtl.reason.includes("unresolved-spec-could-match")),
      `reason=${authCtl?.reason}`,
    );

    // The CORE P1 WIN: clean finding stays un-degraded, even though
    // the repo-wide ratio is high. Pre-P1 would have DEGRADED both.
    assert(
      "CASE-P1.4. logger.log (clean file) is NOT in DEGRADED — the P1 win",
      loggerLog && loggerLog._tier !== "DEGRADED",
      `tier=${loggerLog?._tier}, reason=${loggerLog?.reason}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FP44 — production cleanup must respect test-pinned contracts
// ═════════════════════════════════════════════════════════════
// In --production mode, tests are excluded from symbols.json. That is
// correct for production reachability, but direct test imports often pin
// internal helper contracts. Such exports must be MUTED, not surfaced as
// review-visible cleanup candidates.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-fp44-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-fp44-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "corpus-fp44", type: "module" }),
    );
    write(
      fx,
      "src/contract.ts",
      `export function contractHelper() { return 1; }\n` +
        `export function manifestPinned() { return 2; }\n` +
        `export const trulyDead = 2;\n`,
    );
    write(
      fx,
      "tests/contract.test.ts",
      `import { contractHelper } from '../src/contract';\n` +
        `const requiredExports = { 'src/contract.ts': ['manifestPinned'] };\n` +
        `export const observedByContractTest = contractHelper() + requiredExports['src/contract.ts'].length;\n`,
    );

    runProductionPipeline(fx, out);
    const classify = readClassify(out);
    const fixPlan = readFixPlan(out);
    const proposalSymbols = new Set([
      ...(classify.proposal_C_remove_symbol ?? []).map((p) => p.symbol),
      ...(classify.proposal_A_demote_to_internal ?? []).map((p) => p.symbol),
      ...(classify.proposal_B_review ?? []).map((p) => p.symbol),
    ]);
    const mutedContract = fixPlan.muted.find(
      (s) => s.finding.symbol === "contractHelper",
    );
    const mutedManifest = fixPlan.muted.find(
      (s) => s.finding.symbol === "manifestPinned",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) => s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-FP44.1. test-pinned contractHelper is not review-visible cleanup",
      !proposalSymbols.has("contractHelper"),
      `proposal symbols: ${[...proposalSymbols].join(", ")}`,
    );
    assert(
      "CASE-FP44.2. contractHelper materializes as MUTED testConsumer_FP44",
      mutedContract &&
        mutedContract.evidence?.policy?.reason === "testConsumer_FP44" &&
        mutedContract.evidence.policy.evidence?.testFile ===
          "tests/contract.test.ts",
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FP44.3. export-manifest pinned symbol also materializes as MUTED",
      mutedManifest &&
        mutedManifest.evidence?.policy?.reason === "testConsumer_FP44" &&
        mutedManifest.evidence.policy.evidence?.importKind ===
          "test-export-manifest",
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FP44.4. trulyDead remains review-visible cleanup",
      cleanupDead && cleanupDead.finding.symbol === "trulyDead",
      `cleanup candidates: ${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding.symbol))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FP40 — R-8 / FP-40 regression guard
// ═════════════════════════════════════════════════════════════
// package.json `exports: { ".": "./dist/index.mjs" }` with src/index.ts
// as the real source. Before R-8 the barrel-file detection did a narrow
// `.js → .ts` swap and missed `.mjs`, so the barrel was NOT in
// BARREL_FILES and its public symbols were dead-list candidates. With
// mapOutputToSource the barrel is correctly detected.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-fp40-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-fp40-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-fp40",
        type: "module",
        exports: { ".": "./dist/index.mjs" },
      }),
    );
    // Source entry (what the `exports` target points at once `.mjs` is
    // swapped to `.ts` via OUT_SRC_PAIRS).
    write(
      fx,
      "src/index.ts",
      `export const barrelExport = 1;\n` +
        `export function barrelFn() { return 2; }\n`,
    );
    // A separate file with an actually-dead symbol — the test needs
    // SOMETHING to classify; we just verify barrelExport is NOT in the
    // dead-list, not that the whole list is empty.
    write(fx, "src/other.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const symbols = readSymbols(out);
    const classify = readClassify(out);

    // Pull the dead-production list from symbols.json AND the
    // classifier's proposal buckets. `barrelExport` / `barrelFn` must
    // be in NEITHER (they're the public entry).
    const deadProdSymbols = new Set(
      symbols.deadProdList?.map((d) => d.symbol) ?? [],
    );
    const proposalSymbols = new Set([
      ...(classify.proposal_C_remove_symbol ?? []).map((p) => p.symbol),
      ...(classify.proposal_A_demote_to_internal ?? []).map((p) => p.symbol),
      ...(classify.proposal_B_review ?? []).map((p) => p.symbol),
    ]);

    assert(
      "CASE-FP40.1. barrelExport (from ./dist/index.mjs entry) is NOT dead-listed",
      !deadProdSymbols.has("barrelExport"),
      `deadProdList symbols: ${[...deadProdSymbols].join(", ")}`,
    );
    assert(
      "CASE-FP40.2. barrelFn (from ./dist/index.mjs entry) is NOT dead-listed",
      !deadProdSymbols.has("barrelFn"),
    );
    assert(
      "CASE-FP40.3. barrelExport is NOT proposed for removal",
      !proposalSymbols.has("barrelExport"),
    );
    assert(
      "CASE-FP40.4. trulyDead IS dead-listed (sanity: fixture actually runs the analysis)",
      deadProdSymbols.has("trulyDead"),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FP40B — root exports string/conditional forms are barrels too
// ═════════════════════════════════════════════════════════════
// `detectBarrelFiles` must use normalized exports shapes, not only
// `exports["."]`, otherwise valid root entry forms leak barrel exports
// into dead-export candidates.
for (const [label, exportsField] of [
  ["string-root", "./dist/index.mjs"],
  [
    "conditional-root",
    { import: "./dist/index.mjs", types: "./dist/index.d.ts" },
  ],
]) {
  const fx = mkdtempSync(path.join(tmpdir(), `corpus-fp40b-${label}-`));
  const out = mkdtempSync(path.join(tmpdir(), `corpus-fp40b-${label}-out-`));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: `corpus-fp40b-${label}`,
        type: "module",
        exports: exportsField,
      }),
    );
    write(fx, "src/index.ts", `export const publicRoot = 1;\n`);
    write(fx, "src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const symbols = readSymbols(out);
    const deadProdSymbols = new Set(
      symbols.deadProdList?.map((d) => d.symbol) ?? [],
    );

    assert(
      `CASE-FP40B.${label}. root export form marks src/index.ts as barrel`,
      !deadProdSymbols.has("publicRoot") && deadProdSymbols.has("trulyDead"),
      `deadProdList symbols: ${[...deadProdSymbols].join(", ")}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FP41 — JSX identifier blindness regression guard
// ═════════════════════════════════════════════════════════════
// shadcn/ui-class compound component: `AlertDialog` is imported by
// an app file (live externally). `AlertDialogTrigger` has no external
// consumer but is used via JSX inside AlertDialog's render. The AST
// counter must see the `<AlertDialogTrigger>` JSX usage; before the
// FP-41 fix it matched only `Identifier` nodes and missed JSXIdentifier,
// over-escalating Trigger to Tier C (completely dead). After the fix
// it counts 1 file-internal use and Trigger lands in Tier A.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-fp41-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-fp41-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-fp41",
        type: "module",
      }),
    );
    write(
      fx,
      "src/components/alert-dialog.tsx",
      `import * as React from 'react';\n` +
        `export const AlertDialog = (props) => (\n` +
        `  <div className="root">\n` +
        `    <AlertDialogTrigger onClick={props.onOpen}>\n` +
        `      {props.children}\n` +
        `    </AlertDialogTrigger>\n` +
        `  </div>\n` +
        `);\n` +
        `export const AlertDialogTrigger = (props) => (\n` +
        `  <button onClick={props.onClick}>{props.children}</button>\n` +
        `);\n`,
    );
    // External consumer imports AlertDialog only — Trigger has zero
    // cross-file fan-in.
    write(
      fx,
      "src/app.tsx",
      `import { AlertDialog } from './components/alert-dialog';\n` +
        `export const App = () => <AlertDialog onOpen={() => {}}>open</AlertDialog>;\n`,
    );

    runPipeline(fx, out);
    const classify = readClassify(out);
    const buckets = [
      ...(classify.proposal_C_remove_symbol ?? []).map((p) => ({
        ...p,
        _bucket: "C",
      })),
      ...(classify.proposal_A_demote_to_internal ?? []).map((p) => ({
        ...p,
        _bucket: "A",
      })),
      ...(classify.proposal_B_review ?? []).map((p) => ({
        ...p,
        _bucket: "B",
      })),
    ];
    const trigger = buckets.find((p) => p.symbol === "AlertDialogTrigger");
    const alert = buckets.find((p) => p.symbol === "AlertDialog");

    // AlertDialog is live (imported by src/app.tsx) — MUST NOT be in
    // any dead bucket.
    assert(
      "CASE-FP41.1. AlertDialog (live) is NOT in any dead bucket",
      !alert,
      `AlertDialog bucket=${alert?._bucket}`,
    );

    // Trigger IS in the classifier output (no external consumer).
    assert(
      "CASE-FP41.2. AlertDialogTrigger appears in the classifier output",
      !!trigger,
      `classifier symbols: ${buckets.map((b) => `${b.symbol}(${b._bucket})`).join(", ")}`,
    );

    // Core FP-41 assertion: file-internal JSX usage IS counted. The
    // <AlertDialogTrigger> element inside AlertDialog's render is one
    // valueRefs hit.
    assert(
      "CASE-FP41.3. AlertDialogTrigger fileInternalUses == 1 (JSX use counted)",
      trigger && trigger.fileInternalUses === 1,
      `fileInternalUses=${trigger?.fileInternalUses}`,
    );
    assert(
      "CASE-FP41.4. Trigger's JSX use is tracked as a value reference",
      trigger && trigger.fileInternalRefs?.valueRefs === 1,
      `fileInternalRefs=${JSON.stringify(trigger?.fileInternalRefs)}`,
    );

    // Tier boundary: occ === 1 → Tier A (not Tier C).
    assert(
      "CASE-FP41.5. Trigger classified as A-remove-export, not C-completely-dead",
      trigger && trigger._bucket === "A",
      `bucket=${trigger?._bucket}`,
    );

    // Evidence label preserved through the JSX-aware path.
    assert(
      "CASE-FP41.6. Evidence label remains ast-ident-ref-count",
      trigger && trigger.fileInternalUsesEvidence === "ast-ident-ref-count",
      `evidence=${trigger?.fileInternalUsesEvidence}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1 — root package public surface in workspace repos
// ═════════════════════════════════════════════════════════════
// A pnpm workspace root can itself be the published package. P6-1's
// package/public surface model must include root package.json entries,
// not only child workspaces, and must treat type-only public subpaths as
// public API evidence.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-1-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-1-out-"));
  try {
    write(fx, "pnpm-workspace.yaml", "packages:\n  - examples/*\n");
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-root",
        type: "module",
        exports: {
          ".": { types: "./dist/index.d.ts" },
          "./types": { types: "./types/index.d.ts" },
        },
      }),
    );
    write(
      fx,
      "examples/app/package.json",
      JSON.stringify({ name: "example-app" }),
    );
    write(fx, "src/index.ts", `export function publicEntry() { return 1; }\n`);
    write(
      fx,
      "types/index.d.ts",
      `export interface PublicOptions { enabled?: boolean }\n`,
    );
    write(fx, "src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const fixPlan = readFixPlan(out);
    const visibleSymbols = cleanupSymbols(fixPlan);
    const mutedPublicOptions = fixPlan.muted.find(
      (s) =>
        s.finding.file === "types/index.d.ts" &&
        s.finding.symbol === "PublicOptions",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) => s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-P6-1.1. root export publicEntry is not review-visible cleanup",
      !visibleSymbols.has("publicEntry"),
      `cleanup candidates: ${[...visibleSymbols].join(", ")}`,
    );
    assert(
      "CASE-P6-1.2. type-only public subpath is MUTED as publicApi_FP23",
      mutedPublicOptions &&
        mutedPublicOptions.evidence?.policy?.reason === "publicApi_FP23" &&
        Array.isArray(mutedPublicOptions.evidence.policy.evidence) &&
        mutedPublicOptions.evidence.policy.evidence.some(
          (e) => e.source === "package.exports" && e.subpath === "./types",
        ),
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1.3. unrelated private export remains review-visible",
      cleanupDead && cleanupDead.finding.file === "src/private.ts",
      `cleanup candidates: ${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1a — package imports are internal aliases, not public API
// ═════════════════════════════════════════════════════════════
// Node `package.imports` (`#foo`) helps source files resolve internal
// aliases. It is not an external package surface. A file targeted only
// by `imports` must not be muted as publicApi_FP23.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-imports-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-imports-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-imports",
        type: "module",
        imports: {
          "#internal": "./src/internal.ts",
        },
      }),
    );
    write(fx, "src/internal.ts", `export const internalOnly = 1;\n`);

    runPipeline(fx, out);
    const classify = readClassify(out);
    const fixPlan = readFixPlan(out);
    const mutedInternal = fixPlan.muted.find(
      (s) => s.finding.symbol === "internalOnly",
    );
    const cleanupInternal = findCleanup(
      fixPlan,
      (s) => s.finding.symbol === "internalOnly",
    );

    assert(
      "CASE-P6-1a.1. package.imports exact target is NOT counted as publicApi_FP23",
      classify.summary.excluded.publicApi_FP23 === 0,
      JSON.stringify(classify.summary.excluded),
    );
    assert(
      "CASE-P6-1a.2. #imports-only dead export is not MUTED as public API",
      !(mutedInternal?.evidence?.policy?.reason === "publicApi_FP23"),
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1a.3. #imports-only dead export remains review-visible",
      cleanupInternal && cleanupInternal.finding.file === "src/internal.ts",
      `cleanup candidates: ${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1b — script-driven build entrypoints are not cleanup candidates
// ═════════════════════════════════════════════════════════════
// Some packages build extra entry files from a script-controlled command
// list (`tsup src/client/dev/react.ts ...`). Those files have no static
// import consumer inside source, but the build tool consumes them by path.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-script-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-script-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-script",
        type: "module",
        scripts: {
          build: "esno scripts/build.ts",
        },
      }),
    );
    write(
      fx,
      "scripts/build.ts",
      `export const commands = [\n` +
        `  'npx tsup src/client/dev/react.ts --format esm -d dist/client/dev',\n` +
        `]\n`,
    );
    write(
      fx,
      "src/client/dev/react.ts",
      `export type RegisterSWOptions = { immediate?: boolean };\n` +
        `export function useRegisterSW(_options: RegisterSWOptions = {}) { return {}; }\n`,
    );
    write(fx, "src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const fixPlan = readFixPlan(out);
    const visibleSymbols = cleanupSymbols(fixPlan);
    const mutedUseRegister = fixPlan.muted.find(
      (s) => s.finding.symbol === "useRegisterSW",
    );
    const mutedOptions = fixPlan.muted.find(
      (s) => s.finding.symbol === "RegisterSWOptions",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) => s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-P6-1b.1. script entrypoint function is not review-visible cleanup",
      !visibleSymbols.has("useRegisterSW"),
      `cleanup candidates: ${[...visibleSymbols].join(", ")}`,
    );
    assert(
      "CASE-P6-1b.2. script entrypoint function is MUTED with FP45 evidence",
      mutedUseRegister &&
        mutedUseRegister.evidence?.policy?.reason === "scriptEntrypoint_FP45" &&
        mutedUseRegister.evidence.policy.evidence?.some(
          (e) =>
            e.source === "script-string-literal" &&
            e.scriptFile === "scripts/build.ts",
        ),
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1b.3. all exports in the script entry file are muted",
      mutedOptions &&
        mutedOptions.evidence?.policy?.reason === "scriptEntrypoint_FP45",
      `muted: ${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1b.4. unrelated private export remains review-visible",
      cleanupDead && cleanupDead.finding.file === "src/private.ts",
      `cleanup candidates: ${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1d — declaration-file dependencies resolve from public entries
// ═════════════════════════════════════════════════════════════
// Public/script entrypoints often import reusable types from sibling
// `.d.ts` files (`import type { X } from '../type'`). The resolver must
// see those declaration files; otherwise the source type falsely appears
// as dead even though a public entrypoint re-exports it.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-dts-dep-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-dts-dep-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-dts-dep",
        type: "module",
        scripts: {
          build:
            "tsup src/client/dev/register.ts --format esm -d dist/client/dev",
        },
      }),
    );
    write(
      fx,
      "src/client/type.d.ts",
      `export interface RegisterSWOptions { immediate?: boolean }\n`,
    );
    write(
      fx,
      "src/client/dev/register.ts",
      `import type { RegisterSWOptions } from '../type';\n` +
        `export type { RegisterSWOptions };\n` +
        `export function registerSW(_options: RegisterSWOptions = {}) { return {}; }\n`,
    );
    write(fx, "src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const symbols = readSymbols(out);
    const fixPlan = readFixPlan(out);
    const deadIdentities = new Set(
      (symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`),
    );
    const visibleSymbols = cleanupIdentities(fixPlan);
    const mutedRegister = fixPlan.muted.find(
      (s) =>
        s.finding.file === "src/client/dev/register.ts" &&
        s.finding.symbol === "registerSW",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) => s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-P6-1d.1. sibling .d.ts type import contributes fan-in",
      symbols.fanInByIdentity?.["src/client/type.d.ts::RegisterSWOptions"] ===
        1,
      `fanIn=${symbols.fanInByIdentity?.["src/client/type.d.ts::RegisterSWOptions"]}`,
    );
    assert(
      "CASE-P6-1d.2. source declaration type is not dead-listed",
      !deadIdentities.has("src/client/type.d.ts::RegisterSWOptions"),
      `dead=${[...deadIdentities].join(", ")}`,
    );
    assert(
      "CASE-P6-1d.3. source declaration type is not review-visible cleanup",
      !visibleSymbols.has("src/client/type.d.ts::RegisterSWOptions"),
      `cleanup candidates=${[...visibleSymbols].join(", ")}`,
    );
    assert(
      "CASE-P6-1d.4. public script entry still gets muted by entrypoint policy",
      mutedRegister?.evidence?.policy?.reason === "scriptEntrypoint_FP45",
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1d.5. unrelated private export remains review-visible",
      cleanupDead && cleanupDead.finding.file === "src/private.ts",
      `cleanup candidates=${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1e — package dist exports resolve back to authored source
// ═════════════════════════════════════════════════════════════
// Real packages often publish `exports` that point at `dist/*.js` and
// `dist/*.d.ts`, while the analyzable symbols live in `src/*.ts`. When
// both dist and source files exist in the repo, package-import fan-in
// must still land on the authored source file. Otherwise the source
// symbols look dead even though workspace consumers import the package.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-dist-source-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-dist-source-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-dist-source-root",
        type: "module",
        workspaces: ["packages/*", "app"],
      }),
    );
    write(
      fx,
      "packages/shared/package.json",
      JSON.stringify({
        name: "@scope/shared",
        type: "module",
        exports: {
          ".": {
            types: "./dist/index.d.ts",
            import: "./dist/index.js",
          },
        },
      }),
    );
    write(
      fx,
      "packages/shared/dist/index.js",
      `export function createShared() { return 1; }\n`,
    );
    write(
      fx,
      "packages/shared/dist/index.d.ts",
      `export interface PublicOptions { enabled?: boolean }\n` +
        `export declare function createShared(options?: PublicOptions): number;\n`,
    );
    write(
      fx,
      "packages/shared/src/index.ts",
      `export interface PublicOptions { enabled?: boolean }\n` +
        `export function createShared(_options: PublicOptions = {}) { return 1; }\n` +
        `export const publicButUnreferenced = 1;\n`,
    );
    write(
      fx,
      "packages/shared/src/private.ts",
      `export const trulyDead = 1;\n`,
    );
    write(
      fx,
      "app/package.json",
      JSON.stringify({
        name: "app",
        type: "module",
        dependencies: { "@scope/shared": "workspace:*" },
      }),
    );
    write(
      fx,
      "app/src/main.ts",
      `import { createShared, type PublicOptions } from '@scope/shared';\n` +
        `const options: PublicOptions = { enabled: true };\n` +
        `createShared(options);\n`,
    );

    runPipeline(fx, out);
    const symbols = readSymbols(out);
    const fixPlan = readFixPlan(out);
    const visibleIdentities = cleanupIdentities(fixPlan);
    const cleanupDead = findCleanup(
      fixPlan,
      (s) =>
        s.finding.file === "packages/shared/src/private.ts" &&
        s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-P6-1e.1. package import fan-in lands on source function",
      symbols.fanInByIdentity?.[
        "packages/shared/src/index.ts::createShared"
      ] === 1,
      `fanIn=${symbols.fanInByIdentity?.["packages/shared/src/index.ts::createShared"]}, all=${JSON.stringify(symbols.fanInByIdentity)}`,
    );
    assert(
      "CASE-P6-1e.2. package type import fan-in lands on source interface",
      symbols.fanInByIdentity?.[
        "packages/shared/src/index.ts::PublicOptions"
      ] === 1,
      `fanIn=${symbols.fanInByIdentity?.["packages/shared/src/index.ts::PublicOptions"]}, all=${JSON.stringify(symbols.fanInByIdentity)}`,
    );
    assert(
      "CASE-P6-1e.3. imported public source exports are not review-visible cleanup",
      !visibleIdentities.has("packages/shared/src/index.ts::createShared") &&
        !visibleIdentities.has("packages/shared/src/index.ts::PublicOptions"),
      `cleanup candidates=${[...visibleIdentities].join(", ")}`,
    );
    assert(
      "CASE-P6-1e.4. unreferenced export in package public source file is not review-visible cleanup",
      !visibleIdentities.has(
        "packages/shared/src/index.ts::publicButUnreferenced",
      ),
      `cleanup candidates=${[...visibleIdentities].join(", ")}`,
    );
    assert(
      "CASE-P6-1e.5. unrelated private export remains review-visible",
      cleanupDead,
      `cleanup candidates=${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1f — JS runtime declaration sidecars are not removal candidates
// ═════════════════════════════════════════════════════════════
// A `.d.ts` next to a runtime `.js` module provides TypeScript's view of
// the JS import. The value import fan-in lands on the `.js` file, so the
// declaration sidecar can look dead unless the classifier recognizes the
// paired runtime file.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-dts-sidecar-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-dts-sidecar-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-dts-sidecar",
        type: "module",
      }),
    );
    write(
      fx,
      "src/runtime-module-sources.js",
      `export const RUNTIME_MODULE_SOURCES = { react: 'react-source' };\n`,
    );
    write(
      fx,
      "src/runtime-module-sources.d.ts",
      `export declare const RUNTIME_MODULE_SOURCES: Readonly<{ react: string }>;\n`,
    );
    write(
      fx,
      "src/document.ts",
      `import { RUNTIME_MODULE_SOURCES } from './runtime-module-sources.js';\n` +
        `export const documentSource = RUNTIME_MODULE_SOURCES.react;\n`,
    );
    write(fx, "src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const fixPlan = readFixPlan(out);
    const visibleIdentities = cleanupIdentities(fixPlan);
    const mutedSidecar = fixPlan.muted.find(
      (s) =>
        s.finding.file === "src/runtime-module-sources.d.ts" &&
        s.finding.symbol === "RUNTIME_MODULE_SOURCES",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) =>
        s.finding.file === "src/private.ts" && s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-P6-1f.1. declaration sidecar is not review-visible cleanup",
      !visibleIdentities.has(
        "src/runtime-module-sources.d.ts::RUNTIME_MODULE_SOURCES",
      ),
      `cleanup candidates=${[...visibleIdentities].join(", ")}`,
    );
    assert(
      "CASE-P6-1f.2. declaration sidecar is MUTED with FP48 evidence",
      mutedSidecar?.evidence?.policy?.reason === "declarationSidecar_FP48",
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1f.3. unrelated private export remains review-visible",
      cleanupDead,
      `cleanup candidates=${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-P6-1c — framework and HTML convention entrypoints
// ═════════════════════════════════════════════════════════════
// VitePress loads `.vitepress/config` and `.vitepress/theme/index` by
// convention. Vite HTML entrypoints load `<script type="module" ...>`.
// Both are real consumers outside the import graph.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-p6-framework-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-p6-framework-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-p6-framework",
        type: "module",
        workspaces: ["docs", "app"],
      }),
    );
    write(
      fx,
      "docs/package.json",
      JSON.stringify({
        name: "docs",
        type: "module",
        devDependencies: { vitepress: "1.0.0" },
      }),
    );
    write(
      fx,
      "docs/.vitepress/config.ts",
      `export default { title: 'Docs' };\n`,
    );
    write(
      fx,
      "docs/.vitepress/theme/index.ts",
      `export default { enhanceApp() {} };\n`,
    );
    write(
      fx,
      "docs/.vitepress/contributors.ts",
      `export const teamMembers = [];\n`,
    );
    write(
      fx,
      "app/package.json",
      JSON.stringify({ name: "app", type: "module" }),
    );
    write(
      fx,
      "app/index.html",
      `<div id="app"></div>\n<script type="module" src="/src/main.ts"></script>\n`,
    );
    write(fx, "app/src/main.ts", `export default { mounted: true };\n`);
    write(fx, "app/src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const fixPlan = readFixPlan(out);
    const visibleSymbols = cleanupIdentities(fixPlan);
    const mutedConfig = fixPlan.muted.find(
      (s) => s.finding.file === "docs/.vitepress/config.ts",
    );
    const mutedTheme = fixPlan.muted.find(
      (s) => s.finding.file === "docs/.vitepress/theme/index.ts",
    );
    const mutedMain = fixPlan.muted.find(
      (s) => s.finding.file === "app/src/main.ts",
    );
    const cleanupContrib = findCleanup(
      fixPlan,
      (s) => s.finding.file === "docs/.vitepress/contributors.ts",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) => s.finding.file === "app/src/private.ts",
    );

    assert(
      "CASE-P6-1c.1. VitePress config is MUTED by convention",
      mutedConfig?.evidence?.policy?.reason === "vitePress_FP46",
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1c.2. VitePress theme index is MUTED by convention",
      mutedTheme?.evidence?.policy?.reason === "vitePress_FP46",
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1c.3. HTML module main entrypoint is MUTED with evidence",
      mutedMain?.evidence?.policy?.reason === "htmlEntrypoint_FP47" &&
        mutedMain.evidence.policy.evidence?.some(
          (e) =>
            e.source === "html-module-script" &&
            e.htmlFile === "app/index.html",
        ),
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-P6-1c.4. unrelated .vitepress helper is still review-visible",
      cleanupContrib &&
        visibleSymbols.has("docs/.vitepress/contributors.ts::teamMembers"),
      `cleanup candidates=${[...visibleSymbols].join(", ")}`,
    );
    assert(
      "CASE-P6-1c.5. unrelated app private export remains review-visible",
      cleanupDead && cleanupDead.finding.symbol === "trulyDead",
      `cleanup candidates=${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FRAMEWORK-POLICY-1 — framework mutes are package-scoped
// ═════════════════════════════════════════════════════════════
// Framework policy must require both package-scoped activation evidence
// and a concrete protected convention. A root Next dependency protects
// the root app router and root/src proxy files, but must not leak into a
// nested workspace package with its own package.json. Nested app/*
// middleware-shaped files are also review-visible unless they match the
// top-level Next middleware/proxy convention.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-framework-policy-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-framework-policy-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-framework-policy",
        type: "module",
        dependencies: { next: "15.0.0" },
        workspaces: ["packages/*"],
      }),
    );
    write(fx, "app/page.tsx", `export function RootPage() { return null; }\n`);
    write(
      fx,
      "proxy.ts",
      `export function proxy() { return Response.json({ ok: true }); }\n` +
        `export const config = { matcher: ['/api/:path*'] };\n`,
    );
    write(
      fx,
      "app/foo/middleware.ts",
      `export function middleware() { return null; }\n`,
    );
    write(
      fx,
      "src/instrumentation-client.ts",
      `export default function clientInstrumentation() {}\n`,
    );
    write(
      fx,
      "packages/tool/package.json",
      JSON.stringify({
        name: "@fixture/tool",
        type: "module",
      }),
    );
    write(
      fx,
      "packages/tool/app/page.tsx",
      `export function ToolPage() { return null; }\n`,
    );

    runProductionPipeline(fx, out);
    const classify = readClassify(out);
    const fixPlan = readFixPlan(out);
    const visible = [
      ...fixPlan.safeFixes,
      ...fixPlan.reviewFixes,
      ...fixPlan.degraded,
    ];
    const mutedSymbol = (file, symbol) =>
      fixPlan.muted.find(
        (s) => s.finding.file === file && s.finding.symbol === symbol,
      );
    const visibleSymbol = (file, symbol) =>
      visible.find(
        (s) => s.finding.file === file && s.finding.symbol === symbol,
      );

    assert(
      "CASE-FRAMEWORK-POLICY-1.1. root Next app route is MUTED by framework policy",
      mutedSymbol("app/page.tsx", "RootPage")?.evidence?.policy?.reason ===
        "frameworkSentinel_FP27",
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-1.2. root Next proxy export is MUTED by framework policy",
      mutedSymbol("proxy.ts", "proxy")?.evidence?.policy?.reason ===
        "frameworkSentinel_FP27",
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-1.3. nested non-Next app route stays visible",
      !mutedSymbol("packages/tool/app/page.tsx", "ToolPage") &&
        visibleSymbol("packages/tool/app/page.tsx", "ToolPage"),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-1.4. nested app middleware path stays visible",
      !mutedSymbol("app/foo/middleware.ts", "middleware") &&
        visibleSymbol("app/foo/middleware.ts", "middleware"),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-1.5. phase-1 framework counters are emitted",
      classify.summary.frameworkPolicy?.mutedFindings?.next >= 2 &&
        classify.summary.frameworkPolicy?.reviewHintFindings?.next >= 1 &&
        classify.summary.frameworkPolicy?.pathShapedCandidatesKeptVisible
          ?.middleware >= 1,
      JSON.stringify(classify.summary.frameworkPolicy),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FRAMEWORK-POLICY-2 — export-level framework conventions
// ═════════════════════════════════════════════════════════════
// The matrix is intentionally export-aware. Route files may contain both
// framework-consumed exports and ordinary helpers; only the protected
// export names should become MUTED.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-framework-policy-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-framework-policy-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-framework-policy-exports",
        type: "module",
        workspaces: ["packages/*"],
      }),
    );
    write(
      fx,
      "packages/hono/package.json",
      JSON.stringify({
        name: "@fixture/hono-app",
        type: "module",
        dependencies: { hono: "^4.0.0" },
      }),
    );
    write(
      fx,
      "packages/hono/src/server.ts",
      `import { Hono } from 'hono';\n` +
        `const app = new Hono();\n` +
        `export function health(c) { return c.text('ok'); }\n` +
        `export function looseHelper() { return 'loose'; }\n` +
        `app.get('/health', health);\n`,
    );
    write(
      fx,
      "packages/svelte/package.json",
      JSON.stringify({
        name: "@fixture/svelte-app",
        type: "module",
        dependencies: { "@sveltejs/kit": "^2.0.0" },
      }),
    );
    write(
      fx,
      "packages/svelte/src/routes/blog/[slug]/+page.server.ts",
      `export function load() { return {}; }\n` +
        `export const entries = () => [{ slug: 'a' }];\n` +
        `export function privateHelper() { return 1; }\n`,
    );
    write(
      fx,
      "packages/astro/package.json",
      JSON.stringify({
        name: "@fixture/astro-app",
        type: "module",
        dependencies: { astro: "^5.0.0" },
      }),
    );
    write(
      fx,
      "packages/astro/src/pages/api/user.ts",
      `export function GET() { return new Response('ok'); }\n` +
        `export function helper() { return 'helper'; }\n`,
    );
    write(
      fx,
      "packages/router/package.json",
      JSON.stringify({
        name: "@fixture/router-app",
        type: "module",
        dependencies: { "@react-router/dev": "^7.0.0" },
      }),
    );
    write(
      fx,
      "packages/router/app/routes/home.tsx",
      `export async function loader() { return null; }\n` +
        `export async function clientLoader() { return null; }\n`,
    );

    runProductionPipeline(fx, out);
    const classify = readClassify(out);
    const fixPlan = readFixPlan(out);
    const visible = [
      ...fixPlan.safeFixes,
      ...fixPlan.reviewFixes,
      ...fixPlan.degraded,
    ];
    const mutedSymbol = (file, symbol) =>
      fixPlan.muted.find(
        (s) => s.finding.file === file && s.finding.symbol === symbol,
      );
    const visibleSymbol = (file, symbol) =>
      visible.find(
        (s) => s.finding.file === file && s.finding.symbol === symbol,
      );

    assert(
      "CASE-FRAMEWORK-POLICY-2.1. Hono local route handler is MUTED by route fact",
      mutedSymbol("packages/hono/src/server.ts", "health")?.evidence?.policy
        ?.reason === "frameworkSentinel_FP27",
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2.2. unrelated Hono helper remains visible",
      visibleSymbol("packages/hono/src/server.ts", "looseHelper") &&
        !mutedSymbol("packages/hono/src/server.ts", "looseHelper"),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2.3. SvelteKit load and dynamic entries are MUTED",
      mutedSymbol(
        "packages/svelte/src/routes/blog/[slug]/+page.server.ts",
        "load",
      ) &&
        mutedSymbol(
          "packages/svelte/src/routes/blog/[slug]/+page.server.ts",
          "entries",
        ),
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2.4. ordinary SvelteKit route helper remains visible",
      visibleSymbol(
        "packages/svelte/src/routes/blog/[slug]/+page.server.ts",
        "privateHelper",
      ) &&
        !mutedSymbol(
          "packages/svelte/src/routes/blog/[slug]/+page.server.ts",
          "privateHelper",
        ),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2.5. Astro endpoint GET is MUTED but helper remains visible",
      mutedSymbol("packages/astro/src/pages/api/user.ts", "GET") &&
        visibleSymbol("packages/astro/src/pages/api/user.ts", "helper") &&
        !mutedSymbol("packages/astro/src/pages/api/user.ts", "helper"),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2.6. React Router loader is MUTED while clientLoader is review-visible",
      mutedSymbol("packages/router/app/routes/home.tsx", "loader") &&
        visibleSymbol("packages/router/app/routes/home.tsx", "clientLoader") &&
        !mutedSymbol("packages/router/app/routes/home.tsx", "clientLoader") &&
        classify.summary.frameworkPolicy?.reviewHintFindings?.[
          "react-router"
        ] >= 1,
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}\nframeworkPolicy=${JSON.stringify(classify.summary.frameworkPolicy)}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FRAMEWORK-POLICY-2b — non-Next nested packages are discovered
// ═════════════════════════════════════════════════════════════
// WT-19 is not a Next-only contract. If a scanned file lives under a
// nested package.json outside the declared workspace globs, framework
// sentinel evidence must still be scoped to that nearest package. This
// guards SvelteKit/Astro package owner discovery without relying on
// Next.js app-router conventions.
{
  const fx = mkdtempSync(
    path.join(tmpdir(), "corpus-framework-policy-non-next-"),
  );
  const out = mkdtempSync(
    path.join(tmpdir(), "corpus-framework-policy-non-next-out-"),
  );
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-framework-policy-non-next",
        type: "module",
        workspaces: ["packages/*"],
      }),
    );
    write(
      fx,
      "apps/svelte/package.json",
      JSON.stringify({
        name: "@fixture/nested-svelte",
        type: "module",
        dependencies: { "@sveltejs/kit": "^2.0.0" },
      }),
    );
    write(
      fx,
      "apps/svelte/src/routes/blog/[slug]/+page.server.ts",
      `export function load() { return {}; }\n` +
        `export const entries = () => [{ slug: 'a' }];\n` +
        `export function privateHelper() { return 1; }\n`,
    );
    write(
      fx,
      "apps/astro/package.json",
      JSON.stringify({
        name: "@fixture/nested-astro",
        type: "module",
        dependencies: { astro: "^5.0.0" },
      }),
    );
    write(
      fx,
      "apps/astro/src/pages/api/user.ts",
      `export function GET() { return new Response('ok'); }\n` +
        `export function helper() { return 'helper'; }\n`,
    );

    runProductionPipeline(fx, out);
    const fixPlan = readFixPlan(out);
    const visible = [
      ...fixPlan.safeFixes,
      ...fixPlan.reviewFixes,
      ...fixPlan.degraded,
    ];
    const mutedSymbol = (file, symbol) =>
      fixPlan.muted.find(
        (s) => s.finding.file === file && s.finding.symbol === symbol,
      );
    const visibleSymbol = (file, symbol) =>
      visible.find(
        (s) => s.finding.file === file && s.finding.symbol === symbol,
      );

    const svelteLoad = mutedSymbol(
      "apps/svelte/src/routes/blog/[slug]/+page.server.ts",
      "load",
    );
    const svelteEntries = mutedSymbol(
      "apps/svelte/src/routes/blog/[slug]/+page.server.ts",
      "entries",
    );
    const astroGet = mutedSymbol("apps/astro/src/pages/api/user.ts", "GET");

    assert(
      "CASE-FRAMEWORK-POLICY-2b.1. non-workspace nested SvelteKit load is MUTED by nearest package",
      svelteLoad?.evidence?.policy?.reason === "frameworkSentinel_FP27" &&
        svelteLoad.evidence.policy.evidence?.packageRoot === "apps/svelte" &&
        svelteLoad.evidence.policy.evidence?.activation?.includes(
          "dependency:@sveltejs/kit",
        ),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2b.2. non-workspace nested SvelteKit dynamic entries are MUTED",
      svelteEntries?.evidence?.policy?.reason === "frameworkSentinel_FP27" &&
        svelteEntries.evidence.policy.evidence?.packageRoot === "apps/svelte",
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2b.3. ordinary nested SvelteKit route helper remains visible",
      visibleSymbol(
        "apps/svelte/src/routes/blog/[slug]/+page.server.ts",
        "privateHelper",
      ) &&
        !mutedSymbol(
          "apps/svelte/src/routes/blog/[slug]/+page.server.ts",
          "privateHelper",
        ),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2b.4. non-workspace nested Astro endpoint GET is MUTED by nearest package",
      astroGet?.evidence?.policy?.reason === "frameworkSentinel_FP27" &&
        astroGet.evidence.policy.evidence?.packageRoot === "apps/astro" &&
        astroGet.evidence.policy.evidence?.activation?.includes(
          "dependency:astro",
        ),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
    assert(
      "CASE-FRAMEWORK-POLICY-2b.5. ordinary nested Astro helper remains visible",
      visibleSymbol("apps/astro/src/pages/api/user.ts", "helper") &&
        !mutedSymbol("apps/astro/src/pages/api/user.ts", "helper"),
      `muted=${JSON.stringify(fixPlan.muted)}\nvisible=${JSON.stringify(visible)}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-FP30-SCOPE — h3/Nest-style middleware dirs are not Nuxt/Nitro
// ═════════════════════════════════════════════════════════════
// FP-30 protects Nuxt/Nitro filesystem-routed files. A bare `h3`
// dependency is not enough to mute every `middleware/` or `plugins/`
// directory in a non-Nuxt app; those names are common in NestJS and
// other server projects where exports may be ordinary review candidates.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-fp30-scope-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-fp30-scope-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "corpus-fp30-scope",
        type: "module",
        dependencies: {
          "@nestjs/common": "10.0.0",
          "@nuxt/opencollective": "0.4.1",
          h3: "1.0.0",
        },
      }),
    );
    write(
      fx,
      "middleware/utils.ts",
      `export function mapToExcludeRoute(route: string) { return route.trim(); }\n` +
        `export function isMiddlewareClass(value: unknown) { return typeof value === 'function'; }\n`,
    );
    write(
      fx,
      "src/nest-application.ts",
      `import { mapToExcludeRoute } from '../middleware/utils';\n` +
        `export const route = mapToExcludeRoute('/health');\n`,
    );

    runProductionPipeline(fx, out);
    const classify = readClassify(out);
    const fixPlan = readFixPlan(out);
    const mutedMiddleware = fixPlan.muted.find(
      (s) =>
        s.finding.file === "middleware/utils.ts" &&
        s.finding.symbol === "isMiddlewareClass",
    );
    const visibleMiddleware = [
      ...fixPlan.safeFixes,
      ...fixPlan.reviewFixes,
      ...fixPlan.degraded,
    ].find(
      (s) =>
        s.finding.file === "middleware/utils.ts" &&
        s.finding.symbol === "isMiddlewareClass",
    );

    assert(
      "CASE-FP30-SCOPE.1. h3 alone does not activate Nuxt/Nitro mute policy",
      classify.summary.excluded.nuxtNitro_FP30 === 0,
      JSON.stringify(classify.summary.excluded),
    );
    assert(
      "CASE-FP30-SCOPE.2. Nest-style middleware helper is not MUTED as nuxtNitro_FP30",
      !mutedMiddleware,
      `muted=${JSON.stringify(fixPlan.muted)}`,
    );
    assert(
      "CASE-FP30-SCOPE.3. ordinary middleware helper remains review-visible",
      !!visibleMiddleware &&
        visibleMiddleware.evidence?.policy?.reason !== "nuxtNitro_FP30",
      `visible=${JSON.stringify(visibleMiddleware)}\nmuted=${JSON.stringify(fixPlan.muted)}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-DECL-1 — exported class/const signatures protect type dependencies
// ═════════════════════════════════════════════════════════════
// A type that is only used by exported declarations in the same file is safe
// to demote, not delete. Class fields/method signatures and exported variable
// type annotations can depend on the local binding even without cross-file
// fan-in.
{
  const fx = mkdtempSync(path.join(tmpdir(), "corpus-decl-surface-"));
  const out = mkdtempSync(path.join(tmpdir(), "corpus-decl-surface-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "corpus-decl-surface", type: "module" }),
    );
    write(
      fx,
      "src/api.ts",
      `export interface HiddenResult { id: string }\n` +
        `export class PublicRuntime {\n` +
        `  value!: HiddenResult;\n` +
        `  get(): HiddenResult {\n` +
        `    const local: HiddenResult = { id: 'x' };\n` +
        `    return local;\n` +
        `  }\n` +
        `}\n` +
        `export const makePublic: () => HiddenResult = () => {\n` +
        `  const local: HiddenResult = { id: 'x' };\n` +
        `  return local;\n` +
        `};\n`,
    );
    write(fx, "src/private.ts", `export const trulyDead = 1;\n`);

    runPipeline(fx, out);
    const fixPlan = readFixPlan(out);
    const cleanupHidden = findCleanup(
      fixPlan,
      (s) =>
        s.finding.file === "src/api.ts" && s.finding.symbol === "HiddenResult",
    );
    const degradedHidden = fixPlan.degraded.find(
      (s) =>
        s.finding.file === "src/api.ts" && s.finding.symbol === "HiddenResult",
    );
    const cleanupDead = findCleanup(
      fixPlan,
      (s) =>
        s.finding.file === "src/private.ts" && s.finding.symbol === "trulyDead",
    );

    assert(
      "CASE-DECL-1.1. exported class/const signature type is demote-only SAFE_FIX",
      cleanupHidden?.finding?.declarationExportDependency === true &&
        cleanupHidden.finding.safeAction?.kind ===
          "demote_export_declaration" &&
        cleanupHidden.finding.safeAction?.strongerActionBlockers?.includes(
          "local-refs-present",
        ),
      `cleanup candidates=${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
    assert(
      "CASE-DECL-1.2. exported class/const signature type is not DEGRADED when demote preserves binding",
      !degradedHidden,
      `degraded=${JSON.stringify(fixPlan.degraded.map((s) => ({ reason: s.reason, finding: s.finding })))}`,
    );
    assert(
      "CASE-DECL-1.3. unrelated private export remains review-visible",
      cleanupDead,
      `cleanup candidates=${JSON.stringify(cleanupEntries(fixPlan).map((s) => s.finding))}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// FP BUDGET GATE
// ═════════════════════════════════════════════════════════════
// Every assertion above is a precision invariant. Budget is zero —
// if any of them fail, the corpus blocks the release. Bumping the
// budget silently papers over a regression; fix the cause instead.
const FP_BUDGET = 0;
it("FP budget gate. precision failures stay within zero budget", () => {
  expect(precisionFailures).toBeLessThanOrEqual(FP_BUDGET);
});
