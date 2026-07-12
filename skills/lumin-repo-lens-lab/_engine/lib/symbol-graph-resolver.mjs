import path from "node:path";

import { relPath } from "./paths.mjs";
import {
  explainUnresolvedSpecifier,
  isGeneratedVirtualResolution,
  isNonSourceAssetResolution,
  makeResolver,
} from "./resolver-core.mjs";
import { resolvePythonImport } from "./python.mjs";
import { resolveGoImport } from "./tree-sitter-langs.mjs";

function increment(map, key) {
  map.set(key, (map.get(key) ?? 0) + 1);
}

function counterSuffix(value) {
  const text = String(value ?? "unknown")
    .replace(/[^A-Za-z0-9]+/g, " ")
    .trim();
  if (!text) return "Unknown";
  return text
    .split(/\s+/)
    .map((part) => `${part[0].toUpperCase()}${part.slice(1)}`)
    .join("");
}

function languageBucket(filePath) {
  if (/\.(?:astro|svelte|vue)$/i.test(filePath)) return "Sfc";
  if (/\.(?:md|mdx)$/i.test(filePath)) return "Mdx";
  if (/\.(?:[cm]?[jt]sx?)$/i.test(filePath)) return "JsTs";
  if (/\.py$/i.test(filePath)) return "Python";
  if (/\.go$/i.test(filePath)) return "Go";
  return "Other";
}

function outcomeBucket(target) {
  if (target === "EXTERNAL") return "external";
  if (target === "UNRESOLVED_INTERNAL") return "unresolved-internal";
  if (isGeneratedVirtualResolution(target)) return "generated-virtual";
  if (isNonSourceAssetResolution(target)) return "non-source-asset";
  if (typeof target === "string" && target.length > 0) return "resolved";
  return "unresolved-relative";
}

function isRustResolvedRelativeUse(use) {
  return (
    typeof use === "object" &&
    use?.resolverStage === "relative" &&
    typeof use.resolvedFile === "string" &&
    use.resolvedFile.length > 0
  );
}

