import {
  extractRustJsFactsForSources,
  extractRustJsHybridBatch,
} from './extract-ts-rust-hybrid.mjs';

function checkedResult(result, filePath) {
  if (!result || result.filePath !== filePath) {
    throw new Error(`js-ts-extract-artifact: missing result for ${filePath}`);
  }
  if (result.error) {
    throw new Error(`js-ts-extract-artifact: ${filePath}: ${result.error}`);
  }
  if (
    !Array.isArray(result.defs) ||
    !Array.isArray(result.uses) ||
    !Array.isArray(result.reExports)
  ) {
    throw new Error(`js-ts-extract-artifact: malformed facts for ${filePath}`);
  }
  return result;
}

export function createDefinitionsAndUsesExtractor({ root, files }) {
  if (typeof root !== 'string' || root.length === 0) {
    throw new TypeError('createDefinitionsAndUsesExtractor: root is required');
  }
  if (!Array.isArray(files)) {
    throw new TypeError('createDefinitionsAndUsesExtractor: files must be an array');
  }
  if (new Set(files).size !== files.length) {
    throw new Error('createDefinitionsAndUsesExtractor: duplicate file path');
  }

  const { results } = extractRustJsHybridBatch({
    root,
    files,
    sourceFiles: files,
    label: 'canon rust-js extractor',
  });
  const scopedFiles = new Set(files);
  return function extractDefinitionsAndUsesFromIndex(filePath) {
    if (!scopedFiles.has(filePath)) {
      throw new Error(
        `createDefinitionsAndUsesExtractor: file is outside the scoped index: ${filePath}`,
      );
    }
    return checkedResult(results.get(filePath), filePath);
  };
}

// Compatibility entrypoint for focused callers. Repository scans must use
// createDefinitionsAndUsesExtractor() so the Rust owner runs in bounded batches.
export function extractDefinitionsAndUses(filePath, options = {}) {
  const [result] = extractRustJsFactsForSources(
    [
      {
        filePath,
        artifactFilePath: options.artifactFilePath ?? filePath,
      },
    ],
    { sourceFiles: options.sourceFiles ?? [] },
  );
  return checkedResult(result, filePath);
}
