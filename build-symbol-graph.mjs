// build-symbol-graph.mjs — Symbol-level export/import graph (parameterized)
//
// For each file:
// - collect top-level export definitions (not re-exports)
// - collect import/re-export specifiers (uses)
// - build (definition file, symbol) -> consumer set mapping
// - derive: dead exports, symbol fan-in, top consumers
//
// Usage: node build-symbol-graph.mjs --root <repo> [--output <dir>]

import path from "node:path";
import { performance } from "node:perf_hooks";

import { detectBarrelFiles } from "./_lib/alias-map.mjs";
import { parseCliArgs } from "./_lib/cli.mjs";
import { detectRepoMode } from "./_lib/repo-mode.mjs";
import { buildAliasMap } from "./_lib/alias-map.mjs";
import {
  isGeneratedVirtualResolution,
  isNonSourceAssetResolution,
} from "./_lib/resolver-core.mjs";
import { collectMdxImportConsumers } from "./_lib/mdx-consumers.mjs";
import { normalizeGeneratedArtifactsMode } from "./_lib/generated-artifact-mode.mjs";
import { DEFAULT_IMPORT_META_GLOB_CAP } from "./_lib/import-meta-glob-expansion.mjs";
import { isTestLikePath } from "./_lib/test-paths.mjs";
import { fileExists, relPath } from "./_lib/paths.mjs";
import {
  buildSourceUseAssemblyRequest,
  remapSourceUseRecordIdInputs,
  sourceUseAssemblyNeedsSourceFiles,
  sourceUseProjectionRecordId,
  sourceUseRecordIdRemap,
} from "./_lib/source-use-assembly-request.mjs";
import {
  createSourceUseRecordBuilder,
  isRelativeSourceUse as isSourceUseAssemblyCandidate,
  isRustResolvedRelativeUse,
  looksLikeNonSourceAssetSpecifier,
  sourceUseRequiresSymbolName as sourceUseAssemblyRequiresSymbolName,
} from "./_lib/source-use-record-builder.mjs";
import {
  buildSymbolGraphSfcInputs,
  sfcGlobalComponentResolutionSpec,
} from "./_lib/symbol-graph-sfc-inputs.mjs";
import { STRICT_IDENTITY_MODE } from "./_lib/incremental-snapshot.mjs";
import { isPythonAvailable } from "./_lib/python.mjs";
import {
  isTreeSitterAvailable,
  findGoModule,
} from "./_lib/tree-sitter-langs.mjs";
import { createProducerPhaseTimer } from "./_lib/producer-phase-timing.mjs";
import {
  discoverSymbolGraphFacts,
  SYMBOL_GRAPH_FACT_SCHEMA_VERSION,
  SYMBOL_GRAPH_PARSER_IDENTITY,
  SYMBOL_GRAPH_PRODUCER_ID,
  SYMBOL_GRAPH_PRODUCER_VERSION,
} from "./_lib/symbol-graph-discovery.mjs";
import { discoverSymbolGraphSfcFacts } from "./_lib/symbol-graph-sfc-discovery.mjs";
import { createSymbolGraphResolver } from "./_lib/symbol-graph-resolver.mjs";
import { finalizeSymbolGraphArtifact } from "./_lib/symbol-graph-finalizer.mjs";
import { buildSymbolGraphArtifactRequest } from "./_lib/symbol-graph-request.mjs";
import {
  planInlineSourceUses,
  planOutOfBandSourceUses,
  planSfcComponentSourceUses,
} from "./_lib/symbol-graph-source-use-planner.mjs";

const cli = parseCliArgs({
  incremental: { type: "boolean", default: false },
  "no-incremental": { type: "boolean", default: false },
  "cache-root": { type: "string" },
  "clear-incremental-cache": { type: "boolean", default: false },
  "generated-artifacts": { type: "string", default: "default" },
});
const { root: ROOT, output, verbose } = cli;
const phaseTimer = createProducerPhaseTimer({
  producer: "build-symbol-graph.mjs",
  output,
});
let GENERATED_ARTIFACTS_MODE = "default";
try {
  GENERATED_ARTIFACTS_MODE = normalizeGeneratedArtifactsMode(
    cli.raw?.["generated-artifacts"],
  );
} catch (error) {
  console.error(`[symbols] ${error.message}`);
  process.exit(2);
}
const SOURCE_USE_ASSEMBLY_PATH_TABLE = true;
const SOURCE_USE_ASSEMBLY_ENUM_TABLE = true;
const SOURCE_USE_ASSEMBLY_SPECIFIER_TABLE = true;
const SOURCE_USE_ASSEMBLY_RECORD_ROWS = true;
const SOURCE_USE_ASSEMBLY_NAME_TABLE = true;
const SOURCE_USE_ASSEMBLY_TYPE_ONLY_STATE = true;
const SYMBOL_GRAPH_PATH_TABLE = true;
const pyEnabled = isPythonAvailable();
const tsEnabled = await isTreeSitterAvailable();
const goModule = findGoModule(ROOT);
const languageSupport = {
  ts: { enabled: true, reason: null },
  js: { enabled: true, reason: null },
  python: pyEnabled
    ? { enabled: true, reason: null, extractor: "python-ast-batch" }
    : { enabled: false, reason: "python executable unavailable" },
  go: tsEnabled
    ? { enabled: true, reason: null, extractor: "tree-sitter-wasm" }
    : { enabled: false, reason: "tree-sitter unavailable" },
};

const repoMode = detectRepoMode(ROOT);
const aliasMap = buildAliasMap(ROOT, repoMode, { exclude: cli.exclude });
let symbolResolver = null;

function resolveSpecifier(from, use, lane = "source-use") {
  if (!symbolResolver) {
    throw new Error("symbol resolver used before repo snapshot initialization");
  }
  return symbolResolver.resolve(from, use, lane);
}

if (verbose) console.error(`[symbols] root: ${ROOT}, mode: ${repoMode.mode}`);

// Per-language extractors live in `_lib/extract-{ts,py,go}.mjs`
// since v1.10.1. Each returns the canonical
// {filePath, defs, uses, reExports, loc, [pyDunderAll]} shape — the
// main scan loop below doesn't switch on language after this point.

