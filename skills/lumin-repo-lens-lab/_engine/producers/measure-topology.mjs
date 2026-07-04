#!/usr/bin/env node
// m2s1-topology.mjs — File-level import graph (parameterized)
//
// Usage:
//   node measure-topology.mjs --root <repo> [--output <dir>] [--include-tests] \
//        [--include-type-edges] [--cache-root <dir>] [--no-incremental] \
//        [--clear-incremental-cache] [--rust-topology-scanner off|compare|prefer] \
//        [--rust-topology-scanner-bin <path>] [--rust-topology-timeout-ms <ms>] \
//        [--rust-sidecar-source-commit <sha>] \
//        [--rust-topology-prefer-gate] [--rust-topology-prefer-gate-corpus <name>] \
//        [--rust-topology-prefer-quorum <file>] \
//        [--verbose]

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseOxcOrThrow } from '../lib/parse-oxc.mjs';
import { parseCliArgs } from '../lib/cli.mjs';
import { detectRepoMode } from '../lib/repo-mode.mjs';
import { buildAliasMap } from '../lib/alias-map.mjs';
import { makeResolver, isNonSourceAssetResolution, isResolvedFile } from '../lib/resolver-core.mjs';
import { runAuditCoreJsonResultFile } from '../lib/audit-core.mjs';
import { collectFiles } from '../lib/collect-files.mjs';
import { JS_FAMILY_LANGS } from '../lib/lang.mjs';
import { relPath, buildSubmoduleResolver } from '../lib/paths.mjs';
import {
  MODULE_EDGE_SCANNER_POLICY_VERSION,
  scanJsModuleEdgesFast,
} from '../lib/js-module-edge-scanner.mjs';
import { compareRustTopologyScanner } from '../lib/rust-topology-scanner.mjs';
import {
  evaluateRustTopologyPreferGate,
  readRustTopologyPreferQuorum,
  RUST_TOPOLOGY_PREFER_QUORUM_PATH,
} from '../lib/rust-topology-prefer-gate.mjs';
import {
  compareTopologyArtifactContract,
  evaluateRustTopologyPrefer,
  hashFileSha256,
} from '../lib/rust-topology-prefer.mjs';
import {
  loadCache,
  saveCache,
  pickChangedFiles,
  cacheBanner,
} from '../lib/incremental.mjs';
import {
  clearIncrementalCache,
  openIncrementalCacheStore,
} from '../lib/incremental-cache-store.mjs';
import {
  isPythonAvailable,
  extractPythonBatch,
  resolvePythonImport,
} from '../lib/python.mjs';
import {
  isTreeSitterAvailable,
  extractTreeSitterBatch,
  findGoModule,
  resolveGoImport,
} from '../lib/tree-sitter-langs.mjs';
import { createProducerPhaseTimer } from '../lib/producer-phase-timing.mjs';

const cli = parseCliArgs({
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
  'include-type-edges': { type: 'boolean', default: false },
  'rust-topology-scanner': { type: 'string', default: 'off' },
  'rust-topology-scanner-bin': { type: 'string' },
  'rust-topology-timeout-ms': { type: 'string', default: '60000' },
  'rust-sidecar-source-commit': { type: 'string' },
  'rust-topology-prefer-gate': { type: 'boolean', default: false },
  'rust-topology-prefer-gate-corpus': { type: 'string' },
  'rust-topology-prefer-quorum': { type: 'string' },
});
const { root, output, verbose } = cli;
const producerDir = path.dirname(fileURLToPath(import.meta.url));
const labRoot = path.basename(producerDir) === 'producers'
  ? path.resolve(producerDir, '..', '..')
  : producerDir;
