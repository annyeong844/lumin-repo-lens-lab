import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { ENTRY_SURFACE_REQUEST_SCHEMA_VERSION } from './entry-surface.mjs';

export const ENTRY_SURFACE_SCHEMA_VERSION = 'entry-surface.v1';

export function projectEntrySurfaceArtifact(facts) {
  return runAuditCoreJsonResultFile(
    ['entry-surface-artifact', '--input', '-'],
    'entry-surface-artifact',
    {
      input: JSON.stringify({
        ...facts,
        schemaVersion: facts?.schemaVersion ?? ENTRY_SURFACE_REQUEST_SCHEMA_VERSION,
      }),
    },
  );
}
