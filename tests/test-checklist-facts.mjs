// Regression test for `checklist-facts.mjs` — the pre-compute layer
// for templates/REVIEW_CHECKLIST.md.
//
// Asserts the JSON artifact:
//   - reports A2/E2 from a fresh AST pass (no pipeline prereq)
//   - degrades gracefully on missing artifacts (gate: 'unknown',
//     available: false) — NEVER crashes
//   - lists `_not_computed` items explicitly so the checklist walker
//     can't silently skip them

import { execSync } from 'node:child_process';
import {
  writeFileSync, readFileSync, mkdirSync, rmSync, mkdtempSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(script, args) {
  execSync(`"${NODE}" "${path.join(DIR, script)}" ${args.map((a) => `"${a}"`).join(' ')}`, {
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

// ═════════════════════════════════════════════════════════════
// CASE-DEGRADES-CLEANLY — no pipeline artifacts present
// checklist-facts must NOT crash on a bare output dir;
// AST-backed items still work, artifact-backed items mark
// `available: false` with a human-readable `reason`.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-bare-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-bare-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-bare', type: 'module' }));
    write(fx, 'src/ok.ts', `export const trivial = 1;\n`);

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-DEGRADES.1. meta.schemaVersion set',
      typeof cf.meta?.schemaVersion === 'number' && cf.meta.schemaVersion >= 1);
    assert('CASE-DEGRADES.2. A2 computed (fresh AST pass, no prereq)',
      cf.A2_function_size?.gate === 'ok' && cf.A2_function_size?.buckets?.big === 0);
    assert('CASE-DEGRADES.3. A5 degrades cleanly when topology.json absent',
      cf.A5_decoupling_ratio?.available === false &&
      cf.A5_decoupling_ratio?.gate === 'unknown');
    assert('CASE-DEGRADES.4. A6 same',
      cf.A6_circular_deps?.available === false);
    assert('CASE-DEGRADES.5. B3 degrades cleanly when fix-plan.json absent',
      cf.B3_dead_code?.available === false);
    assert('CASE-DEGRADES.6. C5 degrades cleanly when triage.json absent',
      cf.C5_lint_enforcement?.available === false);
    assert('CASE-DEGRADES.7. C7 degrades cleanly when barrels.json absent',
      cf.C7_barrel_amplification?.available === false);
    assert('CASE-DEGRADES.7b. B1/B2 shape drift degrades cleanly when shape-index.json absent',
      cf.B1B2_shape_drift?.available === false &&
      cf.B1B2_shape_drift?.gate === 'unknown',
      JSON.stringify(cf.B1B2_shape_drift));
    assert('CASE-DEGRADES.8. E2 computed (fresh AST pass)',
      cf.E2_silent_catch?.count === 0 &&
      cf.E2_silent_catch?.gate === 'ok' &&
      cf.E2_silent_catch?.analysis === 'oxc-ast-catch-clause');
    assert('CASE-DEGRADES.9. _not_computed lists >= 20 items so walker sees them',
      Array.isArray(cf._not_computed) && cf._not_computed.length >= 20);

    // v1.10.3 schema additions — citation hints + context-check flags.
    assert('CASE-DEGRADES.10. schemaVersion >= 2 after v1.10.3 hint wiring',
      cf.meta.schemaVersion >= 2);
    assert('CASE-DEGRADES.11. A2 carries a citation hint reviewer can copy',
      typeof cf.A2_function_size._citation_hint === 'string' &&
      cf.A2_function_size._citation_hint.startsWith('[grounded,'));
    assert('CASE-DEGRADES.12. A6 _context_check_required === false (structural gate)',
      cf.A6_circular_deps._context_check_required === false);
    assert('CASE-DEGRADES.13. A2 _context_check_required === true (threshold gate)',
      cf.A2_function_size._context_check_required === true);
    assert('CASE-DEGRADES.14. missing-input citation labels as [확인 불가] with scan range',
      typeof cf.A5_decoupling_ratio._citation_hint === 'string' &&
      cf.A5_decoupling_ratio._citation_hint.includes('확인 불가') &&
      cf.A5_decoupling_ratio._citation_hint.includes('scan range'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-A2 — trigger the function-size gate
// A 160-LOC function should land in `oversized[]` and push gate
// to at least `watch`.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-a2-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-a2-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-a2', type: 'module' }));
    // Build 160-LOC functions. 160 > 150 → big bucket; 100-150 → medium.
    const body = Array.from({ length: 160 }, (_, i) => `  const x${i} = ${i};`).join('\n');
    write(fx, 'src/huge.ts',
      `export function huge() {\n${body}\n  return 0;\n}\n`);
    write(fx, 'tests/huge.test.ts',
      `export function testHuge() {\n${body}\n  return 0;\n}\n`);
    write(fx, 'scripts/huge-smoke.mjs',
      `export function scriptHuge() {\n${body}\n  return 0;\n}\n`);

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-A2.1. oversized has at least one entry',
      cf.A2_function_size.oversized.length >= 1,
      `got ${cf.A2_function_size.oversized.length} oversized`);
    const huge = cf.A2_function_size.oversized.find((o) => o.name === 'huge');
    assert('CASE-A2.2. `huge` is surfaced by name',
      !!huge, `oversized: ${JSON.stringify(cf.A2_function_size.oversized.map((o) => o.name))}`);
    assert('CASE-A2.3. `huge` loc > 150 recorded',
      huge && huge.loc > 150, `loc=${huge?.loc}`);
    assert('CASE-A2.4. gate at least `watch`',
      cf.A2_function_size.gate === 'watch' || cf.A2_function_size.gate === 'fix',
      `gate=${cf.A2_function_size.gate}, buckets=${JSON.stringify(cf.A2_function_size.buckets)}`);
    assert('CASE-A2.5. oversized entries carry production/test/script roles',
      cf.A2_function_size.oversizedByRole.production.some((o) => o.name === 'huge') &&
      cf.A2_function_size.oversizedByRole.test.some((o) => o.name === 'testHuge') &&
      cf.A2_function_size.oversizedByRole.script.some((o) => o.name === 'scriptHuge'),
      JSON.stringify(cf.A2_function_size.oversizedByRole));
    assert('CASE-A2.6. roleBuckets count big functions separately',
      cf.A2_function_size.roleBuckets.production.big === 1 &&
      cf.A2_function_size.roleBuckets.test.big === 1 &&
      cf.A2_function_size.roleBuckets.script.big === 1,
      JSON.stringify(cf.A2_function_size.roleBuckets));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-A5 — full cross-edge list + healthy layered flow
// High cross-submodule ratios are not automatically unhealthy when every
// edge flows from entry/test/script surfaces into the `_lib` engine.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-a5-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-a5-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-a5', type: 'module' }));
    write(fx, 'src/ok.ts', `export const ok = 1;\n`);
    write(out, 'topology.json', JSON.stringify({
      summary: { internalEdges: 100 },
      crossSubmoduleEdges: [
        { from: 'root', to: '_lib', count: 60 },
        { from: 'tests', to: '_lib', count: 20 },
      ],
      crossSubmoduleTop: [
        { edge: 'root → _lib', count: 60 },
      ],
      sccs: [],
    }));

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-A5.1. A5 prefers full crossSubmoduleEdges over display top-30',
      cf.A5_decoupling_ratio.crossSubmoduleEdgeSource === 'full-list' &&
      cf.A5_decoupling_ratio.crossSubmoduleEdgesSum === 80,
      JSON.stringify(cf.A5_decoupling_ratio));
    assert('CASE-A5.2. healthy layered flow downgrades raw fix to ok',
      cf.A5_decoupling_ratio.rawGate === 'fix' &&
      cf.A5_decoupling_ratio.gate === 'ok' &&
      cf.A5_decoupling_ratio.reviewedEdgesSum === 0,
      JSON.stringify(cf.A5_decoupling_ratio));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-A5-INVERSION — unhealthy direction still trips the gate
// The downgrade is narrow: engine → root remains a structural smell.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-a5-inv-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-a5-inv-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-a5-inv', type: 'module' }));
    write(fx, 'src/ok.ts', `export const ok = 1;\n`);
    write(out, 'topology.json', JSON.stringify({
      summary: { internalEdges: 100 },
      crossSubmoduleEdges: [
        { from: '_lib', to: 'root', count: 60 },
      ],
      sccs: [],
    }));

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-A5-INVERSION.1. non-layered edge keeps fix gate',
      cf.A5_decoupling_ratio.rawGate === 'fix' &&
      cf.A5_decoupling_ratio.gate === 'fix' &&
      cf.A5_decoupling_ratio.reviewedEdgesSum === 60,
      JSON.stringify(cf.A5_decoupling_ratio));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-B1B2 — exact exported shape drift is an observation cue
// shape-index.json can ground exact duplicate type shapes, but the
// checklist still treats broader duplication/shape drift as LLM judgment.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-b1b2-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-b1b2-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-b1b2', type: 'module' }));
    write(fx, 'src/web.ts',
      `export interface SubagentActivityState { id: string; status: 'idle' | 'running' }\n`);
    write(fx, 'src/daemon.ts',
      `export type DaemonActivityView = { status: 'idle' | 'running'; id: string };\n`);
    write(fx, 'src/other.ts',
      `export interface DifferentShape { id: number; status: string }\n`);

    run('build-shape-index.mjs', ['--root', fx, '--output', out]);
    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));
    const shape = cf.B1B2_shape_drift;

    assert('CASE-B1B2.1. shape-index input is recorded',
      cf.meta.inputsPresent['shape-index.json'] === true,
      JSON.stringify(cf.meta.inputsPresent));
    assert('CASE-B1B2.2. exact duplicate shape group trips watch, not fix',
      shape.gate === 'watch' &&
      shape.exactDuplicateGroups === 1 &&
      shape.duplicateIdentityCount === 2,
      JSON.stringify(shape));
    assert('CASE-B1B2.3. top group keeps concrete identities and fields',
      shape.topGroups[0]?.identities.includes('src/web.ts::SubagentActivityState') &&
      shape.topGroups[0]?.identities.includes('src/daemon.ts::DaemonActivityView') &&
      shape.topGroups[0]?.fieldNames.includes('id') &&
      shape.topGroups[0]?.fieldNames.includes('status'),
      JSON.stringify(shape.topGroups));
    assert('CASE-B1B2.4. citation hint names B1B2_shape_drift counts',
      shape._citation_hint.includes('B1B2_shape_drift.exactDuplicateGroups = 1') &&
      shape._citation_hint.includes('nearShapeCandidateCount') &&
      shape._context_check_required === true,
      shape._citation_hint);
    assert('CASE-B1B2.5. broader B1/B2 remain in _not_computed as judgment',
      cf._not_computed.some((item) => item.item === 'B1' && item.reason.includes('function clone cues')) &&
      cf._not_computed.some((item) => item.item === 'B2' && item.reason.includes('domain/vocab judgment')),
      JSON.stringify(cf._not_computed.filter((item) => item.item === 'B1' || item.item === 'B2')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-B1B2-NEAR — near exported shapes surface as review cues.
// This is deliberately not a semantic verdict: field/name overlap is
// enough to ask for review, never enough to auto-merge types.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-b1b2-near-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-b1b2-near-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-b1b2-near', type: 'module' }));
    write(fx, 'src/activity-state.ts',
      `export interface SubagentActivityState {\n` +
      `  id: string;\n` +
      `  status: 'idle' | 'running';\n` +
      `  updatedAt: string;\n` +
      `}\n`);
    write(fx, 'src/activity-view.ts',
      `export interface SubagentActivityView {\n` +
      `  id: string;\n` +
      `  status: 'idle' | 'running';\n` +
      `  label: string;\n` +
      `}\n`);
    write(fx, 'src/unrelated.ts',
      `export interface BuildResult {\n` +
      `  ok: boolean;\n` +
      `  durationMs: number;\n` +
      `}\n`);

    run('build-shape-index.mjs', ['--root', fx, '--output', out]);
    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));
    const shape = cf.B1B2_shape_drift;
    const near = shape.nearShapeCandidates?.[0];

    assert('CASE-B1B2-NEAR.1. near-only shape drift trips watch without exact duplicate',
      shape.gate === 'watch' &&
      shape.exactDuplicateGroups === 0 &&
      shape.nearShapeCandidateCount === 1,
      JSON.stringify(shape));
    assert('CASE-B1B2-NEAR.2. near candidate carries identities and shared fields',
      near?.identities.includes('src/activity-state.ts::SubagentActivityState') &&
      near?.identities.includes('src/activity-view.ts::SubagentActivityView') &&
      near?.sharedFieldNames.includes('id') &&
      near?.sharedFieldNames.includes('status') &&
      near?.fieldJaccard >= 0.5,
      JSON.stringify(near));
    assert('CASE-B1B2-NEAR.3. near candidate is explicitly a review cue, not proof',
      /review cue/.test(near?.reason ?? '') &&
      /not proof/.test(near?.reason ?? ''),
      JSON.stringify(near));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-B1-FUNCTION-CLONES — normalized helper clone cues are grounded
