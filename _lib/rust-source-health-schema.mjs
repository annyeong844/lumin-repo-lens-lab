export const RUST_SOURCE_HEALTH_SCHEMA_VERSION = 1;
export const RUST_SOURCE_HEALTH_POLICY_VERSION = 'm6-rust-source-health-syntax-v1';
export const RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES = 16 * 1024 * 1024;
export const RUST_SOURCE_HEALTH_PARSER = Object.freeze({
  kind: 'ra_ap_syntax',
  version: '0.0.337',
  editionPolicy: 'fixed',
  edition: '2021',
  editionSource: 'm6-policy-default',
});

export function sortRustHealthArtifact(artifact) {
  const files = Object.fromEntries(
    Object.entries(artifact.files ?? {})
      .sort(([left], [right]) => compareCodeUnit(left, right))
      .map(([file, value]) => [
        file,
        {
          ...value,
          signals: [...(value.signals ?? [])].sort(compareSignals),
          parse: {
            ...(value.parse ?? {}),
            errors: [...(value.parse?.errors ?? [])].sort(compareParseErrors),
          },
        },
      ]),
  );
  const skippedFiles = [...(artifact.skippedFiles ?? [])]
    .sort((left, right) => compareCodeUnit(String(left.path), String(right.path)));
  return { ...artifact, skippedFiles, files };
}

function compareCodeUnit(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function compareSignals(left, right) {
  return (
    Number(left?.location?.byteStart ?? 0) -
      Number(right?.location?.byteStart ?? 0) ||
    compareCodeUnit(String(left?.kind ?? ''), String(right?.kind ?? ''))
  );
}

function compareParseErrors(left, right) {
  return (
    Number(left?.location?.byteStart ?? 0) -
      Number(right?.location?.byteStart ?? 0) ||
    compareCodeUnit(String(left?.message ?? ''), String(right?.message ?? ''))
  );
}

export function stableObject(value = {}) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => compareCodeUnit(left, right)),
  );
}

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function isSha256(value) {
  return typeof value === 'string' && /^sha256:[a-f0-9]{64}$/i.test(value);
}

function isSafeRelativeSlashPath(value) {
  return typeof value === 'string' &&
    value.length > 0 &&
    !value.startsWith('/') &&
    !value.startsWith('\\') &&
    !value.includes('\\') &&
    !/^[A-Za-z]:/.test(value) &&
    !value.split('/').some((segment) =>
      segment.length === 0 || segment === '.' || segment === '..'
    );
}

function isLocation(value) {
  return isPlainObject(value) &&
    Number.isInteger(value.line) &&
    value.line > 0 &&
    Number.isInteger(value.column) &&
    value.column > 0 &&
    Number.isInteger(value.endLine) &&
    value.endLine > 0 &&
    Number.isInteger(value.endColumn) &&
    value.endColumn > 0 &&
    Number.isInteger(value.byteStart) &&
    value.byteStart >= 0 &&
    Number.isInteger(value.byteEnd) &&
    value.byteEnd >= value.byteStart;
}

export function summarizeRustHealthArtifact(artifact = {}) {
  const safeArtifact = isPlainObject(artifact) ? artifact : {};
  const fileEntries = Object.values(
    isPlainObject(safeArtifact.files) ? safeArtifact.files : {},
  );
  const signals = fileEntries.flatMap((entry) => entry.signals ?? []);
  const parseErrors = fileEntries.flatMap((entry) => entry.parse?.errors ?? []);
  const signalsByKind = {};
  for (const signal of signals) {
    const kind = String(signal.kind ?? '');
    signalsByKind[kind] = (signalsByKind[kind] ?? 0) + 1;
  }
  return {
    files: fileEntries.length,
    skippedFiles: Array.isArray(safeArtifact.skippedFiles) ? safeArtifact.skippedFiles.length : 0,
    parseErrorFiles: fileEntries.filter((entry) => entry.parse?.ok === false).length,
    parseErrors: parseErrors.length,
    functions: fileEntries.reduce((sum, entry) => sum + Number(entry.facts?.functions ?? 0), 0),
    unsafeBlocks: fileEntries.reduce((sum, entry) => sum + Number(entry.facts?.unsafeBlocks ?? 0), 0),
    unsafeFunctions: fileEntries.reduce(
      (sum, entry) => sum + Number(entry.facts?.unsafeFunctions ?? 0),
      0,
    ),
    signals: signals.length,
    signalsByKind,
  };
}