const PRODUCER_ID = SYMBOL_GRAPH_PRODUCER_ID;
const PRODUCER_VERSION = SYMBOL_GRAPH_PRODUCER_VERSION;
const FACT_SCHEMA_VERSION = SYMBOL_GRAPH_FACT_SCHEMA_VERSION;
const PARSER_IDENTITY = SYMBOL_GRAPH_PARSER_IDENTITY;
const discovery = await discoverSymbolGraphFacts({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  repoMode: repoMode.mode,
  pythonEnabled: pyEnabled,
  treeSitterEnabled: tsEnabled,
  cacheRoot: cli.raw?.["cache-root"],
  clearCache: cli.raw?.["clear-incremental-cache"],
  noIncremental: cli.raw?.["no-incremental"],
  phaseTimer,
  verbose,
});
const {
  files,
  scannedJsSourceFiles,
  mdxSourceFiles,
  sfcSourceFiles,
  cacheStore,
  nextCache,
  fileData,
  useCount,
  warnings,
  incrementalEnabled,
  changedFiles,
  reusedFiles,
  droppedFiles,
  invalidatedFiles,
  assembleSymbolGraphStarted,
} = discovery;
symbolResolver = createSymbolGraphResolver({
  root: ROOT,
  aliasMap,
  sourceFiles: scannedJsSourceFiles,
  goModule,
});

// ─── 심볼 그래프 구축 ─────────────────────────────────────
// defIndex: Map<filePath, Map<symbolName, defInfo>>
const assembleDefIndexStarted = Date.now();
const defIndex = new Map();
for (const [f, info] of fileData) {
  const m = new Map();
  for (const d of info.defs) {
    if (!m.has(d.name)) m.set(d.name, d);
  }
  defIndex.set(f, m);
}
phaseTimer.recordPhase(
  "assemble-def-index",
  Date.now() - assembleDefIndexStarted,
);

function buildFanInInputs() {
  const identityCount = [...defIndex.values()].reduce(
    (count, definitions) => count + definitions.size,
    0,
  );

  return {
    consumerEntries: [],
    namespaceUserEntries: [],
    consumerSymbolCount: 0,
    identityCount,
  };
}

function buildDeadCandidateInputs() {
  const barrelFiles = [...detectBarrelFiles(ROOT, repoMode)];
  const testLikeFiles = files
    .map((file) => relPath(ROOT, file))
    .filter((file) => isTestLikePath(file));
  return { barrelFiles, testLikeFiles };
}

const sfcDiscovery = discoverSymbolGraphSfcFacts({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files,
  sfcSourceFiles,
  fileData,
  repoMode,
  phaseTimer,
});
const {
  scriptImportConsumers: sfcImportConsumers,
  scriptSources: sfcScriptSources,
  styleAssetReferences: sfcStyleAssets,
  templateComponentRefs: sfcTemplateRefs,
  globalComponentRegistrations: sfcGlobalRegistrations,
  generatedComponentManifests: sfcGeneratedManifests,
  frameworkConventionComponents: sfcFrameworkConventions,
} = sfcDiscovery;

let rustResolvedRelativeUses = 0;

function stripStyleAssetResourceQuery(spec) {
  const q = spec.indexOf("?");
  const h = spec.indexOf("#");
  const candidates = [];
  if (q >= 0) candidates.push(q);
  if (h > 0) candidates.push(h);
  return candidates.length ? spec.slice(0, Math.min(...candidates)) : spec;
}

function existingRelativeSpecifierTarget(consumerFile, spec) {
  if (typeof spec !== "string") return null;
  if (!spec.startsWith("./") && !spec.startsWith("../")) return null;
  const target = path.resolve(
    path.dirname(consumerFile),
    stripStyleAssetResourceQuery(spec),
  );
  return fileExists(target) ? target : null;
}

function existingRelativeNonSourceAssetTarget(consumerFile, spec) {
  if (!looksLikeNonSourceAssetSpecifier(spec)) return null;
  return existingRelativeSpecifierTarget(consumerFile, spec);
}

function existingExtensionlessRelativeRawTarget(consumerFile, spec) {
  if (typeof spec !== "string") return null;
  if (!spec.startsWith("./") && !spec.startsWith("../")) return null;
  const stripped = stripSpecifierResourceQuery(spec);
  const fileName = stripped.split("/").at(-1) ?? stripped;
  if (fileName.includes(".")) return null;
  const target = path.resolve(path.dirname(consumerFile), stripped);
  return fileExists(target) ? target : null;
}

function unresolvedInternalEvidence(consumerFile, use) {
  return symbolResolver.unresolvedEvidence(consumerFile, use);
}

const sourceUseRelPathCache = new Map();
let sourceUseRelPathCacheHits = 0;
let sourceUseRelPathCacheMisses = 0;

function sourceUseRelPath(value) {
  const cached = sourceUseRelPathCache.get(value);
  if (cached !== undefined) {
    sourceUseRelPathCacheHits++;
    return cached;
  }
  sourceUseRelPathCacheMisses++;
  const normalized = relPath(ROOT, value);
  sourceUseRelPathCache.set(value, normalized);
  return normalized;
}

const assembleSourceUsesStarted = Date.now();
let sourceUseCandidateBuildMs = 0;
let sourceUseJsResolutionLoopMs = 0;
const sourceUseTimings = {
  resolve: 0,
  external: 0,
  asset: 0,
  unresolved: 0,
  generatedVirtual: 0,
  namespaceReExport: 0,
  resolvedInternal: 0,
};
const sourceUseBranchCounts = {
  external: 0,
  asset: 0,
  unresolved: 0,
  generatedVirtual: 0,
  namespaceReExport: 0,
  resolvedInternal: 0,
  skippedNamespaceAlias: 0,
  generatedVirtualUnresolved: 0,
  namespaceReExportMiss: 0,
  namespaceReExportEscape: 0,
  namespaceReExportMember: 0,
  sideEffectOnly: 0,
  reExportNamespaceSkip: 0,
  broadNamespace: 0,
  directConsumer: 0,
  importMetaGlobResolved: 0,
  importMetaGlobUnsupported: 0,
};
function addSourceUseTiming(name, started) {
  sourceUseTimings[name] += performance.now() - started;
}

function incrementSourceUseBranch(name) {
  sourceUseBranchCounts[name] = (sourceUseBranchCounts[name] ?? 0) + 1;
}

function sourceUseRecordId(consumerFile, index) {
  return `${sourceUseRelPath(consumerFile)}#${index}`;
}

function outOfBandSourceUseRecordId(source, index, use) {
  return sourceUseProjectionRecordId(ROOT, source, index, use);
}