function looksLikeNonSourceAssetSpecifier(spec) {
  const stripped = String(spec ?? "").split(/[?#]/, 1)[0];
  const fileName = stripped.split("/").at(-1) ?? stripped;
  const dot = fileName.lastIndexOf(".");
  if (dot <= 0 || dot === fileName.length - 1) return false;
  return !/\.(?:ts|tsx|js|jsx|mjs|cjs|mts|cts|d\.ts|d\.mts|d\.cts)$/i.test(
    stripped,
  );
}

export function createSymbolGraphResolver({
  root,
  aliasMap,
  sourceFiles,
  goModule,
}) {
  const raw = makeResolver(root, aliasMap, { sourceFiles });
  const memoBefore = typeof raw.memoStats === "function" ? raw.memoStats() : null;
  const stageBefore = typeof raw.stageStats === "function" ? raw.stageStats() : null;
  const languageCounts = new Map();
  const outcomeCounts = new Map();
  const laneCounts = new Map();
  const unresolvedCache = new Map();
  const externalFastPathCache = new Map();
  let callCount = 0;
  let rawJsCallCount = 0;
  let unresolvedCacheHits = 0;
  let unresolvedCacheMisses = 0;
  let externalFastPathCacheHits = 0;
  let externalFastPathCacheMisses = 0;

  function resolve(from, use, lane = "source-use") {
    callCount++;
    increment(languageCounts, languageBucket(from));
    increment(laneCounts, lane);
    const spec = typeof use === "string" ? use : use.fromSpec;
    let target;
    if (from.endsWith(".py")) {
      const isFromImport = typeof use === "object" ? !!use.pyIsFromImport : false;
      const level = typeof use === "object" ? (use.pyLevel ?? 0) : 0;
      const names =
        typeof use === "object" && use.name && use.name !== "*"
          ? [use.name]
          : [];
      target = resolvePythonImport(
        root,
        from,
        spec,
        isFromImport,
        names,
        level,
      )[0] ?? null;
    } else if (from.endsWith(".go")) {
      target = resolveGoImport(root, goModule, spec)[0] ?? null;
    } else if (isRustResolvedRelativeUse(use)) {
      target = use.resolvedFile;
      increment(outcomeCounts, "rust-resolved-relative");
      return target;
    } else {
      rawJsCallCount++;
      target = raw(from, spec);
    }
    increment(outcomeCounts, outcomeBucket(target));
    return target;
  }

  function canFastPathExternal(consumerFile, use) {
    if (typeof use?.fromSpec !== "string" || use.fromSpec.length === 0) {
      return false;
    }
    if (consumerFile.endsWith(".py") || consumerFile.endsWith(".go")) return false;
    if (isRustResolvedRelativeUse(use) || use.kind === "import-meta-glob") return false;
    if (
      use.fromSpec.startsWith(".") ||
      use.fromSpec.startsWith("/") ||
      use.fromSpec.startsWith("#") ||
      use.fromSpec.includes("?") ||
      looksLikeNonSourceAssetSpecifier(use.fromSpec)
    ) {
      return false;
    }
    if (typeof raw.canFastPathExternal !== "function") return false;
    const key = `${consumerFile}\0${use.fromSpec}`;
    if (externalFastPathCache.has(key)) {
      externalFastPathCacheHits++;
      return externalFastPathCache.get(key);
    }
    externalFastPathCacheMisses++;
    const result = raw.canFastPathExternal(consumerFile, use.fromSpec);
    externalFastPathCache.set(key, result);
    return result;
  }

  function unresolvedEvidence(consumerFile, use) {
    const spec = typeof use === "string" ? use : use.fromSpec;
    const key = `${consumerFile}\0${spec}`;
    let explanation = unresolvedCache.get(key);
    if (explanation !== undefined) {
      unresolvedCacheHits++;
    } else {
      unresolvedCacheMisses++;
      explanation = explainUnresolvedSpecifier(
        root,
        aliasMap,
        consumerFile,
        spec,
      ) ?? {};
      unresolvedCache.set(key, explanation);
    }
    if (typeof use !== "object") return { ...explanation };
    return {
      ...explanation,
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
              root,
              path.resolve(path.dirname(consumerFile), use.affectedDir),
            ),
          }
        : {}),
    };
  }

  function recordTelemetry(phaseTimer) {
    const memoAfter = typeof raw.memoStats === "function" ? raw.memoStats() : null;
    const stageAfter = typeof raw.stageStats === "function" ? raw.stageStats() : null;
    phaseTimer.setCounter("sourceUseResolverCallCountFinal", callCount);
    phaseTimer.setCounter("sourceUseResolverRawJsCallCountFinal", rawJsCallCount);
    phaseTimer.setCounter("sourceUseUnresolvedExplanationCacheHits", unresolvedCacheHits);
    phaseTimer.setCounter("sourceUseUnresolvedExplanationCacheMisses", unresolvedCacheMisses);
    phaseTimer.setCounter("sourceUseUnresolvedExplanationCacheSize", unresolvedCache.size);
    phaseTimer.setCounter("sourceUseExternalFastPathCacheHits", externalFastPathCacheHits);
    phaseTimer.setCounter("sourceUseExternalFastPathCacheMisses", externalFastPathCacheMisses);
    phaseTimer.setCounter("sourceUseExternalFastPathCacheSize", externalFastPathCache.size);
    for (const [language, count] of languageCounts) {
      phaseTimer.setCounter(
        `sourceUseResolverLanguage${counterSuffix(language)}CallCount`,
        count,
      );
    }
    for (const [outcome, count] of outcomeCounts) {
      phaseTimer.setCounter(
        `sourceUseResolverOutcome${counterSuffix(outcome)}Count`,
        count,
      );
    }
    for (const [lane, count] of laneCounts) {
      phaseTimer.setCounter(
        `sourceUseResolverLane${counterSuffix(lane)}CallCount`,
        count,
      );
    }
    if (memoBefore && memoAfter) {
      phaseTimer.setCounter("sourceUseResolverMemoHits", memoAfter.hits - memoBefore.hits);
      phaseTimer.setCounter("sourceUseResolverMemoMisses", memoAfter.misses - memoBefore.misses);
      phaseTimer.setCounter("sourceUseResolverMemoSize", memoAfter.size);
      phaseTimer.setCounter("symbolResolverMemoHits", memoAfter.hits);
      phaseTimer.setCounter("symbolResolverMemoMisses", memoAfter.misses);
      phaseTimer.setCounter("symbolResolverMemoSize", memoAfter.size);
    }
    if (stageBefore && stageAfter) {
      const fields = [
        ["PatternMatches", "patternMatches"],
        ["ProbeHits", "probeHits"],
        ["ProbeMisses", "probeMisses"],
        ["FallbackHits", "fallbackHits"],
        ["UnresolvedInternalResults", "unresolvedInternalResults"],
      ];
      for (const [stageName, after] of Object.entries(stageAfter)) {
        const before = stageBefore[stageName] ?? {};
        const stem = `${stageName[0].toUpperCase()}${stageName.slice(1)}`;
        for (const [suffix, key] of [
          ["Attempts", "attempts"],
          ["Results", "terminalResults"],
          ["Count", "count"],
          ["CacheHits", "cacheHits"],
          ["CacheMisses", "cacheMisses"],
          ...fields,
          ["Ms", "wallMs"],
        ]) {
          phaseTimer.setCounter(
            `sourceUseResolverStage${stem}${suffix}`,
            (after[key] ?? 0) - (before[key] ?? 0),
          );
        }
      }
    }
  }

  return {
    canFastPathExternal,
    get callCount() {
      return callCount;
    },
    get rawJsCallCount() {
      return rawJsCallCount;
    },
    languageBucket,
    recordTelemetry,
    resolve,
    unresolvedEvidence,
  };
}