const phaseTimer = createProducerPhaseTimer({
  producer: 'measure-topology.mjs',
  output,
});
const isIncremental = cli.raw['no-incremental'] !== true;
const cacheStore = openIncrementalCacheStore({
  root,
  cacheRoot: cli.raw['cache-root'],
});
if (cli.raw['clear-incremental-cache'] === true) {
  clearIncrementalCache(cacheStore);
}
const topologyCacheDir = path.join(cacheStore.repoCacheDir, 'legacy');
// Two lenses for SCC analysis:
//   default (runtime lens) — type-only imports excluded; tracks what actually
//     ships to production. `import type {X}` is erased at compile.
//   --include-type-edges (static lens) — matches dep-cruiser's
//     --ts-pre-compilation-deps: includes the compile-time type-layer graph.
// Report findings with the lens explicitly labeled.
const includeTypeEdges = !!cli.raw['include-type-edges'];
const rustScannerMode = cli.raw['rust-topology-scanner'] ?? 'off';
if (!['off', 'compare', 'prefer'].includes(rustScannerMode)) {
  throw new Error(`unsupported --rust-topology-scanner mode: ${rustScannerMode}`);
}
if (rustScannerMode === 'compare' && isIncremental) {
  throw new Error('--rust-topology-scanner compare requires --no-incremental in M2');
}

if (verbose) console.error(`[m2s1] root: ${root}`);
const repoMode = detectRepoMode(root);
if (verbose) console.error(`[m2s1] mode: ${repoMode.mode}, workspaces: ${repoMode.workspaceDirs.length}`);

const aliasMap = buildAliasMap(root, repoMode, { exclude: cli.exclude });
if (verbose) console.error(`[m2s1] alias entries: ${aliasMap.size}`);

const resolve = makeResolver(root, aliasMap);
const candidateLangList = [...JS_FAMILY_LANGS, 'py', 'go'];
const candidateFiles = phaseTimer.runPhase('collect-files', () => collectFiles(root, {
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  languages: candidateLangList,
}));
const pyCandidates = candidateFiles.filter((f) => f.endsWith('.py'));
const goCandidates = candidateFiles.filter((f) => f.endsWith('.go'));
const pyEnabled = pyCandidates.length > 0 ? isPythonAvailable() : false;
const tsEnabled = goCandidates.length > 0 ? await isTreeSitterAvailable() : false;
const files = candidateFiles.filter((f) => {
  if (f.endsWith('.py')) return pyEnabled;
  if (f.endsWith('.go')) return tsEnabled;
  return true;
});
phaseTimer.setCounter('filesCollected', files.length);
const pyTotal = pyCandidates.length;
const goTotal = goCandidates.length;
const pythonStatus = pyTotal === 0
  ? 'skipped, 0 .py'
  : `${pyEnabled ? 'on' : 'off'}, ${pyTotal} .py`;
const goStatus = goTotal === 0
  ? 'skipped, 0 .go'
  : `${tsEnabled ? 'on' : 'off'}, ${goTotal} .go`;
console.error(
  `[m2s1] scanning ${files.length} files (python=${pythonStatus}, go=${goStatus}) ...`
);
const goModule = goTotal > 0 && tsEnabled ? findGoModule(root) : null;
if (goTotal > 0 && verbose) console.error(`[m2s1] go.mod: ${goModule?.moduleName ?? 'none'}`);

// ─── per-file processor (pure: file → {loc, edges, externalCount, unresolvedCount, parseError}) ─
// Dispatches on file extension:
//   .py  → Python AST via subprocess batch (python.mjs)
//   .go  → Tree-sitter WASM batch (tree-sitter-langs.mjs)
//   else → oxc-parser (TypeScript/JavaScript)
let pyResults = new Map();
let tsResults = new Map(); // tree-sitter results (go, future: rust, java...)
const scannerRiskCounts = new Map();
const scannerFallbackExamples = new Map();
const SCANNER_FALLBACK_EXAMPLE_LIMIT = 5;
const rustComparableJsResults = [];

function recordScannerFallbackRisk(reason, file) {
  const key = String(reason ?? 'unknown');
  scannerRiskCounts.set(key, (scannerRiskCounts.get(key) ?? 0) + 1);
  phaseTimer.incrementCounter(`scannerRisk_${key}`);
  const examples = scannerFallbackExamples.get(key) ?? [];
  if (examples.length < SCANNER_FALLBACK_EXAMPLE_LIMIT) {
    examples.push(relPath(root, file));
    scannerFallbackExamples.set(key, examples);
  }
}

