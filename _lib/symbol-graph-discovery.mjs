import { createHash } from "node:crypto";
import path from "node:path";

import { goExtractShape as extractGoShape } from "./extract-go.mjs";
import { pythonExtractShape as extractPythonShape } from "./extract-py.mjs";
import { extractRustJsHybridBatch } from "./extract-ts-rust-hybrid.mjs";
import {
  buildContextFingerprint,
  buildRepoSnapshot,
} from "./incremental-snapshot.mjs";
import {
  clearIncrementalCache,
  getReusableFact,
  loadProducerCache,
  openIncrementalCacheStore,
  putFact,
  saveProducerCache,
  strictCacheKeyForEntry,
} from "./incremental-cache-store.mjs";
import { JS_FAMILY_LANGS, SFC_FAMILY_LANGS } from "./lang.mjs";
import { relPath } from "./paths.mjs";
import { extractPythonBatch } from "./python.mjs";
import { extractTreeSitterBatch } from "./tree-sitter-langs.mjs";

const MDX_FAMILY_LANGS = ["mdx"];

export const SYMBOL_GRAPH_PRODUCER_ID = "symbols";
export const SYMBOL_GRAPH_PRODUCER_VERSION = 1;
export const SYMBOL_GRAPH_FACT_SCHEMA_VERSION = 5;
export const SYMBOL_GRAPH_PARSER_IDENTITY =
  "symbol-graph-extractors:v6-rust-js-dynamic-opacity";

function isJsFamilyFile(filePath) {
  return JS_FAMILY_LANGS.includes(
    path.extname(filePath).slice(1).toLowerCase(),
  );
}

export function isSfcFamilyFile(filePath) {
  return SFC_FAMILY_LANGS.includes(
    path.extname(filePath).slice(1).toLowerCase(),
  );
}

