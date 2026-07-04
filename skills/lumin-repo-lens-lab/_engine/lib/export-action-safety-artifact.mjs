import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const EXPORT_ACTION_SAFETY_REQUEST_SCHEMA_VERSION = 'lumin-export-action-safety-producer-request.v1';

export function projectExportActionSafetyArtifact(facts) {
  return runAuditCoreJsonResultFile(
    ['export-action-safety-artifact', '--input', '-'],
    'export-action-safety-artifact',
    {
      input: JSON.stringify({
        ...facts,
        schemaVersion: facts?.schemaVersion ?? EXPORT_ACTION_SAFETY_REQUEST_SCHEMA_VERSION,
      }),
    },
  );
}
