// Tests for _lib/pre-write-lookup-dep.mjs — P1-2 step 5.2.
//
// Pinning rules from docs/history/phases/p1/p1-2.md §4.2 + §5.2:
//   - DEPENDENCY_AVAILABLE / DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS /
//     DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE / NEW_PACKAGE.
//   - existingImports = { examples, observedImportCount, countConfidence }.
//     countConfidence 'sample-only' NEVER triggers Watch-for eligibility.
//   - packageRoot() normalizes scoped / bare / subpath; excludes relative/absolute.
//   - "unused" / "cleanup" NEVER appear in rendered output.
//   - Rendered output is the render module's concern; the lookup here
//     just returns structured data that avoids those words by construction.

import {
  lookupDependency,
  packageRoot,
  isWatchForEligible,
} from '../_lib/pre-write-lookup-dep.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── packageRoot() normalization ──────────────────────────────

assert('PR1. bare package → itself',
  packageRoot('dayjs') === 'dayjs');
assert('PR2. bare package subpath → root',
  packageRoot('dayjs/plugin/utc') === 'dayjs');
assert('PR3. scoped bare → itself',
  packageRoot('@scope/pkg') === '@scope/pkg');
assert('PR4. scoped subpath → two-segment root',
  packageRoot('@scope/pkg/sub/path') === '@scope/pkg');
assert('PR5. relative path → null',
  packageRoot('./relative') === null);
assert('PR6. parent-relative → null',
  packageRoot('../up/mod') === null);
assert('PR7. absolute path → null',
  packageRoot('/abs/path') === null);
assert('PR8. malformed scoped (no slash) → null',
  packageRoot('@malformed') === null);
assert('PR9. empty → null',
  packageRoot('') === null);
assert('PR10. null → null',
  packageRoot(null) === null);

// ── Fixture helpers ─────────────────────────────────────────

function buildPkg({ dependencies = {}, devDependencies = {}, peerDependencies = {} } = {}) {
  return { dependencies, devDependencies, peerDependencies };
}

function buildSymbols({ dependencyImportConsumers = [] } = {}) {
  return {
    meta: { supports: { dependencyImportConsumers: true } },
    uses: {
      resolvedInternal: 0,
      external: dependencyImportConsumers.length,
      unresolvedInternal: 0,
      mdxConsumers: 0,
      unresolvedInternalRatio: 0,
    },
    dependencyImportConsumers,
  };
}

function buildLegacySymbols({ uses = [] } = {}) {
  return { uses };
}

// ═══ DEPENDENCY_AVAILABLE with observed consumers ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1.0.0' } });
  const sym = buildSymbols({
    dependencyImportConsumers: [
      { file: 'src/a.ts', fromSpec: 'dayjs', kind: 'import' },
      { file: 'src/b.ts', fromSpec: 'dayjs/plugin/utc', kind: 'import' },
    ],
  });
  const r = lookupDependency('dayjs', { packageJson: pkg, symbols: sym });
  assert('T1. dependency + ≥1 consumer → DEPENDENCY_AVAILABLE',
    r.result === 'DEPENDENCY_AVAILABLE', `result=${r.result}`);
  assert('T1b. declaredIn = dependencies',
    r.declaredIn === 'dependencies');
  assert('T1c. examples list contains both consumer files',
    r.existingImports.examples.length === 2);
  assert('T1d. observedImportCount = 2, countConfidence = grounded',
    r.existingImports.observedImportCount === 2 &&
    r.existingImports.countConfidence === 'grounded');
  assert('T1e. grounded citation uses dependencyImportConsumers field',
    r.citations.some((c) => /symbols\.json\.dependencyImportConsumers/.test(c)),
    JSON.stringify(r.citations));
}

// ═══ DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS ═══

