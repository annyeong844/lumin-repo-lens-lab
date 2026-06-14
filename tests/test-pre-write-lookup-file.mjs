// Tests for _lib/pre-write-lookup-file.mjs — P1-2 step 5.1.
//
// Pinning rules from docs/history/phases/p1/p1-2.md §4.1 + §5.1:
//   - FILE_EXISTS requires POSITIVE evidence (topology node OR defIndex entry).
//   - NEW_FILE requires topology.meta.complete === true AND path absent from
//     topology.nodes AND path absent from symbols.filesWithParseErrors.
//   - symbols.defIndex absence alone NEVER yields NEW_FILE.
//   - Without topology.meta.complete: true, absence-from-topology → FILE_STATUS_UNKNOWN.
//   - boundary.status is ALWAYS 'NOT_EVALUATED' in P1-2 (no planned from→to edge).
//   - triage.json absent → boundary NOT_EVALUATED + [확인 불가, reason: triage absent].

import { lookupFile } from '../_lib/pre-write-lookup-file.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Fixture builders ─────────────────────────────────────────

function buildTopology({ nodes = {}, edges = [], complete = false } = {}) {
  return {
    meta: { tool: 'measure-topology.mjs', ...(complete ? { complete: true } : {}) },
    nodes,  // object form: { 'src/a.ts': { loc: 42 }, ... }
    edges,
  };
}

function buildSymbols({ defIndex = {}, filesWithParseErrors = [] } = {}) {
  return {
    meta: { schemaVersion: 3, supports: { anyContamination: false, identityFanIn: true, reExportRecords: 'file-level' } },
    defIndex,
    filesWithParseErrors,
  };
}

// ═══ FILE_EXISTS via topology ═══

{
  const topology = buildTopology({
    nodes: { 'src/utils/date.ts': { loc: 42 } },
    complete: true,
  });
  const symbols = buildSymbols();
  const r = lookupFile('src/utils/date.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T1. file listed in topology.nodes → FILE_EXISTS',
    r.result === 'FILE_EXISTS', `result=${r.result}`);
  assert('T1b. loc populated from topology',
    r.loc === 42);
}

// ═══ FILE_EXISTS via symbols.defIndex fallback ═══

{
  // No topology, but defIndex has the file.
  const symbols = buildSymbols({
    defIndex: { 'src/legacy/old.ts': { someExport: { kind: 'const', line: 1 } } },
  });
  const r = lookupFile('src/legacy/old.ts', { topology: null, symbols, triage: null, root: '/root' });
  assert('T2. topology absent + defIndex has file → FILE_EXISTS',
    r.result === 'FILE_EXISTS');
  assert('T2b. loc null when topology absent',
    r.loc === null);
  assert('T2c. inboundFanIn unavailable when topology absent',
    r.inboundFanIn === null && r.inboundFanInConfidence === 'unavailable');
}

// ═══ NEW_FILE — full contract ═══

{
  const topology = buildTopology({
    nodes: { 'src/existing.ts': { loc: 10 } },
    complete: true,
  });
  const symbols = buildSymbols({ filesWithParseErrors: [] });
  const r = lookupFile('src/utils/time.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T3. topology.meta.complete=true + absent + no parse error → NEW_FILE',
    r.result === 'NEW_FILE', `result=${r.result}`);
}

// NEW_FILE can still carry a domain-cluster warning when the same
// directory already contains a prefix family. This is the vibe-coder
// guard: "new file" does not mean "new domain".
{
  const topology = buildTopology({
    nodes: {
      'lib/cardNewsGenerator.js': { loc: 120 },
      'lib/cardNewsPlanner.js': { loc: 80 },
      'lib/cardNewsJobStore.js': { loc: 40 },
      'lib/other.js': { loc: 10 },
    },
    complete: true,
  });
  const symbols = buildSymbols({ filesWithParseErrors: [] });
  const r = lookupFile('lib/cardNewsService.js', { topology, symbols, triage: null, root: '/root' });
  assert('T3b. absent file with sibling prefix cluster still stays NEW_FILE',
    r.result === 'NEW_FILE', `result=${r.result}`);
  assert('T3c. domain cluster is detected from topology nodes',
    r.domainCluster?.kind === 'DOMAIN_CLUSTER_DETECTED' &&
    r.domainCluster.matchCount === 3 &&
    r.domainCluster.prefixPath === 'lib/cardNews',
    JSON.stringify(r.domainCluster));
  assert('T3d. domain cluster totals known LOC',
    r.domainCluster.totalLoc === 240,
    JSON.stringify(r.domainCluster));
}