// observations, but still not semantic merge verdicts.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-b1-fn-clones-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-b1-fn-clones-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-b1-fn-clones', type: 'module' }));
    write(fx, 'src/a.ts',
      `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
      `  const dollars = cents / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
      `}\n`);
    write(fx, 'src/b.ts',
      `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
      `  const amount = value / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);\n` +
      `}\n`);

    run('build-function-clone-index.mjs', ['--root', fx, '--output', out]);
    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));
    const clones = cf.B1_duplicate_implementation;

    assert('CASE-B1-FUNCTION-CLONES.1. function-clones input is recorded',
      cf.meta.inputsPresent['function-clones.json'] === true,
      JSON.stringify(cf.meta.inputsPresent));
    assert('CASE-B1-FUNCTION-CLONES.2. same-structure function group trips watch, not fix',
      clones.gate === 'watch' &&
      clones.structureGroupCandidates === 1 &&
      clones.candidateIdentityCount === 2,
      JSON.stringify(clones));
    assert('CASE-B1-FUNCTION-CLONES.3. top group names both concrete helper identities',
      clones.topStructureGroups[0]?.identities.includes('src/a.ts::formatCurrencyCents') &&
      clones.topStructureGroups[0]?.identities.includes('src/b.ts::renderPaymentTotal') &&
      /not proof of semantic equivalence/.test(clones.topStructureGroups[0]?.reason ?? ''),
      JSON.stringify(clones.topStructureGroups));
    assert('CASE-B1-FUNCTION-CLONES.4. citation hint names B1 duplicate implementation counts',
      clones._citation_hint.includes('B1_duplicate_implementation.exactBodyGroups') &&
      clones._citation_hint.includes('structureGroupCandidates') &&
      clones._context_check_required === true,
      clones._citation_hint);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-B1-FUNCTION-NEAR — near function candidates are grounded
// review-only cues distinct from exact/structure clone groups.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-b1-fn-near-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-b1-fn-near-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-b1-fn-near', type: 'module' }));
    write(fx, 'src/date-a.ts',
      `export function formatDate(value: Date) {\n` +
      `  const formatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' });\n` +
      `  return formatter.format(value);\n` +
      `}\n`);
    write(fx, 'src/date-b.ts',
      `export function dateFormat(input: Date) {\n` +
      `  return new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' }).format(input);\n` +
      `}\n`);

    run('build-function-clone-index.mjs', ['--root', fx, '--output', out]);
    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));
    const clones = cf.B1_duplicate_implementation;
    const near = clones.topNearFunctionCandidates?.[0];

    assert('CASE-B1-FUNCTION-NEAR.1. near candidates trip watch without exact/structure groups',
      clones.gate === 'watch' &&
      clones.exactBodyGroups === 0 &&
      clones.structureGroupCandidates === 0 &&
      clones.nearFunctionCandidates === 1,
      JSON.stringify(clones));
    assert('CASE-B1-FUNCTION-NEAR.2. top near candidate is explicitly review-only',
      near?.identities.includes('src/date-a.ts::formatDate') &&
      near?.identities.includes('src/date-b.ts::dateFormat') &&
      near?.risk === 'review-only' &&
      /not proof of semantic equivalence/.test(near?.reason ?? ''),
      JSON.stringify(near));
    assert('CASE-B1-FUNCTION-NEAR.3. citation hint includes nearFunctionCandidates',
      clones._citation_hint.includes('nearFunctionCandidates') &&
      clones._context_check_required === true,
      clones._citation_hint);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-E2 — trigger the silent-catch gate