export function rustHealthInvariantProblems(artifact) {
  const expected = summarizeRustHealthArtifact(artifact);
  const safeArtifact = isPlainObject(artifact) ? artifact : {};
  const actual = isPlainObject(safeArtifact.summary) ? safeArtifact.summary : {};
  const problems = [];
  for (const [key, value] of Object.entries(expected)) {
    if (key === 'signalsByKind') {
      if (
        JSON.stringify(stableObject(actual.signalsByKind ?? {})) !==
        JSON.stringify(stableObject(value))
      ) {
        problems.push('summary.signalsByKind mismatch');
      }
    } else if (actual[key] !== value) {
      problems.push(`summary.${key} expected ${value} but found ${actual[key]}`);
    }
  }
  return problems;
}

function isIsoTimestamp(value) {
  return typeof value === 'string' && !Number.isNaN(Date.parse(value));
}

function validateRustHealthArtifactShape(artifact, { requireWrapperMeta }) {
  const problems = [];
  if (!isPlainObject(artifact)) {
    problems.push('artifact must be an object');
    artifact = {};
  }
  if (artifact?.schemaVersion !== RUST_SOURCE_HEALTH_SCHEMA_VERSION) {
    problems.push('schemaVersion mismatch');
  }
  if (artifact?.meta?.producer !== 'rust-source-health') {
    problems.push('meta.producer mismatch');
  }
  if (artifact?.meta?.mode !== 'syntax-only') {
    problems.push('meta.mode mismatch');
  }
  if (
    !Number.isInteger(artifact?.meta?.runtime?.threadCount) ||
    artifact.meta.runtime.threadCount <= 0
  ) {
    problems.push('meta.runtime.threadCount invalid');
  }
  if (
    !Number.isInteger(artifact?.meta?.runtime?.workerStackBytes) ||
    artifact.meta.runtime.workerStackBytes < RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES
  ) {
    problems.push('meta.runtime.workerStackBytes invalid');
  }
  const limits = artifact?.meta?.limits;
  for (const limit of ['syntax-only', 'no-type-info', 'no-trait-solving', 'no-borrow-check']) {
    if (!Array.isArray(limits) || !limits.includes(limit)) {
      problems.push(`meta.limits missing ${limit}`);
    }
  }
  if (artifact?.meta?.policy?.version !== RUST_SOURCE_HEALTH_POLICY_VERSION) {
    problems.push('policy.version mismatch');
  }
  const thresholds = artifact?.meta?.policy?.thresholds;
  if (
    !Number.isInteger(thresholds?.maxFunctionLines) ||
    thresholds.maxFunctionLines <= 0
  ) {
    problems.push('policy.thresholds.maxFunctionLines invalid');
  }
  if (
    !Number.isInteger(thresholds?.maxImplLines) ||
    thresholds.maxImplLines <= 0
  ) {
    problems.push('policy.thresholds.maxImplLines invalid');
  }
  if (artifact?.meta?.parser?.kind !== RUST_SOURCE_HEALTH_PARSER.kind) {
    problems.push('parser.kind mismatch');
  }
  if (artifact?.meta?.parser?.version !== RUST_SOURCE_HEALTH_PARSER.version) {
    problems.push('parser.version mismatch');
  }
  if (artifact?.meta?.parser?.editionPolicy !== RUST_SOURCE_HEALTH_PARSER.editionPolicy) {
    problems.push('parser.editionPolicy mismatch');
  }
  if (artifact?.meta?.parser?.edition !== RUST_SOURCE_HEALTH_PARSER.edition) {
    problems.push('parser.edition mismatch');
  }
  if (artifact?.meta?.parser?.editionSource !== RUST_SOURCE_HEALTH_PARSER.editionSource) {
    problems.push('parser.editionSource mismatch');
  }
  if (requireWrapperMeta) {
    if (!isIsoTimestamp(artifact?.meta?.generated)) {
      problems.push('meta.generated invalid');
    }
    if (
      typeof artifact?.meta?.sidecar?.sourceCommit !== 'string' ||
      artifact.meta.sidecar.sourceCommit.length === 0
    ) {
      problems.push('meta.sidecar.sourceCommit missing');
    }
    if (!isSha256(artifact?.meta?.sidecar?.binarySha256)) {
      problems.push('meta.sidecar.binarySha256 invalid');
    }
    if (!isPlainObject(artifact?.meta?.input?.pathPolicy)) {
      problems.push('meta.input.pathPolicy missing');
    }
  }
  if (!isPlainObject(artifact?.files)) {
    problems.push('files must be an object');
  }
  if (!Array.isArray(artifact?.skippedFiles)) {
    problems.push('skippedFiles must be an array');
  }
  const allowedSkippedReasons = new Set(['excluded-by-path-policy', 'invalid-utf8']);
  const skippedFiles = Array.isArray(artifact?.skippedFiles)
    ? artifact.skippedFiles
    : [];
  for (const skipped of skippedFiles) {
    if (!isSafeRelativeSlashPath(skipped.path)) {
      problems.push('skippedFiles.path invalid');
      if (typeof skipped.path === 'string' && skipped.path.length > 0) {
        problems.push(`skippedFiles.${skipped.path}.path invalid`);
      }
    }
    if (!allowedSkippedReasons.has(skipped.reason)) {
      problems.push(`skippedFiles.${skipped.path ?? '<unknown>'}.reason invalid`);
    }
  }
  const allowedPathClassifications = new Set(['source', 'test', 'generated']);
  for (const [filePath, file] of Object.entries(artifact?.files ?? {})) {
    if (!isSafeRelativeSlashPath(filePath)) {
      problems.push(`files.${filePath}.path key invalid`);
    }
    if (!isSha256(file?.sha256)) {
      problems.push(`files.${filePath}.sha256 invalid`);
    }
    if (!isPlainObject(file?.facts)) {
      problems.push(`files.${filePath}.facts missing`);
    }
    for (const key of [
      'items',
      'functions',
      'maxFunctionLines',
      'unsafeBlocks',
      'unsafeFunctions',
    ]) {
      if (!Number.isInteger(file?.facts?.[key]) || file.facts[key] < 0) {
        problems.push(`files.${filePath}.facts.${key} invalid`);
      }
    }
    if (!Array.isArray(file?.signals)) {
      problems.push(`files.${filePath}.signals must be an array`);
    }
    if (!isPlainObject(file?.parse) || typeof file.parse.ok !== 'boolean') {
      problems.push(`files.${filePath}.parse invalid`);
    }
    if (!isPlainObject(file?.path) || !Array.isArray(file.path.classifications)) {
      problems.push(`files.${filePath}.path invalid`);
    } else {
      if (
        !file.path.classifications.every((classification) =>
          typeof classification === 'string' &&
            allowedPathClassifications.has(classification)
        )
      ) {
        problems.push(`files.${filePath}.path.classifications invalid`);
      }
      if (typeof file.path.suppressed !== 'boolean') {
        problems.push(`files.${filePath}.path.suppressed invalid`);
      }
    }
    const signals = Array.isArray(file?.signals) ? file.signals : [];
    for (const signal of signals) {
      if (typeof signal.kind !== 'string' || signal.kind.length === 0) {
        problems.push(`files.${filePath}.signal.kind invalid`);
      }
      if (signal.severity !== 'review') {
        problems.push(`files.${filePath}.signal.severity mismatch`);
      }
      if (signal.claim !== 'syntax-only') {
        problems.push(`files.${filePath}.signal.claim mismatch`);
      }
      if (!isLocation(signal.location)) {
        problems.push(`files.${filePath}.signal.location invalid`);
      }
    }
    if (!Array.isArray(file?.parse?.errors)) {
      problems.push(`files.${filePath}.parse.errors must be an array`);
    }
    const parseErrors = Array.isArray(file?.parse?.errors) ? file.parse.errors : [];
    if (file?.parse?.ok === true && parseErrors.length > 0) {
      problems.push(`files.${filePath}.parse.ok true with parse errors`);
    }
    if (file?.parse?.ok === false && parseErrors.length === 0) {
      problems.push(`files.${filePath}.parse.ok false without parse errors`);
    }
    for (const parseError of parseErrors) {
      if (typeof parseError.message !== 'string' || parseError.message.length === 0) {
        problems.push(`files.${filePath}.parse.error.message invalid`);
      }
      if (parseError.claim !== 'syntax-only') {
        problems.push(`files.${filePath}.parse.error.claim mismatch`);
      }
      if (!isLocation(parseError.location)) {
        problems.push(`files.${filePath}.parse.error.location invalid`);
      }
    }
  }
  problems.push(...rustHealthInvariantProblems(artifact));
  return problems;
}

export function validateRustHealthSidecarArtifact(artifact) {
  return validateRustHealthArtifactShape(artifact, { requireWrapperMeta: false });
}

export function validateRustHealthFinalArtifact(artifact) {
  return validateRustHealthArtifactShape(artifact, { requireWrapperMeta: true });
}

export const validateRustHealthArtifact = validateRustHealthFinalArtifact;
