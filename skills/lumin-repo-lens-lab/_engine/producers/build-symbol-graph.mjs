// build-symbol-graph.mjs — Symbol-level export/import graph (parameterized)
//
// For each file:
// - collect top-level export definitions (not re-exports)
// - collect import/re-export specifiers (uses)
// - build (definition file, symbol) -> consumer set mapping
// - derive: dead exports, symbol fan-in, top consumers
//
// Usage: node build-symbol-graph.mjs --root <repo> [--output <dir>]

import { createHash } from "node:crypto";
import {
  closeSync,
  openSync,
  readFileSync,
  readSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";

import { detectBarrelFiles } from "../lib/alias-map.mjs";
import { extractDefinitionsAndUses } from "../lib/extract-ts.mjs";
import { extractRustJsHybridBatch } from "../lib/extract-ts-rust-hybrid.mjs";
import { goExtractShape } from "../lib/extract-go.mjs";
import { pythonExtractShape } from "../lib/extract-py.mjs";
import { parseCliArgs } from "../lib/cli.mjs";
import { detectRepoMode } from "../lib/repo-mode.mjs";
import { buildAliasMap } from "../lib/alias-map.mjs";
import {
  explainUnresolvedSpecifier,
  isGeneratedVirtualResolution,
  isNonSourceAssetResolution,
  makeResolver,
} from "../lib/resolver-core.mjs";
import { collectMdxImportConsumers } from "../lib/mdx-consumers.mjs";
import {
  collectSfcFrameworkConventionComponents,
  collectSfcGeneratedComponentManifests,
  collectSfcGlobalComponentRegistrations,
  collectSfcImportConsumers,
  collectSfcScriptSources,
  collectSfcStyleAssetReferences,
  collectSfcTemplateComponentRefs,
} from "../lib/sfc-consumers.mjs";
import { buildGeneratedConsumerBlindZones } from "../lib/generated-blind-zone-relevance.mjs";
import { isGeneratedArtifactMissingRecord } from "../lib/generated-artifact-evidence.mjs";
import { normalizeGeneratedArtifactsMode } from "../lib/generated-artifact-mode.mjs";
import {
  DEFAULT_IMPORT_META_GLOB_CAP,
  expandImportMetaGlobPattern,
} from "../lib/import-meta-glob-expansion.mjs";
import { JS_FAMILY_LANGS, SFC_FAMILY_LANGS } from "../lib/lang.mjs";
import { isTestLikePath } from "../lib/test-paths.mjs";
import { fileExists, relPath } from "../lib/paths.mjs";
import {
  auditCoreRuntimeFeatureEnabled,
  runAuditCoreJsonResultFile,
  runAuditCoreJsonToResultFile,
} from "../lib/audit-core.mjs";
import { buildAnyContaminationFacts } from "../lib/any-contamination.mjs";
import {
  buildContextFingerprint,
  buildRepoSnapshot,
  STRICT_IDENTITY_MODE,
} from "../lib/incremental-snapshot.mjs";
import {
  clearIncrementalCache,
  getReusableFact,
  loadProducerCache,
  openIncrementalCacheStore,
  putFact,
  saveProducerCache,
  strictCacheKeyForEntry,
} from "../lib/incremental-cache-store.mjs";
import {
  isPythonAvailable,
  extractPythonBatch,
  resolvePythonImport,
} from "../lib/python.mjs";
import {
  isTreeSitterAvailable,
  extractTreeSitterBatch,
  findGoModule,
  resolveGoImport,
} from "../lib/tree-sitter-langs.mjs";
import { createProducerPhaseTimer } from "../lib/producer-phase-timing.mjs";

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
const SOURCE_USE_ASSEMBLY_PATH_TABLE =
  auditCoreRuntimeFeatureEnabled("sourceUseAssemblyPathTable");
const SOURCE_USE_ASSEMBLY_ENUM_TABLE =
  auditCoreRuntimeFeatureEnabled("sourceUseAssemblyEnumTable");
const SOURCE_USE_ASSEMBLY_SPECIFIER_TABLE = auditCoreRuntimeFeatureEnabled(
  "sourceUseAssemblySpecifierTable",
);
const SYMBOL_GRAPH_PATH_TABLE =
  auditCoreRuntimeFeatureEnabled("symbolGraphPathTable");
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

const SYMBOL_GRAPH_SUMMARY_PREFIX_BYTES = 64 * 1024;

function readFilePrefix(filePath, byteLimit) {
  const fd = openSync(filePath, "r");
  try {
    const buffer = Buffer.alloc(byteLimit);
    const bytesRead = readSync(fd, buffer, 0, buffer.length, 0);
    return buffer.toString("utf8", 0, bytesRead);
  } finally {
    closeSync(fd);
  }
}

function extractJsonObjectAfterKey(text, key) {
  const marker = `"${key}"`;
  const markerIndex = text.indexOf(marker);
  if (markerIndex < 0) return null;
  const colonIndex = text.indexOf(":", markerIndex + marker.length);
  if (colonIndex < 0) return null;
  const start = text.indexOf("{", colonIndex + 1);
  if (start < 0) return null;

  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let index = start; index < text.length; index++) {
    const ch = text[index];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === '"') {
        inString = false;
      }
      continue;
    }
    if (ch === '"') {
      inString = true;
    } else if (ch === "{") {
      depth++;
    } else if (ch === "}") {
      depth--;
      if (depth === 0) return text.slice(start, index + 1);
    }
  }
  return null;
}

function readSymbolGraphArtifactSummary(outPath) {
  const prefix = readFilePrefix(outPath, SYMBOL_GRAPH_SUMMARY_PREFIX_BYTES);
  const summaryText = extractJsonObjectAfterKey(prefix, "artifactSummary");
  if (summaryText) {
    return JSON.parse(summaryText);
  }
  phaseTimer.setCounter("symbolGraphArtifactSummaryFullParseFallback", 1);
  const writtenSymbols = JSON.parse(readFileSync(outPath, "utf8"));
  return {
    totalUsesResolved: writtenSymbols.totalUsesResolved,
    unresolvedUses: writtenSymbols.unresolvedUses,
    uses: writtenSymbols.uses,
    resolvedInternalEdgeCount: Array.isArray(writtenSymbols.resolvedInternalEdges)
      ? writtenSymbols.resolvedInternalEdges.length
      : undefined,
    deadTotal: writtenSymbols.deadTotal,
    trulyDead: writtenSymbols.trulyDead,
    deadInProd: writtenSymbols.deadInProd,
    deadInTest: writtenSymbols.deadInTest,
  };
}

const repoMode = detectRepoMode(ROOT);
const aliasMap = buildAliasMap(ROOT, repoMode, { exclude: cli.exclude });
let _resolveRaw = null;
// Extension-aware resolver: Python files use the Python module resolver;
// anything else falls through to the TS/JS alias-aware resolver. EXTERNAL
// (stdlib / npm) is collapsed to `null` for consistent downstream handling.
function resolveSpecifier(from, use) {
  // `use` is the richer import record; callers that only have spec string can
  // pass { fromSpec: spec } for legacy behavior.
  const spec = typeof use === "string" ? use : use.fromSpec;
  if (from.endsWith(".py")) {
    const isFromImport = typeof use === "object" ? !!use.pyIsFromImport : false;
    const level = typeof use === "object" ? (use.pyLevel ?? 0) : 0;
    const names =
      typeof use === "object" && use.name && use.name !== "*" ? [use.name] : [];
    const hits = resolvePythonImport(
      ROOT,
      from,
      spec,
      isFromImport,
      names,
      level,
    );
    return hits[0] ?? null;
  }
  if (from.endsWith(".go")) {
    const hits = resolveGoImport(ROOT, goModule, spec);
    return hits[0] ?? null;
  }
  if (!_resolveRaw) {
    throw new Error("symbol resolver used before repo snapshot initialization");
  }
  if (isRustResolvedRelativeUse(use)) return use.resolvedFile;
  const r = _resolveRaw(from, spec);
  // v1.9.7: preserve resolver sentinels so the caller can distinguish
  // external packages (react, oxc-parser) from failed local aliases
  // (@/components/X that matched a tsconfig path but the file wasn't
  // found). Both used to collapse to null here, inflating
  // unresolvedUses with legitimate external imports and triggering
  // false resolver-blindness alerts.
  return r;
}

if (verbose) console.error(`[symbols] root: ${ROOT}, mode: ${repoMode.mode}`);

// Per-language extractors live in `_lib/extract-{ts,py,go}.mjs`
// since v1.10.1. Each returns the canonical
// {filePath, defs, uses, reExports, loc, [pyDunderAll]} shape — the
// main scan loop below doesn't switch on language after this point.

// ─── 전체 스캔 (incremental-aware, multi-language) ───────
const MDX_FAMILY_LANGS = ["mdx"];
const langList = [...JS_FAMILY_LANGS, ...SFC_FAMILY_LANGS, ...MDX_FAMILY_LANGS];
if (pyEnabled) langList.push("py");
if (tsEnabled) langList.push("go");

const PRODUCER_ID = "symbols";
const PRODUCER_VERSION = 1;
const FACT_SCHEMA_VERSION = 5;
const PARSER_IDENTITY = "symbol-graph-extractors:v5-rust-js-relative-resolve";
const incrementalEnabled = cli.raw?.["no-incremental"] !== true;

function isJsFamilyFile(filePath) {
  return JS_FAMILY_LANGS.includes(
    path.extname(filePath).slice(1).toLowerCase(),
  );
}

function isSfcFamilyFile(filePath) {
  return SFC_FAMILY_LANGS.includes(
    path.extname(filePath).slice(1).toLowerCase(),
  );
}

function isMdxFamilyFile(filePath) {
  return MDX_FAMILY_LANGS.includes(
    path.extname(filePath).slice(1).toLowerCase(),
  );
}

function countNestedMapEntries(map) {
  let count = 0;
  for (const inner of map.values()) count += inner?.size ?? 0;
  return count;
}

function buildSourceSetFingerprint(root, sourceFiles) {
  const normalized = [...sourceFiles].map((file) => relPath(root, file)).sort();
  const hash = createHash("sha256");
  for (const file of normalized) {
    hash.update(file, "utf8");
    hash.update("\n", "utf8");
  }
  return `sha256:${hash.digest("hex")}`;
}

const contextFingerprint = buildContextFingerprint({
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  languages: langList,
  producerContext: {
    producer: PRODUCER_ID,
    producerVersion: PRODUCER_VERSION,
    factSchemaVersion: FACT_SCHEMA_VERSION,
    parserIdentity: PARSER_IDENTITY,
    repoMode: repoMode.mode,
    pythonEnabled: pyEnabled,
    treeSitterEnabled: tsEnabled,
  },
});
const snapshot = phaseTimer.runPhase("snapshot", () =>
  buildRepoSnapshot({
    root: ROOT,
    includeTests: cli.includeTests,
    exclude: cli.exclude,
    languages: langList,
    contextFingerprint,
    hashContents: incrementalEnabled,
  }),
);
const snapshotEntries = Object.values(snapshot.files);
const files = snapshotEntries.map((entry) => entry.absPath);
const scannedJsSourceFiles = new Set(files.filter(isJsFamilyFile));
const jsSourceSetFingerprint = buildSourceSetFingerprint(ROOT, scannedJsSourceFiles);
_resolveRaw = makeResolver(ROOT, aliasMap, { sourceFiles: scannedJsSourceFiles });
const mdxSourceFiles = files.filter(isMdxFamilyFile);
const sfcSourceFiles = files.filter(isSfcFamilyFile);
const jsTotal = files.filter(isJsFamilyFile).length;
const mdxTotal = mdxSourceFiles.length;
const sfcTotal = sfcSourceFiles.length;
const pyTotal = files.filter((f) => f.endsWith(".py")).length;
const goTotal = files.filter((f) => f.endsWith(".go")).length;
phaseTimer.setCounter("snapshotFiles", files.length);
phaseTimer.setCounter(
  "snapshotReadableFiles",
  snapshotEntries.filter((entry) => entry.readable).length,
);
phaseTimer.setCounter(
  "snapshotUnreadableFiles",
  snapshotEntries.filter((entry) => !entry.readable).length,
);
phaseTimer.setCounter("snapshotJsFiles", jsTotal);
phaseTimer.setCounter("snapshotMdxFiles", mdxTotal);
phaseTimer.setCounter("snapshotSfcFiles", sfcTotal);
phaseTimer.setCounter("snapshotPythonFiles", pyTotal);
phaseTimer.setCounter("snapshotGoFiles", goTotal);
console.error(
  `[symbols] scanning ${files.length} files (mdx=${mdxTotal}, sfc=${sfcTotal}, python=${pyEnabled ? `on, ${pyTotal} .py` : "off"}, go=${tsEnabled ? `on, ${goTotal} .go` : "off"})`,
);

const cacheStore = openIncrementalCacheStore({
  root: ROOT,
  cacheRoot: cli.raw?.["cache-root"],
});
if (cli.raw?.["clear-incremental-cache"] === true) {
  clearIncrementalCache(cacheStore);
}

const producerCacheMeta = {
  producerId: PRODUCER_ID,
  producerVersion: PRODUCER_VERSION,
  factSchemaVersion: FACT_SCHEMA_VERSION,
  parserIdentity: PARSER_IDENTITY,
  scanFingerprint: contextFingerprint,
  configFingerprint: contextFingerprint,
};

function producerCacheMetaForEntry(entry) {
  if (entry && isJsFamilyFile(entry.absPath)) {
    return {
      ...producerCacheMeta,
      sourceSetFingerprint: jsSourceSetFingerprint,
    };
  }
  return producerCacheMeta;
}

const priorCache = incrementalEnabled
  ? loadProducerCache(cacheStore, PRODUCER_ID)
  : { entries: {}, meta: { loadStatus: "disabled" } };
const nextProducerCache = { entries: {}, meta: { loadStatus: "new" } };
const nextCache = { version: 1, entries: {} };
const currentStrictKeys = new Set();
const changed = [];
let changedFiles = 0;
let reusedFiles = 0;
let invalidatedFiles = 0;

const cacheClassificationStarted = Date.now();
for (const entry of snapshotEntries) {
  currentStrictKeys.add(strictCacheKeyForEntry(entry));

  if (!entry.readable) {
    changedFiles++;
    nextCache.entries[entry.absPath] = { parseError: true };
    continue;
  }

  const reuse = incrementalEnabled
    ? getReusableFact(priorCache, {
        snapshotEntry: entry,
        producerMeta: producerCacheMetaForEntry(entry),
      })
    : { status: "miss", reason: "disabled-by-flag" };

  if (reuse.status === "hit") {
    reusedFiles++;
    nextCache.entries[entry.absPath] = reuse.payload;
    putFact(nextProducerCache, {
      snapshotEntry: entry,
      producerMeta: producerCacheMetaForEntry(entry),
      payload: reuse.payload,
    });
    continue;
  }

  if (reuse.reason !== "missing-entry" && reuse.reason !== "disabled-by-flag") {
    invalidatedFiles++;
  }
  changedFiles++;
  changed.push(entry.absPath);
}
phaseTimer.recordPhase(
  "cache-classification",
  Date.now() - cacheClassificationStarted,
);

const droppedFiles = Object.keys(priorCache.entries ?? {}).filter(
  (key) => !currentStrictKeys.has(key),
).length;
phaseTimer.setCounter("changedFiles", changedFiles);
phaseTimer.setCounter("reusedFiles", reusedFiles);
phaseTimer.setCounter("droppedFiles", droppedFiles);
phaseTimer.setCounter("invalidatedFiles", invalidatedFiles);

