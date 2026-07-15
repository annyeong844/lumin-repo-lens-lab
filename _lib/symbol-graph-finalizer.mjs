import { createHash } from "node:crypto";
import {
  closeSync,
  openSync,
  readFileSync,
  readSync,
  statSync,
  writeSync,
} from "node:fs";

import {
  AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION,
  auditCoreRuntimeCandidateSignature,
  runAuditCoreJsonToResultFile,
} from "./audit-core.mjs";
import {
  loadProducerArtifactCache,
  saveProducerArtifactCache,
} from "./incremental-cache-store.mjs";
import { atomicWriteFile } from "./atomic-write.mjs";

const ARTIFACT_CACHE_VERSION = 2;
const SUMMARY_PREFIX_BYTES = 64 * 1024;
const COPY_BUFFER_BYTES = 1024 * 1024;

function isJsonWhitespace(byte) {
  return byte === 0x20 || byte === 0x09 || byte === 0x0a || byte === 0x0d;
}

function locateTopLevelObjectValue(filePath, targetKey) {
  const fd = openSync(filePath, "r");
  const buffer = Buffer.allocUnsafe(SUMMARY_PREFIX_BYTES);
  let position = 0;
  let depth = 0;
  let inString = false;
  let escaped = false;
  let capturingKey = false;
  let keyBytes = [];
  let expectingKey = false;
  let pendingTarget = false;
  let targetStart = null;
  try {
    for (;;) {
      const bytesRead = readSync(fd, buffer, 0, buffer.length, position);
      if (bytesRead === 0) return null;
      for (let index = 0; index < bytesRead; index++) {
        const byte = buffer[index];
        const absolute = position + index;
        if (inString) {
          if (escaped) {
            escaped = false;
            if (capturingKey) keyBytes.push(byte);
          } else if (byte === 0x5c) {
            escaped = true;
            if (capturingKey) keyBytes.push(byte);
          } else if (byte === 0x22) {
            inString = false;
            if (capturingKey) {
              pendingTarget =
                Buffer.from(keyBytes).toString("utf8") === targetKey;
              capturingKey = false;
            }
          } else if (capturingKey) {
            keyBytes.push(byte);
          }
          continue;
        }

        if (targetStart !== null) {
          if (byte === 0x22) {
            inString = true;
          } else if (byte === 0x7b || byte === 0x5b) {
            depth++;
          } else if (byte === 0x7d || byte === 0x5d) {
            depth--;
            if (depth === 1) return { start: targetStart, end: absolute + 1 };
          }
          continue;
        }

        if (depth === 0) {
          if (isJsonWhitespace(byte)) continue;
          if (byte !== 0x7b) return null;
          depth = 1;
          expectingKey = true;
          continue;
        }

        if (depth === 1 && expectingKey) {
          if (isJsonWhitespace(byte)) continue;
          if (byte === 0x7d) return null;
          if (byte !== 0x22) return null;
          inString = true;
          capturingKey = true;
          keyBytes = [];
          expectingKey = false;
          continue;
        }

        if (depth === 1 && pendingTarget) {
          if (isJsonWhitespace(byte) || byte === 0x3a) continue;
          if (byte !== 0x7b) return null;
          targetStart = absolute;
          depth++;
          pendingTarget = false;
          continue;
        }

        if (byte === 0x22) {
          inString = true;
        } else if (byte === 0x7b || byte === 0x5b) {
          depth++;
        } else if (byte === 0x7d || byte === 0x5d) {
          depth--;
        } else if (depth === 1 && byte === 0x2c) {
          expectingKey = true;
          pendingTarget = false;
        }
      }
      position += bytesRead;
    }
  } finally {
    closeSync(fd);
  }
}

function readFileRange(filePath, start, end) {
  const buffer = Buffer.allocUnsafe(end - start);
  const fd = openSync(filePath, "r");
  let offset = 0;
  try {
    while (offset < buffer.length) {
      const bytesRead = readSync(
        fd,
        buffer,
        offset,
        buffer.length - offset,
        start + offset,
      );
      if (bytesRead === 0) {
        throw new Error("unexpected EOF while reading JSON object");
      }
      offset += bytesRead;
    }
    return buffer.toString("utf8");
  } finally {
    closeSync(fd);
  }
}

function writeAll(fd, buffer) {
  let offset = 0;
  while (offset < buffer.length) {
    offset += writeSync(fd, buffer, offset, buffer.length - offset);
  }
}

function copyFromOffset(sourceFd, targetFd, sourceOffset) {
  const buffer = Buffer.allocUnsafe(COPY_BUFFER_BYTES);
  let position = sourceOffset;
  for (;;) {
    const bytesRead = readSync(sourceFd, buffer, 0, buffer.length, position);
    if (bytesRead === 0) return;
    writeAll(targetFd, buffer.subarray(0, bytesRead));
    position += bytesRead;
  }
}

function copyByteCount(sourceFd, targetFd, byteCount) {
  const buffer = Buffer.allocUnsafe(COPY_BUFFER_BYTES);
  let position = 0;
  while (position < byteCount) {
    const bytesRead = readSync(
      sourceFd,
      buffer,
      0,
      Math.min(buffer.length, byteCount - position),
      position,
    );
    if (bytesRead === 0) {
      throw new Error("unexpected EOF while restoring artifact");
    }
    writeAll(targetFd, buffer.subarray(0, bytesRead));
    position += bytesRead;
  }
}

function restoreCachedArtifact(cacheHit, outPath, context) {
  if (cacheHit?.status !== "hit" || !cacheHit.artifactPath) {
    throw new Error(
      "symbol graph artifact restore requires a verified cache hit",
    );
  }
  const locatedMeta = locateTopLevelObjectValue(cacheHit.artifactPath, "meta");
  if (!locatedMeta) {
    throw new Error("cached symbols.json does not contain a top-level meta object");
  }
  const meta = JSON.parse(
    readFileRange(cacheHit.artifactPath, locatedMeta.start, locatedMeta.end),
  );
  if (!meta || typeof meta !== "object" || Array.isArray(meta)) {
    throw new Error("cached symbols.json meta must be an object");
  }
  meta.generated = context.generated;
  if (context.incremental == null) {
    delete meta.incremental;
  } else {
    meta.incremental = context.incremental;
  }

  const currentMeta = Buffer.from(JSON.stringify(meta), "utf8");
  atomicWriteFile(outPath, (tmpPath) => {
    const sourceFd = openSync(cacheHit.artifactPath, "r");
    let targetFd;
    try {
      targetFd = openSync(tmpPath, "w");
      copyByteCount(sourceFd, targetFd, locatedMeta.start);
      writeAll(targetFd, currentMeta);
      copyFromOffset(sourceFd, targetFd, locatedMeta.end);
    } finally {
      if (targetFd !== undefined) closeSync(targetFd);
      closeSync(sourceFd);
    }
  });
}

function readArtifactSummary(outPath, phaseTimer) {
  const summary = locateTopLevelObjectValue(outPath, "artifactSummary");
  if (summary) {
    return JSON.parse(readFileRange(outPath, summary.start, summary.end));
  }

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
  if (stableRequest.context) delete stableRequest.context.incremental;
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
        restoreCachedArtifact(lookup, outPath, request.context);
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
      statSync(outPath).size,
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
