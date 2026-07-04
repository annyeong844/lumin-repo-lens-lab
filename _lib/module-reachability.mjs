// PCEF P2 module reachability artifact.
//
// JS owns compatibility and artifact I/O. lumin-audit-core owns deterministic
// graph projection from symbols.json and entry-surface.json facts.

import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION =
  'lumin-module-reachability-producer-request.v1';

export function buildModuleReachabilityArtifact(request) {
  return runAuditCoreJsonResultFile(
    ['module-reachability-artifact', '--input', '-'],
    'module-reachability-artifact',
    {
      input: JSON.stringify({
        schemaVersion: MODULE_REACHABILITY_REQUEST_SCHEMA_VERSION,
        root: request.root ?? process.cwd(),
        generated: request.generated ?? new Date().toISOString(),
        symbols: request.symbolsData ?? request.symbols ?? {},
        entrySurface: request.entrySurface ?? {},
        maxFilesVisited: request.maxFilesVisited,
        maxEdgesVisited: request.maxEdgesVisited,
      }),
    }
  );
}