if (incrementalEnabled) {
  console.error(
    `[symbols-incremental] changed=${changedFiles} reused=${reusedFiles} dropped=${droppedFiles} invalidated=${invalidatedFiles}`,
  );
}

// Pre-batch Python files among the changed set.
const extractChangedFilesStarted = Date.now();
const changedPy = changed.filter((f) => f.endsWith(".py"));
const changedJs = changed.filter(isJsFamilyFile);
const changedMdx = changed.filter(isMdxFamilyFile);
const changedSfc = changed.filter(isSfcFamilyFile);
// v1.8.2: collect non-fatal failure records for explicit inclusion in
// the artifact. Previously these went to stderr (or got silently
// swallowed at a deeper level). The `warnings[]` field in
// `symbols.json.meta` lets CI consumers, SARIF emission, and downstream
// tools like `triage-repo` see what couldn't be processed — and decide
// how to react.
const warnings = [];
const extractPhaseMs = {
  pythonBatch: 0,
  goBatch: 0,
  rustJsBatch: 0,
  jsFiles: 0,
  mdxFiles: 0,
  sfcFiles: 0,
  pythonShapes: 0,
  goShapes: 0,
};
function timeExtractPhase(bucket, action) {
  const started = Date.now();
  try {
    return action();
  } finally {
    extractPhaseMs[bucket] += Date.now() - started;
  }
}

let pyBatch = new Map();
if (changedPy.length > 0 && pyEnabled) {
  const pythonBatchStarted = Date.now();
  try {
    pyBatch = extractPythonBatch(changedPy) ?? new Map();
    // Python extractor surfaces stream-parse failures via a __meta__ key.
    const pyMeta = pyBatch.get("__meta__");
    if (pyMeta?.parseFailures > 0) {
      warnings.push({
        code: "python-ndjson-parse-failure",
        count: pyMeta.parseFailures,
        message: `${pyMeta.parseFailures} stray non-JSON lines in extractor stdout`,
      });
    }
    pyBatch.delete("__meta__");
  } catch (e) {
    console.error(`[symbols] python batch failed: ${e.message}`);
    warnings.push({
      code: "python-batch-crashed",
      message: e.message,
      affected: changedPy.length,
    });
  } finally {
    extractPhaseMs.pythonBatch += Date.now() - pythonBatchStarted;
  }
}

// Pre-batch Go files (and any other tree-sitter languages).
const changedTs = changed.filter((f) => f.endsWith(".go"));
phaseTimer.setCounter("changedJsFiles", changedJs.length);
phaseTimer.setCounter("changedMdxFiles", changedMdx.length);
phaseTimer.setCounter("changedSfcFiles", changedSfc.length);
phaseTimer.setCounter("changedPythonFiles", changedPy.length);
phaseTimer.setCounter("changedGoFiles", changedTs.length);
let tsBatch = new Map();
if (changedTs.length > 0 && tsEnabled) {
  const goBatchStarted = Date.now();
  try {
    tsBatch = (await extractTreeSitterBatch(changedTs)) ?? new Map();
  } catch (e) {
    console.error(`[symbols] tree-sitter batch failed: ${e.message}`);
    warnings.push({
      code: "tree-sitter-batch-crashed",
      message: e.message,
      affected: changedTs.length,
    });
  } finally {
    extractPhaseMs.goBatch += Date.now() - goBatchStarted;
  }
}

let rustJsHybrid = {
  results: new Map(),
  summary: {
    candidateFiles: changedJs.length,
    eligibleFiles: 0,
    fallbackFiles: changedJs.length,
    rustExtractedFiles: 0,
    rustResolvedRelativeUses: 0,
    rustParseErrorFiles: 0,
    readErrorFiles: 0,
    commandFailedFiles: 0,
    batchCount: 0,
    inputBytes: 0,
    sourceBytes: 0,
    fallbackByReason: {},
  },
  warnings: [],
};
if (changedJs.length > 0) {
  const rustJsBatchStarted = Date.now();
  try {
    rustJsHybrid = extractRustJsHybridBatch({
      root: ROOT,
      files: changedJs,
      sourceFiles: scannedJsSourceFiles,
      verbose,
    });
    warnings.push(...rustJsHybrid.warnings);
  } finally {
    extractPhaseMs.rustJsBatch += Date.now() - rustJsBatchStarted;
  }
}

let parseErrors = 0;
let extractedFiles = 0;
let extractedJsFiles = 0;
let extractedMdxFiles = 0;
let extractedSfcFiles = 0;
let extractedPythonFiles = 0;
let extractedGoFiles = 0;
for (const f of changed) {
  const entry = snapshot.files[relPath(ROOT, f)];
  try {
    let payload;
    if (f.endsWith(".py")) {
      const pyRec = pyBatch.get(f);
      if (!pyRec || pyRec.error) {
        parseErrors++;
        if (pyRec?.error && verbose)
          console.error(`py fail: ${f}: ${pyRec.error}`);
        nextCache.entries[f] = { parseError: true };
        if (incrementalEnabled && entry) {
          putFact(nextProducerCache, {
            snapshotEntry: entry,
            producerMeta: producerCacheMetaForEntry(entry),
            payload: nextCache.entries[f],
          });
        }
        continue;
      }
      payload = timeExtractPhase("pythonShapes", () =>
        pythonExtractShape(f, pyRec),
      );
    } else if (f.endsWith(".go")) {
      const goRec = tsBatch.get(f);
      if (!goRec || goRec.error) {
        parseErrors++;
        if (goRec?.error && verbose)
          console.error(`go fail: ${f}: ${goRec.error}`);
        nextCache.entries[f] = { parseError: true };
        if (incrementalEnabled && entry) {
          putFact(nextProducerCache, {
            snapshotEntry: entry,
            producerMeta: producerCacheMetaForEntry(entry),
            payload: nextCache.entries[f],
          });
        }
        continue;
      }
      payload = timeExtractPhase("goShapes", () => goExtractShape(f, goRec));
    } else if (isMdxFamilyFile(f)) {
      payload = timeExtractPhase("mdxFiles", () => ({
        defs: [],
        uses: [],
        reExports: [],
        loc: 0,
      }));
    } else if (isSfcFamilyFile(f)) {
      payload = timeExtractPhase("sfcFiles", () => ({
        defs: [],
        uses: [],
        reExports: [],
        loc: 0,
      }));
    } else {
      const rustResult = rustJsHybrid.results.get(f);
      if (rustResult?.error) {
        parseErrors++;
        if (verbose)
          console.error(`js rust parse fail: ${f}: ${rustResult.error}`);
        nextCache.entries[f] = { parseError: true };
        if (incrementalEnabled && entry) {
          putFact(nextProducerCache, {
            snapshotEntry: entry,
            producerMeta: producerCacheMetaForEntry(entry),
            payload: nextCache.entries[f],
          });
        }
        continue;
      }
      payload =
        rustResult ??
        timeExtractPhase("jsFiles", () =>
          extractDefinitionsAndUses(f, {
            artifactFilePath: relPath(ROOT, f),
          }),
        );
    }
    nextCache.entries[f] = { ...payload, parseError: false };
    extractedFiles++;
    if (f.endsWith(".py")) {
      extractedPythonFiles++;
    } else if (f.endsWith(".go")) {
      extractedGoFiles++;
    } else if (isJsFamilyFile(f)) {
      extractedJsFiles++;
    } else if (isMdxFamilyFile(f)) {
      extractedMdxFiles++;
    } else if (isSfcFamilyFile(f)) {
      extractedSfcFiles++;
    }
    if (incrementalEnabled && entry) {
      putFact(nextProducerCache, {
        snapshotEntry: entry,
        producerMeta: producerCacheMetaForEntry(entry),
        payload: nextCache.entries[f],
      });
    }
  } catch (e) {
    parseErrors++;
    console.error(`parse fail: ${f}: ${e.message}`);
    nextCache.entries[f] = { parseError: true };
    if (incrementalEnabled && entry) {
      putFact(nextProducerCache, {
        snapshotEntry: entry,
        producerMeta: producerCacheMetaForEntry(entry),
        payload: nextCache.entries[f],
      });
    }
  }
}
phaseTimer.recordPhase("extract-python-batch", extractPhaseMs.pythonBatch);
phaseTimer.recordPhase("extract-go-batch", extractPhaseMs.goBatch);
phaseTimer.recordPhase("extract-rust-js-batch", extractPhaseMs.rustJsBatch);
phaseTimer.recordPhase("extract-js-files", extractPhaseMs.jsFiles);
phaseTimer.recordPhase("extract-mdx-files", extractPhaseMs.mdxFiles);
phaseTimer.recordPhase("extract-sfc-files", extractPhaseMs.sfcFiles);
phaseTimer.recordPhase("extract-python-shapes", extractPhaseMs.pythonShapes);
phaseTimer.recordPhase("extract-go-shapes", extractPhaseMs.goShapes);
phaseTimer.recordPhase(
  "extract-changed-files",
  Date.now() - extractChangedFilesStarted,
);
// Cached parse errors still count in aggregate.
for (const [f, entry] of Object.entries(nextCache.entries)) {
  if (!changed.includes(f) && entry?.parseError) parseErrors++;
}
phaseTimer.setCounter("extractedFiles", extractedFiles);
phaseTimer.setCounter("extractedJsFiles", extractedJsFiles);
phaseTimer.setCounter("extractedMdxFiles", extractedMdxFiles);
phaseTimer.setCounter("extractedSfcFiles", extractedSfcFiles);
phaseTimer.setCounter("extractedPythonFiles", extractedPythonFiles);
phaseTimer.setCounter("extractedGoFiles", extractedGoFiles);
phaseTimer.setCounter("parseErrorCount", parseErrors);
phaseTimer.setCounter(
  "rustJsExtractorCandidateFiles",
  rustJsHybrid.summary.candidateFiles,
);
phaseTimer.setCounter(
  "rustJsExtractorEligibleFiles",
  rustJsHybrid.summary.eligibleFiles,
);
phaseTimer.setCounter(
  "rustJsExtractorFallbackFiles",
  rustJsHybrid.summary.fallbackFiles,
);
phaseTimer.setCounter(
  "rustJsExtractorExtractedFiles",
  rustJsHybrid.summary.rustExtractedFiles,
);
phaseTimer.setCounter(
  "rustJsExtractorResolvedRelativeUses",
  rustJsHybrid.summary.rustResolvedRelativeUses ?? 0,
);
phaseTimer.setCounter(
  "rustJsExtractorParseErrorFiles",
  rustJsHybrid.summary.rustParseErrorFiles,
);
phaseTimer.setCounter(
  "rustJsExtractorReadErrorFiles",
  rustJsHybrid.summary.readErrorFiles,
);
phaseTimer.setCounter(
  "rustJsExtractorCommandFailedFiles",
  rustJsHybrid.summary.commandFailedFiles,
);
phaseTimer.setCounter("rustJsExtractorBatchCount", rustJsHybrid.summary.batchCount);
phaseTimer.setCounter("rustJsExtractorInputBytes", rustJsHybrid.summary.inputBytes);
phaseTimer.setCounter("rustJsExtractorSourceBytes", rustJsHybrid.summary.sourceBytes ?? 0);
for (const [reason, count] of Object.entries(
  rustJsHybrid.summary.fallbackByReason ?? {},
)) {
  const suffix = reason
    .split("-")
    .filter(Boolean)
    .map((part) => `${part[0]?.toUpperCase() ?? ""}${part.slice(1)}`)
    .join("");
  phaseTimer.setCounter(`rustJsExtractorFallback${suffix}Files`, count);
}

const assembleSymbolGraphStarted = Date.now();
const assembleFileDataStarted = Date.now();
const fileData = new Map();
let definitionCount = 0;
let useCount = 0;
let reExportCount = 0;
let typeEscapeCount = 0;
let dynamicImportOpacityCount = 0;
let cjsRequireOpacityCount = 0;
let classMethodCount = 0;
let localOperationCount = 0;
for (const [f, entry] of Object.entries(nextCache.entries)) {
  if (entry.parseError || entry.defs === undefined) continue;
  definitionCount += (entry.defs ?? []).length;
  useCount += (entry.uses ?? []).length;
  reExportCount += (entry.reExports ?? []).length;
  typeEscapeCount += (entry.typeEscapes ?? []).length;
  dynamicImportOpacityCount += (entry.dynamicImportOpacity ?? []).length;
  cjsRequireOpacityCount += (entry.cjsRequireOpacity ?? []).length;
  classMethodCount += (entry.classMethods ?? []).length;
  localOperationCount += (entry.localOperations ?? []).length;
  fileData.set(f, {
    filePath: f,
    defs: entry.defs ?? [],
    uses: entry.uses ?? [],
    reExports: entry.reExports ?? [],
    classMethods: entry.classMethods ?? [],
    localOperations: entry.localOperations ?? [],
    typeEscapes: entry.typeEscapes ?? [],
    loc: entry.loc ?? 0,
    dynamicImportOpacity: entry.dynamicImportOpacity ?? [],
    cjsExportSurface: entry.cjsExportSurface ?? null,
    cjsRequireOpacity: entry.cjsRequireOpacity ?? [],
    // v1.7.2: Python-specific `__all__` declaration. Present only for .py
    // files where the module declared `__all__ = [...]`. When present,
    // only the listed names are considered exported; other top-level
    // names are module-private and excluded from the dead-list.
    ...(entry.pyDunderAll !== undefined
      ? { pyDunderAll: entry.pyDunderAll }
      : {}),
  });
}

if (incrementalEnabled)
  saveProducerCache(cacheStore, PRODUCER_ID, nextProducerCache);
phaseTimer.setCounter("fileDataFiles", fileData.size);
phaseTimer.setCounter("definitionCount", definitionCount);
phaseTimer.setCounter("useCount", useCount);
phaseTimer.setCounter("reExportCount", reExportCount);
phaseTimer.setCounter("typeEscapeCount", typeEscapeCount);
phaseTimer.setCounter("dynamicImportOpacityCount", dynamicImportOpacityCount);
phaseTimer.setCounter("cjsRequireOpacityCount", cjsRequireOpacityCount);
phaseTimer.setCounter("classMethodCount", classMethodCount);
phaseTimer.setCounter("localOperationCount", localOperationCount);
phaseTimer.recordPhase(
  "assemble-file-data",
  Date.now() - assembleFileDataStarted,
);
console.log(`[parse] errors: ${parseErrors}`);

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

const consumerEntries = [];
const namespaceUserEntries = [];
const consumerSymbolKeys = new Set();
function addConsumer(defFile, name, consumerFile, use = null) {
  if (typeof name !== "string" || name.length === 0) return;
  const space =
    use && typeof use === "object" && use.typeOnly === true ? "type" : "value";
  consumerEntries.push({
    defFile,
    symbol: name,
    consumerFile,
    space,
  });
  consumerSymbolKeys.add(JSON.stringify([defFile, name]));
}

// namespace import의 정확한 사용을 모르므로 "전체 파일 사용" 으로 기록
function addNamespaceUser(defFile, consumerFile) {
  namespaceUserEntries.push({ defFile, consumerFile });
}

function buildFanInInputs() {
  const identityKeys = new Set();

  for (const [defFile, defs] of defIndex) {
    const relFile = relPath(ROOT, defFile);
    for (const symbol of defs.keys()) {
      identityKeys.add(`${relFile}::${symbol}`);
    }
  }
  for (const key of consumerSymbolKeys) {
    const [defFile, symbol] = JSON.parse(key);
    identityKeys.add(`${relPath(ROOT, defFile)}::${symbol}`);
  }

  return {
    consumerEntries: [...consumerEntries],
    namespaceUserEntries: [...namespaceUserEntries],
    consumerSymbolCount: consumerSymbolKeys.size,
    identityCount: identityKeys.size,
  };
}