// Two undocumented empty catches in the fixture. Gate stays `watch`
// (count 1-3); four or more would flip to `fix`.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-e2-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-e2-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-e2', type: 'module' }));
    write(fx, 'src/a.ts',
      `export function a() {\n` +
      `  try { JSON.parse('x'); } catch {}\n` +
      `  try { JSON.parse('y'); } catch (e) {}\n` +
      `  try { JSON.parse('z'); } catch { /* intentionally optional */ }\n` +
      `  try { JSON.parse('q'); } catch (e) {\n` +
      `    // intentionally optional\n` +
      `  }\n` +
      `}\n`);
    write(fx, 'src/b.ts',
      `// Non-silent — should NOT be counted:\n` +
      `export function b() {\n` +
      `  try { JSON.parse('z'); } catch (e) { console.error(e); }\n` +
      `}\n`);

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-E2.1. silent catch count == 2 (both empty variants)',
      cf.E2_silent_catch.count === 2,
      `count=${cf.E2_silent_catch.count}, sites=${JSON.stringify(cf.E2_silent_catch.sites)}`);
    assert('CASE-E2.2. non-silent catch in b.ts is NOT counted',
      !cf.E2_silent_catch.sites.some((s) => s.file.endsWith('b.ts')));
    assert('CASE-E2.3. gate=`watch` at count 2',
      cf.E2_silent_catch.gate === 'watch',
      `gate=${cf.E2_silent_catch.gate}`);
    assert('CASE-E2.4. documented empty catches are separated from gate count',
      cf.E2_silent_catch.documentedCount === 2 &&
      cf.E2_silent_catch.documentedSites.every((s) => s.file.endsWith('a.ts')),
      JSON.stringify(cf.E2_silent_catch));
    assert('CASE-E2.5. anonymous catch clauses are counted separately from silent count',
      cf.E2_silent_catch.anonymousCount === 2 &&
      cf.E2_silent_catch.nonEmptyAnonymousCount === 0,
      JSON.stringify(cf.E2_silent_catch));
    assert('CASE-E2.6. E2 sites carry AST-backed localization details',
      cf.E2_silent_catch.analysis === 'oxc-ast-catch-clause' &&
      cf.E2_silent_catch.sites.every((s) =>
        s.fileRole === 'production' && typeof s.bodyStatementCount === 'number'),
      JSON.stringify(cf.E2_silent_catch));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-E2-ANON — non-empty anonymous catches are still worth watching
