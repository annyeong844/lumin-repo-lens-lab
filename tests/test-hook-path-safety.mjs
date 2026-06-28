import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  getToolTargetPath,
  resolveAuditRoot,
  resolvePackageRoot,
  resolveWorkspaceRoot,
  safeRepoPathForToolInput,
  safeRepoPathSyntactic,
} from '../_lib/hook-path-safety.mjs';

let passed = 0;
let failed = 0;

function check(label, fn) {
  try {
    fn();
    passed++;
    console.log(`  PASS  ${label}`);
  } catch (error) {
    failed++;
    console.log(`  FAIL  ${label}\n        ${error?.message ?? error}`);
  }
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), 'lrl-hook-path-'));
  const pkg = path.join(root, 'packages', 'app');
  const src = path.join(pkg, 'src');
  mkdirSync(src, { recursive: true });
  writeFileSync(path.join(root, 'pnpm-workspace.yaml'), 'packages:\n  - packages/*\n');
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ private: true }));
  writeFileSync(path.join(pkg, 'package.json'), JSON.stringify({ name: 'app' }));
  writeFileSync(path.join(src, 'a.ts'), 'export const a = 1;\n');
  return { root, pkg, src };
}

check('HPS1. workspace root resolves from pnpm workspace marker', () => {
  const fx = fixture();
  try {
    assert.equal(resolveWorkspaceRoot(fx.src), fx.root);
    assert.equal(resolvePackageRoot(fx.src), fx.pkg);
    assert.equal(resolveAuditRoot(fx.src), path.join(fx.root, '.audit'));
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPS2. tool target extraction covers mutating file tools only', () => {
  assert.equal(getToolTargetPath('Edit', { file_path: 'src/a.ts' }), 'src/a.ts');
  assert.equal(getToolTargetPath('Write', { file_path: 'src/a.ts' }), 'src/a.ts');
  assert.equal(getToolTargetPath('MultiEdit', { file_path: 'src/a.ts' }), 'src/a.ts');
  assert.equal(getToolTargetPath('Read', { file_path: 'src/a.ts' }), null);
  assert.equal(getToolTargetPath('Edit', {}), null);
});

check('HPS3. tool input paths stay inside the workspace root', () => {
  const fx = fixture();
  try {
    const ok = safeRepoPathForToolInput(fx.pkg, 'src/a.ts');
    assert.equal(ok.ok, true, JSON.stringify(ok));
    assert.equal(ok.repoRoot, fx.root);
    assert.equal(ok.repoRel, 'packages/app/src/a.ts');
    assert.equal(ok.ext, '.ts');
    assert.equal(ok.exists, true);
    assert.equal(ok.kind, 'file');

    const escaped = safeRepoPathForToolInput(fx.pkg, '../../../outside.ts');
    assert.equal(escaped.ok, false, JSON.stringify(escaped));
    assert.equal(escaped.reason, 'outside-repo');
  } finally {
    rmSync(fx.root, { recursive: true, force: true });
  }
});

check('HPS4. syntactic repo-relative path validation does not touch disk', () => {
  assert.equal(safeRepoPathSyntactic('src/a.ts').ok, true);
  assert.equal(safeRepoPathSyntactic('src/missing.ts').ok, true);
  assert.equal(safeRepoPathSyntactic('../outside.ts').ok, false);
  assert.equal(safeRepoPathSyntactic('/abs/path.ts').ok, false);
  assert.equal(safeRepoPathSyntactic('src\\a.ts').ok, false);
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