{
  const pkg = buildPkg({ devDependencies: { eslint: '9.0.0' } });
  const sym = buildSymbols();  // no uses
  const r = lookupDependency('eslint', { packageJson: pkg, symbols: sym });
  assert('T2. declared but no observed imports → DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS',
    r.result === 'DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS');
  assert('T2b. declaredIn = devDependencies',
    r.declaredIn === 'devDependencies');
  assert('T2c. observedImportCount = 0',
    r.existingImports.observedImportCount === 0);
  assert('T2d. citation carries [확인 불가] + "import graph only"',
    r.citations.some((c) => /확인 불가/.test(c) && /import graph/i.test(c)));
  assert('T2e. citation NEVER uses the word "unused"',
    !r.citations.some((c) => /\bunused\b/i.test(c)));
  assert('T2f. citation NEVER uses the word "cleanup"',
    !r.citations.some((c) => /\bcleanup\b/i.test(c)));
}

// ═══ Declared dependency with unavailable import graph ═══

{
  const pkg = buildPkg({ dependencies: { eslint: '9.0.0' } });
  const r = lookupDependency('eslint', { packageJson: pkg, symbols: null });
  assert('T2g. declared + symbols absent → DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE',
    r.result === 'DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE',
    `result=${r.result}`);
  assert('T2h. unavailable import graph is not reported as observedImportCount=0',
    r.existingImports.observedImportCount === null &&
    r.existingImports.countConfidence === 'unavailable',
    JSON.stringify(r.existingImports));
  assert('T2i. unavailable import graph carries [확인 불가] citation',
    r.citations.some((c) => /확인 불가/.test(c) && /symbols\.json absent/.test(c)),
    JSON.stringify(r.citations));
}

{
  const pkg = buildPkg({ dependencies: { eslint: '9.0.0' } });
  const r = lookupDependency('eslint', { packageJson: pkg, symbols: {} });
  assert('T2j. symbols without uses[] also means import graph unavailable',
    r.result === 'DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE' &&
    r.existingImports.countConfidence === 'unavailable',
    JSON.stringify(r));
}

// ═══ peerDependencies ═══

{
  const pkg = buildPkg({ peerDependencies: { react: '>=18' } });
  const sym = buildSymbols({
    dependencyImportConsumers: [{ file: 'src/app.tsx', fromSpec: 'react', kind: 'import' }],
  });
  const r = lookupDependency('react', { packageJson: pkg, symbols: sym });
  assert('T3. peerDependency + consumer → DEPENDENCY_AVAILABLE',
    r.result === 'DEPENDENCY_AVAILABLE');
  assert('T3b. declaredIn = peerDependencies',
    r.declaredIn === 'peerDependencies');
}

// ═══ NEW_PACKAGE ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1.0.0' } });
  const sym = buildSymbols();
  const r = lookupDependency('axios', { packageJson: pkg, symbols: sym });
  assert('T4. absent from all declarations → NEW_PACKAGE',
    r.result === 'NEW_PACKAGE');
  assert('T4b. declaredIn = null',
    r.declaredIn === null);
  assert('T4c. citation mentions package.json absence',
    r.citations.some((c) => /package\.json/.test(c)));
}

// ═══ Scoped package ═══

{
  const pkg = buildPkg({ dependencies: { '@anthropic/sdk': '0.1.0' } });
  const sym = buildSymbols({
    dependencyImportConsumers: [{ file: 'src/ai.ts', fromSpec: '@anthropic/sdk/client', kind: 'import' }],
  });
  const r = lookupDependency('@anthropic/sdk', { packageJson: pkg, symbols: sym });
  assert('T5. scoped package with subpath consumer → DEPENDENCY_AVAILABLE',
    r.result === 'DEPENDENCY_AVAILABLE');
  assert('T5b. subpath consumer normalized under scoped root',
    r.existingImports.examples.length === 1 &&
    r.existingImports.examples[0].fromSpec === '@anthropic/sdk/client');
}

// ═══ Subpath lookup normalizes to root ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1.0.0' } });
  const sym = buildSymbols({
    dependencyImportConsumers: [
      { file: 'src/a.ts', fromSpec: 'dayjs/plugin/utc', kind: 'import' },
    ],
  });
  // Caller passes a subpath as depName; lookup normalizes.
  const r = lookupDependency('dayjs/plugin/utc', { packageJson: pkg, symbols: sym });
  assert('T6. subpath depName matches root declaration',
    r.result === 'DEPENDENCY_AVAILABLE');
}

