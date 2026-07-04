// Resolver capability and diagnostics artifact bridge.
//
// JS owns compatibility and artifact I/O. lumin-audit-core owns deterministic
// capability metadata and diagnostics projection from symbols.json facts.

import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const RESOLVER_CAPABILITIES_SCHEMA_VERSION = 'resolver-capabilities.v1';
export const RESOLVER_DIAGNOSTICS_SCHEMA_VERSION = 'resolver-diagnostics.v1';
export const RESOLVER_VERSION = 'resolver-2026-05-v1';
export const RESOLVER_DIAGNOSTICS_REQUEST_SCHEMA_VERSION =
  'lumin-resolver-diagnostics-producer-request.v1';

export function buildResolverDiagnosticsArtifacts(symbolsData, {
  capabilityArtifact = 'resolver-capabilities.json',
} = {}) {
  return runAuditCoreJsonResultFile(
    ['resolver-diagnostics-artifacts', '--input', '-'],
    'resolver-diagnostics-artifacts',
    {
      input: JSON.stringify({
        schemaVersion: RESOLVER_DIAGNOSTICS_REQUEST_SCHEMA_VERSION,
        symbols: symbolsData ?? {},
        capabilityArtifact,
      }),
    }
  );
}

export function buildResolverCapabilitiesArtifact() {
  return buildResolverDiagnosticsArtifacts({}).capabilities;
}

export function buildResolverDiagnosticsArtifact(symbolsData, options = {}) {
  return buildResolverDiagnosticsArtifacts(symbolsData, options).diagnostics;
}
