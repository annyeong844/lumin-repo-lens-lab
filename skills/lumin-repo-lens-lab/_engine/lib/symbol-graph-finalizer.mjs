import { createHash } from "node:crypto";
import { closeSync, openSync, readFileSync, readSync, statSync } from "node:fs";

import {
  AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION,
  auditCoreRuntimeCandidateSignature,
  runAuditCoreJsonToResultFile,
} from "./audit-core.mjs";
import {
  loadProducerArtifactCache,
  restoreProducerArtifactCache,
  saveProducerArtifactCache,
} from "./incremental-cache-store.mjs";

const ARTIFACT_CACHE_VERSION = 1;
const SUMMARY_PREFIX_BYTES = 64 * 1024;

function readFilePrefix(filePath, byteLimit) {
  const fd = openSync(filePath, "r");
  try {
    const buffer = Buffer.alloc(byteLimit);
    const bytesRead = readSync(fd, buffer, 0, buffer.length, 0);
    return buffer.toString("utf8", 0, bytesRead);
  } finally {
    closeSync(fd);
  }
}

function extractJsonObjectAfterKey(text, key) {
  const marker = `"${key}"`;
  const markerIndex = text.indexOf(marker);
  if (markerIndex < 0) return null;
  const colonIndex = text.indexOf(":", markerIndex + marker.length);
  if (colonIndex < 0) return null;
  const start = text.indexOf("{", colonIndex + 1);
  if (start < 0) return null;

  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let index = start; index < text.length; index++) {
    const ch = text[index];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === '"') {
        inString = false;
      }
      continue;
    }
    if (ch === '"') {
      inString = true;
    } else if (ch === "{") {
      depth++;
    } else if (ch === "}") {
      depth--;
      if (depth === 0) return text.slice(start, index + 1);
    }
  }
  return null;
}

function readArtifactSummary(outPath, phaseTimer) {
  const prefix = readFilePrefix(outPath, SUMMARY_PREFIX_BYTES);
  const summaryText = extractJsonObjectAfterKey(prefix, "artifactSummary");
  if (summaryText) return JSON.parse(summaryText);

  phaseTimer.setCounter("symbolGraphArtifactSummaryFullParseFallback", 1);
  const artifact = JSON.parse(readFileSync(outPath, "utf8"));
  return {
    totalUsesResolved: artifact.totalUsesResolved,
    unresolvedUses: artifact.unresolvedUses,
    uses: artifact.uses,
    resolvedInternalEdgeCount: Array.isArray(artifact.resolvedInternalEdges)
      ? artifact.resolvedInternalEdges.length
      : undefined,
    deadTotal: artifact.deadTotal,
    trulyDead: artifact.trulyDead,
    deadInProd: artifact.deadInProd,
    deadInTest: artifact.deadInTest,
    generatedConsumerBlindZoneCount: Array.isArray(
      artifact.generatedConsumerBlindZones,
    )
      ? artifact.generatedConsumerBlindZones.length
      : undefined,
  };
}

function requireArtifactSummary(summary) {
  const requiredCounts = [
    "totalUsesResolved",
    "unresolvedUses",
    "resolvedInternalEdgeCount",
    "deadTotal",
    "trulyDead",
    "deadInProd",
    "deadInTest",
    "generatedConsumerBlindZoneCount",
  ];
  for (const field of requiredCounts) {
    if (!Number.isFinite(summary?.[field])) {
      throw new Error(`symbols.json artifactSummary missing numeric ${field}`);
    }
  }
  if (!summary.uses || typeof summary.uses !== "object") {
    throw new Error("symbols.json artifactSummary missing uses object");
  }
  const requiredUseCounts = [
    "resolvedInternal",
    "resolvedGeneratedVirtual",
    "nonSourceAsset",
    "external",
    "unresolvedInternal",
    "mdxConsumers",
    "sfcScriptConsumers",
    "sfcScriptSrcReachability",
    "sfcStyleAssetReferences",
    "sfcTemplateComponentRefs",
    "sfcGlobalComponentRegistrations",
    "sfcGeneratedComponentManifests",
    "sfcFrameworkConventionComponents",
  ];
  for (const field of requiredUseCounts) {
    if (!Number.isFinite(summary.uses[field])) {
      throw new Error(
        `symbols.json artifactSummary.uses missing numeric ${field}`,
      );
    }
  }
  return summary;
}

function cacheIdentity(request, producer) {
  const stableRequest = {
    ...request,
    context: request.context ? { ...request.context } : request.context,
  };
  if (stableRequest.context) delete stableRequest.context.generated;
  const requestJson = JSON.stringify(stableRequest);
  const contract = JSON.stringify({
    cacheVersion: ARTIFACT_CACHE_VERSION,
    producerId: producer.id,
    producerVersion: producer.version,
    factSchemaVersion: producer.factSchemaVersion,
    parserIdentity: producer.parserIdentity,
    auditCoreBridgeContractVersion: AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION,
    auditCoreCandidateSignature: auditCoreRuntimeCandidateSignature(),
  });
  const hash = createHash("sha256");
  hash.update(contract, "utf8");
  hash.update("\n", "utf8");
  hash.update(requestJson, "utf8");
  return {
    identity: `sha256:${hash.digest("hex")}`,
    logicalRequestBytes: Buffer.byteLength(requestJson, "utf8"),
  };
}