export function isMdxFamilyFile(filePath) {
  return MDX_FAMILY_LANGS.includes(
    path.extname(filePath).slice(1).toLowerCase(),
  );
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

function emptyRustJsBatch(candidateFiles) {
  return {
    results: new Map(),
    summary: {
      candidateFiles,
      eligibleFiles: 0,
      fallbackFiles: 0,
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
}

function setSnapshotCounters({
  phaseTimer,
  snapshotEntries,
  files,
  mdxSourceFiles,
  sfcSourceFiles,
  pythonEnabled,
  treeSitterEnabled,
}) {
  const jsTotal = files.filter(isJsFamilyFile).length;
  const pyTotal = files.filter((file) => file.endsWith(".py")).length;
  const goTotal = files.filter((file) => file.endsWith(".go")).length;
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
  phaseTimer.setCounter("snapshotMdxFiles", mdxSourceFiles.length);
  phaseTimer.setCounter("snapshotSfcFiles", sfcSourceFiles.length);
  phaseTimer.setCounter("snapshotPythonFiles", pyTotal);
  phaseTimer.setCounter("snapshotGoFiles", goTotal);
  console.error(
    `[symbols] scanning ${files.length} files (mdx=${mdxSourceFiles.length}, sfc=${sfcSourceFiles.length}, python=${pythonEnabled ? `on, ${pyTotal} .py` : "off"}, go=${treeSitterEnabled ? `on, ${goTotal} .go` : "off"})`,
  );
}

function setExtractionCounters(phaseTimer, counts, rustJsSummary) {
  for (const [name, value] of Object.entries(counts)) {
    phaseTimer.setCounter(name, value);
  }
  phaseTimer.setCounter(
    "rustJsExtractorCandidateFiles",
    rustJsSummary.candidateFiles,
  );
  phaseTimer.setCounter(
    "rustJsExtractorEligibleFiles",
    rustJsSummary.eligibleFiles,
  );
  phaseTimer.setCounter(
    "rustJsExtractorFallbackFiles",
    rustJsSummary.fallbackFiles,
  );
  phaseTimer.setCounter(
    "rustJsExtractorExtractedFiles",
    rustJsSummary.rustExtractedFiles,
  );
  phaseTimer.setCounter(
    "rustJsExtractorResolvedRelativeUses",
    rustJsSummary.rustResolvedRelativeUses ?? 0,
  );
  phaseTimer.setCounter(
    "rustJsExtractorParseErrorFiles",
    rustJsSummary.rustParseErrorFiles,
  );
  phaseTimer.setCounter(
    "rustJsExtractorReadErrorFiles",
    rustJsSummary.readErrorFiles,
  );
  phaseTimer.setCounter(
    "rustJsExtractorCommandFailedFiles",
    rustJsSummary.commandFailedFiles,
  );
  phaseTimer.setCounter("rustJsExtractorBatchCount", rustJsSummary.batchCount);
  phaseTimer.setCounter("rustJsExtractorInputBytes", rustJsSummary.inputBytes);
  phaseTimer.setCounter(
    "rustJsExtractorSourceBytes",
    rustJsSummary.sourceBytes ?? 0,
  );
}

function assembleFileData(nextCache) {
  const fileData = new Map();
  const counts = {
    definitionCount: 0,
    useCount: 0,
    reExportCount: 0,
    typeEscapeCount: 0,
    dynamicImportOpacityCount: 0,
    cjsRequireOpacityCount: 0,
    classMethodCount: 0,
    localOperationCount: 0,
  };
  for (const [filePath, entry] of Object.entries(nextCache.entries)) {
    if (entry.parseError || entry.defs === undefined) continue;
    counts.definitionCount += (entry.defs ?? []).length;
    counts.useCount += (entry.uses ?? []).length;
    counts.reExportCount += (entry.reExports ?? []).length;
    counts.typeEscapeCount += (entry.typeEscapes ?? []).length;
    counts.dynamicImportOpacityCount +=
      (entry.dynamicImportOpacity ?? []).length;
    counts.cjsRequireOpacityCount += (entry.cjsRequireOpacity ?? []).length;
    counts.classMethodCount += (entry.classMethods ?? []).length;
    counts.localOperationCount += (entry.localOperations ?? []).length;
    fileData.set(filePath, {
      filePath,
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
      ...(entry.pyDunderAll !== undefined
        ? { pyDunderAll: entry.pyDunderAll }
        : {}),
    });
  }
  return { fileData, counts };
}

export async function discoverSymbolGraphFacts({
  root,
  includeTests,
  exclude,
  repoMode,
  pythonEnabled,
  treeSitterEnabled,
  cacheRoot,
  clearCache,
  noIncremental,
  phaseTimer,
  verbose,
}) {
  const languages = [...JS_FAMILY_LANGS, ...SFC_FAMILY_LANGS, ...MDX_FAMILY_LANGS];
  if (pythonEnabled) languages.push("py");
  if (treeSitterEnabled) languages.push("go");
  const incrementalEnabled = noIncremental !== true;
  const contextFingerprint = buildContextFingerprint({
    includeTests,
    exclude,
    languages,
    producerContext: {
      producer: SYMBOL_GRAPH_PRODUCER_ID,
      producerVersion: SYMBOL_GRAPH_PRODUCER_VERSION,
      factSchemaVersion: SYMBOL_GRAPH_FACT_SCHEMA_VERSION,
      parserIdentity: SYMBOL_GRAPH_PARSER_IDENTITY,
      repoMode,
      pythonEnabled,
      treeSitterEnabled,
    },
  });
  const snapshot = phaseTimer.runPhase("snapshot", () =>
    buildRepoSnapshot({
      root,
      includeTests,
      exclude,
      languages,
      contextFingerprint,
      hashContents: incrementalEnabled,
    }),
  );
  const snapshotEntries = Object.values(snapshot.files);
  const files = snapshotEntries.map((entry) => entry.absPath);
  const fileSizes = new Map(
    snapshotEntries
      .filter((entry) => Number.isFinite(entry.size) && entry.size >= 0)
      .map((entry) => [entry.absPath, entry.size]),
  );
  const scannedJsSourceFiles = new Set(files.filter(isJsFamilyFile));
  const jsSourceSetFingerprint = buildSourceSetFingerprint(
    root,
    scannedJsSourceFiles,
  );
  const mdxSourceFiles = files.filter(isMdxFamilyFile);
  const sfcSourceFiles = files.filter(isSfcFamilyFile);
  setSnapshotCounters({
    phaseTimer,
    snapshotEntries,
    files,
    mdxSourceFiles,
    sfcSourceFiles,
    pythonEnabled,
    treeSitterEnabled,
  });

  const cacheStore = openIncrementalCacheStore({ root, cacheRoot });
  if (clearCache === true) clearIncrementalCache(cacheStore);
  const producerCacheMeta = {
    producerId: SYMBOL_GRAPH_PRODUCER_ID,
    producerVersion: SYMBOL_GRAPH_PRODUCER_VERSION,
    factSchemaVersion: SYMBOL_GRAPH_FACT_SCHEMA_VERSION,
    parserIdentity: SYMBOL_GRAPH_PARSER_IDENTITY,
    scanFingerprint: contextFingerprint,
    configFingerprint: contextFingerprint,
  };
  const producerCacheMetaForEntry = (entry) =>
    entry && isJsFamilyFile(entry.absPath)
      ? { ...producerCacheMeta, sourceSetFingerprint: jsSourceSetFingerprint }
      : producerCacheMeta;
  const priorCache = incrementalEnabled
    ? loadProducerCache(cacheStore, SYMBOL_GRAPH_PRODUCER_ID)
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

  const extractionStarted = Date.now();
  const changedSet = new Set(changed);
  const changedPy = changed.filter((file) => file.endsWith(".py"));
  const changedJs = changed.filter(isJsFamilyFile);
  const changedMdx = changed.filter(isMdxFamilyFile);
  const changedSfc = changed.filter(isSfcFamilyFile);
  const changedGo = changed.filter((file) => file.endsWith(".go"));
  phaseTimer.setCounter("changedJsFiles", changedJs.length);
  phaseTimer.setCounter("changedMdxFiles", changedMdx.length);
  phaseTimer.setCounter("changedSfcFiles", changedSfc.length);
  phaseTimer.setCounter("changedPythonFiles", changedPy.length);
  phaseTimer.setCounter("changedGoFiles", changedGo.length);
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
  const timeExtractPhase = (bucket, action) => {
    const started = Date.now();
    try {
      return action();
    } finally {
      extractPhaseMs[bucket] += Date.now() - started;
    }
  };

  let pythonBatch = new Map();
  if (changedPy.length > 0 && pythonEnabled) {
    const started = Date.now();
    try {
      pythonBatch = extractPythonBatch(changedPy) ?? new Map();
      const meta = pythonBatch.get("__meta__");
      if (meta?.parseFailures > 0) {
        warnings.push({
          code: "python-ndjson-parse-failure",
          count: meta.parseFailures,
          message: `${meta.parseFailures} stray non-JSON lines in extractor stdout`,
        });
      }
      pythonBatch.delete("__meta__");
    } catch (error) {
      console.error(`[symbols] python batch failed: ${error.message}`);
      warnings.push({
        code: "python-batch-crashed",
        message: error.message,
        affected: changedPy.length,
      });
    } finally {
      extractPhaseMs.pythonBatch += Date.now() - started;
    }
  }

  let goBatch = new Map();
  if (changedGo.length > 0 && treeSitterEnabled) {
    const started = Date.now();
    try {
      goBatch = (await extractTreeSitterBatch(changedGo)) ?? new Map();
    } catch (error) {
      console.error(`[symbols] tree-sitter batch failed: ${error.message}`);
      warnings.push({
        code: "tree-sitter-batch-crashed",
        message: error.message,
        affected: changedGo.length,
      });
    } finally {
      extractPhaseMs.goBatch += Date.now() - started;
    }
  }

  let rustJsBatch = emptyRustJsBatch(changedJs.length);
  if (changedJs.length > 0) {
    const started = Date.now();
    try {
      rustJsBatch = extractRustJsHybridBatch({
        root,
        files: changedJs,
        fileSizes,
        sourceFiles: scannedJsSourceFiles,
      });
      warnings.push(...rustJsBatch.warnings);
    } finally {
      extractPhaseMs.rustJsBatch += Date.now() - started;
    }
  }
  const missingRustJsResults = changedJs.filter(
    (file) => !rustJsBatch.results.has(file),
  );
  if (missingRustJsResults.length > 0) {
    throw new Error(
      `symbols rust-js extractor omitted ${missingRustJsResults.length} changed file result(s)`,
    );
  }

  let parseErrors = 0;
  const extracted = {
    extractedFiles: 0,
    extractedJsFiles: 0,
    extractedMdxFiles: 0,
    extractedSfcFiles: 0,
    extractedPythonFiles: 0,
    extractedGoFiles: 0,
  };
  for (const file of changed) {
    const snapshotEntry = snapshot.files[relPath(root, file)];
    try {
      let payload;
      if (file.endsWith(".py")) {
        const record = pythonBatch.get(file);
        if (!record || record.error) {
          parseErrors++;
          if (record?.error && verbose) {
            console.error(`py fail: ${file}: ${record.error}`);
          }
          nextCache.entries[file] = { parseError: true };
          if (incrementalEnabled && snapshotEntry) {
            putFact(nextProducerCache, {
              snapshotEntry,
              producerMeta: producerCacheMetaForEntry(snapshotEntry),
              payload: nextCache.entries[file],
            });
          }
          continue;
        }
        payload = timeExtractPhase("pythonShapes", () =>
          extractPythonShape(file, record),
        );
      } else if (file.endsWith(".go")) {
        const record = goBatch.get(file);
        if (!record || record.error) {
          parseErrors++;
          if (record?.error && verbose) {
            console.error(`go fail: ${file}: ${record.error}`);
          }
          nextCache.entries[file] = { parseError: true };
          if (incrementalEnabled && snapshotEntry) {
            putFact(nextProducerCache, {
              snapshotEntry,
              producerMeta: producerCacheMetaForEntry(snapshotEntry),
              payload: nextCache.entries[file],
            });
          }
          continue;
        }
        payload = timeExtractPhase("goShapes", () =>
          extractGoShape(file, record),
        );
      } else if (isMdxFamilyFile(file)) {
        payload = timeExtractPhase("mdxFiles", () => ({
          defs: [],
          uses: [],
          reExports: [],
          loc: 0,
        }));
      } else if (isSfcFamilyFile(file)) {
        payload = timeExtractPhase("sfcFiles", () => ({
          defs: [],
          uses: [],
          reExports: [],
          loc: 0,
        }));
      } else {
        const result = rustJsBatch.results.get(file);
        if (result.error) {
          parseErrors++;
          if (verbose) {
            console.error(`js rust parse fail: ${file}: ${result.error}`);
          }
          nextCache.entries[file] = { parseError: true };
          if (incrementalEnabled && snapshotEntry) {
            putFact(nextProducerCache, {
              snapshotEntry,
              producerMeta: producerCacheMetaForEntry(snapshotEntry),
              payload: nextCache.entries[file],
            });
          }
          continue;
        }
        payload = result;
      }
      nextCache.entries[file] = { ...payload, parseError: false };
      extracted.extractedFiles++;
      if (file.endsWith(".py")) extracted.extractedPythonFiles++;
      else if (file.endsWith(".go")) extracted.extractedGoFiles++;
      else if (isJsFamilyFile(file)) extracted.extractedJsFiles++;
      else if (isMdxFamilyFile(file)) extracted.extractedMdxFiles++;
      else if (isSfcFamilyFile(file)) extracted.extractedSfcFiles++;
      if (incrementalEnabled && snapshotEntry) {
        putFact(nextProducerCache, {
          snapshotEntry,
          producerMeta: producerCacheMetaForEntry(snapshotEntry),
          payload: nextCache.entries[file],
        });
      }
    } catch (error) {
      parseErrors++;
      console.error(`parse fail: ${file}: ${error.message}`);
      nextCache.entries[file] = { parseError: true };
      if (incrementalEnabled && snapshotEntry) {
        putFact(nextProducerCache, {
          snapshotEntry,
          producerMeta: producerCacheMetaForEntry(snapshotEntry),
          payload: nextCache.entries[file],
        });
      }
    }
  }
  for (const [file, entry] of Object.entries(nextCache.entries)) {
    if (!changedSet.has(file) && entry?.parseError) parseErrors++;
  }
  phaseTimer.recordPhase("extract-python-batch", extractPhaseMs.pythonBatch);
  phaseTimer.recordPhase("extract-go-batch", extractPhaseMs.goBatch);
  phaseTimer.recordPhase("extract-rust-js-batch", extractPhaseMs.rustJsBatch);
  phaseTimer.recordPhase("extract-js-files", extractPhaseMs.jsFiles);
  phaseTimer.recordPhase("extract-mdx-files", extractPhaseMs.mdxFiles);
  phaseTimer.recordPhase("extract-sfc-files", extractPhaseMs.sfcFiles);
  phaseTimer.recordPhase("extract-python-shapes", extractPhaseMs.pythonShapes);
  phaseTimer.recordPhase("extract-go-shapes", extractPhaseMs.goShapes);
  phaseTimer.recordPhase("extract-changed-files", Date.now() - extractionStarted);
  setExtractionCounters(
    phaseTimer,
    { ...extracted, parseErrorCount: parseErrors },
    rustJsBatch.summary,
  );

  const assembleSymbolGraphStarted = Date.now();
  const assembleStarted = Date.now();
  const { fileData, counts } = assembleFileData(nextCache);
  if (incrementalEnabled) {
    saveProducerCache(cacheStore, SYMBOL_GRAPH_PRODUCER_ID, nextProducerCache);
  }
  phaseTimer.setCounter("fileDataFiles", fileData.size);
  for (const [name, value] of Object.entries(counts)) {
    phaseTimer.setCounter(name, value);
  }
  phaseTimer.recordPhase("assemble-file-data", Date.now() - assembleStarted);
  console.log(`[parse] errors: ${parseErrors}`);

  return {
    files,
    scannedJsSourceFiles,
    mdxSourceFiles,
    sfcSourceFiles,
    cacheStore,
    nextCache,
    fileData,
    useCount: counts.useCount,
    warnings,
    incrementalEnabled,
    changedFiles,
    reusedFiles,
    droppedFiles,
    invalidatedFiles,
    assembleSymbolGraphStarted,
  };
}