function resolveTopologyEdge(fromFile, source, flags, edgesOut) {
  const target = resolve(fromFile, source);
  if (target === 'EXTERNAL') return 'external';
  if (isNonSourceAssetResolution(target)) return 'asset';
  if (isResolvedFile(target)) {
    edgesOut.push({
      to: target,
      typeOnly: !!flags.typeOnly,
      ...(flags.dynamic ? { dynamic: true } : {}),
      ...(flags.reExport ? { reExport: true } : {}),
    });
    return 'resolved';
  }
  return 'unresolved';
}

function processFilePython(f) {
  const r = pyResults.get(f);
  if (!r) return { readError: true };
  if (r.error) {
    if (verbose) console.error(`[m2s1] py error: ${relPath(root, f)}: ${r.error}`);
    return { loc: r.loc ?? 0, edges: [], externalCount: 0, unresolvedCount: 0, parseError: true };
  }
  const edges = [];
  let externalCount = 0;
  for (const imp of r.imports ?? []) {
    const hits = resolvePythonImport(
      root, f, imp.source, imp.isFromImport, imp.imported, imp.level
    );
    if (hits.length === 0) {
      externalCount++;
    } else {
      for (const hit of hits) edges.push({ to: hit });
    }
  }
  return { loc: r.loc ?? 0, edges, externalCount, unresolvedCount: 0, parseError: false };
}

function processFileGo(f) {
  const r = tsResults.get(f);
  if (!r) return { readError: true };
  if (r.error) {
    if (verbose) console.error(`[m2s1] go error: ${relPath(root, f)}: ${r.error}`);
    return { loc: r.loc ?? 0, edges: [], externalCount: 0, unresolvedCount: 0, parseError: true };
  }
  const edges = [];
  let externalCount = 0;
  for (const imp of r.imports ?? []) {
    const hits = resolveGoImport(root, goModule, imp.source);
    if (hits.length === 0) {
      externalCount++; // stdlib or 3rd-party
    } else {
      for (const hit of hits) edges.push({ to: hit });
    }
  }
  return { loc: r.loc ?? 0, edges, externalCount, unresolvedCount: 0, parseError: false };
}