function stripSpecifierResourceQuery(spec) {
  const q = spec.indexOf("?");
  const h = spec.indexOf("#");
  const candidates = [];
  if (q >= 0) candidates.push(q);
  if (h > 0) candidates.push(h);
  return candidates.length ? spec.slice(0, Math.min(...candidates)) : spec;
}

const sourceUseRecordBuilder = createSourceUseRecordBuilder({
  normalizePath: sourceUseRelPath,
  unresolvedEvidence: unresolvedInternalEvidence,
  existingRelativeTarget: existingRelativeSpecifierTarget,
});
const sourceUseAssemblyRecord = sourceUseRecordBuilder.record;
const generatedVirtualSourceUseAssemblyRecord =
  sourceUseRecordBuilder.generatedVirtualRecord;
const nonSourceAssetSourceUseAssemblyRecord =
  sourceUseRecordBuilder.nonSourceAssetRecord;
const unresolvedSourceUseAssemblyRecord =
  sourceUseRecordBuilder.unresolvedRecord;
const terminalSourceUseRecord = sourceUseRecordBuilder.terminalRecord;

const embeddedSourceUseAssemblyRecords = [];

function enqueueUnresolvedSourceUseAssemblyRecord(
  recordId,
  consumerFile,
  use,
  resolverStage,
) {
  const record = unresolvedSourceUseAssemblyRecord(
    recordId,
    consumerFile,
    use,
    resolverStage,
  );
  if (!record) return false;
  embeddedSourceUseAssemblyRecords.push(record);
  return true;
}

function enqueueGeneratedVirtualSourceUseAssemblyRecord(
  recordId,
  consumerFile,
  use,
  surface,
) {
  const record = generatedVirtualSourceUseAssemblyRecord(
    recordId,
    consumerFile,
    use,
    surface,
  );
  if (!record) return false;
  embeddedSourceUseAssemblyRecords.push(record);
  return true;
}

function enqueueNonSourceAssetSourceUseAssemblyRecord(
  recordId,
  consumerFile,
  use,
) {
  const record = nonSourceAssetSourceUseAssemblyRecord(
    recordId,
    consumerFile,
    use,
  );
  if (!record) return false;
  embeddedSourceUseAssemblyRecords.push(record);
  return true;
}

function enqueueResolvedSourceUseAssemblyRecord(
  recordId,
  consumerFile,
  use,
  target,
) {
  const record = sourceUseAssemblyRecord(recordId, consumerFile, {
    ...use,
    resolvedFile: target,
    resolverStage: "resolved-internal",
  });
  const kind = record?.kind ?? "import";
  if (
    !record ||
    typeof record.fromSpec !== "string" ||
    record.fromSpec.length === 0 ||
    (sourceUseAssemblyRequiresSymbolName(kind) &&
      (typeof record.name !== "string" || record.name.length === 0))
  ) {
    return false;
  }
  embeddedSourceUseAssemblyRecords.push(record);
  return true;
}

function requireSourceUseAssemblyRecord(enqueued, recordId, outcome) {
  if (enqueued) return;
  throw new Error(`source-use assembly refused ${outcome} record ${recordId}`);
}

function buildOutOfBandSourceUseAssemblyCandidateRecords(consumers, source) {
  return planOutOfBandSourceUses({
    root: ROOT,
    consumers,
    source,
    recordBuilder: sourceUseRecordBuilder,
    canFastPathExternal: canFastPathExternalSourceUse,
    existingRelativeNonSourceAssetTarget,
  });
}

function buildSfcComponentSourceUseAssemblyCandidateRecords(
  consumers,
  source,
  options,
) {
  return planSfcComponentSourceUses({
    root: ROOT,
    consumers,
    source,
    ...options,
    recordBuilder: sourceUseRecordBuilder,
    canFastPathExternal: canFastPathExternalSourceUse,
    existingRelativeNonSourceAssetTarget,
    existingExtensionlessRelativeRawTarget,
    resolve: resolveSpecifier,
  });
}

function planSourceUseAssembly() {
  const candidateBuildStarted = performance.now();
  const candidates = planInlineSourceUses({
    root: ROOT,
    fileData,
    recordBuilder: sourceUseRecordBuilder,
    canFastPathExternal: canFastPathExternalSourceUse,
    existingRelativeNonSourceAssetTarget,
  });
  sourceUseCandidateBuildMs += performance.now() - candidateBuildStarted;
  embeddedSourceUseAssemblyRecords.push(...candidates.records);
  embeddedSourceUseAssemblyRecords.push(...candidates.requiresRustResolution);
  phaseTimer.setCounter("sourceUsePreHandledNamespaceReExportMissCount", 0);
  phaseTimer.setCounter(
    "sourceUseRustAssemblyNamespaceReExportCandidateCount",
    candidates.namespaceReExportCandidateCount,
  );
  phaseTimer.setCounter(
    "sourceUsePreHandledExternalCount",
    candidates.records.filter((record) => record.resolverStage === "external")
      .length,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyResolvableRelativeCandidateCount",
    candidates.requiresRustResolution.length,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyResolvableRelativeSkippedCount",
    0,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyResolvableRelativeDeferredCount",
    candidates.requiresRustResolution.length,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyCandidateCount",
    candidates.records.length + candidates.requiresRustResolution.length,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyEmbeddedCount",
    candidates.records.length + candidates.requiresRustResolution.length,
  );
  phaseTimer.setCounter(
    "sourceUseJsResolutionRequiredCount",
    candidates.requiresJsResolution.length,
  );
  const jsResolutionLanguageCounts = { JsTs: 0, Python: 0, Go: 0, Other: 0 };
  for (const entry of candidates.requiresJsResolution) {
    jsResolutionLanguageCounts[
      symbolResolver.languageBucket(entry.consumerFile)
    ]++;
  }
  for (const [language, count] of Object.entries(jsResolutionLanguageCounts)) {
    phaseTimer.setCounter(`sourceUseRecordsJsResolved${language}`, count);
  }
  return candidates.requiresJsResolution;
}

function canFastPathExternalSourceUse(consumerFile, use) {
  return symbolResolver.canFastPathExternal(consumerFile, use);
}

const sourceUsesRequiringJsResolution = planSourceUseAssembly();