// A `catch { return null; }` is not an empty silent catch, but it still
// discards the error identity. Surface it so E2 doesn't report "ok" while
// anonymous catch sites exist.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-e2-anon-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-e2-anon-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-e2-anon', type: 'module' }));
    write(fx, 'src/a.ts',
      `export function a(raw: string) {\n` +
      `  try { return JSON.parse(raw); } catch { return null; }\n` +
      `}\n`);

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-E2-ANON.1. non-empty anonymous catch does not inflate empty silent count',
      cf.E2_silent_catch.count === 0,
      JSON.stringify(cf.E2_silent_catch));
    assert('CASE-E2-ANON.2. non-empty anonymous catch is surfaced as watch evidence',
      cf.E2_silent_catch.nonEmptyAnonymousCount === 1 &&
      cf.E2_silent_catch.anonymousCount === 1 &&
      cf.E2_silent_catch.gate === 'watch',
      JSON.stringify(cf.E2_silent_catch));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-E2-UNUSED-PARAM — a non-empty catch with an unused parameter
// is not an empty silent catch, but it still discards the error identity.
// Surface it with AST-backed evidence while leaving used parameters alone.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-e2-unused-param-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-e2-unused-param-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-e2-unused-param', type: 'module' }));
    write(fx, 'src/a.ts',
      `export function ignored(raw: string) {\n` +
      `  try { return JSON.parse(raw); } catch (err) { return null; }\n` +
      `}\n` +
      `export function logged(raw: string) {\n` +
      `  try { return JSON.parse(raw); } catch (err) { console.error(err); return null; }\n` +
      `}\n`);

    run('checklist-facts.mjs', ['--root', fx, '--output', out]);
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-E2-UNUSED.1. unused catch parameter does not inflate empty silent count',
      cf.E2_silent_catch.count === 0,
      JSON.stringify(cf.E2_silent_catch));
    assert('CASE-E2-UNUSED.2. unused catch parameter is surfaced as watch evidence',
      cf.E2_silent_catch.unusedParamCount === 1 &&
      cf.E2_silent_catch.unusedParamSites[0]?.paramName === 'err' &&
      cf.E2_silent_catch.gate === 'watch',
      JSON.stringify(cf.E2_silent_catch));
    assert('CASE-E2-UNUSED.3. catch parameter used in the body is not counted',
      cf.E2_silent_catch.unusedParamSites.length === 1 &&
      cf.E2_silent_catch.unusedParamSites[0]?.line === 2,
      JSON.stringify(cf.E2_silent_catch));
    assert('CASE-E2-UNUSED.4. citation hint names unusedParamCount',
      cf.E2_silent_catch._citation_hint.includes('unusedParamCount = 1') &&
      cf.E2_silent_catch._citation_hint.includes('analysis = oxc-ast-catch-clause'),
      cf.E2_silent_catch._citation_hint);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-C5 — no-restricted-imports counts as boundary evidence
