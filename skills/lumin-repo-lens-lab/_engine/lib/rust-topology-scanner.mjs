import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";

import { MODULE_EDGE_SCANNER_POLICY_VERSION } from "./js-module-edge-scanner.mjs";

const SCHEMA_VERSION = 1;
const DEFAULT_MISMATCH_SAMPLE_LIMIT = 10;

export function normalizeScannerPath(value) {
  return String(value ?? "").replaceAll("\\", "/");
}

function boolValue(value) {
  return value === true;
}

function normalizeEdge(edge = {}) {
  return {
    source: String(edge.source ?? ""),
    line: Number.isFinite(edge.line) ? edge.line : 0,
    dynamic: boolValue(edge.dynamic),
    typeOnly: boolValue(edge.typeOnly),
    reExport: boolValue(edge.reExport),
  };
}

function edgeKey(edge) {
  return [
    edge.source,
    String(edge.line),
    edge.dynamic ? "1" : "0",
    edge.typeOnly ? "1" : "0",
    edge.reExport ? "1" : "0",
  ].join("\0");
}

function compareEdges(a, b) {
  return (
    a.source.localeCompare(b.source) ||
    a.line - b.line ||
    Number(a.dynamic) - Number(b.dynamic) ||
    Number(a.typeOnly) - Number(b.typeOnly) ||
    Number(a.reExport) - Number(b.reExport)
  );
}

function normalizeRisk(risk) {
  return [...new Set((Array.isArray(risk) ? risk : []).map(String))].sort();
}

export function normalizeScannerResult(result = {}) {
  return {
    file: normalizeScannerPath(result.file),
    ok: result.ok === true,
    loc: Number.isFinite(result.loc) ? result.loc : 0,
    edges: (Array.isArray(result.edges) ? result.edges : [])
      .map(normalizeEdge)
      .sort(compareEdges),
    risk: normalizeRisk(result.risk),
  };
}

function diffArrayByKey(left, right, keyFn) {
  const rightCounts = new Map();
  for (const item of right) {
    const key = keyFn(item);
    rightCounts.set(key, (rightCounts.get(key) ?? 0) + 1);
  }
  const only = [];
  for (const item of left) {
    const key = keyFn(item);
    const count = rightCounts.get(key) ?? 0;
    if (count > 0) rightCounts.set(key, count - 1);
    else only.push(item);
  }
  return only;
}

function compareOne(jsResult, rustResult) {
  const js = normalizeScannerResult(jsResult);
  const rust = normalizeScannerResult(rustResult);
  const jsOnlyEdges = diffArrayByKey(js.edges, rust.edges, edgeKey);
  const rustOnlyEdges = diffArrayByKey(rust.edges, js.edges, edgeKey);
  if (jsOnlyEdges.length > 0 || rustOnlyEdges.length > 0) {
    return {
      file: js.file || rust.file,
      kind: "edge-mismatch",
      jsOnly: jsOnlyEdges,
      rustOnly: rustOnlyEdges,
    };
  }

  const jsOnlyRisk = js.risk.filter((item) => !rust.risk.includes(item));
  const rustOnlyRisk = rust.risk.filter((item) => !js.risk.includes(item));
  if (jsOnlyRisk.length > 0 || rustOnlyRisk.length > 0) {
    return {
      file: js.file || rust.file,
      kind: "risk-mismatch",
      jsOnly: jsOnlyRisk,
      rustOnly: rustOnlyRisk,
    };
  }
  return null;
}

function comparisonMetadata({
  status,
  mode,
  binary,
  timeoutMs,
  elapsedMs,
  filesCompared,
  mismatches,
  mismatchSamples,
  extra = {},
}) {
  return {
    attempted: true,
    mode,
    status,
    binary,
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    filesCompared,
    mismatches,
    mismatchSamples,
    mismatchSampleLimit: DEFAULT_MISMATCH_SAMPLE_LIMIT,
    timeoutMs,
    elapsedMs,
    ...extra,
  };
}

function parseJson(value) {
  try {
    return { ok: true, value: JSON.parse(value) };
  } catch (error) {
    return { ok: false, error };
  }
}

function fileSetMismatch(jsResults, rustResults) {
  const jsFiles = new Set(jsResults.map((entry) => normalizeScannerPath(entry.file)));
  const rustFiles = new Set(
    rustResults.map((entry) => normalizeScannerPath(entry.file)),
  );
  const jsOnlyFiles = [...jsFiles].filter((file) => !rustFiles.has(file)).sort();
  const rustOnlyFiles = [...rustFiles].filter((file) => !jsFiles.has(file)).sort();
  if (jsOnlyFiles.length === 0 && rustOnlyFiles.length === 0) return null;
  return {
    kind: "count-mismatch",
    count: jsOnlyFiles.length + rustOnlyFiles.length,
    jsOnlyFiles: jsOnlyFiles.slice(0, DEFAULT_MISMATCH_SAMPLE_LIMIT),
    rustOnlyFiles: rustOnlyFiles.slice(0, DEFAULT_MISMATCH_SAMPLE_LIMIT),
  };
}

