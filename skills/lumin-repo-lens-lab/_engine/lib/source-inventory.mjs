import { readFileSync } from 'node:fs';
import path from 'node:path';

export const SOURCE_INVENTORY_SCHEMA_VERSION = 'lumin-source-inventory.v2';
export const SOURCE_INVENTORY_POLICY_VERSION = 'lumin-source-walk.v1';

let configuredInventory = null;
const loadedInventoryCache = new Map();

function normalizedComparablePath(value) {
  const resolved = path.resolve(value);
  return process.platform === 'win32' ? resolved.toLowerCase() : resolved;
}

function isWithinRoot(root, candidate) {
  const rel = path.relative(root, candidate);
  return rel === '' || (!rel.startsWith('..') && !path.isAbsolute(rel));
}

function validateStringArray(value, field) {
  if (!Array.isArray(value) || value.some((item) => typeof item !== 'string')) {
    throw new Error(`source inventory ${field} must be an array of strings`);
  }
}

function arraysEqual(left, right) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function compareUtf8(left, right) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

function validateRunId(runId) {
  if (typeof runId !== 'string' || !/^[A-Za-z0-9._-]{1,128}$/.test(runId)) {
    throw new Error('source inventory runId must contain 1-128 safe identifier characters');
  }
}

function validateSortedUnique(values, field) {
  if (values.length === 0) throw new Error(`source inventory ${field} must not be empty`);
  for (let index = 1; index < values.length; index += 1) {
    if (compareUtf8(values[index - 1], values[index]) >= 0) {
      throw new Error(`source inventory ${field} must be strictly sorted and unique`);
    }
  }
}

function validateRelativePath(file, previous) {
  if (
    file.length === 0 ||
    file.includes('\0') ||
    file.includes('\\') ||
    path.posix.isAbsolute(file) ||
    path.posix.normalize(file) !== file ||
    file === '.' ||
    file === '..' ||
    file.startsWith('../')
  ) {
    throw new Error(`source inventory contains unsafe repo-relative path: ${file}`);
  }
  if (previous !== null && compareUtf8(previous, file) >= 0) {
    throw new Error(`source inventory files must be strictly sorted and unique: ${file}`);
  }
}

export function buildSourceInventoryArtifact({
  runId,
  root,
  analysisIncludeTests,
  exclude = [],
  languages,
  files,
}) {
  validateRunId(runId);
  const resolvedRoot = path.resolve(root);
  const repoRelativeFiles = files.map((file) => {
    const resolvedFile = path.resolve(file);
    if (!isWithinRoot(resolvedRoot, resolvedFile) || resolvedFile === resolvedRoot) {
      throw new Error(`source inventory file is outside root: ${file}`);
    }
    return path.relative(resolvedRoot, resolvedFile).split(path.sep).join('/');
  }).toSorted(compareUtf8);

  for (let index = 0; index < repoRelativeFiles.length; index += 1) {
    validateRelativePath(repoRelativeFiles[index], index === 0 ? null : repoRelativeFiles[index - 1]);
  }

  const sortedLanguages = [...new Set(languages)].toSorted(compareUtf8);
  const countsByLanguage = Object.fromEntries(sortedLanguages.map((language) => [language, 0]));
  for (const file of repoRelativeFiles) {
    const language = path.posix.extname(file).slice(1);
    if (Object.hasOwn(countsByLanguage, language)) countsByLanguage[language] += 1;
  }

  return {
    schemaVersion: SOURCE_INVENTORY_SCHEMA_VERSION,
    producer: 'triage-repo.mjs',
    runId,
    root: resolvedRoot,
    pathMode: 'repo-relative',
    walkScope: {
      includeTests: true,
      exclude: [...exclude],
      languages: sortedLanguages,
      policyVersion: SOURCE_INVENTORY_POLICY_VERSION,
    },
    analysisScope: {
      includeTests: analysisIncludeTests === true,
      exclude: [...exclude],
    },
    fileCount: repoRelativeFiles.length,
    countsByLanguage,
    files: repoRelativeFiles,
  };
}

export function configureSourceInventory(inputPath, runId, analysisScope) {
  if (!inputPath) return null;
  validateRunId(runId);
  const resolved = path.resolve(inputPath);
  const next = {
    path: resolved,
    runId,
    includeTests: analysisScope.includeTests === true,
    exclude: [...analysisScope.exclude],
  };
  if (configuredInventory !== null) {
    const same =
      normalizedComparablePath(configuredInventory.path) === normalizedComparablePath(next.path) &&
      configuredInventory.runId === next.runId &&
      configuredInventory.includeTests === next.includeTests &&
      arraysEqual(configuredInventory.exclude, next.exclude);
    if (!same) {
      throw new Error('source inventory is already configured with a different run contract');
    }
  }
  configuredInventory = next;
  return resolved;
}

export function activeSourceInventory() {
  return configuredInventory;
}

