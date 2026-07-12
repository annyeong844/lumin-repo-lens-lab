import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION = 'lumin-barrel-discipline-producer-request.v1';

export function projectBarrelDisciplineArtifact(facts) {
  return runAuditCoreJsonResultFile(
    ['barrel-discipline-artifact', '--input', '-'],
    'barrel-discipline-artifact',
    {
      input: JSON.stringify({
        ...facts,
        schemaVersion: facts?.schemaVersion ?? BARREL_DISCIPLINE_REQUEST_SCHEMA_VERSION,
      }),
    },
  );
}