const sourceUseJsResolutionLoopStarted = performance.now();
for (const {
  consumerFile,
  useIndex,
  use: u,
} of sourceUsesRequiringJsResolution) {
  const resolveStarted = performance.now();
  const target = resolveSpecifier(consumerFile, u, "source-use-js-resolution");
  addSourceUseTiming("resolve", resolveStarted);
  if (isRustResolvedRelativeUse(u)) rustResolvedRelativeUses++;
  const recordId = sourceUseRecordId(consumerFile, useIndex);
  const branchStarted = performance.now();
  const { branch, outcome, record } = terminalSourceUseRecord(
    recordId,
    consumerFile,
    u,
    target,
  );
  requireSourceUseAssemblyRecord(Boolean(record), recordId, outcome);
  embeddedSourceUseAssemblyRecords.push(record);
  incrementSourceUseBranch(branch);
  addSourceUseTiming(branch, branchStarted);
}
sourceUseJsResolutionLoopMs +=
  performance.now() - sourceUseJsResolutionLoopStarted;
phaseTimer.recordPhase("assemble-source-use-resolve", sourceUseTimings.resolve);
phaseTimer.recordPhase(
  "assemble-source-use-external",
  sourceUseTimings.external,
);
phaseTimer.recordPhase("assemble-source-use-asset", sourceUseTimings.asset);
phaseTimer.recordPhase(
  "assemble-source-use-unresolved",
  sourceUseTimings.unresolved,
);
phaseTimer.recordPhase(
  "assemble-source-use-generated-virtual",
  sourceUseTimings.generatedVirtual,
);
phaseTimer.recordPhase(
  "assemble-source-use-namespace-reexport",
  sourceUseTimings.namespaceReExport,
);
phaseTimer.recordPhase(
  "assemble-source-use-resolved-internal",
  sourceUseTimings.resolvedInternal,
);
phaseTimer.recordPhase(
  "assemble-source-use-candidate-build",
  sourceUseCandidateBuildMs,
);
phaseTimer.recordPhase(
  "assemble-source-use-js-resolution",
  sourceUseJsResolutionLoopMs,
);
const sourceUseMeasuredBranchMs =
  sourceUseTimings.resolve +
  sourceUseTimings.external +
  sourceUseTimings.asset +
  sourceUseTimings.unresolved +
  sourceUseTimings.generatedVirtual +
  sourceUseTimings.namespaceReExport +
  sourceUseTimings.resolvedInternal;
phaseTimer.setCounter(
  "sourceUseJsResolutionLoopOverheadMs",
  Math.max(0, sourceUseJsResolutionLoopMs - sourceUseMeasuredBranchMs),
);
phaseTimer.setCounter("sourceUseCandidateBuildMs", sourceUseCandidateBuildMs);
phaseTimer.setCounter(
  "sourceUseJsResolutionLoopMs",
  sourceUseJsResolutionLoopMs,
);
phaseTimer.setCounter("sourceUseRelPathCacheHits", sourceUseRelPathCacheHits);
phaseTimer.setCounter(
  "sourceUseRelPathCacheMisses",
  sourceUseRelPathCacheMisses,
);
phaseTimer.setCounter("sourceUseRelPathCacheSize", sourceUseRelPathCache.size);
phaseTimer.setCounter("sourceUseResolveMs", sourceUseTimings.resolve);
phaseTimer.setCounter(
  "sourceUseRustResolvedRelativeCount",
  rustResolvedRelativeUses,
);
phaseTimer.setCounter("sourceUseExternalMs", sourceUseTimings.external);
phaseTimer.setCounter("sourceUseAssetMs", sourceUseTimings.asset);
phaseTimer.setCounter("sourceUseUnresolvedMs", sourceUseTimings.unresolved);
phaseTimer.setCounter(
  "sourceUseGeneratedVirtualMs",
  sourceUseTimings.generatedVirtual,
);
phaseTimer.setCounter(
  "sourceUseNamespaceReExportMs",
  sourceUseTimings.namespaceReExport,
);
phaseTimer.setCounter(
  "sourceUseResolvedInternalMs",
  sourceUseTimings.resolvedInternal,
);
for (const [name, count] of Object.entries(sourceUseBranchCounts)) {
  phaseTimer.setCounter(
    `sourceUse${name[0].toUpperCase()}${name.slice(1)}BranchCount`,
    count,
  );
}
phaseTimer.setCounter("sourceUseFilesProcessed", fileData.size);
phaseTimer.setCounter("sourceUseRecordsProcessed", useCount);
phaseTimer.setCounter(
  "sourceUseRecordsJsResolved",
  sourceUsesRequiringJsResolution.length,
);
const sourceUseResolverCallCountAfterMainAssembly = symbolResolver.callCount;
const sourceUseResolverRawJsCallCountAfterMainAssembly =
  symbolResolver.rawJsCallCount;
phaseTimer.setCounter(
  "sourceUseResolverCallCount",
  sourceUseResolverCallCountAfterMainAssembly,
);
phaseTimer.setCounter(
  "sourceUseResolverRawJsCallCount",
  sourceUseResolverRawJsCallCountAfterMainAssembly,
);
phaseTimer.recordPhase(
  "assemble-source-uses",
  Date.now() - assembleSourceUsesStarted,
);

function processOutOfBandImportConsumers(
  consumers,
  source,
  handledRecords = new Set(),
) {
  for (let index = 0; index < consumers.length; index++) {
    const u = consumers[index];
    const recordId = outOfBandSourceUseRecordId(source, index, u);
    if (handledRecords.has(recordId)) {
      continue;
    }
    const target = resolveSpecifier(
      u.consumerFile,
      u,
      "out-of-band-import-consumer",
    );
    const { outcome, record } = terminalSourceUseRecord(
      recordId,
      u.consumerFile,
      u,
      target,
      source,
    );
    requireSourceUseAssemblyRecord(Boolean(record), recordId, outcome);
    embeddedSourceUseAssemblyRecords.push(record);
  }
}

const SFC_SCRIPT_SRC_SOURCE_EXTS = [
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".mts",
  ".cts",
  ".d.ts",
  ".d.mts",
  ".d.cts",
];
const SFC_SCRIPT_SRC_INDEX_EXTS = SFC_SCRIPT_SRC_SOURCE_EXTS.map(
  (ext) => `/index${ext}`,
);

function isScannedJsSourceFile(filePath) {
  if (typeof filePath !== "string" || filePath.length === 0) return false;
  return (
    scannedJsSourceFiles.has(filePath) ||
    scannedJsSourceFiles.has(path.resolve(filePath))
  );
}