function loadValidatedArtifact(inputPath, expected) {
  const cacheKey = JSON.stringify([
    normalizedComparablePath(inputPath),
    expected.runId,
    expected.includeTests,
    expected.exclude,
  ]);
  const cached = loadedInventoryCache.get(cacheKey);
  if (cached) return cached;

  let artifact;
  try {
    artifact = JSON.parse(readFileSync(inputPath, 'utf8'));
  } catch (error) {
    throw new Error(`source inventory unreadable at ${inputPath}: ${error.message}`);
  }

  if (!artifact || typeof artifact !== 'object' || Array.isArray(artifact)) {
    throw new Error('source inventory must be a JSON object');
  }
  if (artifact.schemaVersion !== SOURCE_INVENTORY_SCHEMA_VERSION) {
    throw new Error(`source inventory has unsupported schemaVersion: ${artifact.schemaVersion}`);
  }
  if (artifact.producer !== 'triage-repo.mjs') {
    throw new Error(`source inventory has unsupported producer: ${artifact.producer}`);
  }
  validateRunId(artifact.runId);
  if (artifact.runId !== expected.runId) {
    throw new Error(`source inventory runId mismatch: expected ${expected.runId}`);
  }
  if (artifact.pathMode !== 'repo-relative') {
    throw new Error(`source inventory has unsupported pathMode: ${artifact.pathMode}`);
  }
  if (typeof artifact.root !== 'string' || !path.isAbsolute(artifact.root)) {
    throw new Error('source inventory root must be an absolute path');
  }
  if (!artifact.walkScope || typeof artifact.walkScope !== 'object') {
    throw new Error('source inventory walkScope must be an object');
  }
  if (artifact.walkScope.includeTests !== true) {
    throw new Error('source inventory walkScope must include tests');
  }
  validateStringArray(artifact.walkScope.exclude, 'walkScope.exclude');
  validateStringArray(artifact.walkScope.languages, 'walkScope.languages');
  validateSortedUnique(artifact.walkScope.languages, 'walkScope.languages');
  if (artifact.walkScope.policyVersion !== SOURCE_INVENTORY_POLICY_VERSION) {
    throw new Error(
      `source inventory has unsupported walk policy: ${artifact.walkScope.policyVersion}`,
    );
  }
  if (!artifact.analysisScope || typeof artifact.analysisScope !== 'object') {
    throw new Error('source inventory analysisScope must be an object');
  }
  if (artifact.analysisScope.includeTests !== expected.includeTests) {
    throw new Error('source inventory analysisScope.includeTests mismatch');
  }
  validateStringArray(artifact.analysisScope.exclude, 'analysisScope.exclude');
  if (
    !arraysEqual(artifact.walkScope.exclude, expected.exclude) ||
    !arraysEqual(artifact.analysisScope.exclude, expected.exclude)
  ) {
    throw new Error('source inventory exclude scope mismatch');
  }
  validateStringArray(artifact.files, 'files');
  if (!Number.isSafeInteger(artifact.fileCount) || artifact.fileCount !== artifact.files.length) {
    throw new Error('source inventory fileCount does not match files.length');
  }
  if (
    !artifact.countsByLanguage ||
    typeof artifact.countsByLanguage !== 'object' ||
    Array.isArray(artifact.countsByLanguage)
  ) {
    throw new Error('source inventory countsByLanguage must be an object');
  }

  const countLanguages = Object.keys(artifact.countsByLanguage).toSorted(compareUtf8);
  if (!arraysEqual(countLanguages, artifact.walkScope.languages)) {
    throw new Error('source inventory countsByLanguage keys must match walkScope.languages');
  }
  const expectedCounts = Object.fromEntries(
    artifact.walkScope.languages.map((language) => [language, 0]),
  );
  for (const language of artifact.walkScope.languages) {
    if (!Number.isSafeInteger(artifact.countsByLanguage[language])) {
      throw new Error(`source inventory count for ${language} must be a safe integer`);
    }
  }

  const inventoryRoot = path.resolve(artifact.root);
  const absoluteFiles = [];
  let previous = null;
  for (const file of artifact.files) {
    validateRelativePath(file, previous);
    previous = file;
    const absolute = path.resolve(inventoryRoot, ...file.split('/'));
    if (!isWithinRoot(inventoryRoot, absolute)) {
      throw new Error(`source inventory path escapes root: ${file}`);
    }
    const language = path.posix.extname(file).slice(1);
    if (!Object.hasOwn(expectedCounts, language)) {
      throw new Error(`source inventory file has unsupported language extension: ${file}`);
    }
    expectedCounts[language] += 1;
    absoluteFiles.push(absolute);
  }
  for (const language of artifact.walkScope.languages) {
    if (artifact.countsByLanguage[language] !== expectedCounts[language]) {
      throw new Error(`source inventory count mismatch for ${language}`);
    }
  }
  const loaded = { artifact, inventoryRoot, absoluteFiles };
  loadedInventoryCache.set(cacheKey, loaded);
  return loaded;
}

export function loadSourceInventory(inputPath, requestedRoot, expected) {
  const loaded = loadValidatedArtifact(inputPath, expected);
  const resolvedRequestedRoot = path.resolve(requestedRoot);
  if (!isWithinRoot(loaded.inventoryRoot, resolvedRequestedRoot)) {
    throw new Error(
      `source inventory root ${loaded.inventoryRoot} does not contain requested root ${resolvedRequestedRoot}`,
    );
  }
  return {
    artifact: loaded.artifact,
    inventoryRoot: loaded.inventoryRoot,
    requestedRoot: resolvedRequestedRoot,
    absoluteFiles: loaded.absoluteFiles.filter(
      (file) => isWithinRoot(resolvedRequestedRoot, file) && file !== resolvedRequestedRoot,
    ),
  };
}