function processFileTs(f) {
  let src;
  try {
    src = readFileSync(f, 'utf8');
  } catch {
    return { readError: true };
  }
  phaseTimer.incrementCounter('jsFilesProcessed');
  phaseTimer.incrementCounter('jsBytesRead', Buffer.byteLength(src, 'utf8'));
  const loc = src.split('\n').length;
  const edgesOut = [];
  let externalCount = 0;
  let unresolvedCount = 0;

  const scannerStarted = Date.now();
  phaseTimer.incrementCounter('scannerFilesAttempted');
  const scanned = scanJsModuleEdgesFast(src, { filename: f });
  phaseTimer.incrementCounter('scannerMs', Date.now() - scannerStarted);
  if (scanned.ok) {
    rustComparableJsResults.push({
      file: f,
      ok: true,
      loc: scanned.loc ?? loc,
      edges: scanned.edges ?? [],
      risk: [],
    });
    phaseTimer.incrementCounter('scannerAcceptedFiles');
    for (const edge of scanned.edges ?? []) {
      const outcome = resolveTopologyEdge(f, edge.source, edge, edgesOut);
      if (outcome === 'external') externalCount++;
      else if (outcome === 'unresolved') unresolvedCount++;
    }
    return {
      loc: scanned.loc ?? loc,
      edges: edgesOut,
      externalCount,
      unresolvedCount,
      parseError: false,
      scannerMode: 'fast-module-edge',
    };
  }
  phaseTimer.incrementCounter('scannerFallbackFiles');
  rustComparableJsResults.push({
    file: f,
    ok: false,
    loc: scanned.loc ?? loc,
    edges: [],
    risk: scanned.risk ?? [],
  });
  for (const reason of scanned.risk ?? []) recordScannerFallbackRisk(reason, f);

  // v0.6.8 FP-18 sync-back: dynamic `import('./x')` edges must surface in
  // topology — SKILL.md promises dynamic imports are ALWAYS in both the
  // runtime and static lens. Previously only top-level ImportDeclaration and
  // re-export with source were read; dynamic imports live inside function
  // bodies, arrow expressions, conditionals, object literals, etc. so we
  // need a recursive walker (same logic as build-symbol-graph.mjs). Edges
  // get `dynamic: true` for provenance; `typeOnly: false` so they survive
  // the runtime-lens filter.
  function walkDynamic(node) {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'ImportExpression') {
      const s = node.source;
      if (s && (s.type === 'Literal' || s.type === 'StringLiteral') &&
          typeof s.value === 'string') {
        const outcome = resolveTopologyEdge(f, s.value, { typeOnly: false, dynamic: true }, edgesOut);
        if (outcome === 'external') externalCount++;
        else if (outcome === 'unresolved') unresolvedCount++;
      }
    }
    for (const key of Object.keys(node)) {
      if (key === 'type' || key === 'start' || key === 'end') continue;
      const v = node[key];
      if (Array.isArray(v)) {
        for (const n of v) walkDynamic(n);
      } else if (v && typeof v === 'object' && typeof v.type === 'string') {
        walkDynamic(v);
      }
    }
  }

  // v1.8.3: helper centralizes oxc error escalation; see _lib/parse-oxc.mjs.
  try {
    phaseTimer.incrementCounter('oxcParseCalls');
    const r = parseOxcOrThrow(f, src);
    for (const node of r.program.body) {
      if (node.type === 'ImportDeclaration') {
        const outcome = resolveTopologyEdge(f, node.source.value, {
          typeOnly: node.importKind === 'type',
        }, edgesOut);
        if (outcome === 'external') externalCount++;
        else if (outcome === 'unresolved') unresolvedCount++;
      } else if (
        (node.type === 'ExportNamedDeclaration' || node.type === 'ExportAllDeclaration') &&
        node.source
      ) {
        // v1.8.3: detect type-only re-exports so the runtime-lens
        // topology doesn't attribute cycles to `export type { X } from
        // './types'`. Three TypeScript syntactic forms:
        //   (1) `export type { X } from ...`      → node.exportKind === 'type'
        //   (2) `export type * from ...`          → node.exportKind === 'type'
        //   (3) `export { type X, type Y } ...`   → every specifier has exportKind='type'
        // Mixed forms (e.g. `export { X, type Y }`) must keep the edge
        // because X is still a runtime re-export.
        const specs = node.specifiers ?? [];
        const allSpecsTypeOnly = specs.length > 0 && specs.every((s) => s.exportKind === 'type');
        const typeOnly = node.exportKind === 'type' || allSpecsTypeOnly;
        const outcome = resolveTopologyEdge(f, node.source.value, {
          reExport: true,
          typeOnly,
        }, edgesOut);
        if (outcome === 'external') externalCount++;
        else if (outcome === 'unresolved') unresolvedCount++;
      }
    }
    // Sweep the entire AST once for dynamic import() expressions anywhere.
    walkDynamic(r.program);
  } catch (e) {
    phaseTimer.incrementCounter('oxcParseErrors');
    if (verbose) console.error(`[m2s1] parse error: ${relPath(root, f)}: ${e.message}`);
    return { loc, edges: [], externalCount: 0, unresolvedCount: 0, parseError: true };
  }
  return { loc, edges: edgesOut, externalCount, unresolvedCount, parseError: false };
}

function processFile(f) {
  if (f.endsWith('.py')) return processFilePython(f);
  if (f.endsWith('.go')) return processFileGo(f);
  return processFileTs(f);
}

function buildRustTopologyEntryFromScannerResult(f, rustResult) {
  const edgesOut = [];
  let externalCount = 0;
  let unresolvedCount = 0;
  if (!rustResult || rustResult.ok !== true) {
    return {
      loc: rustResult?.loc ?? 0,
      edges: [],
      externalCount: 0,
      unresolvedCount: 0,
      parseError: false,
      scannerMode: 'rust-module-edge-risk',
    };
  }
  for (const edge of rustResult.edges ?? []) {
    const outcome = resolveTopologyEdge(f, edge.source, edge, edgesOut);
    if (outcome === 'external') externalCount++;
    else if (outcome === 'unresolved') unresolvedCount++;
  }
  return {
    loc: rustResult.loc ?? 0,
    edges: edgesOut,
    externalCount,
    unresolvedCount,
    parseError: false,
    scannerMode: 'rust-module-edge',
  };
}