function buildDeadCandidateInputs() {
  const barrelFiles = [...detectBarrelFiles(ROOT, repoMode)];
  const testLikeFiles = files
    .map((file) => relPath(ROOT, file))
    .filter((file) => isTestLikePath(file));
  return { barrelFiles, testLikeFiles };
}

let totalUses = 0;
let unresolvedUses = 0;
// v1.9.7 FP-36 counters: external packages vs genuine scanner
// blind spots. Feeds into fix-plan's resolverBlindness gate.
let resolvedInternalUses = 0;
let resolvedGeneratedVirtualUses = 0;
let rustResolvedRelativeUses = 0;
let nonSourceAssetUses = 0;
let externalUses = 0;
let unresolvedInternalUses = 0;
let mdxConsumerUses = 0;
let sfcScriptConsumerUses = 0;
let sfcScriptSrcReachabilityUses = 0;
let sfcStyleAssetReferenceUses = 0;
let sfcTemplateComponentRefUses = 0;
let sfcGlobalComponentRegistrationUses = 0;
let sfcGeneratedComponentManifestUses = 0;
let sfcFrameworkConventionComponentUses = 0;
const dependencyImportConsumers = [];
// Spec-frequency counter for topUnresolvedSpecifiers artifact.
// Keyed by "prefix" (everything up to first /) so "@/foo/a" and
// "@/foo/b" roll up to "@/" — gives users actionable feedback
// ("add a tsconfig path for `@/`").
const unresolvedInternalByPrefix = new Map();
const prefixExamples = new Map();
// v1.10.0 P1: full set of unique unresolved specifiers for per-finding
// taint matching in classify-dead-exports. Lets the classifier ask "does
// any unresolved import look like it could resolve to THIS dead symbol's
// file?" rather than relying on the repo-wide unresolvedInternalRatio.
const unresolvedInternalSpecifiers = new Set();
const unresolvedInternalSpecifierRecords = [];
const generatedConsumerBlindZoneUnresolvedRecords = [];
const resolvedInternalEdges = [];
const sfcStyleAssetReferenceInputs = [];
const sfcTemplateComponentRefInputs = [];
const sfcGlobalComponentRegistrationInputs = [];
const sfcGeneratedComponentManifestInputs = [];
const sfcFrameworkConventionComponentInputs = [];
const generatedVirtualSurfaces = new Map();
const generatedVirtualImportConsumers = [];
function prefixOf(spec) {
  const slash = spec.indexOf("/");
  return slash > 0 ? spec.slice(0, slash + 1) : spec;
}

function edgeKindForUse(use) {
  const kind = typeof use === "object" ? use.kind : "import";
  if (kind === "import") return "import-named";
  if (kind === "default") return "import-default";
  if (kind === "namespace" || kind === "namespace-member")
    return "import-namespace";
  if (kind === "import-side-effect") return "import-side-effect";
  if (kind === "reExport") return "reexport-named";
  if (kind === "reExportAll") return "reexport-broad";
  if (kind === "reExportNamespace") return "reexport-namespace";
  if (kind === "imported-namespace-member") return "reexport-namespace-member";
  if (kind === "imported-namespace-escape") return "reexport-namespace-escape";
  if (kind === "dynamic" || kind === "dynamic-member") return "dynamic-literal";
  if (kind === "cjs-side-effect-only") return "cjs-side-effect";
  if (kind === "cjs-require-exact") return "cjs-require-exact";
  if (kind === "cjs-namespace-member") return "cjs-namespace-member";
  if (kind === "cjs-namespace-escape") return "cjs-namespace-escape";
  if (kind === "cjs-reexport-broad") return "cjs-reexport-broad";
  return kind;
}

function isImportedNamespaceAliasUse(use) {
  return (
    use?.kind === "imported-namespace-member" ||
    use?.kind === "imported-namespace-escape"
  );
}

function isRustResolvedRelativeUse(use) {
  return (
    typeof use === "object" &&
    use?.resolverStage === "relative" &&
    typeof use.resolvedFile === "string" &&
    use.resolvedFile.length > 0
  );
}

function addResolvedInternalEdge(consumerFile, target, use) {
  const fromSpec = typeof use === "string" ? use : use.fromSpec;
  resolvedInternalEdges.push({
    from: relPath(ROOT, consumerFile),
    to: relPath(ROOT, target),
    kind: edgeKindForUse(use),
    source: fromSpec,
    typeOnly: typeof use === "object" ? !!use.typeOnly : false,
    ...(typeof use === "object" && Number.isFinite(use.line)
      ? { line: use.line }
      : {}),
    ...(typeof use === "object" && use.sfcLanguage
      ? { sfcLanguage: use.sfcLanguage }
      : {}),
  });
}

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

function extensionlessRelativeRawTargetExists(consumerFile, spec) {
  if (typeof spec !== "string") return false;
  if (!spec.startsWith("./") && !spec.startsWith("../")) return false;
  const stripped = stripSpecifierResourceQuery(spec);
  const fileName = stripped.split("/").at(-1) ?? stripped;
  if (fileName.includes(".")) return false;
  return fileExists(path.resolve(path.dirname(consumerFile), stripped));
}

function targetFileLang(filePath) {
  return path.extname(filePath).slice(1).toLowerCase();
}

function isJsFamilyTarget(filePath) {
  return (
    JS_FAMILY_LANGS.includes(targetFileLang(filePath)) ||
    /\.d\.(ts|mts|cts)$/i.test(filePath)
  );
}

function isSfcFamilyTarget(filePath) {
  return SFC_FAMILY_LANGS.includes(targetFileLang(filePath));
}

function unresolvedInternalSpecifierRecord(consumerFile, use) {
  const spec = typeof use === "string" ? use : use.fromSpec;
  if (typeof spec !== "string" || spec.length === 0) return null;
  const unresolvedEvidence = unresolvedInternalEvidence(consumerFile, use);
  return {
    specifier: spec,
    consumerFile: relPath(ROOT, consumerFile),
    fromHint: relPath(ROOT, consumerFile),
    kind: typeof use === "object" ? (use.kind ?? "import") : "import",
    ...(typeof use === "object" && typeof use.typeOnly === "boolean"
      ? { typeOnly: use.typeOnly }
      : {}),
    ...unresolvedEvidence,
  };
}

function recordUnresolvedInternalSpecifier(consumerFile, use) {
  const record = unresolvedInternalSpecifierRecord(consumerFile, use);
  if (!record) return;
  unresolvedInternalSpecifiers.add(record.specifier);
  unresolvedInternalSpecifierRecords.push(record);
}

function recordGeneratedConsumerBlindZoneCandidate(consumerFile, use) {
  const record = unresolvedInternalSpecifierRecord(consumerFile, use);
  if (record) generatedConsumerBlindZoneUnresolvedRecords.push(record);
}

function unresolvedInternalEvidence(consumerFile, use) {
  const spec = typeof use === "string" ? use : use.fromSpec;
  const explanation =
    explainUnresolvedSpecifier(ROOT, aliasMap, consumerFile, spec) ?? {};
  const diagnostic =
    typeof use === "object"
      ? {
          ...(use.reason ? { reason: use.reason } : {}),
          ...(use.resolverStage ? { resolverStage: use.resolverStage } : {}),
          ...(use.outputLevel ? { outputLevel: use.outputLevel } : {}),
          ...(use.unsupportedFamily
            ? { unsupportedFamily: use.unsupportedFamily }
            : {}),
          ...(use.hint ? { hint: use.hint } : {}),
          ...(Array.isArray(use.targetCandidates)
            ? { targetCandidates: use.targetCandidates }
            : {}),
          ...(use.affectedPackageScope
            ? { affectedPackageScope: use.affectedPackageScope }
            : {}),
          ...(typeof use.matchCount === "number"
            ? { matchCount: use.matchCount }
            : {}),
          ...(typeof use.cap === "number" ? { cap: use.cap } : {}),
          ...(use.scanPolicy ? { scanPolicy: use.scanPolicy } : {}),
          ...(use.affectedDir
            ? {
                affectedPackageScope: relPath(
                  ROOT,
                  path.resolve(path.dirname(consumerFile), use.affectedDir),
                ),
              }
            : {}),
        }
      : {};
  return {
    ...explanation,
    ...diagnostic,
  };
}

const namespaceReExportsByFile = new Map();
const namedReExportsByFile = new Map();
const namespaceReExportDiagnostics = [];

function addNamespaceReExport(
  barrelFile,
  exportedName,
  targetFile,
  sourceSpec,
) {
  if (!namespaceReExportsByFile.has(barrelFile)) {
    namespaceReExportsByFile.set(barrelFile, new Map());
  }
  namespaceReExportsByFile.get(barrelFile).set(exportedName, {
    targetFile,
    sourceSpec,
  });
}

function getNamespaceReExport(barrelFile, exportedName) {
  return namespaceReExportsByFile.get(barrelFile)?.get(exportedName) ?? null;
}

function addNamedReExport(barrelFile, exportedName, targetFile, sourceSpec) {
  if (!namedReExportsByFile.has(barrelFile)) {
    namedReExportsByFile.set(barrelFile, new Map());
  }
  namedReExportsByFile.get(barrelFile).set(exportedName, {
    targetFile,
    sourceSpec,
  });
}

function getNamedReExport(barrelFile, exportedName) {
  return namedReExportsByFile.get(barrelFile)?.get(exportedName) ?? null;
}

function namespaceReExportChainEntry(
  kind,
  barrelFile,
  exportedName,
  targetFile,
  sourceSpec,
) {
  return {
    kind,
    file: relPath(ROOT, barrelFile),
    exportedName,
    targetFile: relPath(ROOT, targetFile),
    source: sourceSpec,
  };
}

function resolveNamespaceReExport(barrelFile, exportedName, seen = new Set()) {
  const key = `${barrelFile}::${exportedName}`;
  if (seen.has(key)) return null;
  seen.add(key);

  const direct = getNamespaceReExport(barrelFile, exportedName);
  if (direct) {
    return {
      targetFile: direct.targetFile,
      sourceSpec: direct.sourceSpec,
      chain: [
        namespaceReExportChainEntry(
          "namespace-reexport",
          barrelFile,
          exportedName,
          direct.targetFile,
          direct.sourceSpec,
        ),
      ],
    };
  }

  const named = getNamedReExport(barrelFile, exportedName);
  if (!named) return null;
  const nested = resolveNamespaceReExport(named.targetFile, exportedName, seen);
  if (!nested) return null;
  return {
    targetFile: nested.targetFile,
    sourceSpec: nested.sourceSpec,
    chain: [
      namespaceReExportChainEntry(
        "named-reexport",
        barrelFile,
        exportedName,
        named.targetFile,
        named.sourceSpec,
      ),
      ...(nested.chain ?? []),
    ],
  };
}

function addNamespaceReExportDiagnostic(
  consumerFile,
  importFile,
  use,
  reExport,
) {
  namespaceReExportDiagnostics.push({
    kind: "opaque-namespace-escape",
    reason: "namespace-object-escaped",
    consumerFile: relPath(ROOT, consumerFile),
    importFile: relPath(ROOT, importFile),
    exportedName: use.name,
    targetFile: relPath(ROOT, reExport.targetFile),
    source: use.fromSpec,
    ...(typeof use.line === "number" ? { line: use.line } : {}),
    ...(Array.isArray(reExport.chain) && reExport.chain.length > 0
      ? { chain: reExport.chain }
      : {}),
  });
}

function namespaceReExportSourceUseRecordId(barrelFile, useIndex, use) {
  return outOfBandSourceUseRecordId("namespace-reexport-map", useIndex, {
    consumerFile: barrelFile,
    fromSpec: use?.fromSpec,
  });
}

function isNamespaceReExportMapSourceUseCandidate(barrelFile, use) {
  if (use?.kind !== "reExportNamespace" && use?.kind !== "reExport") {
    return false;
  }
  if (!use.name || use.name === "*" || use.typeOnly === true) return false;
  if (!isSourceUseAssemblyCandidate(use)) return false;
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  if (extensionlessRelativeRawTargetExists(barrelFile, use.fromSpec)) {
    return false;
  }
  return true;
}

function buildNamespaceReExportSourceUseAssemblyRecords() {
  const records = [];
  for (const [barrelFile, info] of fileData) {
    const uses = info.uses ?? [];
    for (let useIndex = 0; useIndex < uses.length; useIndex++) {
      const use = uses[useIndex];
      if (!isNamespaceReExportMapSourceUseCandidate(barrelFile, use)) {
        continue;
      }
      const record = sourceUseAssemblyRecord(
        namespaceReExportSourceUseRecordId(barrelFile, useIndex, use),
        barrelFile,
        {
          ...use,
          consumerSource: "namespace-reexport-map",
          resolverStage: "relative",
        },
      );
      if (record) records.push(record);
    }
  }
  return records;
}

function namespaceReExportSourceUseAssemblyTarget(
  resolution,
  barrelFile,
  useIndex,
  use,
) {
  if (!isNamespaceReExportMapSourceUseCandidate(barrelFile, use)) return null;
  const recordId = namespaceReExportSourceUseRecordId(barrelFile, useIndex, use);
  if (!resolution.handled.has(recordId)) return null;
  const resolved = resolution.resolvedByRecordId.get(recordId);
  return typeof resolved === "string" && resolved.length > 0 ? resolved : null;
}

const assembleNamespaceReExportsStarted = Date.now();
const namespaceReExportSourceUseAssemblyRecords =
  buildNamespaceReExportSourceUseAssemblyRecords();
const hasNamespaceReExportMapCandidates =
  namespaceReExportSourceUseAssemblyRecords.some(
    (record) => record?.kind === "reExportNamespace",
  );
const namespaceReExportSourceUseAssemblyResolution =
  hasNamespaceReExportMapCandidates
    ? resolveSourceUseAssemblyRecords(namespaceReExportSourceUseAssemblyRecords)
    : {
      handled: new Set(),
      resolvedByRecordId: new Map(),
      skippedCount: 0,
    };
phaseTimer.setCounter(
  "namespaceReExportSourceUseRustAssemblyCandidateCount",
  namespaceReExportSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "namespaceReExportSourceUseRustAssemblyBypassedNamedOnlyCount",
  hasNamespaceReExportMapCandidates
    ? 0
    : namespaceReExportSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "namespaceReExportSourceUseRustAssemblyResolvedCount",
  namespaceReExportSourceUseAssemblyResolution.handled.size,
);
phaseTimer.setCounter(
  "namespaceReExportSourceUseRustAssemblySkippedCount",
  namespaceReExportSourceUseAssemblyResolution.skippedCount,
);
if (hasNamespaceReExportMapCandidates) {
  for (const [barrelFile, info] of fileData) {
    const uses = info.uses ?? [];
    for (let useIndex = 0; useIndex < uses.length; useIndex++) {
      const use = uses[useIndex];
      if (use?.kind !== "reExportNamespace" && use?.kind !== "reExport") continue;
      if (!use.name || use.name === "*" || use.typeOnly === true) continue;
      const target = namespaceReExportSourceUseAssemblyTarget(
        namespaceReExportSourceUseAssemblyResolution,
        barrelFile,
        useIndex,
        use,
      ) ?? resolveSpecifier(barrelFile, use);
      if (!target || target === "EXTERNAL" || target === "UNRESOLVED_INTERNAL")
        continue;
      if (
        isGeneratedVirtualResolution(target) ||
        isNonSourceAssetResolution(target)
      )
        continue;
      if (use.kind === "reExportNamespace") {
        addNamespaceReExport(barrelFile, use.name, target, use.fromSpec);
      } else {
        addNamedReExport(barrelFile, use.name, target, use.fromSpec);
      }
    }
  }
}
phaseTimer.setCounter(
  "namespaceReExportFileCount",
  namespaceReExportsByFile.size,
);
phaseTimer.setCounter(
  "namespaceReExportEntryCount",
  countNestedMapEntries(namespaceReExportsByFile),
);
phaseTimer.setCounter("namedReExportFileCount", namedReExportsByFile.size);
phaseTimer.setCounter(
  "namedReExportEntryCount",
  countNestedMapEntries(namedReExportsByFile),
);
phaseTimer.recordPhase(
  "assemble-namespace-reexports",
  Date.now() - assembleNamespaceReExportsStarted,
);

