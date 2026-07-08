import { readFileSync } from 'node:fs';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { relPath } from './paths.mjs';

export const RUST_JS_EXTRACTOR_POLICY_VERSION = 'rust-js-extract-hybrid.v6';
const REQUEST_SCHEMA_VERSION = 'lumin-js-ts-extract-request.v1';
const MAX_BATCH_SOURCE_BYTES = 16 * 1024 * 1024;

// These are file-level fallback guards for semantic lanes the Rust foundation
// does not own yet. Dynamic import `.then(...)` callback member precision and
// nonliteral opacity stay on the JS extractor until Rust reaches full parity.
const STATIC_UNSUPPORTED_PATTERNS = [
  ['cjs-require', /\brequire\s*\(/],
];
const MODULE_EXPORTS_ASSIGNMENT_PATTERN =
  /(?:^|[^\w$.])module\s*\.\s*exports\s*(?:(?:\.\s*[$A-Z_a-z][$\w]*)|(?:\[\s*[^\]]+\s*\]))?\s*=/m;
const EXPORTS_MEMBER_ASSIGNMENT_PATTERN =
  /(?:^|[^\w$.])exports\s*(?:(?:\.\s*[$A-Z_a-z][$\w]*)|(?:\[\s*[^\]]+\s*\]))\s*=/m;

function emptyReasonCounts() {
  return Object.create(null);
}

function incrementReason(counts, reason) {
  counts[reason] = (counts[reason] ?? 0) + 1;
}

function importBindingNames(source) {
  const named = new Set();
  const namespace = new Set();
  const importPattern =
    /\bimport\s+(?:type\s+)?(?:[$A-Z_a-z][$\w]*\s*,\s*)?(?:(?:\*\s+as\s+([$A-Z_a-z][$\w]*))|\{([^}]*)\})\s+from\s*["'][^"']+["']/gms;

  for (const match of source.matchAll(importPattern)) {
    if (match[1]) namespace.add(match[1]);
    const specifierText = match[2];
    if (!specifierText) continue;
    for (const rawPart of specifierText.split(',')) {
      const part = rawPart.trim().replace(/^type\s+/, '');
      if (!part) continue;
      const aliasMatch = part.match(/\bas\s+([$A-Z_a-z][$\w]*)$/);
      const localName = aliasMatch?.[1] ?? part.match(/^([$A-Z_a-z][$\w]*)/)?.[1];
      if (localName) named.add(localName);
    }
  }

  return { named, namespace };
}

function syntaxStateAtOffset(source, offset) {
  let state = 'code';
  let escaped = false;

  for (let i = 0; i < offset; i++) {
    const ch = source[i];
    const next = source[i + 1];

    if (state === 'line-comment') {
      if (ch === '\n' || ch === '\r') state = 'code';
      continue;
    }
    if (state === 'block-comment') {
      if (ch === '*' && next === '/') {
        i++;
        state = 'code';
      }
      continue;
    }
    if (state === 'single-string' || state === 'double-string' || state === 'template-string') {
      const end = state === 'single-string' ? "'" : state === 'double-string' ? '"' : '`';
      if (escaped) {
        escaped = false;
      } else if (ch === '\\') {
        escaped = true;
      } else if (ch === end) {
        state = 'code';
      }
      continue;
    }

    if (ch === '/' && next === '/') {
      i++;
      state = 'line-comment';
    } else if (ch === '/' && next === '*') {
      i++;
      state = 'block-comment';
    } else if (ch === "'") {
      state = 'single-string';
    } else if (ch === '"') {
      state = 'double-string';
    } else if (ch === '`') {
      state = 'template-string';
    }
  }

  return state;
}

function hasUnsupportedDynamicImport(source) {
  const dynamicImportPattern = /\bimport\s*\(/g;
  for (const match of source.matchAll(dynamicImportPattern)) {
    if (syntaxStateAtOffset(source, match.index) !== 'code') continue;
    const rest = source.slice(match.index + match[0].length);
    if (/^\s*(["'])(?:\\.|(?!\1)[\s\S])*\1\s*\)\s*\.then\s*\(/.test(rest)) {
      return true;
    }
    if (/^\s*(["'])(?:\\.|(?!\1)[\s\S])*\1\s*\)/.test(rest)) {
      continue;
    }
    return true;
  }
  return false;
}

function hasCjsExportSurface(source) {
  return (
    MODULE_EXPORTS_ASSIGNMENT_PATTERN.test(source) ||
    EXPORTS_MEMBER_ASSIGNMENT_PATTERN.test(source)
  );
}

export function rustJsEligibilityForSource(source) {
  for (const [reason, pattern] of STATIC_UNSUPPORTED_PATTERNS) {
    if (pattern.test(source)) return { eligible: false, reason };
  }
  if (hasCjsExportSurface(source)) {
    return { eligible: false, reason: 'cjs-export-surface' };
  }
  if (hasUnsupportedDynamicImport(source)) {
    return { eligible: false, reason: 'dynamic-import' };
  }
  const imports = importBindingNames(source);
  if (imports.namespace.size > 0) {
    return { eligible: false, reason: 'namespace-import' };
  }

  return { eligible: true, reason: null };
}

function pushRequestFile(batch, root, filePath) {
  batch.push({
    filePath,
    artifactFilePath: relPath(root, filePath),
  });
}

export function extractRustJsHybridBatch({
  root,
  files,
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

    summary.eligibleFiles++;
    const sourceBytes = Buffer.byteLength(source, 'utf8');
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
