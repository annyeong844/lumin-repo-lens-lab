// PCEF P2: public deep-import risk blocks entry-unreachable confidence support.

import {
  getPublicDeepImportRisk,
  hasPublicDeepImportRisk,
} from '../_lib/package-exports.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

assert('D1. private package has no public deep-import risk',
  hasPublicDeepImportRisk({
    private: true,
    exports: { './*': './src/*' },
  }, 'src/internal.ts') === false);

assert('D2. publishable package without exports has deep-import risk',
  hasPublicDeepImportRisk({
    name: 'pkg',
    main: './dist/index.js',
  }, 'src/internal.ts') === true);

assert('D2b. package without a package name has no public deep-import contract',
  hasPublicDeepImportRisk({
    type: 'module',
    main: './src/index.js',
  }, 'src/internal.ts') === false);

assert('D3. root-only exports do not expose arbitrary internals',
  hasPublicDeepImportRisk({
    name: 'pkg',
    exports: {
      '.': {
        types: './dist/index.d.ts',
        import: './dist/index.mjs',
        require: './dist/index.cjs',
      },
      './package.json': './package.json',
    },
  }, 'src/internal.ts') === false);

assert('D4. wildcard exports expose matching source files',
  hasPublicDeepImportRisk({
    name: 'pkg',
    exports: { './src/*': './src/*' },
  }, 'src/internal.ts') === true);

assert('D5. conditional wildcard exports expose matching files',
  hasPublicDeepImportRisk({
    name: 'pkg',
    exports: {
      './features/*': {
        import: './src/features/*.ts',
        types: './src/features/*.d.ts',
      },
    },
  }, 'src/features/foo.ts') === true);

assert('D6. explicit file export exposes that file',
  hasPublicDeepImportRisk({
    name: 'pkg',
    exports: { './internals/foo': './src/internals/foo.ts' },
  }, 'src/internals/foo.ts') === true);

assert('D7. null export leaf blocks exposure',
  hasPublicDeepImportRisk({
    name: 'pkg',
    exports: { './internals/*': null, '.': './dist/index.js' },
  }, 'src/internals/foo.ts') === false);

assert('D8. array fallback exposes a matching leaf',
  hasPublicDeepImportRisk({
    name: 'pkg',
    exports: { './x': ['./dist/x.mjs', './dist/x.cjs'] },
  }, 'dist/x.mjs') === true);

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    main: './dist/index.js',
  }, 'src/internal.ts');
  assert('D9. deep-import risk detail explains unknown publish surface without exports',
    detail.risk === true &&
      detail.reason === 'exports-absent-publish-surface-unknown' &&
      detail.publishSurfaceSource === 'implicit-npm-surface' &&
      detail.packageName === 'pkg' &&
      detail.relFileFromPkgRoot === 'src/internal.ts',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    exports: { './src/*': './src/*' },
  }, 'src/internal.ts');
  assert('D10. deep-import risk detail explains wildcard exposure',
    detail.risk === true &&
      detail.reason === 'wildcard-exposes-file' &&
      detail.matchedExport === './src/*',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    exports: { './internals/foo': './src/internals/foo.ts' },
  }, 'src/internals/foo.ts');
  assert('D11. deep-import risk detail explains explicit exposure',
    detail.risk === true &&
      detail.reason === 'explicitly-exposed-file' &&
      detail.matchedExport === './src/internals/foo.ts',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    type: 'module',
    main: './src/index.js',
  }, 'src/internal.ts');
  assert('D12. no-name package detail stays non-risk and explains why',
    detail.risk === false &&
      detail.reason === 'package-name-absent',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist'],
  }, 'src/internal.ts');
  assert('D13. package files excluding source clears public deep-import risk',
    detail.risk === false &&
      detail.reason === 'files-excludes-file' &&
      detail.publishSurfaceSource === 'package-json-files',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src'],
  }, 'src/internal.ts');
  assert('D14. package files including source keeps public deep-import risk',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published' &&
      detail.publishSurfaceSource === 'package-json-files' &&
      detail.matchedFilesEntry === 'src',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/index.ts'],
  }, 'src/index.ts');
  assert('D15. exact package files entry includes exact file',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published' &&
      detail.matchedFilesEntry === 'src/index.ts',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/index.ts'],
  }, 'src/other.ts');
  assert('D16. exact package files entry does not include sibling file',
    detail.risk === false &&
      detail.reason === 'files-excludes-file',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: [],
  }, 'src/internal.js');
  assert('D17. empty files array excludes non-entry source file',
    detail.risk === false &&
      detail.reason === 'files-excludes-file',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    main: 'src/index.js',
    files: ['dist'],
  }, 'src/index.js');
  assert('D18. main file remains public risk even when files excludes it',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.publishSurfaceSource === 'npm-always-included' &&
      detail.matchedAlwaysIncludedRule === 'main' &&
      detail.matchedPackageJsonField === 'main',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist'],
  }, 'index.js');
  assert('D19. default main index.js remains public risk',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'default-main',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    bin: { cli: 'src/cli.js' },
    files: ['dist'],
  }, 'src/cli.js');
  assert('D20. bin file remains public risk even when files excludes it',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'bin' &&
      detail.matchedPackageJsonField === 'bin',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    directories: { bin: 'bin' },
    files: ['dist'],
  }, 'bin/tool.js');
  assert('D21. directories.bin remains public risk even when files excludes it',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'directories.bin' &&
      detail.matchedPackageJsonField === 'directories.bin',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: [],
  }, 'README.md');
  assert('D22. README variant remains public risk with empty files array',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published-always-included' &&
      detail.matchedAlwaysIncludedRule === 'readme',
    JSON.stringify(detail));
}

{
  const direct = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/*'],
  }, 'src/a.ts');
  const nested = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/*'],
  }, 'src/nested/a.ts');
  assert('D23. single star files entry matches direct child only',
    direct.risk === true &&
      direct.reason === 'exports-absent-file-published' &&
      nested.risk === false &&
      nested.reason === 'files-excludes-file',
    JSON.stringify({ direct, nested }));
}

{
  const direct = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/**/*.ts'],
  }, 'src/a.ts');
  const nested = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['src/**/*.ts'],
  }, 'src/nested/a.ts');
  assert('D24. globstar files entry matches direct and nested children',
    direct.risk === true &&
      nested.risk === true &&
      direct.reason === 'exports-absent-file-published' &&
      nested.reason === 'exports-absent-file-published',
    JSON.stringify({ direct, nested }));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist', { bad: true }],
  }, 'src/internal.ts');
  assert('D25. unsupported files entry fails closed when no inclusion is proven',
    detail.risk === true &&
      detail.reason === 'exports-absent-files-unsupported' &&
      detail.publishSurfaceSource === 'package-json-files',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['dist', { bad: true }],
  }, 'dist/index.js');
  assert('D26. supported inclusion wins before unsupported files entry fallback',
    detail.risk === true &&
      detail.reason === 'exports-absent-file-published' &&
      detail.matchedFilesEntry === 'dist',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['C:/repo/src/internal.ts'],
  }, 'src/internal.ts');
  assert('D27. drive-letter files entry fails closed',
    detail.risk === true &&
      detail.reason === 'exports-absent-files-unsupported',
    JSON.stringify(detail));
}

{
  const detail = getPublicDeepImportRisk({
    name: 'pkg',
    files: ['..\\src\\internal.ts'],
  }, 'src/internal.ts');
  assert('D28. backslash and parent traversal files entry fails closed',
    detail.risk === true &&
      detail.reason === 'exports-absent-files-unsupported',
    JSON.stringify(detail));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