const assembleSourceUsesStarted = Date.now();
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
const sourceUseResolverStatsBefore =
  typeof _resolveRaw.memoStats === "function" ? _resolveRaw.memoStats() : null;
const sourceUseResolverStageStatsBefore =
  typeof _resolveRaw.stageStats === "function"
    ? _resolveRaw.stageStats()
    : null;

function addSourceUseTiming(name, started) {
  sourceUseTimings[name] += performance.now() - started;
}

function incrementSourceUseBranch(name) {
  sourceUseBranchCounts[name] = (sourceUseBranchCounts[name] ?? 0) + 1;
}

function importMetaGlobDiagnosticUse(use, expansion) {
  const unresolvedEvidence = {
    reason: expansion.reason ?? use.reason ?? "import-meta-glob-unsupported",
    resolverStage: "import-meta-glob",
    outputLevel: "unsupported",
    unsupportedFamily: "dynamic-modules",
    hint: use.hint ?? "dynamic-module-surface",
    ...(typeof expansion.matchCount === "number"
      ? { matchCount: expansion.matchCount }
      : {}),
    ...(typeof expansion.cap === "number" ? { cap: expansion.cap } : {}),
    ...(expansion.scanPolicy ? { scanPolicy: expansion.scanPolicy } : {}),
    ...(expansion.affectedPackageScope
      ? { affectedPackageScope: expansion.affectedPackageScope }
      : {}),
  };
  return {
    ...use,
    ...unresolvedEvidence,
    unresolvedEvidence,
  };
}

function sourceUseRecordId(consumerFile, index) {
  return `${relPath(ROOT, consumerFile)}#${index}`;
}

function outOfBandSourceUseRecordId(source, index, use) {
  const consumerFile = relPath(ROOT, use?.consumerFile ?? "");
  const fromSpec = use?.fromSpec ?? "";
  return `${source}:${index}:${consumerFile}:${fromSpec}`;
}

function isSourceUseAssemblyCandidate(use) {
  return (
    typeof use === "object" &&
    typeof use?.fromSpec === "string" &&
    (use.fromSpec.startsWith("./") || use.fromSpec.startsWith("../"))
  );
}

function stripSpecifierResourceQuery(spec) {
  const q = spec.indexOf("?");
  const h = spec.indexOf("#");
  const candidates = [];
  if (q >= 0) candidates.push(q);
  if (h > 0) candidates.push(h);
  return candidates.length ? spec.slice(0, Math.min(...candidates)) : spec;
}

function looksLikeNonSourceAssetSpecifier(spec) {
  if (typeof spec !== "string") return false;
  const stripped = stripSpecifierResourceQuery(spec);
  const fileName = stripped.split("/").at(-1) ?? stripped;
  const dot = fileName.lastIndexOf(".");
  if (dot <= 0 || dot === fileName.length - 1) return false;
  return !/\.(?:ts|tsx|js|jsx|mjs|cjs|mts|cts|d\.ts|d\.mts|d\.cts)$/i.test(
    stripped,
  );
}

function sourceUseAssemblyRequiresSymbolName(kind) {
  return ![
    "cjs-side-effect-only",
    "import-side-effect",
    "reExportNamespace",
    "sfc-script-src",
    "namespace",
    "reExportAll",
    "dynamic",
    "import-meta-glob",
    "dynamic-import-meta-glob",
    "cjs-namespace-escape",
    "cjs-reexport-broad",
  ].includes(kind);
}

function isInlineSourceUseAssemblyCandidate(use) {
  if (
    !isSourceUseAssemblyCandidate(use) &&
    typeof use?.resolvedFile !== "string"
  ) {
    return false;
  }
  if (
    !isRustResolvedRelativeUse(use) &&
    use?.resolverStage !== "resolved-internal"
  ) {
    return false;
  }
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  const kind = use.kind ?? "import";
  if (
    sourceUseAssemblyRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  ) {
    return false;
  }
  return true;
}

function isResolvableRelativeSourceUseAssemblyCandidate(use) {
  if (!isSourceUseAssemblyCandidate(use)) return false;
  if (typeof use?.resolvedFile === "string" && use.resolvedFile.length > 0) {
    return false;
  }
  if (use?.kind === "import-meta-glob") return false;
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  const kind = use.kind ?? "import";
  if (
    sourceUseAssemblyRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  ) {
    return false;
  }
  return true;
}

function sourceUseAssemblyPath(value) {
  return typeof value === "string" && value.length > 0 ? relPath(ROOT, value) : value;
}

function sourceUseAssemblyKind(value) {
  return typeof value === "string" && value !== "import" ? value : undefined;
}

function sourceUseAssemblyTypeOnly(value) {
  return value === true ? true : undefined;
}

function sourceUseAssemblyResolverStage(stage, resolvedFile) {
  if (
    stage === "resolved-internal" &&
    typeof resolvedFile === "string" &&
    resolvedFile.length > 0
  ) {
    return undefined;
  }
  return stage;
}

function sourceUseAssemblyConsumerSource(value) {
  return value === "source-import" ? undefined : value;
}

function sourceUseAssemblyRecord(recordId, consumerFile, use) {
  if (
    !isSourceUseAssemblyCandidate(use) &&
    typeof use?.resolvedFile !== "string"
  ) {
    return null;
  }
  return {
    recordId,
    consumerFile: sourceUseAssemblyPath(consumerFile),
    resolvedFile: sourceUseAssemblyPath(use.resolvedFile),
    fromSpec: use.fromSpec,
    name: use.name,
    memberName: use.memberName,
    kind: sourceUseAssemblyKind(use.kind),
    typeOnly: sourceUseAssemblyTypeOnly(use.typeOnly),
    line: Number.isFinite(use.line) ? use.line : undefined,
    sfcLanguage: use.sfcLanguage,
    resolverStage: sourceUseAssemblyResolverStage(
      use.resolverStage,
      use.resolvedFile,
    ),
    consumerSource: sourceUseAssemblyConsumerSource(use.consumerSource),
  };
}

const embeddedSourceUseAssemblyRecords = [];

function enqueueExternalSourceUseAssemblyRecord(recordId, consumerFile, use, source) {
  const fromSpec = typeof use === "string" ? use : use?.fromSpec;
  if (typeof fromSpec !== "string" || fromSpec.length === 0) return false;
  embeddedSourceUseAssemblyRecords.push({
    recordId,
    consumerFile: sourceUseAssemblyPath(consumerFile),
    fromSpec,
    kind: sourceUseAssemblyKind(typeof use === "object" ? use.kind : undefined),
    typeOnly: sourceUseAssemblyTypeOnly(
      typeof use === "object" ? use.typeOnly : undefined,
    ),
    typeOnlyPresent: typeof use === "object" && typeof use.typeOnly === "boolean",
    resolverStage: "external",
    consumerSource: sourceUseAssemblyConsumerSource(source),
  });
  return true;
}

function enqueueUnresolvedSourceUseAssemblyRecord(
  recordId,
  consumerFile,
  use,
  resolverStage,
) {
  const fromSpec = typeof use === "string" ? use : use?.fromSpec;
  if (typeof fromSpec !== "string" || fromSpec.length === 0) return false;
  const unresolvedEvidence =
    typeof use === "object" &&
    use?.unresolvedEvidence &&
    typeof use.unresolvedEvidence === "object"
      ? use.unresolvedEvidence
      : unresolvedInternalEvidence(consumerFile, use);
  if (isGeneratedArtifactMissingRecord(unresolvedEvidence)) return false;
  embeddedSourceUseAssemblyRecords.push({
    recordId,
    consumerFile: sourceUseAssemblyPath(consumerFile),
    fromSpec,
    kind: sourceUseAssemblyKind(typeof use === "object" ? use.kind : undefined),
    typeOnly: sourceUseAssemblyTypeOnly(
      typeof use === "object" ? use.typeOnly : undefined,
    ),
    typeOnlyPresent: typeof use === "object" && typeof use.typeOnly === "boolean",
    resolverStage,
    unresolvedEvidence,
  });
  return true;
}

function enqueueGeneratedVirtualSourceUseAssemblyRecord(
  recordId,
  consumerFile,
  use,
  surface,
) {
  const fromSpec = typeof use === "string" ? use : use?.fromSpec;
  if (
    typeof fromSpec !== "string" ||
    fromSpec.length === 0 ||
    !isGeneratedVirtualResolution(surface)
  ) {
    return false;
  }
  embeddedSourceUseAssemblyRecords.push({
    recordId,
    consumerFile: sourceUseAssemblyPath(consumerFile),
    fromSpec,
    name: typeof use === "object" ? use.name : undefined,
    kind: sourceUseAssemblyKind(typeof use === "object" ? use.kind : undefined),
    typeOnly: sourceUseAssemblyTypeOnly(
      typeof use === "object" ? use.typeOnly : undefined,
    ),
    typeOnlyPresent: typeof use === "object" && typeof use.typeOnly === "boolean",
    resolverStage: "generated-virtual",
    generatedVirtualSurface: surface,
    unresolvedEvidence: unresolvedInternalEvidence(consumerFile, use),
  });
  return true;
}

function enqueueNonSourceAssetSourceUseAssemblyRecord(recordId, consumerFile, use) {
  const fromSpec = typeof use === "string" ? use : use?.fromSpec;
  if (typeof fromSpec !== "string" || fromSpec.length === 0) return false;
  embeddedSourceUseAssemblyRecords.push({
    recordId,
    resolverStage: "non-source-asset",
  });
  return true;
}

function enqueueResolvedSourceUseAssemblyRecord(recordId, consumerFile, use, target) {
  const record = sourceUseAssemblyRecord(recordId, consumerFile, {
    ...use,
    resolvedFile: target,
    resolverStage: "resolved-internal",
  });
  if (!record || !isInlineSourceUseAssemblyCandidate(record)) return false;
  embeddedSourceUseAssemblyRecords.push(record);
  return true;
}

function buildSourceUseAssemblyCandidates() {
  const records = [];
  const requiresResolution = [];
  const preHandled = new Set();
  for (const [consumerFile, info] of fileData) {
    for (let index = 0; index < info.uses.length; index++) {
      const use = info.uses[index];
      const recordId = sourceUseRecordId(consumerFile, index);
      if (isUnresolvableNamespaceReExportUse(use)) {
        preHandled.add(recordId);
        incrementSourceUseBranch("namespaceReExport");
        incrementSourceUseBranch("namespaceReExportMiss");
        continue;
      }
      if (use?.kind === "import-meta-glob") {
        const record = sourceUseAssemblyRecord(recordId, consumerFile, {
          ...use,
          resolverStage: "relative",
        });
        if (record) records.push(record);
        continue;
      }
      if (!isInlineSourceUseAssemblyCandidate(use)) {
        if (isResolvableRelativeSourceUseAssemblyCandidate(use)) {
          const record = sourceUseAssemblyRecord(recordId, consumerFile, {
            ...use,
            resolverStage: "relative",
          });
          if (record) requiresResolution.push(record);
        }
        continue;
      }
      const record = sourceUseAssemblyRecord(
        recordId,
        consumerFile,
        use,
      );
      if (record) records.push(record);
    }
  }
  return { records, requiresResolution, preHandled };
}

function isOutOfBandSourceUseAssemblyCandidate(use) {
  if (!isSourceUseAssemblyCandidate(use)) return false;
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  const kind = use.kind ?? "import";
  if (
    sourceUseAssemblyRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  ) {
    return false;
  }
  return true;
}

function buildOutOfBandSourceUseAssemblyCandidateRecords(consumers, source) {
  const records = [];
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    if (!isOutOfBandSourceUseAssemblyCandidate(use)) continue;
    const record = sourceUseAssemblyRecord(
      outOfBandSourceUseRecordId(source, index, use),
      use.consumerFile,
      {
        ...use,
        consumerSource: source,
        resolverStage: "relative",
      },
    );
    if (record) records.push(record);
  }
  return records;
}

function buildSfcComponentSourceUseAssemblyCandidateRecords(
  consumers,
  source,
  { consumerFileForUse, fromSpecForUse, kind },
) {
  const records = [];
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    const consumerFile = consumerFileForUse(use);
    const fromSpec = fromSpecForUse(use);
    if (
      typeof consumerFile !== "string" ||
      consumerFile.length === 0 ||
      typeof fromSpec !== "string" ||
      fromSpec.length === 0
    ) {
      continue;
    }
    if (extensionlessRelativeRawTargetExists(consumerFile, fromSpec)) {
      continue;
    }
    const record = sourceUseAssemblyRecord(
      outOfBandSourceUseRecordId(source, index, { consumerFile, fromSpec }),
      consumerFile,
      {
        fromSpec,
        kind,
        name: "*",
        typeOnly: false,
        consumerSource: source,
        resolverStage: "relative",
      },
    );
    if (record && isOutOfBandSourceUseAssemblyCandidate(record)) {
      records.push(record);
    }
  }
  return records;
}

function sfcComponentSourceUseAssemblyTarget(
  resolution,
  source,
  index,
  consumerFile,
  fromSpec,
) {
  if (
    typeof consumerFile !== "string" ||
    consumerFile.length === 0 ||
    typeof fromSpec !== "string" ||
    fromSpec.length === 0
  ) {
    return null;
  }
  const recordId = outOfBandSourceUseRecordId(source, index, {
    consumerFile,
    fromSpec,
  });
  if (!resolution.handled.has(recordId)) return null;
  const resolved = resolution.resolvedByRecordId.get(recordId);
  return typeof resolved === "string" && resolved.length > 0 ? resolved : null;
}

function resolveSourceUseAssemblyRecords(records) {
  if (records.length === 0) {
    return {
      handled: new Set(),
      resolvedByRecordId: new Map(),
      unresolvedInternalSpecifierRecords: [],
      skippedCount: 0,
    };
  }
  const request = buildSourceUseAssemblyRequest(records, {
    includeSourceFiles: true,
    compactPaths: SOURCE_USE_ASSEMBLY_PATH_TABLE,
    compactEnums: SOURCE_USE_ASSEMBLY_ENUM_TABLE,
    compactSpecifiers: SOURCE_USE_ASSEMBLY_SPECIFIER_TABLE,
  });
  const response = runAuditCoreJsonResultFile(
    ["source-use-assembly-artifact", "--input", "-"],
    "source-use-assembly-artifact",
    { input: JSON.stringify(request) },
  );
  return {
    handled: new Set(
      Array.isArray(response.handledRecordIds)
        ? response.handledRecordIds
        : [],
    ),
    resolvedByRecordId: new Map(
      Array.isArray(response.resolvedRecordTargets)
        ? response.resolvedRecordTargets
          .filter((entry) =>
            typeof entry?.recordId === "string" &&
            typeof entry?.resolvedFile === "string" &&
            entry.resolvedFile.length > 0
          )
          .map((entry) => [entry.recordId, entry.resolvedFile])
        : [],
    ),
    unresolvedInternalSpecifierRecords: Array.isArray(
      response.unresolvedInternalSpecifierRecords,
    )
      ? response.unresolvedInternalSpecifierRecords
      : [],
    skippedCount: Array.isArray(response.skippedRecords)
      ? response.skippedRecords.length
      : 0,
  };
}

