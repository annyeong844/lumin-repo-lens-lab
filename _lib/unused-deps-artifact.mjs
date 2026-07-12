import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const UNUSED_DEPS_SCHEMA_VERSION = 'unused-deps.v1';
export const UNUSED_DEPS_POLICY_VERSION = 'unused-deps-review-policy-v1';
export const UNUSED_DEPS_REQUEST_SCHEMA_VERSION =
  'lumin-unused-deps-producer-request.v1';

export function buildUnusedDepsArtifact(request = {}) {
  return runAuditCoreJsonResultFile(
    ['unused-deps-artifact', '--input', '-'],
    'build-unused-deps-artifact',
    {
      input: JSON.stringify({
        schemaVersion: UNUSED_DEPS_REQUEST_SCHEMA_VERSION,
        root: request.root,
        includeTests: request.includeTests ?? true,
        exclude: request.exclude ?? [],
        packageRecords: request.packageRecords ?? [],
        symbols: request.symbols ?? {},
      }),
    },
  );
}