// Suffix/token domain cluster: `artifact-loader.mjs` should notice a
// directory that already has `*-artifact.mjs` siblings. Prefix-only
// detection missed this real package-surface pattern.
{
  const topology = buildTopology({
    nodes: {
      '_lib/artifacts.mjs': { loc: 80 },
      '_lib/check-canon-artifact.mjs': { loc: 40 },
      '_lib/post-write-artifact.mjs': { loc: 30 },
      '_lib/pre-write-artifact.mjs': { loc: 30 },
      '_lib/shape-index-artifact.mjs': { loc: 120 },
      '_lib/symbol-graph-artifact.mjs': { loc: 100 },
      '_lib/other.mjs': { loc: 10 },
    },
    complete: true,
  });
  const symbols = buildSymbols({ filesWithParseErrors: [] });
  const r = lookupFile('_lib/artifact-loader.mjs', { topology, symbols, triage: null, root: '/root' });
  assert('T3e. suffix/token artifact domain cluster detected',
    r.result === 'NEW_FILE' &&
    r.domainCluster?.kind === 'DOMAIN_CLUSTER_DETECTED' &&
    r.domainCluster.matchKind === 'domain-token' &&
    r.domainCluster.matchCount === 6,
    JSON.stringify(r.domainCluster));
  assert('T3f. artifact cluster examples include suffix-form siblings',
    r.domainCluster.examples.some((e) => e.file === '_lib/post-write-artifact.mjs') &&
    r.domainCluster.examples.some((e) => e.file === '_lib/artifacts.mjs'),
    JSON.stringify(r.domainCluster.examples));
}

// Strong prefix sibling: `merge-with-defaults.util.ts` and
// `merge-with-values.util.ts` share a two-token domain prefix. Even one
// sibling is useful as a watch-for cue; semantic sameness is still NOT
// claimed.
{
  const topology = buildTopology({
    nodes: {
      'src/utils/merge-with-values.util.ts': { loc: 44 },
      'src/utils/deep-merge.util.ts': { loc: 25 },
    },
    complete: true,
  });
  const symbols = buildSymbols({ filesWithParseErrors: [] });
  const r = lookupFile('src/utils/merge-with-defaults.util.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T3g. strong two-token prefix sibling emits a domain watch-for cue',
    r.result === 'NEW_FILE' &&
    r.domainCluster?.kind === 'DOMAIN_CLUSTER_DETECTED' &&
    r.domainCluster.matchCount === 1 &&
    r.domainCluster.prefixPath === 'src/utils/mergeWith',
    JSON.stringify(r.domainCluster));
  assert('T3h. strong prefix sibling does not claim semantic deepMerge reuse',
    r.domainCluster.examples.length === 1 &&
    r.domainCluster.examples[0].file === 'src/utils/merge-with-values.util.ts',
    JSON.stringify(r.domainCluster.examples));
}

// ═══ FILE_STATUS_UNKNOWN — topology absent ═══

{
  const symbols = buildSymbols();  // no defIndex entry either
  const r = lookupFile('src/utils/time.ts', { topology: null, symbols, triage: null, root: '/root' });
  assert('T4. topology absent + defIndex empty → FILE_STATUS_UNKNOWN',
    r.result === 'FILE_STATUS_UNKNOWN');
  assert('T4b. [확인 불가] citation mentions topology',
    r.citations.some((c) => /확인 불가/.test(c) && /topology/.test(c)),
    `citations=${JSON.stringify(r.citations)}`);
}

// ═══ FILE_STATUS_UNKNOWN — topology present but not complete ═══

{
  const topology = buildTopology({
    nodes: { 'src/existing.ts': { loc: 10 } },
    complete: false,  // NOT a promise of completeness
  });
  const symbols = buildSymbols();
  const r = lookupFile('src/utils/time.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T5. topology present without complete=true + absent → FILE_STATUS_UNKNOWN',
    r.result === 'FILE_STATUS_UNKNOWN',
    `result=${r.result}`);
}

// ═══ CRITICAL: defIndex absence alone does NOT yield NEW_FILE ═══

{
  const symbols = buildSymbols({ defIndex: {} });
  const r = lookupFile('src/any.ts', { topology: null, symbols, triage: null, root: '/root' });
  assert('T6. topology absent + empty defIndex → FILE_STATUS_UNKNOWN (NEVER NEW_FILE)',
    r.result === 'FILE_STATUS_UNKNOWN',
    `result=${r.result}`);
}

// ═══ Parse-error file IS NOT NEW_FILE ═══

{
  const topology = buildTopology({
    nodes: { 'src/clean.ts': { loc: 20 } },
    complete: true,
  });
  // The file failed to parse, so it's not in topology.nodes but IS real.
  const symbols = buildSymbols({ filesWithParseErrors: ['src/broken.ts'] });
  const r = lookupFile('src/broken.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T7. topology complete + absent from nodes BUT in parseErrors → FILE_STATUS_UNKNOWN (NOT NEW_FILE)',
    r.result === 'FILE_STATUS_UNKNOWN',
    `result=${r.result}`);
  assert('T7b. citation mentions parse-error reason',
    r.citations.some((c) => /parse/i.test(c)),
    `citations=${JSON.stringify(r.citations)}`);
}

// ═══ inboundFanIn from topology edges ═══

{
  const topology = buildTopology({
    nodes: { 'src/util/shared.ts': { loc: 50 } },
    edges: [
      { from: 'src/app/a.ts', to: 'src/util/shared.ts' },
      { from: 'src/app/b.ts', to: 'src/util/shared.ts' },
      { from: 'src/app/c.ts', to: 'src/util/shared.ts' },
    ],
    complete: true,
  });
  const symbols = buildSymbols();
  const r = lookupFile('src/util/shared.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T8. inboundFanIn counted from topology.edges',
    r.inboundFanIn === 3,
    `inboundFanIn=${r.inboundFanIn}`);
  assert('T8b. inboundFanInConfidence grounded',
    r.inboundFanInConfidence === 'grounded');
}