function recordCacheMiss(phaseTimer, reason) {
  const counter = {
    "missing-manifest": "symbolGraphFinalizerCacheMissMissing",
    "missing-artifact": "symbolGraphFinalizerCacheMissMissing",
    "malformed-manifest": "symbolGraphFinalizerCacheMissIncompatible",
    "incompatible-manifest": "symbolGraphFinalizerCacheMissIncompatible",
    "identity-mismatch": "symbolGraphFinalizerCacheMissIdentityMismatch",
    "size-mismatch": "symbolGraphFinalizerCacheMissCorrupt",
    "hash-mismatch": "symbolGraphFinalizerCacheMissCorrupt",
    "artifact-read-failed": "symbolGraphFinalizerCacheMissCorrupt",
    "restore-failed": "symbolGraphFinalizerCacheMissRestoreFailed",
  }[reason];
  if (counter) phaseTimer.setCounter(counter, 1);
}

export function finalizeSymbolGraphArtifact({
  request,
  outPath,
  incrementalEnabled,
  cacheStore,
  producer,
  phaseTimer,
}) {
  const writeStarted = Date.now();
  const identityStarted = Date.now();
  const identity = incrementalEnabled ? cacheIdentity(request, producer) : null;
  phaseTimer.recordPhase(
    "symbol-graph-finalizer-cache-identity",
    Date.now() - identityStarted,
  );
  phaseTimer.setCounter(
    "symbolGraphArtifactLogicalRequestBytes",
    identity?.logicalRequestBytes ?? 0,
  );
  phaseTimer.setCounter(
    "symbolGraphFinalizerCacheEnabled",
    incrementalEnabled ? 1 : 0,
  );

  let lookup = { status: "miss", reason: "disabled" };
  let restored = false;
  const lookupStarted = Date.now();
  if (incrementalEnabled) {
    lookup = loadProducerArtifactCache(
      cacheStore,
      producer.id,
      identity.identity,
    );
    if (lookup.status === "hit") {
      try {
        restoreProducerArtifactCache(lookup, outPath);
        restored = true;
      } catch {
        lookup = { status: "miss", reason: "restore-failed" };
      }
    }
  }
  phaseTimer.recordPhase(
    "symbol-graph-finalizer-cache-lookup",
    Date.now() - lookupStarted,
  );
  phaseTimer.setCounter("symbolGraphFinalizerCacheHit", restored ? 1 : 0);
  phaseTimer.setCounter(
    "symbolGraphFinalizerCacheMiss",
    incrementalEnabled && !restored ? 1 : 0,
  );
  if (incrementalEnabled && !restored) {
    recordCacheMiss(phaseTimer, lookup.reason);
  }

  if (restored) {
    phaseTimer.setCounter(
      "symbolGraphFinalizerCacheRestoredBytes",
      lookup.artifactBytes,
    );
    phaseTimer.setCounter("symbolGraphArtifactRequestBytes", 0);
    phaseTimer.recordPhase("symbol-graph-artifact-request-json", 0);
    phaseTimer.recordPhase("symbol-graph-artifact-request-write", 0);
    phaseTimer.recordPhase("symbol-graph-artifact-command", 0);
  } else {
    const requestJsonStarted = Date.now();
    const requestJson = JSON.stringify(request);
    phaseTimer.recordPhase(
      "symbol-graph-artifact-request-json",
      Date.now() - requestJsonStarted,
    );
    phaseTimer.setCounter(
      "symbolGraphArtifactRequestBytes",
      Buffer.byteLength(requestJson, "utf8"),
    );
    phaseTimer.recordPhase("symbol-graph-artifact-request-write", 0);
    const commandStarted = Date.now();
    try {
      runAuditCoreJsonToResultFile(
        ["symbol-graph-artifact", "--input", "-"],
        "symbol-graph-artifact",
        outPath,
        { input: requestJson },
      );
    } finally {
      phaseTimer.recordPhase(
        "symbol-graph-artifact-command",
        Date.now() - commandStarted,
      );
    }

    if (incrementalEnabled) {
      const storeStarted = Date.now();
      try {
        const stored = saveProducerArtifactCache(cacheStore, producer.id, {
          requestIdentity: identity.identity,
          artifactPath: outPath,
        });
        phaseTimer.setCounter("symbolGraphFinalizerCacheStored", 1);
        phaseTimer.setCounter(
          "symbolGraphFinalizerCacheStoredBytes",
          stored.artifactBytes,
        );
        phaseTimer.setCounter(
          "symbolGraphFinalizerCacheCleanupFailed",
          stored.cleanupFailures,
        );
      } catch (error) {
        phaseTimer.setCounter("symbolGraphFinalizerCacheStoreFailed", 1);
        console.error(
          `[symbols-incremental] finalizer artifact cache store failed: ${error.message}`,
        );
      } finally {
        phaseTimer.recordPhase(
          "symbol-graph-finalizer-cache-store",
          Date.now() - storeStarted,
        );
      }
    }
  }

  phaseTimer.setCounter("symbolsJsonBytes", statSync(outPath).size);
  const summary = requireArtifactSummary(
    readArtifactSummary(outPath, phaseTimer),
  );
  phaseTimer.recordPhase("write-artifact", Date.now() - writeStarted);
  return summary;
}