function sourceUseAssemblyRecordsWithResolvedTargets(records, resolution) {
  return records
    .filter((record) => resolution.handled.has(record.recordId))
    .map((record) => {
      const resolvedFile = resolution.resolvedByRecordId.get(record.recordId);
      return typeof resolvedFile === "string" && resolvedFile.length > 0
        ? { ...record, resolvedFile: sourceUseAssemblyPath(resolvedFile) }
        : record;
    });
}

function sourceUseAssemblyNeedsSourceFiles(records) {
  return records.some((record) =>
    record?.resolverStage === "relative" &&
    typeof record.resolvedFile !== "string"
  );
}

function sourceUseAssemblyReExportEntries(map) {
  const entries = [];
  for (const [barrelFile, byName] of map) {
    for (const [exportedName, target] of byName) {
      entries.push({
        barrelFile: relPath(ROOT, barrelFile),
        exportedName,
        targetFile: relPath(ROOT, target.targetFile),
        sourceSpec: target.sourceSpec,
      });
    }
  }
  return entries;
}

function compactSourceUseAssemblyRecordIds(records) {
  return records.map((record, index) => ({
    ...record,
    recordId: `r${index}`,
  }));
}

function compactSourceUseAssemblyRecordPaths(records) {
  const pathTable = [];
  const pathIds = new Map();
  const pathId = (value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = pathIds.get(value);
    if (existing !== undefined) return existing;
    const id = pathTable.length;
    pathTable.push(value);
    pathIds.set(value, id);
    return id;
  };
  return {
    pathTable,
    records: records.map((record) => {
      const { consumerFile, resolvedFile, ...rest } = record;
      const consumerFileId = pathId(consumerFile);
      const resolvedFileId = pathId(resolvedFile);
      return {
        ...rest,
        ...(consumerFileId !== null ? { consumerFileId } : {}),
        ...(resolvedFileId !== null ? { resolvedFileId } : {}),
      };
    }),
  };
}

function compactSourceUseAssemblyRecordEnums(records) {
  const kindTable = [];
  const resolverStageTable = [];
  const consumerSourceTable = [];
  const tableId = (table, ids, value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = ids.get(value);
    if (existing !== undefined) return existing;
    const id = table.length;
    table.push(value);
    ids.set(value, id);
    return id;
  };
  const kindIds = new Map();
  const resolverStageIds = new Map();
  const consumerSourceIds = new Map();
  return {
    kindTable,
    resolverStageTable,
    consumerSourceTable,
    records: records.map((record) => {
      const { kind, resolverStage, consumerSource, ...rest } = record;
      const kindId = tableId(kindTable, kindIds, kind);
      const resolverStageId = tableId(
        resolverStageTable,
        resolverStageIds,
        resolverStage,
      );
      const consumerSourceId = tableId(
        consumerSourceTable,
        consumerSourceIds,
        consumerSource,
      );
      return {
        ...rest,
        ...(kindId !== null ? { kindId } : {}),
        ...(resolverStageId !== null ? { resolverStageId } : {}),
        ...(consumerSourceId !== null ? { consumerSourceId } : {}),
      };
    }),
  };
}

function compactSourceUseAssemblyRecordSpecifiers(records) {
  const specifierTable = [];
  const specifierIds = new Map();
  const specifierId = (value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = specifierIds.get(value);
    if (existing !== undefined) return existing;
    const id = specifierTable.length;
    specifierTable.push(value);
    specifierIds.set(value, id);
    return id;
  };
  return {
    specifierTable,
    records: records.map((record) => {
      const { fromSpec, ...rest } = record;
      const fromSpecId = specifierId(fromSpec);
      return {
        ...rest,
        ...(fromSpecId !== null ? { fromSpecId } : {}),
      };
    }),
  };
}

function buildSourceUseAssemblyRequest(
  records,
  {
    includeSourceFiles = true,
    compactRecordIds = false,
    compactPaths = false,
    compactEnums = false,
    compactSpecifiers = false,
  } = {},
) {
  const sourceFiles = includeSourceFiles
    ? [...scannedJsSourceFiles].map((file) => relPath(ROOT, file))
    : [];
  const namespaceReExports = sourceUseAssemblyReExportEntries(namespaceReExportsByFile);
  const namedReExports = sourceUseAssemblyReExportEntries(namedReExportsByFile);
  let outputRecords = compactRecordIds
    ? compactSourceUseAssemblyRecordIds(records)
    : records;
  let pathTable = [];
  if (compactPaths) {
    const compacted = compactSourceUseAssemblyRecordPaths(outputRecords);
    outputRecords = compacted.records;
    pathTable = compacted.pathTable;
  }
  let kindTable = [];
  let resolverStageTable = [];
  let consumerSourceTable = [];
  if (compactEnums) {
    const compacted = compactSourceUseAssemblyRecordEnums(outputRecords);
    outputRecords = compacted.records;
    kindTable = compacted.kindTable;
    resolverStageTable = compacted.resolverStageTable;
    consumerSourceTable = compacted.consumerSourceTable;
  }
  let specifierTable = [];
  if (compactSpecifiers) {
    const compacted = compactSourceUseAssemblyRecordSpecifiers(outputRecords);
    outputRecords = compacted.records;
    specifierTable = compacted.specifierTable;
  }
  return {
    schemaVersion: "lumin-source-use-assembly-request.v1",
    root: ROOT,
    ...(DEFAULT_IMPORT_META_GLOB_CAP !== 64
      ? { importMetaGlobCap: DEFAULT_IMPORT_META_GLOB_CAP }
      : {}),
    ...(sourceFiles.length > 0 ? { sourceFiles } : {}),
    ...(namespaceReExports.length > 0 ? { namespaceReExports } : {}),
    ...(namedReExports.length > 0 ? { namedReExports } : {}),
    ...(pathTable.length > 0 ? { pathTable } : {}),
    ...(kindTable.length > 0 ? { kindTable } : {}),
    ...(resolverStageTable.length > 0 ? { resolverStageTable } : {}),
    ...(consumerSourceTable.length > 0 ? { consumerSourceTable } : {}),
    ...(specifierTable.length > 0 ? { specifierTable } : {}),
    records: outputRecords,
  };
}

function symbolArtifactFileDataRecord(filePath, info) {
  return {
    filePath,
    ...((info.reExports?.length ?? 0) > 0
      ? { reExports: info.reExports }
      : {}),
    ...((info.classMethods?.length ?? 0) > 0
      ? { classMethods: info.classMethods }
      : {}),
    ...((info.localOperations?.length ?? 0) > 0
      ? { localOperations: info.localOperations }
      : {}),
    ...((info.dynamicImportOpacity?.length ?? 0) > 0
      ? { dynamicImportOpacity: info.dynamicImportOpacity }
      : {}),
    ...(info.cjsExportSurface !== undefined && info.cjsExportSurface !== null
      ? { cjsExportSurface: info.cjsExportSurface }
      : {}),
    ...((info.cjsRequireOpacity?.length ?? 0) > 0
      ? { cjsRequireOpacity: info.cjsRequireOpacity }
      : {}),
    ...(info.pyDunderAll !== undefined
      ? { pyDunderAll: info.pyDunderAll }
      : {}),
  };
}

function symbolArtifactParseErrorCacheEntries(entries) {
  return Object.fromEntries(
    Object.entries(entries)
      .filter(([, entry]) => entry?.parseError === true)
      .map(([filePath]) => [filePath, { parseError: true }]),
  );
}

function compactSymbolGraphArtifactPaths(request) {
  const pathTable = [];
  const pathIds = new Map();
  const pathId = (value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = pathIds.get(value);
    if (existing !== undefined) return existing;
    const id = pathTable.length;
    pathTable.push(value);
    pathIds.set(value, id);
    return id;
  };
  const fileIds = request.files
    .map(pathId)
    .filter((id) => id !== null);
  const defIndex = request.defIndex.map(({ filePath, ...entry }) => {
    const filePathId = pathId(filePath);
    return {
      ...entry,
      ...(filePathId !== null ? { filePathId } : { filePath }),
    };
  });
  const fileData = request.fileData.map(({ filePath, ...entry }) => {
    const filePathId = pathId(filePath);
    return {
      ...entry,
      ...(filePathId !== null ? { filePathId } : { filePath }),
    };
  });
  return {
    ...request,
    ...(pathTable.length > 0 ? { pathTable } : {}),
    files: fileIds.length === request.files.length ? [] : request.files,
    ...(fileIds.length === request.files.length ? { fileIds } : {}),
    defIndex,
    fileData,
  };
}

function runSourceUseAssembly() {
  const candidates = buildSourceUseAssemblyCandidates();
  const resolvedCandidates =
    resolveSourceUseAssemblyRecords(candidates.requiresResolution);
  const resolvedRecords = sourceUseAssemblyRecordsWithResolvedTargets(
    candidates.requiresResolution,
    resolvedCandidates,
  );
  embeddedSourceUseAssemblyRecords.push(...candidates.records);
  embeddedSourceUseAssemblyRecords.push(...resolvedRecords);
  generatedConsumerBlindZoneUnresolvedRecords.push(
    ...resolvedCandidates.unresolvedInternalSpecifierRecords,
  );
  phaseTimer.setCounter(
    "sourceUsePreHandledNamespaceReExportMissCount",
    candidates.preHandled.size,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyResolvableRelativeCandidateCount",
    candidates.requiresResolution.length,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyResolvableRelativeSkippedCount",
    resolvedCandidates.skippedCount,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyCandidateCount",
    candidates.records.length + candidates.requiresResolution.length,
  );
  phaseTimer.setCounter(
    "sourceUseRustAssemblyEmbeddedCount",
    candidates.records.length + resolvedRecords.length,
  );
  return {
    handled: new Set([
      ...candidates.preHandled,
      ...candidates.records.map((record) => record.recordId),
      ...resolvedRecords.map((record) => record.recordId),
    ]),
  };
}

function isUnresolvableNamespaceReExportUse(use) {
  return (
    isSourceUseAssemblyCandidate(use) &&
    namespaceReExportsByFile.size === 0 &&
    (use?.kind === "imported-namespace-member" ||
      use?.kind === "imported-namespace-escape")
  );
}

const rustSourceUseAssembly = runSourceUseAssembly();

for (const [consumerFile, info] of fileData) {
  for (let useIndex = 0; useIndex < info.uses.length; useIndex++) {
    const u = info.uses[useIndex];
    if (rustSourceUseAssembly.handled.has(sourceUseRecordId(consumerFile, useIndex))) {
      continue;
    }
    if (u?.kind === "import-meta-glob") {
      const branchStarted = performance.now();
      const expansion = expandImportMetaGlobPattern({
        root: ROOT,
        consumerFile,
        pattern: u.fromSpec,
        scannedSourceFileSet: scannedJsSourceFiles,
        cap: DEFAULT_IMPORT_META_GLOB_CAP,
      });

      if (expansion.ok) {
        incrementSourceUseBranch("importMetaGlobResolved");
        for (const targetFile of expansion.targets) {
          enqueueResolvedSourceUseAssemblyRecord(
            `${sourceUseRecordId(consumerFile, useIndex)}:glob:${relPath(ROOT, targetFile)}`,
            consumerFile,
            {
              ...u,
              kind: "dynamic-import-meta-glob",
              outputLevel: "resolved",
            },
            targetFile,
          );
        }
        addSourceUseTiming("resolvedInternal", branchStarted);
      } else {
        incrementSourceUseBranch("importMetaGlobUnsupported");
        if (
          enqueueUnresolvedSourceUseAssemblyRecord(
            sourceUseRecordId(consumerFile, useIndex),
            consumerFile,
            importMetaGlobDiagnosticUse(u, expansion),
            "unresolved-relative",
          )
        ) {
          recordGeneratedConsumerBlindZoneCandidate(
            consumerFile,
            importMetaGlobDiagnosticUse(u, expansion),
          );
          addSourceUseTiming("unresolved", branchStarted);
          continue;
        }
        unresolvedInternalUses++;
        unresolvedUses++;
        recordUnresolvedInternalSpecifier(
          consumerFile,
          importMetaGlobDiagnosticUse(u, expansion),
        );
        addSourceUseTiming("unresolved", branchStarted);
      }
      continue;
    }

    const resolveStarted = performance.now();
    const target = resolveSpecifier(consumerFile, u);
    addSourceUseTiming("resolve", resolveStarted);
    if (isRustResolvedRelativeUse(u)) rustResolvedRelativeUses++;
    if (target === "EXTERNAL") {
      const branchStarted = performance.now();
      incrementSourceUseBranch("external");
      if (
        enqueueExternalSourceUseAssemblyRecord(
          sourceUseRecordId(consumerFile, useIndex),
          consumerFile,
          u,
          "source-import",
        )
      ) {
        addSourceUseTiming("external", branchStarted);
        continue;
      }
      if (isImportedNamespaceAliasUse(u)) {
        incrementSourceUseBranch("skippedNamespaceAlias");
        addSourceUseTiming("external", branchStarted);
        continue;
      }
      // External npm package. NOT a blind spot for dead-export
      // analysis — external packages don't consume internal exports.
      externalUses++;
      unresolvedUses++; // legacy counter for backward-compat
      addSourceUseTiming("external", branchStarted);
      continue;
    }
    if (isNonSourceAssetResolution(target)) {
      const branchStarted = performance.now();
      incrementSourceUseBranch("asset");
      if (
        enqueueNonSourceAssetSourceUseAssemblyRecord(
          sourceUseRecordId(consumerFile, useIndex),
          consumerFile,
          u,
        )
      ) {
        addSourceUseTiming("asset", branchStarted);
        continue;
      }
      nonSourceAssetUses++;
      addSourceUseTiming("asset", branchStarted);
      continue;
    }
    if (target === "UNRESOLVED_INTERNAL") {
      const branchStarted = performance.now();
      incrementSourceUseBranch("unresolved");
      if (
        enqueueUnresolvedSourceUseAssemblyRecord(
          sourceUseRecordId(consumerFile, useIndex),
          consumerFile,
          u,
          "unresolved-internal",
        )
      ) {
        recordGeneratedConsumerBlindZoneCandidate(consumerFile, u);
        addSourceUseTiming("unresolved", branchStarted);
        continue;
      }
      // Local alias matched (e.g. `@/*` from tsconfig paths) but no
      // target file. THIS is a real blind spot — we probably missed
      // a legitimate consumer.
      unresolvedInternalUses++;
      unresolvedUses++;
      const spec = typeof u === "string" ? u : u.fromSpec;
      const p = prefixOf(spec);
      unresolvedInternalByPrefix.set(
        p,
        (unresolvedInternalByPrefix.get(p) ?? 0) + 1,
      );
      if (!prefixExamples.has(p)) prefixExamples.set(p, spec);
      recordUnresolvedInternalSpecifier(consumerFile, u);
      addSourceUseTiming("unresolved", branchStarted);
      continue;
    }
    if (isGeneratedVirtualResolution(target)) {
      const branchStarted = performance.now();
      incrementSourceUseBranch("generatedVirtual");
      enqueueGeneratedVirtualSourceUseAssemblyRecord(
        sourceUseRecordId(consumerFile, useIndex),
        consumerFile,
        u,
        target,
      );
      addSourceUseTiming("generatedVirtual", branchStarted);
      continue;
    }
    if (!target) {
      const branchStarted = performance.now();
      incrementSourceUseBranch("unresolved");
      if (
        enqueueUnresolvedSourceUseAssemblyRecord(
          sourceUseRecordId(consumerFile, useIndex),
          consumerFile,
          u,
          "unresolved-relative",
        )
      ) {
        recordGeneratedConsumerBlindZoneCandidate(consumerFile, u);
        addSourceUseTiming("unresolved", branchStarted);
        continue;
      }
      // null — relative path that didn't resolve, or malformed spec.
      // Treat conservatively as internal: a relative path that
      // doesn't find a file is more likely a scanner/parse issue than
      // an external package.
      unresolvedInternalUses++;
      unresolvedUses++;
      recordUnresolvedInternalSpecifier(consumerFile, u);
      addSourceUseTiming("unresolved", branchStarted);
      continue;
    }
    if (
      enqueueResolvedSourceUseAssemblyRecord(
        sourceUseRecordId(consumerFile, useIndex),
        consumerFile,
        u,
        target,
      )
    ) {
      continue;
    }
    if (
      u.kind === "imported-namespace-member" ||
      u.kind === "imported-namespace-escape"
    ) {
      const branchStarted = performance.now();
      incrementSourceUseBranch("namespaceReExport");
      const reExport = resolveNamespaceReExport(target, u.name);
      if (!reExport) {
        incrementSourceUseBranch("namespaceReExportMiss");
        addSourceUseTiming("namespaceReExport", branchStarted);
        continue;
      }
      totalUses++;
      resolvedInternalUses++;
      addResolvedInternalEdge(consumerFile, reExport.targetFile, u);
      if (u.kind === "imported-namespace-escape") {
        incrementSourceUseBranch("namespaceReExportEscape");
        addNamespaceReExportDiagnostic(consumerFile, target, u, reExport);
        addNamespaceUser(reExport.targetFile, consumerFile);
      } else if (u.memberName) {
        incrementSourceUseBranch("namespaceReExportMember");
        addConsumer(reExport.targetFile, u.memberName, consumerFile, {
          ...u,
          name: u.memberName,
        });
      }
      addSourceUseTiming("namespaceReExport", branchStarted);
      continue;
    }
    const branchStarted = performance.now();
    incrementSourceUseBranch("resolvedInternal");
    totalUses++;
    resolvedInternalUses++;
    addResolvedInternalEdge(consumerFile, target, u);
    // v0.6.6 FP-18: dynamic `import()` treated like namespace — whole-file
    // consumer, since we can't statically know which symbol the caller uses.
    // PCEF P0: CJS side-effect-only imports evaluate the file but do not
    // consume named exports, while CJS namespace escapes/re-exports are broad.
    if (u.kind === "cjs-side-effect-only" || u.kind === "import-side-effect") {
      incrementSourceUseBranch("sideEffectOnly");
      addSourceUseTiming("resolvedInternal", branchStarted);
      continue;
    }
    if (u.kind === "reExportNamespace") {
      incrementSourceUseBranch("reExportNamespaceSkip");
      addSourceUseTiming("resolvedInternal", branchStarted);
      continue;
    }
    if (
      u.kind === "namespace" ||
      u.kind === "reExportAll" ||
      u.kind === "dynamic" ||
      u.kind === "cjs-namespace-escape" ||
      u.kind === "cjs-reexport-broad"
    ) {
      incrementSourceUseBranch("broadNamespace");
      addNamespaceUser(target, consumerFile);
    } else {
      incrementSourceUseBranch("directConsumer");
      addConsumer(target, u.name, consumerFile, u);
    }
    addSourceUseTiming("resolvedInternal", branchStarted);
  }
}
const sourceUseResolverStatsAfter =
  typeof _resolveRaw.memoStats === "function" ? _resolveRaw.memoStats() : null;