// triage-repo must pass the rule through so checklist-facts can
// ground C5 without borrowing evidence from fixture/corpus configs.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-c5-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-c5-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-c5', type: 'module' }));
    write(fx, 'eslint.config.mjs',
      `export default [{\n` +
      `  rules: {\n` +
      `    'no-restricted-imports': ['error', { patterns: ['../*.mjs'] }],\n` +
      `  },\n` +
      `}];\n`);
    write(fx, 'src/ok.ts', `export const ok = 1;\n`);

    run('triage-repo.mjs', ['--root', fx, '--output', out]);
    run('checklist-facts.mjs', ['--root', fx, '--output', out]);

    const triage = JSON.parse(readFileSync(path.join(out, 'triage.json'), 'utf8'));
    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-C5.1. triage records no-restricted-imports boundary rule',
      triage.boundaries.some((b) => b.rule === 'no-restricted-imports' && b.file === 'eslint.config.mjs'),
      JSON.stringify(triage.boundaries));
    assert('CASE-C5.2. checklist C5 gate is ok from no-restricted-imports',
      cf.C5_lint_enforcement.gate === 'ok' &&
      cf.C5_lint_enforcement.boundaryRulePresent === true,
      JSON.stringify(cf.C5_lint_enforcement));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═════════════════════════════════════════════════════════════
