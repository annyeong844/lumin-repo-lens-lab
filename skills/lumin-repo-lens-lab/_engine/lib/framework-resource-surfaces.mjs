import { runAuditCoreJsonResultFile } from './audit-core.mjs';

export const FRAMEWORK_RESOURCE_SURFACE_POLICY_VERSION = 'framework-resource-surface-policy-v1';
export const FRAMEWORK_RESOURCE_SURFACE_SCHEMA_VERSION = 'framework-resource-surfaces.v1';
export const FRAMEWORK_RESOURCE_SURFACE_REQUEST_SCHEMA_VERSION =
  'lumin-framework-resource-surfaces-producer-request.v1';

export function classifyFrameworkResourceSurfaces(request = {}) {
  return runAuditCoreJsonResultFile(
    ['framework-resource-surfaces-artifact', '--input', '-'],
    'framework-resource-surfaces-artifact',
    {
      input: JSON.stringify({
        schemaVersion: FRAMEWORK_RESOURCE_SURFACE_REQUEST_SCHEMA_VERSION,
        root: request.root ?? process.cwd(),
        files: request.files ?? [],
        packageRecords: request.packageRecords ?? [],
        contentsByFile: request.contentsByFile ?? {},
      }),
    },
  );
}