function buildRustCandidateEntries({ jsEntries, rustResults }) {
  const rustByFile = new Map(
    (rustResults ?? []).map((entry) => [String(entry.file).replaceAll('\\', '/'), entry]),
  );
  const entries = globalThis.structuredClone(jsEntries);
  for (const file of rustComparableJsResults.map((entry) => entry.file)) {
    const key = file.replaceAll('\\', '/');
    entries[file] = buildRustTopologyEntryFromScannerResult(file, rustByFile.get(key));
  }
  return entries;
}

// ─── incremental-aware processing loop ───────────────────
const cache = isIncremental ? loadCache(topologyCacheDir, 'topology') : { version: 1, entries: {} };
const { changed, unchanged, dropped, nextCache } = isIncremental
  ? pickChangedFiles(files, cache)
  : { changed: files, unchanged: [], dropped: [], nextCache: { version: 1, entries: {} } };
phaseTimer.setCounter('changedFiles', changed.length);
phaseTimer.setCounter('unchangedFiles', unchanged.length);
phaseTimer.setCounter('droppedFiles', dropped.length);

if (isIncremental) {
  console.error(cacheBanner('m2s1', changed, unchanged, dropped));
}

// Pre-batch Python files among the changed set (one subprocess).
const processChangedFilesStarted = Date.now();
const changedPy = changed.filter((f) => f.endsWith('.py'));
if (changedPy.length > 0 && pyEnabled) {
  try {
    pyResults = extractPythonBatch(changedPy) ?? new Map();
    if (verbose) console.error(`[m2s1] python batch: ${pyResults.size}/${changedPy.length}`);
  } catch (e) {
    console.error(`[m2s1] python batch failed: ${e.message}`);
  }
}

// Pre-batch tree-sitter languages (currently Go) among changed set.
const changedTs = changed.filter((f) => f.endsWith('.go'));
if (changedTs.length > 0 && tsEnabled) {
  try {
    tsResults = (await extractTreeSitterBatch(changedTs)) ?? new Map();
    if (verbose) console.error(`[m2s1] tree-sitter batch: ${tsResults.size}/${changedTs.length}`);
  } catch (e) {
    console.error(`[m2s1] tree-sitter batch failed: ${e.message}`);
  }
}

for (const f of changed) {
  const payload = processFile(f);
  if (payload.readError) continue;
  nextCache.entries[f] = { ...(nextCache.entries[f] ?? {}), ...payload };
}
phaseTimer.recordPhase('process-changed-files', Date.now() - processChangedFilesStarted);

if (isIncremental) {
  mkdirSync(topologyCacheDir, { recursive: true });
  saveCache(topologyCacheDir, 'topology', nextCache);
}

for (const counterName of [
  'scannerFilesAttempted',
  'scannerAcceptedFiles',
  'scannerFallbackFiles',
  'scannerMs',
  'oxcParseCalls',
  'oxcParseErrors',
]) {
  phaseTimer.setCounter(counterName, phaseTimer.counters[counterName] ?? 0);
}