const sourceUseResolverStageStatsAfter =
  typeof _resolveRaw.stageStats === "function"
    ? _resolveRaw.stageStats()
    : null;
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
phaseTimer.setCounter("sourceUseResolveMs", sourceUseTimings.resolve);
phaseTimer.setCounter("sourceUseRustResolvedRelativeCount", rustResolvedRelativeUses);
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
if (sourceUseResolverStatsBefore && sourceUseResolverStatsAfter) {
  phaseTimer.setCounter(
    "sourceUseResolverMemoHits",
    sourceUseResolverStatsAfter.hits - sourceUseResolverStatsBefore.hits,
  );
  phaseTimer.setCounter(
    "sourceUseResolverMemoMisses",
    sourceUseResolverStatsAfter.misses - sourceUseResolverStatsBefore.misses,
  );
  phaseTimer.setCounter(
    "sourceUseResolverMemoSize",
    sourceUseResolverStatsAfter.size,
  );
  phaseTimer.setCounter(
    "symbolResolverMemoHits",
    sourceUseResolverStatsAfter.hits,
  );
  phaseTimer.setCounter(
    "symbolResolverMemoMisses",
    sourceUseResolverStatsAfter.misses,
  );
  phaseTimer.setCounter(
    "symbolResolverMemoSize",
    sourceUseResolverStatsAfter.size,
  );
}
if (sourceUseResolverStageStatsBefore && sourceUseResolverStageStatsAfter) {
  const extraStageCounterFields = [
    ["PatternMatches", "patternMatches"],
    ["ProbeHits", "probeHits"],
    ["ProbeMisses", "probeMisses"],
    ["FallbackHits", "fallbackHits"],
    ["UnresolvedInternalResults", "unresolvedInternalResults"],
  ];
  for (const [stageName, after] of Object.entries(
    sourceUseResolverStageStatsAfter,
  )) {
    const before = sourceUseResolverStageStatsBefore[stageName] ?? {};
    const stem = `${stageName[0].toUpperCase()}${stageName.slice(1)}`;
    phaseTimer.setCounter(
      `sourceUseResolverStage${stem}Attempts`,
      (after.attempts ?? 0) - (before.attempts ?? 0),
    );
    phaseTimer.setCounter(
      `sourceUseResolverStage${stem}Results`,
      (after.terminalResults ?? 0) - (before.terminalResults ?? 0),
    );
    phaseTimer.setCounter(
      `sourceUseResolverStage${stem}Count`,
      (after.count ?? 0) - (before.count ?? 0),
    );
    phaseTimer.setCounter(
      `sourceUseResolverStage${stem}CacheHits`,
      (after.cacheHits ?? 0) - (before.cacheHits ?? 0),
    );
    phaseTimer.setCounter(
      `sourceUseResolverStage${stem}CacheMisses`,
      (after.cacheMisses ?? 0) - (before.cacheMisses ?? 0),
    );
    for (const [suffix, key] of extraStageCounterFields) {
      phaseTimer.setCounter(
        `sourceUseResolverStage${stem}${suffix}`,
        (after[key] ?? 0) - (before[key] ?? 0),
      );
    }
    phaseTimer.setCounter(
      `sourceUseResolverStage${stem}Ms`,
      (after.wallMs ?? 0) - (before.wallMs ?? 0),
    );
  }
}
phaseTimer.setCounter("sourceUseFilesProcessed", fileData.size);
phaseTimer.setCounter("sourceUseRecordsProcessed", useCount);
phaseTimer.recordPhase(
  "assemble-source-uses",
  Date.now() - assembleSourceUsesStarted,
);

function processOutOfBandImportConsumers(consumers, source, handledRecords = new Set()) {
  let resolvedConsumerUses = 0;
  for (let index = 0; index < consumers.length; index++) {
    const u = consumers[index];
    if (handledRecords.has(outOfBandSourceUseRecordId(source, index, u))) {
      continue;
    }
    const target = resolveSpecifier(u.consumerFile, u);
    if (target === "EXTERNAL") {
      if (
        enqueueExternalSourceUseAssemblyRecord(
          outOfBandSourceUseRecordId(source, index, u),
          u.consumerFile,
          u,
          source,
        )
      ) {
        continue;
      }
      externalUses++;
      unresolvedUses++;
      continue;
    }
    if (isNonSourceAssetResolution(target)) {
      if (
        enqueueNonSourceAssetSourceUseAssemblyRecord(
          outOfBandSourceUseRecordId(source, index, u),
          u.consumerFile,
          u,
        )
      ) {
        continue;
      }
      nonSourceAssetUses++;
      continue;
    }
    if (target === "UNRESOLVED_INTERNAL") {
      if (
        enqueueUnresolvedSourceUseAssemblyRecord(
          outOfBandSourceUseRecordId(source, index, u),
          u.consumerFile,
          u,
          "unresolved-internal",
        )
      ) {
        recordGeneratedConsumerBlindZoneCandidate(u.consumerFile, u);
        continue;
      }
      unresolvedInternalUses++;
      unresolvedUses++;
      const p = prefixOf(u.fromSpec);
      unresolvedInternalByPrefix.set(
        p,
        (unresolvedInternalByPrefix.get(p) ?? 0) + 1,
      );
      if (!prefixExamples.has(p)) prefixExamples.set(p, u.fromSpec);
      recordUnresolvedInternalSpecifier(u.consumerFile, u);
      continue;
    }
    if (isGeneratedVirtualResolution(target)) {
      enqueueGeneratedVirtualSourceUseAssemblyRecord(
        outOfBandSourceUseRecordId(source, index, u),
        u.consumerFile,
        u,
        target,
      );
      continue;
    }
    if (!target) {
      if (
        enqueueUnresolvedSourceUseAssemblyRecord(
          outOfBandSourceUseRecordId(source, index, u),
          u.consumerFile,
          u,
          "unresolved-relative",
        )
      ) {
        recordGeneratedConsumerBlindZoneCandidate(u.consumerFile, u);
        continue;
      }
      unresolvedInternalUses++;
      unresolvedUses++;
      recordUnresolvedInternalSpecifier(u.consumerFile, u);
      continue;
    }
    if (
      enqueueResolvedSourceUseAssemblyRecord(
        outOfBandSourceUseRecordId(source, index, u),
        u.consumerFile,
        u,
        target,
      )
    ) {
      continue;
    }
    totalUses++;
    resolvedInternalUses++;
    resolvedConsumerUses++;
    addResolvedInternalEdge(u.consumerFile, target, u);
    if (u.kind === "import-side-effect") continue;
    if (u.kind === "namespace") {
      addNamespaceUser(target, u.consumerFile);
    } else {
      addConsumer(target, u.name, u.consumerFile, u);
    }
  }
  return resolvedConsumerUses;
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

function processSfcScriptSourceReachability(consumers, handled = new Set()) {
  let resolvedReachabilityUses = 0;
  for (let index = 0; index < consumers.length; index++) {
    const u = consumers[index];
    if (handled.has(outOfBandSourceUseRecordId("sfc-script-src", index, u))) {
      continue;
    }
    let target = resolveSpecifier(u.consumerFile, u);
    if (target === "EXTERNAL") continue;
    if (isNonSourceAssetResolution(target)) {
      nonSourceAssetUses++;
      continue;
    }
    if (target === "UNRESOLVED_INTERNAL" || !target) {
      const diagnosticUse = {
        ...u,
        reason: "sfc-script-src-unresolved",
        resolverStage: "sfc-script-src",
        outputLevel: "unsupported",
        unsupportedFamily: "sfc-script-src",
        hint: "sfc-script-src-reachability",
      };
      unresolvedInternalUses++;
      unresolvedUses++;
      const p = prefixOf(u.fromSpec);
      unresolvedInternalByPrefix.set(
        p,
        (unresolvedInternalByPrefix.get(p) ?? 0) + 1,
      );
      if (!prefixExamples.has(p)) prefixExamples.set(p, u.fromSpec);
      recordUnresolvedInternalSpecifier(u.consumerFile, diagnosticUse);
      continue;
    }
    if (isGeneratedVirtualResolution(target)) {
      generatedVirtualSurfaces.set(target.id, target);
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
        nonSourceAssetUses++;
        continue;
      } else {
        const diagnosticUse = {
          ...u,
          reason: "sfc-script-src-unscanned-target",
          resolverStage: "sfc-script-src",
          outputLevel: "unsupported",
          unsupportedFamily: "sfc-script-src",
          hint: "sfc-script-src-source-target",
        };
        unresolvedInternalUses++;
        unresolvedUses++;
        recordUnresolvedInternalSpecifier(u.consumerFile, diagnosticUse);
        continue;
      }
    }

    totalUses++;
    resolvedInternalUses++;
    resolvedReachabilityUses++;
    addResolvedInternalEdge(u.consumerFile, target, u);
  }
  return resolvedReachabilityUses;
}

function processSfcStyleAssetReferences(consumers) {
  for (const use of consumers) {
    sfcStyleAssetReferenceInputs.push({
      consumerFile: use.consumerFile,
      fromSpec: use.fromSpec,
      source: use.source,
      kind: use.kind,
      styleKind: use.styleKind,
      confidence: use.confidence,
      importSyntax: use.importSyntax,
      line: Number.isFinite(use.line) ? use.line : undefined,
      sfcBlockKind: use.sfcBlockKind,
      sfcLanguage: use.sfcLanguage,
    });
  }
  return 0;
}

function processSfcTemplateComponentRefs(consumers, sourceUseResolution) {
  let recordedRefs = 0;
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    recordedRefs++;
    const input = {
      consumerFile: use.consumerFile,
      tagName: use.tagName,
      normalizedTagName: use.normalizedTagName,
      bindingName: use.bindingName,
      bindingSource: use.bindingSource,
      source: use.source,
      language: use.language,
      templateKind: use.templateKind,
      confidence: use.confidence,
      bindingKind: use.bindingKind,
      importedName: use.importedName,
      memberName: use.memberName,
      line: Number.isFinite(use.line) ? use.line : undefined,
      sfcBlockKind: use.sfcBlockKind,
    };
    if (use.status === "muted") {
      input.status = "muted";
      input.reason = use.reason ?? "sfc-template-component-muted";
      sfcTemplateComponentRefInputs.push(input);
      continue;
    }

    const assemblyTarget = sfcComponentSourceUseAssemblyTarget(
      sourceUseResolution,
      "sfc-template-component-ref",
      index,
      use.consumerFile,
      use.bindingSource,
    );
    if (assemblyTarget) {
      input.status = "resolved";
      input.resolvedFile = assemblyTarget;
      sfcTemplateComponentRefInputs.push(input);
      continue;
    }

    const target = resolveSpecifier(use.consumerFile, {
      ...use,
      fromSpec: use.bindingSource,
      kind: "sfc-template-component-ref",
      name: "*",
      typeOnly: false,
    });
    if (target === "EXTERNAL") {
      input.status = "external";
      input.reason = "sfc-template-component-external-binding";
      sfcTemplateComponentRefInputs.push(input);
      continue;
    }
    if (
      isNonSourceAssetResolution(target) ||
      isGeneratedVirtualResolution(target)
    ) {
      input.status = "muted";
      input.resolvedFile = isNonSourceAssetResolution(target)
        ? existingRelativeSpecifierTarget(use.consumerFile, use.bindingSource)
        : null;
      input.reason = "sfc-template-component-non-source-binding";
      sfcTemplateComponentRefInputs.push(input);
      continue;
    }
    if (target === "UNRESOLVED_INTERNAL" || !target) {
      input.status = "unresolved";
      input.reason = "sfc-template-component-unresolved";
      sfcTemplateComponentRefInputs.push(input);
      continue;
    }

    input.status = "resolved";
    input.resolvedFile = target;
    sfcTemplateComponentRefInputs.push(input);
  }
  return recordedRefs;
}

