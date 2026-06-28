import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

function readRepoFile(relPath) {
  return readFileSync(path.join(ROOT, relPath), 'utf8');
}

describe('maintainer scripts guards', () => {
  it('run-syntax-check reports child process spawn errors explicitly', () => {
    const source = readRepoFile('scripts/run-syntax-check.mjs');

    expect(source).toContain('if (result.error)');
    expect(source).toContain('failed to start node --check');
    expect(source).toContain('result.error.message');
  });

  it('run-tests reports child process spawn errors explicitly', () => {
    const source = readRepoFile('scripts/run-tests.mjs');

    expect(source).toContain('if (result.error)');
    expect(source).toContain('failed to start test suite');
    expect(source).toContain('result.error.message');
  });

  it('publish-public-plugin uses try/catch optional JSON reads', () => {
    const source = readRepoFile('scripts/publish-public-plugin.mjs');
    const packageSurfaceStart = source.indexOf('function validatePackageSurface');
    const gitChangesStart = source.indexOf('function hasActualGitChanges');

    expect(packageSurfaceStart).toBeGreaterThanOrEqual(0);
    expect(gitChangesStart).toBeGreaterThan(packageSurfaceStart);

    const packageSurface = source.slice(packageSurfaceStart, gitChangesStart);

    expect(source).toContain('function readOptionalJson');
    expect(source).toContain("if (error?.code === 'ENOENT') return null;");
    expect(packageSurface).not.toContain(
      "existsSync(path.join(checkoutDir, 'skills/lumin-repo-lens-lab/package-lock.json'))\n    ? readJson",
    );
  });
});