function stripSfcScriptSrcResourceQuery(spec) {
  const value = String(spec ?? "");
  const query = value.indexOf("?");
  const fragment = value.indexOf("#");
  const cuts = [query, fragment].filter((index) => index >= 0);
  return cuts.length === 0 ? value : value.slice(0, Math.min(...cuts));
}

function resolveSfcScriptScannedSourceFallback(consumerFile, fromSpec) {
  if (!isSourceUseAssemblyCandidate({ fromSpec })) return null;
  const base = path.resolve(
    path.dirname(consumerFile),
    stripSfcScriptSrcResourceQuery(fromSpec),
  );
  for (const ext of SFC_SCRIPT_SRC_SOURCE_EXTS) {
    const candidate = base + ext;
    if (isScannedJsSourceFile(candidate)) return candidate;
  }
  for (const ext of SFC_SCRIPT_SRC_INDEX_EXTS) {
    const candidate = base + ext;
    if (isScannedJsSourceFile(candidate)) return candidate;
  }
  return null;
}

function sfcScriptSrcAssemblyUse(use, overrides = {}) {
  return {
    ...use,
    kind: "sfc-script-src",
    typeOnly: false,
    consumerSource: "sfc-script-src",
    ...overrides,
  };
}

function processSfcScriptSourceReachability(consumers, handled = new Set()) {
  for (let index = 0; index < consumers.length; index++) {
    const u = consumers[index];
    const recordId = outOfBandSourceUseRecordId("sfc-script-src", index, u);
    const rustCandidate = handled.has(recordId);
    if (rustCandidate) {
      continue;
    }
    let target = resolveSpecifier(u.consumerFile, u, "sfc-script-src");
    if (target === "EXTERNAL") continue;
    if (isNonSourceAssetResolution(target)) {
      requireSourceUseAssemblyRecord(
        enqueueNonSourceAssetSourceUseAssemblyRecord(
          recordId,
          u.consumerFile,
          sfcScriptSrcAssemblyUse(u),
        ),
        recordId,
        "non-source-asset",
      );
      continue;
    }
    if (target === "UNRESOLVED_INTERNAL" || !target) {
      const diagnosticUse = {
        ...sfcScriptSrcAssemblyUse(u),
        reason: "sfc-script-src-unresolved",
        resolverStage: "sfc-script-src",
        outputLevel: "unsupported",
        unsupportedFamily: "sfc-script-src",
        hint: "sfc-script-src-reachability",
      };
      requireSourceUseAssemblyRecord(
        enqueueUnresolvedSourceUseAssemblyRecord(
          recordId,
          u.consumerFile,
          diagnosticUse,
          target === "UNRESOLVED_INTERNAL"
            ? "unresolved-internal"
            : "unresolved-relative",
        ),
        recordId,
        target === "UNRESOLVED_INTERNAL"
          ? "unresolved-internal"
          : "unresolved-relative",
      );
      continue;
    }
    if (isGeneratedVirtualResolution(target)) {
      requireSourceUseAssemblyRecord(
        enqueueGeneratedVirtualSourceUseAssemblyRecord(
          recordId,
          u.consumerFile,
          sfcScriptSrcAssemblyUse(u),
          target,
        ),
        recordId,
        "generated-virtual",
      );
      continue;
    }
    if (!isScannedJsSourceFile(target)) {
      const sourceTarget = resolveSfcScriptScannedSourceFallback(
        u.consumerFile,
        u.fromSpec,
      );
      if (sourceTarget) {
        target = sourceTarget;
      } else if (typeof target === "string" && fileExists(target)) {
        requireSourceUseAssemblyRecord(
          enqueueNonSourceAssetSourceUseAssemblyRecord(
            recordId,
            u.consumerFile,
            sfcScriptSrcAssemblyUse(u),
          ),
          recordId,
          "non-source-asset",
        );
        continue;
      } else {
        const diagnosticUse = {
          ...sfcScriptSrcAssemblyUse(u),
          reason: "sfc-script-src-unscanned-target",
          resolverStage: "sfc-script-src",
          outputLevel: "unsupported",
          unsupportedFamily: "sfc-script-src",
          hint: "sfc-script-src-source-target",
        };
        requireSourceUseAssemblyRecord(
          enqueueUnresolvedSourceUseAssemblyRecord(
            recordId,
            u.consumerFile,
            diagnosticUse,
            "unresolved-internal",
          ),
          recordId,
          "unresolved-internal",
        );
        continue;
      }
    }

    requireSourceUseAssemblyRecord(
      enqueueResolvedSourceUseAssemblyRecord(
        recordId,
        u.consumerFile,
        sfcScriptSrcAssemblyUse(u),
        target,
      ),
      recordId,
      "resolved-internal",
    );
  }
}

const assembleMdxUsesStarted = Date.now();
const mdxImportConsumers =
  mdxSourceFiles.length > 0
    ? collectMdxImportConsumers({
        root: ROOT,
        includeTests: cli.includeTests,
        exclude: cli.exclude,
        files: mdxSourceFiles,
      })
    : [];
phaseTimer.setCounter(
  "mdxImportConsumerCandidateCount",
  mdxImportConsumers.length,
);
const mdxSourceUseAssemblyRecords =
  buildOutOfBandSourceUseAssemblyCandidateRecords(
    mdxImportConsumers,
    "mdx-import",
  );

const assembleSfcScriptUsesStarted = Date.now();
const sfcScriptSourceUseAssemblyRecords =
  buildOutOfBandSourceUseAssemblyCandidateRecords(
    sfcImportConsumers,
    "sfc-script-import",
  );

const assembleSfcScriptSrcStarted = Date.now();
const sfcScriptSrcSourceUseAssemblyRecords =
  buildOutOfBandSourceUseAssemblyCandidateRecords(
    sfcScriptSources,
    "sfc-script-src",
  );

const outOfBandSourceUseAssemblyResolutionStarted = Date.now();
const outOfBandSourceUseAssemblyRecords = [
  ...mdxSourceUseAssemblyRecords,
  ...sfcScriptSourceUseAssemblyRecords,
  ...sfcScriptSrcSourceUseAssemblyRecords,
];
embeddedSourceUseAssemblyRecords.push(...outOfBandSourceUseAssemblyRecords);
phaseTimer.recordPhase(
  "source-use-out-of-band-rust-assembly",
  Date.now() - outOfBandSourceUseAssemblyResolutionStarted,
);
phaseTimer.setCounter("outOfBandSourceUseRustAssemblySkippedCount", 0);