// CASE-PIPELINE — run the whole pipeline, verify artifact-backed
// items (A5, A6, B3, C5, C7) populate from their sources.
// ═════════════════════════════════════════════════════════════
{
  const fx = mkdtempSync(path.join(tmpdir(), 'cf-pipe-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cf-pipe-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cf-pipe', type: 'module' }));
    write(fx, 'src/entry.ts',
      `import { helper } from './helper.js';\n` +
      `export const live = helper();\n`);
    write(fx, 'src/helper.ts',
      `export function helper() { return 1; }\n` +
      `export const maybeDead = 2;\n`);

    run('triage-repo.mjs',             ['--root', fx, '--output', out]);
    run('measure-topology.mjs',        ['--root', fx, '--output', out]);
    run('build-symbol-graph.mjs',      ['--root', fx, '--output', out]);
    run('classify-dead-exports.mjs',   ['--root', fx, '--output', out]);
    run('rank-fixes.mjs',              ['--root', fx, '--output', out]);
    run('check-barrel-discipline.mjs', ['--root', fx, '--output', out]);
    run('build-shape-index.mjs',       ['--root', fx, '--output', out]);
    run('checklist-facts.mjs',         ['--root', fx, '--output', out]);

    const cf = JSON.parse(readFileSync(path.join(out, 'checklist-facts.json'), 'utf8'));

    assert('CASE-PIPELINE.1. A5 available once topology.json exists',
      cf.A5_decoupling_ratio.available !== false &&
      typeof cf.A5_decoupling_ratio.ratioLowerBound === 'number');
    assert('CASE-PIPELINE.2. A6 reports no cycles for this linear fixture',
      cf.A6_circular_deps.sccCount === 0 && cf.A6_circular_deps.gate === 'ok');
    assert('CASE-PIPELINE.3. B3 available once fix-plan.json exists',
      typeof cf.B3_dead_code.total === 'number');
    assert('CASE-PIPELINE.4. C5 available once triage.json exists (even if no rules)',
      cf.C5_lint_enforcement.available !== false);
    assert('CASE-PIPELINE.5. C7 handles single-package mode',
      cf.C7_barrel_amplification.gate === 'ok');
    assert('CASE-PIPELINE.6. meta.inputsPresent flips the right bits',
      cf.meta.inputsPresent['topology.json'] === true &&
      cf.meta.inputsPresent['fix-plan.json'] === true &&
      cf.meta.inputsPresent['triage.json'] === true &&
      cf.meta.inputsPresent['barrels.json'] === true &&
      cf.meta.inputsPresent['shape-index.json'] === true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