// ─── aggregate ───────────────────────────────────────────
function assembleTopologyArtifactFromEntries({
  sourceEntries,
  files,
  rustMetadata = {},
}) {
  const submoduleOf = buildSubmoduleResolver(root, repoMode);
  const submoduleByFile = {};
  for (const f of files) {
    submoduleByFile[f] = submoduleOf(f);
    for (const e of sourceEntries[f]?.edges ?? []) {
      if (typeof e?.to === 'string' &&
          !e.to.startsWith('external:') &&
          !e.to.startsWith('unresolved:')) {
        submoduleByFile[e.to] = submoduleOf(e.to);
      }
    }
  }

  const performance = {
    filesCollected: phaseTimer.counters.filesCollected ?? files.length,
    changedFiles: phaseTimer.counters.changedFiles ?? files.length,
    unchangedFiles: phaseTimer.counters.unchangedFiles ?? 0,
    droppedFiles: phaseTimer.counters.droppedFiles ?? 0,
    jsFilesProcessed: phaseTimer.counters.jsFilesProcessed ?? 0,
    jsBytesRead: phaseTimer.counters.jsBytesRead ?? 0,
    scannerPolicyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    scannerFilesAttempted: phaseTimer.counters.scannerFilesAttempted ?? 0,
    scannerAcceptedFiles: phaseTimer.counters.scannerAcceptedFiles ?? 0,
    scannerFallbackFiles: phaseTimer.counters.scannerFallbackFiles ?? 0,
    scannerMs: phaseTimer.counters.scannerMs ?? 0,
    scannerRiskCounts: Object.fromEntries([...scannerRiskCounts.entries()].sort(([a], [b]) =>
      a.localeCompare(b))),
    scannerFallbackExamples: Object.fromEntries([...scannerFallbackExamples.entries()].sort(([a], [b]) =>
      a.localeCompare(b))),
    oxcParseCalls: phaseTimer.counters.oxcParseCalls ?? 0,
    oxcParseErrors: phaseTimer.counters.oxcParseErrors ?? 0,
    resolverMemoHits: phaseTimer.counters.resolverMemoHits ?? 0,
    resolverMemoMisses: phaseTimer.counters.resolverMemoMisses ?? 0,
    resolverMemoSize: phaseTimer.counters.resolverMemoSize ?? 0,
  };

  const artifact = runAuditCoreJsonResultFile(
    ['topology-artifact', '--input', '-'],
    'topology-artifact',
    {
      input: JSON.stringify({
        schemaVersion: 'lumin-topology-producer-request.v1',
        generated: new Date().toISOString(),
        root,
        mode: repoMode.mode,
        rootPkgName: repoMode.rootPkgName,
        includeTypeEdges,
        files,
        sourceEntries,
        submoduleByFile,
        performance,
        rustMetadata,
      }),
    }
  );

  return { artifact };
}

const assembleGraphStarted = Date.now();
const sourceEntriesForAssembly = nextCache.entries;
const rustPreferRequested = rustScannerMode === 'prefer';
const rustComparisonMode = rustPreferRequested && isIncremental ? 'off' : rustScannerMode;
const rustScannerComparison = compareRustTopologyScanner({
  mode: rustComparisonMode,
  binary: cli.raw['rust-topology-scanner-bin'],
  root,
  files: rustComparableJsResults.map((entry) => entry.file),
  jsResults: rustComparableJsResults,
  timeoutMs: Number(cli.raw['rust-topology-timeout-ms'] ?? 60000),
});
const rustPreferGateEnabled = cli.raw['rust-topology-prefer-gate'] === true;
const rustPreferQuorumPath = cli.raw['rust-topology-prefer-quorum']
  ? path.resolve(cli.raw['rust-topology-prefer-quorum'])
  : path.join(labRoot, RUST_TOPOLOGY_PREFER_QUORUM_PATH);
const rustPreferQuorumEvidence = rustPreferGateEnabled && !(rustPreferRequested && isIncremental)
  ? readRustTopologyPreferQuorum(rustPreferQuorumPath)
  : null;
const rustTopologyPreferGate = rustPreferGateEnabled && !(rustPreferRequested && isIncremental)
  ? evaluateRustTopologyPreferGate({
      mode: rustScannerMode,
      currentCorpus: cli.raw['rust-topology-prefer-gate-corpus'],
      rustTopologyScanner: rustScannerComparison.metadata,
      quorumEvidence: rustPreferQuorumEvidence,
      quorumEvidencePath: rustPreferQuorumPath,
      policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    })
  : null;

let rustSidecarBinarySha256 = null;
try {
  if (rustPreferRequested && cli.raw['rust-topology-scanner-bin']) {
    rustSidecarBinarySha256 = hashFileSha256(cli.raw['rust-topology-scanner-bin']);
  }
} catch {
  rustSidecarBinarySha256 = null;
}