const mdxSourceUseAssemblyHandled = new Set(
  mdxSourceUseAssemblyRecords.map((record) => record.recordId),
);
phaseTimer.setCounter(
  "mdxSourceUseRustAssemblyCandidateCount",
  mdxSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "mdxSourceUseRustAssemblyEmbeddedCount",
  mdxSourceUseAssemblyRecords.length,
);
processOutOfBandImportConsumers(
  mdxImportConsumers,
  "mdx-import",
  mdxSourceUseAssemblyHandled,
);
phaseTimer.recordPhase(
  "assemble-mdx-uses",
  Date.now() - assembleMdxUsesStarted,
);

const sfcScriptSourceUseAssemblyHandled = new Set(
  sfcScriptSourceUseAssemblyRecords.map((record) => record.recordId),
);
phaseTimer.setCounter(
  "sfcScriptSourceUseRustAssemblyCandidateCount",
  sfcScriptSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sfcScriptSourceUseRustAssemblyEmbeddedCount",
  sfcScriptSourceUseAssemblyRecords.length,
);
processOutOfBandImportConsumers(
  sfcImportConsumers,
  "sfc-script-import",
  sfcScriptSourceUseAssemblyHandled,
);
phaseTimer.recordPhase(
  "assemble-sfc-script-uses",
  Date.now() - assembleSfcScriptUsesStarted,
);

const sfcScriptSrcSourceUseAssemblyHandled = new Set(
  sfcScriptSrcSourceUseAssemblyRecords.map((record) => record.recordId),
);
phaseTimer.setCounter(
  "sfcScriptSrcSourceUseRustAssemblyCandidateCount",
  sfcScriptSrcSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sfcScriptSrcSourceUseRustAssemblyEmbeddedCount",
  sfcScriptSrcSourceUseAssemblyRecords.length,
);
processSfcScriptSourceReachability(
  sfcScriptSources,
  sfcScriptSrcSourceUseAssemblyHandled,
);
phaseTimer.recordPhase(
  "assemble-sfc-script-src-uses",
  Date.now() - assembleSfcScriptSrcStarted,
);

const sfcTemplateComponentSourceUseAssemblyRecords =
  buildSfcComponentSourceUseAssemblyCandidateRecords(
    sfcTemplateRefs,
    "sfc-template-component-ref",
    {
      consumerFileForUse: (use) => use.consumerFile,
      fromSpecForUse: (use) => use.bindingSource,
      kind: "sfc-template-component-ref",
      allowExternal: true,
    },
  );
const sfcGlobalComponentSourceUseAssemblyRecords =
  buildSfcComponentSourceUseAssemblyCandidateRecords(
    sfcGlobalRegistrations,
    "sfc-global-component-registration",
    {
      consumerFileForUse: (use) => use.registrationFile,
      fromSpecForUse: sfcGlobalComponentResolutionSpec,
      kind: "sfc-global-component-registration",
      allowExternal: true,
    },
  );
const sfcGeneratedManifestSourceUseAssemblyRecords =
  buildSfcComponentSourceUseAssemblyCandidateRecords(
    sfcGeneratedManifests,
    "sfc-generated-component-manifest",
    {
      consumerFileForUse: (use) => use.manifestFile,
      fromSpecForUse: (use) => use.bindingSource,
      kind: "sfc-generated-component-manifest",
      allowExternal: true,
    },
  );
