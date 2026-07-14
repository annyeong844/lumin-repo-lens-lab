import { readFileSync } from 'node:fs';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const SFC_FILE_FACTS_REQUEST_SCHEMA_VERSION =
  'lumin-sfc-file-facts-request.v1';
export const SFC_FILE_FACTS_RESPONSE_SCHEMA_VERSION =
  'lumin-sfc-file-facts-response.v1';

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function validateFileResult(result, filePath) {
  if (!isObject(result) || result.filePath !== filePath) {
    throw new Error(`sfc-file-facts-artifact: missing result for ${filePath}`);
  }
  for (const field of [
    'scriptImportConsumers',
    'scriptSources',
    'styleAssetReferences',
    'templateComponentRefs',
    'frameworkConventionComponents',
  ]) {
    if (!Array.isArray(result[field])) {
      throw new Error(
        `sfc-file-facts-artifact: ${filePath} result.${field} must be an array`,
      );
    }
  }
  return result;
}

export function extractSfcFileFactsForSources(files) {
  if (!Array.isArray(files)) {
    throw new TypeError('extractSfcFileFactsForSources: files must be an array');
  }
  if (files.length === 0) return [];
  const expectedPaths = files.map((file) => file?.filePath);
  if (
    expectedPaths.some(
      (filePath) => typeof filePath !== 'string' || filePath.length === 0,
    )
  ) {
    throw new TypeError(
      'extractSfcFileFactsForSources: every file requires filePath',
    );
  }
  if (new Set(expectedPaths).size !== expectedPaths.length) {
    throw new Error('extractSfcFileFactsForSources: duplicate filePath');
  }
  const response = runAuditCoreJsonResultFile(
    ['sfc-file-facts-artifact', '--input', '-'],
    'sfc-file-facts-artifact',
    {
      input: JSON.stringify({
        schemaVersion: SFC_FILE_FACTS_REQUEST_SCHEMA_VERSION,
        files,
      }),
    },
  );
  if (
    !isObject(response) ||
    response.schemaVersion !== SFC_FILE_FACTS_RESPONSE_SCHEMA_VERSION ||
    !Array.isArray(response.files) ||
    response.files.length !== files.length
  ) {
    throw new Error('sfc-file-facts-artifact: malformed result');
  }
  return response.files.map((result, index) =>
    validateFileResult(result, expectedPaths[index]),
  );
}

export function extractSfcFileFacts(files) {
  return extractSfcFileFactsForSources(
    files.map((filePath) => ({
      filePath,
      source: readFileSync(filePath, 'utf8'),
    })),
  );
}