function sfcGlobalComponentResolutionSpec(use) {
  if (use?.status !== "muted") return use?.bindingSource;
  if (use.reason === "sfc-global-component-async-factory") {
    return use.fromSpec;
  }
  if (use.reason === "sfc-global-component-duplicate-registration") {
    return use.bindingSource;
  }
  return null;
}

function processSfcGlobalComponentRegistrations(consumers, sourceUseResolution) {
  let recordedRegistrations = 0;
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    recordedRegistrations++;
    const input = {
      registrationFile: use.registrationFile,
      framework: use.framework,
      api: use.api,
      componentName: use.componentName,
      normalizedTagNames: Array.isArray(use.normalizedTagNames)
        ? [...use.normalizedTagNames]
        : undefined,
      bindingName: use.bindingName,
      bindingSource: use.bindingSource,
      fromSpec: use.fromSpec,
      source: use.source,
      bindingKind: use.bindingKind,
      importedName: use.importedName,
      factoryKind: use.factoryKind,
      ambiguityKey: use.ambiguityKey,
      line: Number.isFinite(use.line) ? use.line : undefined,
    };
    if (use.status === "muted") {
      const mutedSpec = sfcGlobalComponentResolutionSpec(use);
      if (mutedSpec) {
        const assemblyTarget = sfcComponentSourceUseAssemblyTarget(
          sourceUseResolution,
          "sfc-global-component-registration",
          index,
          use.registrationFile,
          mutedSpec,
        );
        const target = assemblyTarget ?? resolveSpecifier(use.registrationFile, {
          ...use,
          fromSpec: mutedSpec,
          kind: "sfc-global-component-registration",
          name: "*",
          typeOnly: false,
        });
        input.status = "muted";
        input.resolvedFile =
          isNonSourceAssetResolution(target)
            ? existingRelativeSpecifierTarget(use.registrationFile, mutedSpec)
            : target &&
                target !== "EXTERNAL" &&
                target !== "UNRESOLVED_INTERNAL" &&
                !isGeneratedVirtualResolution(target)
              ? target
              : null;
        input.reason = use.reason ?? "sfc-global-component-muted";
        sfcGlobalComponentRegistrationInputs.push(input);
        continue;
      }
      input.status = "muted";
      input.reason = use.reason ?? "sfc-global-component-muted";
      sfcGlobalComponentRegistrationInputs.push(input);
      continue;
    }

    const assemblyTarget = sfcComponentSourceUseAssemblyTarget(
      sourceUseResolution,
      "sfc-global-component-registration",
      index,
      use.registrationFile,
      use.bindingSource,
    );
    if (assemblyTarget) {
      input.status = "resolved";
      input.resolvedFile = assemblyTarget;
      sfcGlobalComponentRegistrationInputs.push(input);
      continue;
    }

    const target = resolveSpecifier(use.registrationFile, {
      ...use,
      fromSpec: use.bindingSource,
      kind: "sfc-global-component-registration",
      name: "*",
      typeOnly: false,
    });
    if (target === "EXTERNAL") {
      input.status = "external";
      input.reason = "sfc-global-component-external-binding";
      sfcGlobalComponentRegistrationInputs.push(input);
      continue;
    }
    if (
      isNonSourceAssetResolution(target) ||
      isGeneratedVirtualResolution(target)
    ) {
      input.status = "muted";
      input.resolvedFile = isNonSourceAssetResolution(target)
          ? existingRelativeSpecifierTarget(
              use.registrationFile,
              use.bindingSource,
            )
          : null;
      input.reason = "sfc-global-component-non-source-binding";
      sfcGlobalComponentRegistrationInputs.push(input);
      continue;
    }
    if (target === "UNRESOLVED_INTERNAL" || !target) {
      input.status = "unresolved";
      input.reason = "sfc-global-component-unresolved";
      sfcGlobalComponentRegistrationInputs.push(input);
      continue;
    }

    input.status = "resolved";
    input.resolvedFile = target;
    sfcGlobalComponentRegistrationInputs.push(input);
  }
  return recordedRegistrations;
}