const sfcComponentSourceUseAssemblyStarted = Date.now();
const sfcComponentSourceUseAssemblyRecords = [
  ...sfcTemplateComponentSourceUseAssemblyRecords,
  ...sfcGlobalComponentSourceUseAssemblyRecords,
  ...sfcGeneratedManifestSourceUseAssemblyRecords,
];
embeddedSourceUseAssemblyRecords.push(...sfcComponentSourceUseAssemblyRecords);
const sfcTemplateComponentSourceUseAssemblyRecordIds = new Set(
  sfcTemplateComponentSourceUseAssemblyRecords.map((record) => record.recordId),
);
const sfcGlobalComponentSourceUseAssemblyRecordIds = new Set(
  sfcGlobalComponentSourceUseAssemblyRecords.map((record) => record.recordId),
);
const sfcGeneratedManifestSourceUseAssemblyRecordIds = new Set(
  sfcGeneratedManifestSourceUseAssemblyRecords.map((record) => record.recordId),
);
phaseTimer.recordPhase(
  "sfc-component-source-use-rust-assembly",
  Date.now() - sfcComponentSourceUseAssemblyStarted,
);
phaseTimer.setCounter(
  "sfcComponentSourceUseRustAssemblyCandidateCount",
  sfcComponentSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sfcComponentSourceUseRustAssemblyEmbeddedCount",
  sfcComponentSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter("sfcComponentSourceUseRustAssemblySkippedCount", 0);

const assembleSfcProjectionInputsStarted = Date.now();
const sfcInputs = buildSymbolGraphSfcInputs({
  root: ROOT,
  styleAssetReferences: sfcStyleAssets,
  templateComponentRefs: sfcTemplateRefs,
  globalComponentRegistrations: sfcGlobalRegistrations,
  generatedComponentManifests: sfcGeneratedManifests,
  frameworkConventionComponents: sfcFrameworkConventions,
  templateRecordIds: sfcTemplateComponentSourceUseAssemblyRecordIds,
  globalRecordIds: sfcGlobalComponentSourceUseAssemblyRecordIds,
  generatedManifestRecordIds: sfcGeneratedManifestSourceUseAssemblyRecordIds,
});
phaseTimer.recordPhase(
  "assemble-sfc-projection-inputs",
  Date.now() - assembleSfcProjectionInputsStarted,
);

phaseTimer.setCounter(
  "sourceUseResolverPostSourceUseCallCount",
  Math.max(
    0,
    symbolResolver.callCount - sourceUseResolverCallCountAfterMainAssembly,
  ),
);
phaseTimer.setCounter(
  "sourceUseResolverPostSourceUseRawJsCallCount",
  Math.max(
    0,
    symbolResolver.rawJsCallCount -
      sourceUseResolverRawJsCallCountAfterMainAssembly,
  ),
);
symbolResolver.recordTelemetry(phaseTimer);

console.log(
  `[defs] total symbols: ${[...defIndex.values()].reduce((a, m) => a + m.size, 0)}`,
);
phaseTimer.setCounter(
  "sfcStyleAssetReferenceCount",
  sfcInputs.styleAssetReferences.length,
);
phaseTimer.setCounter(
  "sfcTemplateComponentRefCount",
  sfcInputs.templateComponentRefs.length,
);
phaseTimer.setCounter(
  "sfcGlobalComponentRegistrationCount",
  sfcInputs.globalComponentRegistrations.length,
);
phaseTimer.setCounter(
  "sfcGeneratedComponentManifestCount",
  sfcInputs.generatedComponentManifests.length,
);
phaseTimer.setCounter(
  "sfcFrameworkConventionComponentCount",
  sfcInputs.frameworkConventionComponents.length,
);

// ─── Dead export raw inputs ───────────────────────────────
const assembleDeadCandidatesStarted = Date.now();
const deadCandidateInputs = buildDeadCandidateInputs();
phaseTimer.setCounter(
  "barrelFileCount",
  deadCandidateInputs.barrelFiles.length,
);
phaseTimer.recordPhase(
  "assemble-dead-candidates",
  Date.now() - assembleDeadCandidatesStarted,
);

// ─── Symbol fan-in raw inputs ────────────────────────────
const assembleFanInStarted = Date.now();
const fanInInputs = buildFanInInputs();
phaseTimer.recordPhase("assemble-fan-in", Date.now() - assembleFanInStarted);

// ─── 리포트 ───────────────────────────────────────────────
console.log(`\n\n════════ 1. Top 25 심볼 fan-in ════════`);
console.log(`  Rust-owned fan-in projection is written to symbols.json`);

// ─── Dead 요약 ───────────────────────────────────────────
console.log(`\n\n════════ 2. Dead export 후보 ════════`);
console.log(
  `  Rust-owned dead candidate projection is written to symbols.json`,
);
phaseTimer.setCounter("symbolFanInCount", fanInInputs.consumerSymbolCount);
phaseTimer.setCounter("fanInIdentityCount", fanInInputs.identityCount);
phaseTimer.setCounter("fanInIdentitySpaceCount", fanInInputs.identityCount);
const compactEmbeddedSourceUseRecordIds = true;
const embeddedSourceUseRecordIdRemap = compactEmbeddedSourceUseRecordIds
  ? sourceUseRecordIdRemap(embeddedSourceUseAssemblyRecords)
  : new Map();
const embeddedSourceUseAssemblyRequest = buildSourceUseAssemblyRequest({
  root: ROOT,
  sourceFiles: scannedJsSourceFiles,
  importMetaGlobCap: DEFAULT_IMPORT_META_GLOB_CAP,
  records: embeddedSourceUseAssemblyRecords,
  includeSourceFiles: sourceUseAssemblyNeedsSourceFiles(
    embeddedSourceUseAssemblyRecords,
  ),
  compactRecordIds: compactEmbeddedSourceUseRecordIds,
  omitRecordIds: compactEmbeddedSourceUseRecordIds,
  compactPaths: SOURCE_USE_ASSEMBLY_PATH_TABLE,
  compactEnums: SOURCE_USE_ASSEMBLY_ENUM_TABLE,
  compactSpecifiers: SOURCE_USE_ASSEMBLY_SPECIFIER_TABLE,
  compactNames: SOURCE_USE_ASSEMBLY_NAME_TABLE,
  compactTypeOnly: SOURCE_USE_ASSEMBLY_TYPE_ONLY_STATE,
  compactRows: SOURCE_USE_ASSEMBLY_RECORD_ROWS,
});
phaseTimer.setCounter(
  "sourceUseRustAssemblyTotalEmbeddedCount",
  embeddedSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyCompactedRecordIdCount",
  embeddedSourceUseAssemblyRequest.records?.length ??
    embeddedSourceUseAssemblyRequest.recordRows?.length ??
    0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblySourceFileCount",
  embeddedSourceUseAssemblyRequest.sourceFiles?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyPathTableEnabled",
  SOURCE_USE_ASSEMBLY_PATH_TABLE ? 1 : 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyPathTableCount",
  embeddedSourceUseAssemblyRequest.pathTable?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyEnumTableEnabled",
  SOURCE_USE_ASSEMBLY_ENUM_TABLE ? 1 : 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyKindTableCount",
  embeddedSourceUseAssemblyRequest.kindTable?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyResolverStageTableCount",
  embeddedSourceUseAssemblyRequest.resolverStageTable?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyConsumerSourceTableCount",
  embeddedSourceUseAssemblyRequest.consumerSourceTable?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblySpecifierTableEnabled",
  SOURCE_USE_ASSEMBLY_SPECIFIER_TABLE ? 1 : 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblySpecifierTableCount",
  embeddedSourceUseAssemblyRequest.specifierTable?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyNameTableEnabled",
  SOURCE_USE_ASSEMBLY_NAME_TABLE ? 1 : 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyNameTableCount",
  embeddedSourceUseAssemblyRequest.nameTable?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyTypeOnlyStateEnabled",
  SOURCE_USE_ASSEMBLY_TYPE_ONLY_STATE ? 1 : 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyRecordRowsEnabled",
  SOURCE_USE_ASSEMBLY_RECORD_ROWS ? 1 : 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyRecordRowFieldCount",
  embeddedSourceUseAssemblyRequest.recordRowFields?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyRecordRowCount",
  embeddedSourceUseAssemblyRequest.recordRows?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyNamespaceReExportEntryCount",
  embeddedSourceUseAssemblyRequest.namespaceReExports?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyNamedReExportEntryCount",
  embeddedSourceUseAssemblyRequest.namedReExports?.length ?? 0,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyInputBytes",
  Buffer.byteLength(JSON.stringify(embeddedSourceUseAssemblyRequest), "utf8"),
);
phaseTimer.recordPhase(
  "assemble-symbol-graph",
  Date.now() - assembleSymbolGraphStarted,
);

// ─── 저장 ─────────────────────────────────────────────────
const outPath = path.join(output, "symbols.json");
const generated = new Date().toISOString();
const buildArtifactRequestStarted = Date.now();
const { request: artifactRequest, artifactParseErrorFiles } =
  buildSymbolGraphArtifactRequest({
    root: ROOT,
    context: {
      generated,
      root: ROOT,
      includeTests: cli.includeTests,
      exclude: cli.exclude,
      generatedArtifactsMode: GENERATED_ARTIFACTS_MODE,
      languageSupport,
      warnings,
      incremental: {
        enabled: incrementalEnabled,
        identityMode: incrementalEnabled ? STRICT_IDENTITY_MODE : null,
        cacheVersion: 1,
        cacheRoot: incrementalEnabled ? cacheStore.cacheRoot : null,
        changedFiles,
        reusedFiles,
        droppedFiles,
        invalidatedFiles,
        reason: incrementalEnabled ? null : "disabled-by-flag",
      },
    },
    files,
    defIndex,
    fileData,
    cacheEntries: nextCache.entries,
    sourceUseAssembly: embeddedSourceUseAssemblyRequest,
    graph: {
      fanIn: {
        consumerEntries: fanInInputs.consumerEntries,
        namespaceUserEntries: fanInInputs.namespaceUserEntries,
      },
      deadCandidates: deadCandidateInputs,
      sfc: {
        styleAssetReferences: sfcInputs.styleAssetReferences,
        templateComponentRefs: remapSourceUseRecordIdInputs(
          sfcInputs.templateComponentRefs,
          embeddedSourceUseRecordIdRemap,
        ),
        globalComponentRegistrations: remapSourceUseRecordIdInputs(
          sfcInputs.globalComponentRegistrations,
          embeddedSourceUseRecordIdRemap,
        ),
        generatedComponentManifests: remapSourceUseRecordIdInputs(
          sfcInputs.generatedComponentManifests,
          embeddedSourceUseRecordIdRemap,
        ),
        generatedManifestExternalUses: sfcInputs.generatedManifestExternalUses,
        frameworkConventionComponents: sfcInputs.frameworkConventionComponents,
      },
    },
    compactPaths: SYMBOL_GRAPH_PATH_TABLE,
  });
phaseTimer.recordPhase(
  "build-symbol-artifact-request",
  Date.now() - buildArtifactRequestStarted,
);
phaseTimer.setCounter("symbolGraphArtifactRequestFileCount", files.length);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestFileDataCount",
  artifactRequest.extraction.fileData.length,
);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestDefIndexCount",
  artifactRequest.extraction.defIndex.length,
);
phaseTimer.setCounter(
  "symbolGraphArtifactPathTableEnabled",
  SYMBOL_GRAPH_PATH_TABLE ? 1 : 0,
);
phaseTimer.setCounter(
  "symbolGraphArtifactPathTableCount",
  artifactRequest.extraction.pathTable.length,
);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestParseErrorCacheEntryCount",
  artifactParseErrorFiles.length,
);
const writtenSymbolSummary = finalizeSymbolGraphArtifact({
  request: artifactRequest,
  outPath,
  incrementalEnabled,
  cacheStore,
  producer: {
    id: PRODUCER_ID,
    version: PRODUCER_VERSION,
    factSchemaVersion: FACT_SCHEMA_VERSION,
    parserIdentity: PARSER_IDENTITY,
  },
  phaseTimer,
});
phaseTimer.setCounter("totalUses", writtenSymbolSummary.totalUsesResolved);
phaseTimer.setCounter("unresolvedUses", writtenSymbolSummary.unresolvedUses);
phaseTimer.setCounter(
  "resolvedInternalUses",
  writtenSymbolSummary.uses.resolvedInternal,
);
phaseTimer.setCounter(
  "resolvedGeneratedVirtualUses",
  writtenSymbolSummary.uses.resolvedGeneratedVirtual,
);
phaseTimer.setCounter(
  "nonSourceAssetUses",
  writtenSymbolSummary.uses.nonSourceAsset,
);
phaseTimer.setCounter("externalUses", writtenSymbolSummary.uses.external);
phaseTimer.setCounter(
  "unresolvedInternalUses",
  writtenSymbolSummary.uses.unresolvedInternal,
);
phaseTimer.setCounter(
  "mdxConsumerUses",
  writtenSymbolSummary.uses.mdxConsumers,
);
phaseTimer.setCounter(
  "sfcScriptConsumerUses",
  writtenSymbolSummary.uses.sfcScriptConsumers,
);
phaseTimer.setCounter(
  "sfcScriptSrcReachabilityUses",
  writtenSymbolSummary.uses.sfcScriptSrcReachability,
);
phaseTimer.setCounter(
  "sfcStyleAssetReferenceUses",
  writtenSymbolSummary.uses.sfcStyleAssetReferences,
);
phaseTimer.setCounter(
  "sfcTemplateComponentRefUses",
  writtenSymbolSummary.uses.sfcTemplateComponentRefs,
);
phaseTimer.setCounter(
  "sfcGlobalComponentRegistrationUses",
  writtenSymbolSummary.uses.sfcGlobalComponentRegistrations,
);
phaseTimer.setCounter(
  "sfcGeneratedComponentManifestUses",
  writtenSymbolSummary.uses.sfcGeneratedComponentManifests,
);
phaseTimer.setCounter(
  "sfcFrameworkConventionComponentUses",
  writtenSymbolSummary.uses.sfcFrameworkConventionComponents,
);
phaseTimer.setCounter(
  "resolvedInternalEdgeCount",
  writtenSymbolSummary.resolvedInternalEdgeCount,
);
phaseTimer.setCounter(
  "unresolvedInternalSpecifierRecordCount",
  writtenSymbolSummary.uses.unresolvedInternal,
);
phaseTimer.setCounter("deadCandidateCount", writtenSymbolSummary.deadTotal);
phaseTimer.setCounter("trulyDeadCount", writtenSymbolSummary.trulyDead);
phaseTimer.setCounter(
  "namespaceShadowedDeadCount",
  writtenSymbolSummary.deadTotal - writtenSymbolSummary.trulyDead,
);
phaseTimer.setCounter("deadProductionCount", writtenSymbolSummary.deadInProd);
phaseTimer.setCounter("deadTestCount", writtenSymbolSummary.deadInTest);
phaseTimer.setCounter(
  "generatedConsumerBlindZoneCount",
  writtenSymbolSummary.generatedConsumerBlindZoneCount,
);
phaseTimer.write();
console.log(
  `[uses] total ${writtenSymbolSummary.totalUsesResolved}, unresolved ${writtenSymbolSummary.unresolvedUses}`,
);
console.log(
  `[uses] resolvedInternal: ${writtenSymbolSummary.uses.resolvedInternal}, external: ${writtenSymbolSummary.uses.external}, unresolvedInternal: ${writtenSymbolSummary.uses.unresolvedInternal}`,
);
console.log(
  `[symbols] ${files.length} files, dead production candidates: ${writtenSymbolSummary.deadInProd}`,
);
console.log(`[symbols] saved → ${outPath}`);