const rustCandidateEntries = rustPreferRequested && rustScannerComparison.rustResults.length > 0
  ? buildRustCandidateEntries({
      jsEntries: sourceEntriesForAssembly,
      rustResults: rustScannerComparison.rustResults,
    })
  : null;

const resolverMemoStats = typeof resolve.memoStats === 'function'
  ? resolve.memoStats()
  : { hits: 0, misses: 0, size: 0 };
phaseTimer.setCounter('resolverMemoHits', resolverMemoStats.hits);
phaseTimer.setCounter('resolverMemoMisses', resolverMemoStats.misses);
phaseTimer.setCounter('resolverMemoSize', resolverMemoStats.size);

const baseRustMetadata = {
  ...(rustScannerComparison.metadata
    ? { rustTopologyScanner: rustScannerComparison.metadata }
    : {}),
  ...(rustTopologyPreferGate
    ? { rustTopologyPreferGate }
    : {}),
};

const jsArtifact = assembleTopologyArtifactFromEntries({
  sourceEntries: sourceEntriesForAssembly,
  files,
  rustMetadata: baseRustMetadata,
}).artifact;

const rustCandidateArtifact = rustCandidateEntries
  ? assembleTopologyArtifactFromEntries({
      sourceEntries: rustCandidateEntries,
      files,
      rustMetadata: baseRustMetadata,
    }).artifact
  : null;

const artifactGuard = rustCandidateArtifact
  ? compareTopologyArtifactContract(jsArtifact, rustCandidateArtifact)
  : { status: 'not-run', passed: false };

const rustTopologyPrefer = rustPreferRequested
  ? evaluateRustTopologyPrefer({
      requested: true,
      mode: rustScannerMode,
      isIncremental,
      currentCorpus: cli.raw['rust-topology-prefer-gate-corpus'],
      rustTopologyScanner: rustScannerComparison.metadata,
      rustTopologyPreferGate,
      currentFileCount: jsArtifact.summary.files,
      quorumEvidencePath: rustPreferQuorumPath,
      rustSidecarBinary: cli.raw['rust-topology-scanner-bin'],
      rustSidecarSourceCommit: cli.raw['rust-sidecar-source-commit'],
      expectedRustSidecarSourceCommit: rustPreferQuorumEvidence?.rustSidecarSourceCommit,
      rustSidecarBinarySha256,
      expectedRustSidecarBinarySha256: rustPreferQuorumEvidence?.rustSidecarBinarySha256,
      artifactGuard,
    })
  : null;

const artifact = rustTopologyPrefer?.usedRust && rustCandidateArtifact
  ? rustCandidateArtifact
  : jsArtifact;
if (rustTopologyPrefer) artifact.meta.rustTopologyPrefer = rustTopologyPrefer;
phaseTimer.recordPhase('assemble-graph', Date.now() - assembleGraphStarted);

const outPath = path.join(output, 'topology.json');
const writeArtifactStarted = Date.now();
writeFileSync(outPath, JSON.stringify(artifact, null, 2));
phaseTimer.recordPhase('write-artifact', Date.now() - writeArtifactStarted);
phaseTimer.write();

const lensLabel = includeTypeEdges ? 'static' : 'runtime';
console.log(`[m2s1] ${files.length} files, ${artifact.summary.totalLoc.toLocaleString()} LOC, ${artifact.summary.internalEdges} edges (lens: ${lensLabel})`);
console.log(`[m2s1] SCC ${artifact.summary.sccCount} (max ${artifact.summary.maxSccSize}), 1000 LOC+ ${artifact.summary.oneThousandPlusFiles}`);
console.log(`[m2s1] saved → ${outPath}`);

if (rustTopologyPrefer?.status === 'blocked') {
  console.error(`[m2s1] Rust topology prefer blocked: ${rustTopologyPrefer.reason}`);
  console.error(`[m2s1] diagnostic artifact saved → ${outPath}`);
  process.exitCode = 1;
}