function processSfcGeneratedComponentManifests(consumers, sourceUseResolution) {
  let recordedManifests = 0;
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    recordedManifests++;
    const input = {
      manifestFile: use.manifestFile,
      manifestKind: use.manifestKind,
      componentName: use.componentName,
      normalizedTagNames: Array.isArray(use.normalizedTagNames)
        ? [...use.normalizedTagNames]
        : [],
      bindingSource: use.bindingSource,
      fromSpec: use.fromSpec,
      computedKeySource: use.computedKeySource,
      source: use.source,
      confidence: use.confidence,
      line: Number.isFinite(use.line) ? use.line : undefined,
    };
    if (use.status === "skipped") {
      input.status = "skipped";
      input.reason = use.reason ?? "sfc-framework-generated-manifest-nonliteral";
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    const assemblyTarget = sfcComponentSourceUseAssemblyTarget(
      sourceUseResolution,
      "sfc-generated-component-manifest",
      index,
      use.manifestFile,
      use.bindingSource,
    );
    if (assemblyTarget) {
      input.status = "resolved";
      input.resolvedFile = assemblyTarget;
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    const target = resolveSpecifier(use.manifestFile, {
      ...use,
      fromSpec: use.bindingSource,
      kind: "sfc-generated-component-manifest",
      name: "*",
      typeOnly: false,
    });

    if (target === "EXTERNAL") {
      continue;
    }

    if (isNonSourceAssetResolution(target)) {
      const resolvedFile = existingRelativeSpecifierTarget(
        use.manifestFile,
        use.bindingSource,
      );
      input.status = resolvedFile ? "muted" : "unresolved";
      if (resolvedFile) {
        input.resolvedFile = resolvedFile;
      }
      input.reason = resolvedFile
          ? "sfc-framework-generated-manifest-non-source-binding"
          : "sfc-framework-generated-manifest-unresolved";
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    if (isGeneratedVirtualResolution(target)) {
      input.status = "muted";
      input.reason = "sfc-framework-generated-manifest-non-source-binding";
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    if (target === "UNRESOLVED_INTERNAL" || !target) {
      input.status = "unresolved";
      input.reason = "sfc-framework-generated-manifest-unresolved";
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    if (isSfcFamilyTarget(target)) {
      input.status = "muted";
      input.resolvedFile = target;
      input.reason = "sfc-framework-generated-manifest-non-source-binding";
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    if (isJsFamilyTarget(target)) {
      input.status = "resolved";
      input.resolvedFile = target;
      sfcGeneratedComponentManifestInputs.push(input);
      continue;
    }

    input.status = "muted";
    input.resolvedFile = target;
    input.reason = "sfc-framework-generated-manifest-non-source-binding";
    sfcGeneratedComponentManifestInputs.push(input);
  }
  return recordedManifests;
}

const assembleMdxUsesStarted = Date.now();
const mdxImportConsumers = collectMdxImportConsumers({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files: mdxSourceFiles,
});
phaseTimer.setCounter(
  "mdxImportConsumerCandidateCount",
  mdxImportConsumers.length,
);
const mdxSourceUseAssemblyRecords = buildOutOfBandSourceUseAssemblyCandidateRecords(
  mdxImportConsumers,
  "mdx-import",
);

const assembleSfcScriptUsesStarted = Date.now();
const sfcImportConsumers = collectSfcImportConsumers({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files: sfcSourceFiles,
});
phaseTimer.setCounter(
  "sfcScriptImportConsumerCandidateCount",
  sfcImportConsumers.length,
);
const sfcScriptSourceUseAssemblyRecords = buildOutOfBandSourceUseAssemblyCandidateRecords(
  sfcImportConsumers,
  "sfc-script-import",
);

const assembleSfcScriptSrcStarted = Date.now();
const sfcScriptSources = collectSfcScriptSources({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files: sfcSourceFiles,
});
phaseTimer.setCounter("sfcScriptSrcCandidateCount", sfcScriptSources.length);
const sfcScriptSrcSourceUseAssemblyRecords =
  buildOutOfBandSourceUseAssemblyCandidateRecords(
    sfcScriptSources,
    "sfc-script-src",
  );

const outOfBandSourceUseAssemblyResolutionStarted = Date.now();
const outOfBandSourceUseAssemblyResolution = resolveSourceUseAssemblyRecords([
  ...mdxSourceUseAssemblyRecords,
  ...sfcScriptSourceUseAssemblyRecords,
  ...sfcScriptSrcSourceUseAssemblyRecords,
]);
generatedConsumerBlindZoneUnresolvedRecords.push(
  ...outOfBandSourceUseAssemblyResolution.unresolvedInternalSpecifierRecords,
);
phaseTimer.recordPhase(
  "source-use-out-of-band-rust-assembly",
  Date.now() - outOfBandSourceUseAssemblyResolutionStarted,
);
phaseTimer.setCounter(
  "outOfBandSourceUseRustAssemblySkippedCount",
  outOfBandSourceUseAssemblyResolution.skippedCount,
);

const mdxSourceUseAssemblyHandledRecords = sourceUseAssemblyRecordsWithResolvedTargets(
  mdxSourceUseAssemblyRecords,
  outOfBandSourceUseAssemblyResolution,
);
embeddedSourceUseAssemblyRecords.push(...mdxSourceUseAssemblyHandledRecords);
const mdxSourceUseAssemblyHandled = new Set(
  mdxSourceUseAssemblyHandledRecords.map((record) => record.recordId),
);
phaseTimer.setCounter(
  "mdxSourceUseRustAssemblyCandidateCount",
  mdxSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "mdxSourceUseRustAssemblyEmbeddedCount",
  mdxSourceUseAssemblyHandledRecords.length,
);
mdxConsumerUses = processOutOfBandImportConsumers(
  mdxImportConsumers,
  "mdx-import",
  mdxSourceUseAssemblyHandled,
);
phaseTimer.recordPhase(
  "assemble-mdx-uses",
  Date.now() - assembleMdxUsesStarted,
);

const sfcScriptSourceUseAssemblyHandledRecords =
  sourceUseAssemblyRecordsWithResolvedTargets(
    sfcScriptSourceUseAssemblyRecords,
    outOfBandSourceUseAssemblyResolution,
  );
embeddedSourceUseAssemblyRecords.push(...sfcScriptSourceUseAssemblyHandledRecords);
const sfcScriptSourceUseAssemblyHandled = new Set(
  sfcScriptSourceUseAssemblyHandledRecords.map((record) => record.recordId),
);
phaseTimer.setCounter(
  "sfcScriptSourceUseRustAssemblyCandidateCount",
  sfcScriptSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sfcScriptSourceUseRustAssemblyEmbeddedCount",
  sfcScriptSourceUseAssemblyHandledRecords.length,
);
sfcScriptConsumerUses = processOutOfBandImportConsumers(
  sfcImportConsumers,
  "sfc-script-import",
  sfcScriptSourceUseAssemblyHandled,
);
phaseTimer.recordPhase(
  "assemble-sfc-script-uses",
  Date.now() - assembleSfcScriptUsesStarted,
);

const sfcScriptSrcSourceUseAssemblyHandledRecords =
  sourceUseAssemblyRecordsWithResolvedTargets(
    sfcScriptSrcSourceUseAssemblyRecords,
    outOfBandSourceUseAssemblyResolution,
  );
embeddedSourceUseAssemblyRecords.push(...sfcScriptSrcSourceUseAssemblyHandledRecords);
const sfcScriptSrcSourceUseAssemblyHandled = new Set(
  sfcScriptSrcSourceUseAssemblyHandledRecords.map((record) => record.recordId),
);
phaseTimer.setCounter(
  "sfcScriptSrcSourceUseRustAssemblyCandidateCount",
  sfcScriptSrcSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sfcScriptSrcSourceUseRustAssemblyEmbeddedCount",
  sfcScriptSrcSourceUseAssemblyHandledRecords.length,
);
sfcScriptSrcReachabilityUses =
  processSfcScriptSourceReachability(
    sfcScriptSources,
    sfcScriptSrcSourceUseAssemblyHandled,
  );
phaseTimer.recordPhase(
  "assemble-sfc-script-src-uses",
  Date.now() - assembleSfcScriptSrcStarted,
);

const assembleSfcStyleAssetsStarted = Date.now();
const sfcStyleAssets = collectSfcStyleAssetReferences({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files: sfcSourceFiles,
});
phaseTimer.setCounter("sfcStyleAssetCandidateCount", sfcStyleAssets.length);
sfcStyleAssetReferenceUses = processSfcStyleAssetReferences(sfcStyleAssets);
phaseTimer.recordPhase(
  "assemble-sfc-style-assets",
  Date.now() - assembleSfcStyleAssetsStarted,
);

const assembleSfcTemplateRefsStarted = Date.now();
const sfcTemplateRefs = collectSfcTemplateComponentRefs({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files: sfcSourceFiles,
});
phaseTimer.setCounter(
  "sfcTemplateComponentRefCandidateCount",
  sfcTemplateRefs.length,
);

const assembleSfcGlobalRegistrationsStarted = Date.now();
const sfcGlobalRegistrations = collectSfcGlobalComponentRegistrations({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files,
});
phaseTimer.setCounter(
  "sfcGlobalComponentRegistrationCandidateCount",
  sfcGlobalRegistrations.length,
);

const assembleSfcGeneratedManifestsStarted = Date.now();
const sfcGeneratedManifests = collectSfcGeneratedComponentManifests({
  root: ROOT,
});
phaseTimer.setCounter(
  "sfcGeneratedComponentManifestCandidateCount",
  sfcGeneratedManifests.length,
);
sfcGeneratedComponentManifestUses = 0;

const sfcTemplateComponentSourceUseAssemblyRecords =
  buildSfcComponentSourceUseAssemblyCandidateRecords(
    sfcTemplateRefs,
    "sfc-template-component-ref",
    {
      consumerFileForUse: (use) => use.consumerFile,
      fromSpecForUse: (use) => use.bindingSource,
      kind: "sfc-template-component-ref",
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
    },
  );
const sfcComponentSourceUseAssemblyStarted = Date.now();
const sfcComponentSourceUseAssemblyResolution = resolveSourceUseAssemblyRecords([
  ...sfcTemplateComponentSourceUseAssemblyRecords,
  ...sfcGlobalComponentSourceUseAssemblyRecords,
  ...sfcGeneratedManifestSourceUseAssemblyRecords,
]);
generatedConsumerBlindZoneUnresolvedRecords.push(
  ...sfcComponentSourceUseAssemblyResolution.unresolvedInternalSpecifierRecords,
);
phaseTimer.recordPhase(
  "sfc-component-source-use-rust-assembly",
  Date.now() - sfcComponentSourceUseAssemblyStarted,
);
phaseTimer.setCounter(
  "sfcComponentSourceUseRustAssemblyCandidateCount",
  sfcTemplateComponentSourceUseAssemblyRecords.length +
    sfcGlobalComponentSourceUseAssemblyRecords.length +
    sfcGeneratedManifestSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sfcComponentSourceUseRustAssemblyResolvedCount",
  sfcComponentSourceUseAssemblyResolution.handled.size,
);
phaseTimer.setCounter(
  "sfcComponentSourceUseRustAssemblySkippedCount",
  sfcComponentSourceUseAssemblyResolution.skippedCount,
);

sfcTemplateComponentRefUses = processSfcTemplateComponentRefs(
  sfcTemplateRefs,
  sfcComponentSourceUseAssemblyResolution,
);
phaseTimer.recordPhase(
  "assemble-sfc-template-component-refs",
  Date.now() - assembleSfcTemplateRefsStarted,
);

sfcGlobalComponentRegistrationUses = processSfcGlobalComponentRegistrations(
  sfcGlobalRegistrations,
  sfcComponentSourceUseAssemblyResolution,
);
phaseTimer.recordPhase(
  "assemble-sfc-global-component-registrations",
  Date.now() - assembleSfcGlobalRegistrationsStarted,
);

sfcGeneratedComponentManifestUses = processSfcGeneratedComponentManifests(
  sfcGeneratedManifests,
  sfcComponentSourceUseAssemblyResolution,
);
phaseTimer.recordPhase(
  "assemble-sfc-generated-component-manifests",
  Date.now() - assembleSfcGeneratedManifestsStarted,
);

const assembleSfcFrameworkConventionsStarted = Date.now();
const sfcFrameworkConventions = collectSfcFrameworkConventionComponents({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  files: sfcSourceFiles,
});
phaseTimer.setCounter(
  "sfcFrameworkConventionComponentCandidateCount",
  sfcFrameworkConventions.length,
);
for (const use of sfcFrameworkConventions) {
  sfcFrameworkConventionComponentInputs.push(use);
}
sfcFrameworkConventionComponentUses = sfcFrameworkConventions.length;
phaseTimer.recordPhase(
  "assemble-sfc-framework-convention-components",
  Date.now() - assembleSfcFrameworkConventionsStarted,
);

console.log(`[uses:js-pre-rust] total ${totalUses}, unresolved ${unresolvedUses}`);
console.log(
  `[uses:js-pre-rust] resolvedInternal: ${resolvedInternalUses}, external: ${externalUses}, unresolvedInternal: ${unresolvedInternalUses}`,
);
console.log(
  `[defs] total symbols: ${[...defIndex.values()].reduce((a, m) => a + m.size, 0)}`,
);
phaseTimer.setCounter("totalUses", totalUses);
phaseTimer.setCounter("unresolvedUses", unresolvedUses);
phaseTimer.setCounter("resolvedInternalUses", resolvedInternalUses);
phaseTimer.setCounter(
  "resolvedGeneratedVirtualUses",
  resolvedGeneratedVirtualUses,
);
phaseTimer.setCounter("nonSourceAssetUses", nonSourceAssetUses);
phaseTimer.setCounter("externalUses", externalUses);
phaseTimer.setCounter("unresolvedInternalUses", unresolvedInternalUses);
phaseTimer.setCounter("mdxConsumerUses", mdxConsumerUses);
phaseTimer.setCounter("sfcScriptConsumerUses", sfcScriptConsumerUses);
phaseTimer.setCounter(
  "sfcScriptSrcReachabilityUses",
  sfcScriptSrcReachabilityUses,
);
phaseTimer.setCounter("sfcStyleAssetReferenceUses", sfcStyleAssetReferenceUses);
phaseTimer.setCounter(
  "sfcStyleAssetReferenceCount",
  sfcStyleAssetReferenceInputs.length,
);
phaseTimer.setCounter(
  "sfcTemplateComponentRefUses",
  sfcTemplateComponentRefUses,
);
phaseTimer.setCounter(
  "sfcTemplateComponentRefCount",
  sfcTemplateComponentRefInputs.length,
);
phaseTimer.setCounter(
  "sfcGlobalComponentRegistrationUses",
  sfcGlobalComponentRegistrationUses,
);
phaseTimer.setCounter(
  "sfcGlobalComponentRegistrationCount",
  sfcGlobalComponentRegistrationInputs.length,
);
phaseTimer.setCounter(
  "sfcGeneratedComponentManifestUses",
  sfcGeneratedComponentManifestUses,
);
phaseTimer.setCounter(
  "sfcGeneratedComponentManifestCount",
  sfcGeneratedComponentManifestInputs.length,
);
phaseTimer.setCounter(
  "sfcFrameworkConventionComponentUses",
  sfcFrameworkConventionComponentUses,
);
phaseTimer.setCounter(
  "sfcFrameworkConventionComponentCount",
  sfcFrameworkConventionComponentInputs.length,
);
phaseTimer.setCounter(
  "dependencyImportConsumerCount",
  dependencyImportConsumers.length,
);
phaseTimer.setCounter(
  "resolvedInternalEdgeCount",
  resolvedInternalEdges.length,
);
phaseTimer.setCounter(
  "unresolvedInternalSpecifierCount",
  unresolvedInternalSpecifiers.size,
);
phaseTimer.setCounter(
  "unresolvedInternalSpecifierRecordCount",
  unresolvedInternalSpecifierRecords.length,
);
phaseTimer.setCounter(
  "generatedVirtualSurfaceCount",
  generatedVirtualSurfaces.size,
);
phaseTimer.setCounter(
  "generatedVirtualImportConsumerCount",
  generatedVirtualImportConsumers.length,
);

const assembleGeneratedBlindZonesStarted = Date.now();
const generatedConsumerBlindZones = buildGeneratedConsumerBlindZones(
  {
    unresolvedInternalSpecifierRecords: [
      ...unresolvedInternalSpecifierRecords,
      ...generatedConsumerBlindZoneUnresolvedRecords,
    ],
  },
  {
    root: ROOT,
    includeTests: cli.includeTests,
    exclude: cli.exclude,
    mode: GENERATED_ARTIFACTS_MODE,
  },
);
phaseTimer.recordPhase(
  "assemble-generated-blind-zones",
  Date.now() - assembleGeneratedBlindZonesStarted,
);

// ─── Dead export raw inputs ───────────────────────────────
const assembleDeadCandidatesStarted = Date.now();
const deadCandidateInputs = buildDeadCandidateInputs();
phaseTimer.setCounter("barrelFileCount", deadCandidateInputs.barrelFiles.length);
phaseTimer.recordPhase(
  "assemble-dead-candidates",
  Date.now() - assembleDeadCandidatesStarted,
);

// ─── Symbol fan-in raw inputs ────────────────────────────
const assembleFanInStarted = Date.now();
const fanInInputs = buildFanInInputs();
phaseTimer.recordPhase("assemble-fan-in", Date.now() - assembleFanInStarted);

const assembleAnyContaminationStarted = Date.now();
const anyContaminationFacts = buildAnyContaminationFacts({
  root: ROOT,
  defIndex,
  fileData,
});
phaseTimer.recordPhase(
  "assemble-any-contamination",
  Date.now() - assembleAnyContaminationStarted,
);

// ─── 리포트 ───────────────────────────────────────────────
console.log(`\n\n════════ 1. Top 25 심볼 fan-in ════════`);
console.log(`  Rust-owned fan-in projection is written to symbols.json`);

// ─── Dead 요약 ───────────────────────────────────────────
console.log(`\n\n════════ 2. Dead export 후보 ════════`);
console.log(`  Rust-owned dead candidate projection is written to symbols.json`);
phaseTimer.setCounter("symbolFanInCount", fanInInputs.consumerSymbolCount);
phaseTimer.setCounter("fanInIdentityCount", fanInInputs.identityCount);
phaseTimer.setCounter("fanInIdentitySpaceCount", fanInInputs.identityCount);
const embeddedSourceUseAssemblyRequest = buildSourceUseAssemblyRequest(
  embeddedSourceUseAssemblyRecords,
  {
    includeSourceFiles: sourceUseAssemblyNeedsSourceFiles(
      embeddedSourceUseAssemblyRecords,
    ),
    compactRecordIds: true,
    compactPaths: SOURCE_USE_ASSEMBLY_PATH_TABLE,
    compactEnums: SOURCE_USE_ASSEMBLY_ENUM_TABLE,
    compactSpecifiers: SOURCE_USE_ASSEMBLY_SPECIFIER_TABLE,
  },
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyTotalEmbeddedCount",
  embeddedSourceUseAssemblyRecords.length,
);
phaseTimer.setCounter(
  "sourceUseRustAssemblyCompactedRecordIdCount",
  embeddedSourceUseAssemblyRequest.records.length,
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
phaseTimer.setCounter(
  "namespaceReExportDiagnosticCount",
  namespaceReExportDiagnostics.length,
);
phaseTimer.setCounter(
  "generatedConsumerBlindZoneCount",
  generatedConsumerBlindZones.length,
);
phaseTimer.recordPhase(
  "assemble-symbol-graph",
  Date.now() - assembleSymbolGraphStarted,
);

// ─── 저장 ─────────────────────────────────────────────────
const outPath = path.join(output, "symbols.json");
const requestPath = path.join(output, ".symbols-artifact-request.tmp.json");
const generated = new Date().toISOString();
const buildArtifactRequestStarted = Date.now();
const artifactParseErrorCacheEntries = symbolArtifactParseErrorCacheEntries(
  nextCache.entries,
);
let artifactRequest = {
  schemaVersion: "lumin-symbol-graph-producer-request.v1",
  generated,
  root: ROOT,
  files,
  defIndex: [...defIndex.entries()].map(([filePath, definitions]) => ({
    filePath,
    definitions: Object.fromEntries(definitions),
  })),
  fileData: [...fileData.entries()].map(([filePath, info]) => ({
    ...symbolArtifactFileDataRecord(filePath, info),
  })),
  parseErrors,
  warnings,
  // Rust only uses this legacy field to project filesWithParseErrors. Keep the
  // field for old helper compatibility, but do not resend every cached fact.
  nextCacheEntries: artifactParseErrorCacheEntries,
  unresolvedInternalByPrefix: [...unresolvedInternalByPrefix.entries()].map(
    ([key, count]) => ({ key, count }),
  ),
  prefixExamples: Object.fromEntries(prefixExamples),
  unresolvedInternalSpecifiers: [...unresolvedInternalSpecifiers],
  unresolvedInternalSpecifierRecords,
  languageSupport,
  totalUses,
  unresolvedUses,
  resolvedInternalUses,
  resolvedGeneratedVirtualUses,
  nonSourceAssetUses,
  externalUses,
  dependencyImportConsumers,
  resolvedInternalEdges,
  generatedConsumerBlindZones,
  generatedVirtualSurfaces: [...generatedVirtualSurfaces.values()],
  generatedVirtualImportConsumers,
  unresolvedInternalUses,
  mdxConsumerUses,
  sfcScriptConsumerUses,
  sfcScriptSrcReachabilityUses,
  sfcStyleAssetReferenceUses: 0,
  sfcTemplateComponentRefUses: 0,
  sfcGlobalComponentRegistrationUses: 0,
  sfcGeneratedComponentManifestUses: Math.max(
    0,
    sfcGeneratedComponentManifestUses -
      sfcGeneratedComponentManifestInputs.length,
  ),
  sfcFrameworkConventionComponentUses: 0,
  sfcStyleAssetReferences: [],
  sfcStyleAssetReferenceInputs,
  sfcTemplateComponentRefs: [],
  sfcTemplateComponentRefInputs,
  sfcGlobalComponentRegistrations: [],
  sfcGlobalComponentRegistrationInputs,
  sfcGeneratedComponentManifests: [],
  sfcGeneratedComponentManifestInputs,
  sfcFrameworkConventionComponents: [],
  sfcFrameworkConventionComponentInputs,
  dead: [],
  trulyDead: [],
  deadInProd: [],
  deadInTest: [],
  deadCandidateInputs,
  sourceUseAssembly: embeddedSourceUseAssemblyRequest,
  fanInInputs: {
    consumerEntries: fanInInputs.consumerEntries,
    namespaceUserEntries: fanInInputs.namespaceUserEntries,
  },
  namespaceReExportDiagnostics,
  anyContaminationFacts,
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
};
if (SYMBOL_GRAPH_PATH_TABLE) {
  artifactRequest = compactSymbolGraphArtifactPaths(artifactRequest);
}
phaseTimer.recordPhase(
  "build-symbol-artifact-request",
  Date.now() - buildArtifactRequestStarted,
);
phaseTimer.setCounter("symbolGraphArtifactRequestFileCount", files.length);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestFileDataCount",
  artifactRequest.fileData.length,
);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestDefIndexCount",
  artifactRequest.defIndex.length,
);
phaseTimer.setCounter(
  "symbolGraphArtifactPathTableEnabled",
  SYMBOL_GRAPH_PATH_TABLE ? 1 : 0,
);
phaseTimer.setCounter(
  "symbolGraphArtifactPathTableCount",
  artifactRequest.pathTable?.length ?? 0,
);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestResolvedInternalEdgeCount",
  resolvedInternalEdges.length,
);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestDeadCandidateCount",
  0,
);
phaseTimer.setCounter(
  "symbolGraphArtifactRequestParseErrorCacheEntryCount",
  Object.keys(artifactParseErrorCacheEntries).length,
);
const writeArtifactStarted = Date.now();
try {
  const requestJsonStarted = Date.now();
  const artifactRequestJson = JSON.stringify(artifactRequest);
  phaseTimer.recordPhase(
    "symbol-graph-artifact-request-json",
    Date.now() - requestJsonStarted,
  );
  phaseTimer.setCounter(
    "symbolGraphArtifactRequestBytes",
    Buffer.byteLength(artifactRequestJson, "utf8"),
  );
  const requestWriteStarted = Date.now();
  writeFileSync(requestPath, artifactRequestJson);
  phaseTimer.recordPhase(
    "symbol-graph-artifact-request-write",
    Date.now() - requestWriteStarted,
  );
  const commandStarted = Date.now();
  try {
    runAuditCoreJsonToResultFile(
      ["symbol-graph-artifact", "--input", requestPath],
      "symbol-graph-artifact",
      outPath,
    );
  } finally {
    phaseTimer.recordPhase(
      "symbol-graph-artifact-command",
      Date.now() - commandStarted,
    );
  }
} finally {
  rmSync(requestPath, { force: true });
}
phaseTimer.setCounter("symbolsJsonBytes", statSync(outPath).size);
const writtenSymbolSummary = readSymbolGraphArtifactSummary(outPath);
phaseTimer.setCounter(
  "totalUses",
  writtenSymbolSummary.totalUsesResolved ?? totalUses,
);
phaseTimer.setCounter(
  "resolvedInternalUses",
  writtenSymbolSummary.uses?.resolvedInternal ?? resolvedInternalUses,
);
phaseTimer.setCounter(
  "resolvedInternalEdgeCount",
  writtenSymbolSummary.resolvedInternalEdgeCount ?? resolvedInternalEdges.length,
);
phaseTimer.setCounter("deadCandidateCount", writtenSymbolSummary.deadTotal ?? 0);
phaseTimer.setCounter("trulyDeadCount", writtenSymbolSummary.trulyDead ?? 0);
phaseTimer.setCounter(
  "namespaceShadowedDeadCount",
  (writtenSymbolSummary.deadTotal ?? 0) -
    (writtenSymbolSummary.trulyDead ?? 0),
);
phaseTimer.setCounter("deadProductionCount", writtenSymbolSummary.deadInProd ?? 0);
phaseTimer.setCounter("deadTestCount", writtenSymbolSummary.deadInTest ?? 0);
phaseTimer.recordPhase("write-artifact", Date.now() - writeArtifactStarted);
phaseTimer.write();
console.log(
  `[uses] total ${writtenSymbolSummary.totalUsesResolved ?? totalUses}, unresolved ${writtenSymbolSummary.unresolvedUses ?? unresolvedUses}`,
);
console.log(
  `[uses] resolvedInternal: ${writtenSymbolSummary.uses?.resolvedInternal ?? resolvedInternalUses}, external: ${writtenSymbolSummary.uses?.external ?? externalUses}, unresolvedInternal: ${writtenSymbolSummary.uses?.unresolvedInternal ?? unresolvedInternalUses}`,
);
console.log(
  `[symbols] ${files.length} files, dead production candidates: ${writtenSymbolSummary.deadInProd ?? 0}`,
);
console.log(`[symbols] saved → ${outPath}`);