// ═══ Relative / absolute specifiers excluded from consumer matching ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1.0.0' } });
  const sym = buildSymbols({
    dependencyImportConsumers: [
      { file: 'src/a.ts', fromSpec: './dayjs', kind: 'import' },  // NOT a dayjs consumer
      { file: 'src/b.ts', fromSpec: 'dayjs', kind: 'import' },    // real consumer
    ],
  });
  const r = lookupDependency('dayjs', { packageJson: pkg, symbols: sym });
  assert('T7. relative specifier fromSpec excluded from consumer matching',
    r.existingImports.observedImportCount === 1);
}

// ═══ Sample capping ═══

{
  const pkg = buildPkg({ dependencies: { lodash: '4' } });
  const uses = [];
  for (let i = 0; i < 12; i++) {
    uses.push({ file: `src/f${i}.ts`, fromSpec: 'lodash', kind: 'import' });
  }
  const sym = buildSymbols({ dependencyImportConsumers: uses });
  const r = lookupDependency('lodash', { packageJson: pkg, symbols: sym });
  assert('T8. examples capped at 5',
    r.existingImports.examples.length <= 5);
  assert('T8b. observedImportCount = 12 (true total)',
    r.existingImports.observedImportCount === 12);
  assert('T8c. countConfidence = grounded (full scan)',
    r.existingImports.countConfidence === 'grounded');
}

// ═══ Watch-for eligibility threshold (PINNING) ═══

{
  // High count + grounded confidence → eligible
  const r1 = isWatchForEligible({
    examples: [], observedImportCount: 20, countConfidence: 'grounded',
  });
  assert('T9. grounded + >= threshold → Watch-for eligible',
    r1 === true);

  // Same count, but confidence sample-only → NOT eligible.
  const r2 = isWatchForEligible({
    examples: new Array(20), observedImportCount: null, countConfidence: 'sample-only',
  });
  assert('T10. sample-only confidence NEVER Watch-for eligible (even with high examples.length)',
    r2 === false);

  // Grounded but below threshold → not eligible.
  const r3 = isWatchForEligible({
    examples: [], observedImportCount: 3, countConfidence: 'grounded',
  });
  assert('T11. grounded + below threshold → not eligible',
    r3 === false);
}

// ═══ kind discriminator ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1' } });
  const sym = buildSymbols();
  const r = lookupDependency('dayjs', { packageJson: pkg, symbols: sym });
  assert('T12. result carries kind:"dependency"',
    r.kind === 'dependency');
  assert('T12b. depName preserved',
    r.depName === 'dayjs');
}

// ═══ Citations grounded when declared ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1.0.0' } });
  const sym = buildSymbols({
    dependencyImportConsumers: [{ file: 'src/a.ts', fromSpec: 'dayjs', kind: 'import' }],
  });
  const r = lookupDependency('dayjs', { packageJson: pkg, symbols: sym });
  assert('T13. DEPENDENCY_AVAILABLE carries [grounded, package.json ...] citation',
    r.citations.some((c) => /\[grounded.*package\.json/.test(c)));
}

// ═══ Legacy symbols.uses[] fallback ═══

{
  const pkg = buildPkg({ dependencies: { dayjs: '1.0.0' } });
  const sym = buildLegacySymbols({
    uses: [{ file: 'src/legacy.ts', fromSpec: 'dayjs', kind: 'import' }],
  });
  const r = lookupDependency('dayjs', { packageJson: pkg, symbols: sym });
  assert('T14. legacy symbols.uses[] shape remains supported',
    r.result === 'DEPENDENCY_AVAILABLE' &&
    r.existingImports.observedImportCount === 1,
    JSON.stringify(r));
  assert('T14b. legacy citation names symbols.json.uses',
    r.citations.some((c) => /symbols\.json\.uses/.test(c)),
    JSON.stringify(r.citations));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