export function compareRustTopologyScanner({
  mode = "off",
  binary,
  root,
  files = [],
  jsResults = [],
  timeoutMs = 60000,
} = {}) {
  if (mode === "off") return { useRust: false, metadata: null, rustResults: [] };

  const started = Date.now();
  if (!binary || !existsSync(binary)) {
    return {
      useRust: false,
      metadata: comparisonMetadata({
        status: "binary-not-found",
        mode,
        binary,
        timeoutMs,
        elapsedMs: Date.now() - started,
        filesCompared: 0,
        mismatches: 0,
        mismatchSamples: [],
      }),
      rustResults: [],
    };
  }

  const request = JSON.stringify({
    schemaVersion: SCHEMA_VERSION,
    root,
    files,
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
  });
  const needsShell = process.platform === "win32" && /\.(cmd|bat)$/i.test(binary);
  const child = spawnSync(binary, {
    input: request,
    encoding: "utf8",
    timeout: timeoutMs,
    windowsHide: true,
    shell: needsShell,
    maxBuffer: 1024 * 1024 * 64,
  });
  const elapsedMs = Date.now() - started;

  if (child.error?.code === "ETIMEDOUT" || child.signal === "SIGTERM") {
    return {
      useRust: false,
      metadata: comparisonMetadata({
        status: "timeout",
        mode,
        binary,
        timeoutMs,
        elapsedMs,
        filesCompared: 0,
        mismatches: 0,
        mismatchSamples: [],
      }),
      rustResults: [],
    };
  }

  if (child.status !== 0) {
    return {
      useRust: false,
      metadata: comparisonMetadata({
        status: "non-zero-exit",
        mode,
        binary,
        timeoutMs,
        elapsedMs,
        filesCompared: 0,
        mismatches: 0,
        mismatchSamples: [],
        extra: { exitCode: child.status },
      }),
      rustResults: [],
    };
  }

  const parsed = parseJson(child.stdout);
  if (
    !parsed.ok ||
    parsed.value?.schemaVersion !== SCHEMA_VERSION ||
    !Array.isArray(parsed.value?.files)
  ) {
    return {
      useRust: false,
      metadata: comparisonMetadata({
        status: "invalid-json-output",
        mode,
        binary,
        timeoutMs,
        elapsedMs,
        filesCompared: 0,
        mismatches: 0,
        mismatchSamples: [],
      }),
      rustResults: [],
    };
  }

  if (parsed.value.policyVersion !== MODULE_EDGE_SCANNER_POLICY_VERSION) {
    return {
      useRust: false,
      metadata: comparisonMetadata({
        status: "invalid-json-output",
        mode,
        binary,
        timeoutMs,
        elapsedMs,
        filesCompared: 0,
        mismatches: 0,
        mismatchSamples: [],
        extra: {
          reason: "policy-version-mismatch",
          rustPolicyVersion: parsed.value.policyVersion,
        },
      }),
      rustResults: [],
    };
  }

  const rustResults = parsed.value.files;
  const countMismatch = fileSetMismatch(jsResults, rustResults);
  if (countMismatch) {
    return {
      useRust: false,
      metadata: comparisonMetadata({
        status: "count-mismatch",
        mode,
        binary,
        timeoutMs,
        elapsedMs,
        filesCompared: jsResults.length,
        mismatches: countMismatch.count,
        mismatchSamples: [countMismatch],
        extra: { sidecarTiming: parsed.value.timing ?? null },
      }),
      rustResults,
    };
  }

  const rustByFile = new Map(
    rustResults.map((entry) => [normalizeScannerPath(entry.file), entry]),
  );
  const mismatches = [];
  for (const jsResult of jsResults) {
    const file = normalizeScannerPath(jsResult.file);
    const mismatch = compareOne(jsResult, rustByFile.get(file));
    if (mismatch) mismatches.push(mismatch);
  }
  const status = mismatches.length === 0 ? "matched" : mismatches[0].kind;
  return {
    useRust: false,
    metadata: comparisonMetadata({
      status,
      mode,
      binary,
      timeoutMs,
      elapsedMs,
      filesCompared: jsResults.length,
      mismatches: mismatches.length,
      mismatchSamples: mismatches.slice(0, DEFAULT_MISMATCH_SAMPLE_LIMIT),
      extra: { sidecarTiming: parsed.value.timing ?? null },
    }),
    rustResults,
  };
}
