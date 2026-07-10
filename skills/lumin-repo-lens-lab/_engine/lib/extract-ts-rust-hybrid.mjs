import { readFileSync, statSync } from 'node:fs';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { relPath } from './paths.mjs';

export const RUST_JS_EXTRACTOR_POLICY_VERSION = 'rust-js-extract-hybrid.v12';
const REQUEST_SCHEMA_VERSION = 'lumin-js-ts-extract-request.v1';
const MAX_BATCH_SOURCE_BYTES = 16 * 1024 * 1024;

// These are file-level fallback guards for semantic lanes the Rust foundation
// does not own yet. Dynamic `import(...)` is Rust-owned once the audit-core
// contract reports literal dynamic import evidence and nonliteral opacity.
const STATIC_UNSUPPORTED_PATTERNS = [];
const NEEDS_SOURCE_ELIGIBILITY_SCAN = STATIC_UNSUPPORTED_PATTERNS.length > 0;

function emptyReasonCounts() {
  return Object.create(null);
}

function incrementReason(counts, reason) {
  counts[reason] = (counts[reason] ?? 0) + 1;
}

export function rustJsEligibilityForSource(source) {
  for (const [reason, pattern] of STATIC_UNSUPPORTED_PATTERNS) {
    if (pattern.test(source)) return { eligible: false, reason };
  }

  return { eligible: true, reason: null };
}

function pushRequestFile(batch, root, filePath) {
  batch.push({
    filePath,
    artifactFilePath: relPath(root, filePath),
  });
}

function sourceByteEstimate(filePath, fileSizes) {
  const knownSize = fileSizes?.get?.(filePath);
  if (Number.isFinite(knownSize) && knownSize >= 0) return knownSize;
  try {
    return statSync(filePath).size;
  } catch {
    return 0;
  }
}

export function extractRustJsHybridBatch({
  root,
  files,
  fileSizes,
  sourceFiles = files,
  verbose = false,
}) {
  const results = new Map();
  const warnings = [];
  const fallbackByReason = emptyReasonCounts();
  const summary = {
    policyVersion: RUST_JS_EXTRACTOR_POLICY_VERSION,
    maxBatchSourceBytes: MAX_BATCH_SOURCE_BYTES,
    candidateFiles: files.length,
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
    fallbackByReason,
  };

  let disabledReason = null;
  let batch = [];
  let batchBytes = 0;

  function markFallback(reason) {
    summary.fallbackFiles++;
    incrementReason(fallbackByReason, reason);
  }

  function flushBatch() {
    if (batch.length === 0 || disabledReason) return;
    const request = {
      schemaVersion: REQUEST_SCHEMA_VERSION,
      sourceFiles: [...sourceFiles],
      files: batch,
    };
    const requestText = JSON.stringify(request);
    const currentBatch = batch;
    batch = [];
    batchBytes = 0;
    summary.batchCount++;
    summary.inputBytes += Buffer.byteLength(requestText, 'utf8');

    let response;
    try {
      response = runAuditCoreJsonResultFile(
        ['js-ts-extract-artifact', '--input', '-'],
        'symbols rust-js extractor',
        { input: requestText },
      );
    } catch (error) {
      disabledReason = error?.message ?? 'unknown audit-core failure';
      summary.commandFailedFiles += currentBatch.length;
      for (const file of currentBatch) markFallback('rust-command-failed');
      warnings.push({
        code: 'rust-js-extractor-unavailable',
        message: disabledReason,
        affected: currentBatch.length,
      });
      if (verbose) {
        console.error(`[symbols] rust-js extractor unavailable: ${disabledReason}`);
      }
      return;
    }

    const byFile = new Map(
      Array.isArray(response?.files)
        ? response.files.map((file) => [file.filePath, file])
        : [],
    );
    for (const file of currentBatch) {
      const result = byFile.get(file.filePath);
      if (!result) {
        markFallback('rust-result-missing');
        warnings.push({
          code: 'rust-js-extractor-missing-result',
          file: relPath(root, file.filePath),
          message: 'audit-core did not return a result for an eligible JS/TS file',
        });
        continue;
      }
      results.set(file.filePath, result);
      if (result.error) summary.rustParseErrorFiles++;
      else {
        summary.rustExtractedFiles++;
        summary.rustResolvedRelativeUses += (result.uses ?? []).filter(
          (use) => use?.resolverStage === 'relative' && typeof use.resolvedFile === 'string',
        ).length;
      }
    }
  }

  for (const filePath of files) {
    if (disabledReason) {
      markFallback('rust-command-disabled');
      continue;
    }

    let sourceBytes = sourceByteEstimate(filePath, fileSizes);
    if (NEEDS_SOURCE_ELIGIBILITY_SCAN) {
      let source;
      try {
        source = readFileSync(filePath, 'utf8');
      } catch (error) {
        summary.readErrorFiles++;
        markFallback('read-error');
        warnings.push({
          code: 'rust-js-extractor-read-error',
          file: relPath(root, filePath),
          message: error?.message ?? 'failed to read source',
        });
        continue;
      }

      const eligibility = rustJsEligibilityForSource(source);
      if (!eligibility.eligible) {
        markFallback(eligibility.reason);
        continue;
      }
      sourceBytes = Buffer.byteLength(source, 'utf8');
    }

    summary.eligibleFiles++;
    if (batch.length > 0 && batchBytes + sourceBytes > MAX_BATCH_SOURCE_BYTES) {
      flushBatch();
    }
    if (disabledReason) {
      markFallback('rust-command-disabled');
      continue;
    }
    summary.sourceBytes += sourceBytes;
    pushRequestFile(batch, root, filePath);
    batchBytes += sourceBytes;
  }

  flushBatch();
  return { results, summary, warnings };
}
