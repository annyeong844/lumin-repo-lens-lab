import { readFileSync } from 'node:fs';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { relPath } from './paths.mjs';

export const RUST_JS_EXTRACTOR_POLICY_VERSION = 'rust-js-extract-hybrid.v1';
const REQUEST_SCHEMA_VERSION = 'lumin-js-ts-extract-request.v1';
const MAX_BATCH_SOURCE_BYTES = 16 * 1024 * 1024;

// These are file-level fallback guards for semantic lanes the Rust foundation
// does not own yet. In particular, dynamic `import(...)` must stay on the JS
// extractor until Rust emits dynamic use evidence; otherwise lazy modules can
// look unreferenced to downstream dead-export consumers.
const STATIC_UNSUPPORTED_PATTERNS = [
  ['cjs-require', /\brequire\s*\(/],
  ['cjs-export-surface', /\b(?:module\s*\.\s*)?exports\s*(?:\.|\[|=)/],
  ['dynamic-import', /\bimport\s*\(/],
  ['import-meta', /\bimport\s*\.\s*meta\b/],
  ['comment-type-escape', /@ts-(?:ignore|expect-error|nocheck)\b/],
];

const LOCAL_OPERATION_VERBS = [
  'add',
  'create',
  'delete',
  'destroy',
  'dispatch',
  'emit',
  'fetch',
  'find',
  'get',
  'list',
  'load',
  'lookup',
  'patch',
  'query',
  'read',
  'remove',
  'resolve',
  'retrieve',
  'save',
  'search',
  'send',
  'set',
  'update',
  'upsert',
  'write',
];

const IDENTIFIER_PATTERN = String.raw`[$A-Z_a-z][$\w]*`;
const LOCAL_OPERATION_DECL_PATTERN = String.raw`\b(?:function|const|let|var)\s+(?:${LOCAL_OPERATION_VERBS.join('|')})[$A-Z_a-z0-9_]*\b`;
const INLINE_EXPORTED_LOCAL_OPERATION_PATTERN = new RegExp(
  String.raw`\bexport\s+(?:default\s+)?(?:async\s+)?(?:function(?:\s+${IDENTIFIER_PATTERN})?|const\s+${IDENTIFIER_PATTERN})[\s\S]{0,2000}${LOCAL_OPERATION_DECL_PATTERN}`,
);

function emptyReasonCounts() {
  return Object.create(null);
}

function incrementReason(counts, reason) {
  counts[reason] = (counts[reason] ?? 0) + 1;
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
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

function exportedLocalNames(source) {
  const names = new Set();
  for (const match of source.matchAll(/\bexport\s*\{([^}]*)\}/gms)) {
    const rest = source.slice(match.index + match[0].length);
    if (/^\s*from\b/.test(rest)) continue;
    for (const rawPart of match[1].split(',')) {
      const part = rawPart.trim().replace(/^type\s+/, '');
      if (!part) continue;
      const localName = part.match(/^([$A-Z_a-z][$\w]*)/)?.[1];
      if (localName) names.add(localName);
    }
  }
  for (const match of source.matchAll(/\bexport\s+default\s+([$A-Z_a-z][$\w]*)\b/g)) {
    names.add(match[1]);
  }
  return names;
}

function hasLocalOperationInDeclaration(source, localName) {
  const name = escapeRegExp(localName);
  return [
    new RegExp(String.raw`\b(?:async\s+)?function\s+${name}\b[\s\S]{0,2000}${LOCAL_OPERATION_DECL_PATTERN}`),
    new RegExp(String.raw`\b(?:const|let|var)\s+${name}\b[\s\S]{0,2000}${LOCAL_OPERATION_DECL_PATTERN}`),
    new RegExp(String.raw`\bclass\s+${name}\b[\s\S]{0,2000}${LOCAL_OPERATION_DECL_PATTERN}`),
  ].some((pattern) => pattern.test(source));
}

function hasExportedLocalOperationCandidate(source) {
  if (INLINE_EXPORTED_LOCAL_OPERATION_PATTERN.test(source)) return true;
  for (const localName of exportedLocalNames(source)) {
    if (hasLocalOperationInDeclaration(source, localName)) return true;
  }
  return false;
}

export function rustJsEligibilityForSource(source) {
  for (const [reason, pattern] of STATIC_UNSUPPORTED_PATTERNS) {
    if (pattern.test(source)) return { eligible: false, reason };
  }
  if (hasExportedLocalOperationCandidate(source)) {
    return { eligible: false, reason: 'local-operation-candidate' };
  }

  const imports = importBindingNames(source);
  if (imports.namespace.size > 0) {
    return { eligible: false, reason: 'namespace-import' };
  }

  return { eligible: true, reason: null };
}

function pushRequestFile(batch, root, filePath, source) {
  batch.push({
    filePath,
    artifactFilePath: relPath(root, filePath),
    source,
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
    const currentBatch = batch;
    batch = [];
    batchBytes = 0;
    summary.batchCount++;

    let response;
    try {
      response = runAuditCoreJsonResultFile(
        ['js-ts-extract-artifact', '--input', '-'],
        'symbols rust-js extractor',
        { input: JSON.stringify(request) },
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
    summary.inputBytes += sourceBytes;
    pushRequestFile(batch, root, filePath, source);
    batchBytes += sourceBytes;
  }

  flushBatch();
  return { results, summary, warnings };
}