// ═══ Boundary: ALWAYS NOT_EVALUATED in P1-2 (no planned edges) ═══

{
  // Even with a triage file listing declared boundary rules, P1-2
  // refuses to evaluate — no planned from→to edge is available.
  const topology = buildTopology({ nodes: { 'apps/web/new.ts': null }, complete: true });
  const symbols = buildSymbols();
  const triage = {
    boundaryRules: [
      { from: 'apps/web/*', to: '_lib/*', direction: 'allowed', declaredIn: 'eslint.config.mjs' },
    ],
  };
  const r = lookupFile('apps/web/new.ts', { topology, symbols, triage, root: '/root' });
  assert('T9. P1-2 ALWAYS emits boundary.status = NOT_EVALUATED',
    r.boundary.status === 'NOT_EVALUATED',
    `boundary=${JSON.stringify(r.boundary)}`);
  assert('T9b. boundary.rule is null in NOT_EVALUATED',
    r.boundary.rule === null);
}

// PINNING: no fixture can coax ALLOWED or FORBIDDEN out of P1-2.
{
  const topology = buildTopology({ nodes: {}, complete: true });
  const symbols = buildSymbols();
  const triage = {
    boundaryRules: [
      { from: '*', to: '*', direction: 'allowed', declaredIn: 'anywhere' },
    ],
  };
  const r = lookupFile('any/file.ts', { topology, symbols, triage, root: '/root' });
  assert('T10. even a blanket ALLOW rule cannot produce boundary.status=ALLOWED in P1-2',
    r.boundary.status !== 'ALLOWED' && r.boundary.status !== 'FORBIDDEN',
    `boundary.status=${r.boundary.status}`);
}

// ═══ Triage absent → NOT_EVALUATED with [확인 불가] ═══

{
  const topology = buildTopology({ nodes: { 'src/a.ts': { loc: 1 } }, complete: true });
  const symbols = buildSymbols();
  const r = lookupFile('src/a.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T11. triage absent → boundary NOT_EVALUATED with reason',
    r.boundary.status === 'NOT_EVALUATED');
  assert('T11b. boundary citation mentions triage or planned-edge reason',
    r.citations.some((c) => /triage/.test(c) || /planned.edge/i.test(c)),
    `citations=${JSON.stringify(r.citations)}`);
}

// ═══ test-only tag ═══

{
  const topology = buildTopology({
    nodes: { 'tests/foo.test.ts': { loc: 5 } },
    complete: true,
  });
  const symbols = buildSymbols();
  const r = lookupFile('tests/foo.test.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T12. test-path candidate picks up test-only tag',
    r.tags?.includes('test-only'),
    `tags=${JSON.stringify(r.tags)}`);
}

// ═══ Windows path separator normalization ═══

{
  const topology = buildTopology({
    nodes: { 'src/utils/date.ts': { loc: 42 } },
    complete: true,
  });
  const symbols = buildSymbols();
  // Caller passes Windows-style backslash path; lookup must normalize.
  const r = lookupFile('src\\utils\\date.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T13. Windows backslash intentFile normalizes to match topology.nodes keys',
    r.result === 'FILE_EXISTS',
    `result=${r.result}`);
}

// ═══ Submodule classification via buildSubmoduleResolver ═══

{
  // Without a real repo root we can't test buildSubmoduleResolver fully,
  // but the shape assertion holds: `submodule` is a string or null.
  const topology = buildTopology({ nodes: { 'src/a.ts': { loc: 1 } }, complete: true });
  const symbols = buildSymbols();
  const r = lookupFile('src/a.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T14. submodule field is a string or null',
    typeof r.submodule === 'string' || r.submodule === null);
}

// ═══ Citations coverage ═══

{
  const topology = buildTopology({
    nodes: { 'src/utils/date.ts': { loc: 42 } },
    complete: true,
  });
  const symbols = buildSymbols();
  const r = lookupFile('src/utils/date.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T15. FILE_EXISTS result carries at least one grounded citation',
    r.citations.some((c) => /\[grounded/.test(c)));
}

{
  const symbols = buildSymbols();
  const r = lookupFile('src/anything.ts', { topology: null, symbols, triage: null, root: '/root' });
  assert('T16. FILE_STATUS_UNKNOWN carries [확인 불가] citation',
    r.citations.some((c) => /\[확인 불가/.test(c)));
}

// ═══ kind discriminator ═══

{
  const topology = buildTopology({ nodes: { 'src/a.ts': { loc: 1 } }, complete: true });
  const symbols = buildSymbols();
  const r = lookupFile('src/a.ts', { topology, symbols, triage: null, root: '/root' });
  assert('T17. result carries kind:"file" discriminator',
    r.kind === 'file');
  assert('T17b. intentFile preserved',
    r.intentFile === 'src/a.ts');
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
