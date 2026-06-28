// WT-23 P1: nested local service operations should be indexed as review-only
// pre-write evidence without contaminating export, class-method, or formal
// lookup-name lanes.

import { execFileSync } from 'node:child_process';
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { lookupName } from '../_lib/pre-write-lookup-name.mjs';

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

const REPO = process.cwd();

function runNode(args, cwd = REPO) {
  return execFileSync(process.execPath, args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function writeRepositoryFixture(root) {
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(
    path.join(root, 'package.json'),
    JSON.stringify({ private: true, type: 'module' }, null, 2),
  );
  writeFileSync(
    path.join(root, 'src', 'repository.ts'),
    [
      'export function createRepository(db: any) {',
      '  function getWorld(id: string) {',
      '    return db.world.find(id);',
      '  }',
      '',
      '  const listLibraryDocs = async (worldId: string) => {',
      '    return db.docs.list(worldId);',
      '  };',
      '',
      '  function deleteWorld(id: string) {',
      '    return db.world.delete(id);',
      '  }',
      '',
      '  function normalizeInput(value: string) {',
      '    return value.trim();',
      '  }',
      '',
      '  return { getWorld, listLibraryDocs, deleteWorld, normalizeInput };',
      '}',
      '',
    ].join('\n'),
  );
}

function readSymbolsAfterBuild(root) {
  const out = path.join(root, '.audit');
  runNode([
    'build-symbol-graph.mjs',
    '--root',
    root,
    '--output',
    out,
    '--no-incremental',
  ]);
  return JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
}

{
  const root = mkdtempSync(
    path.join(os.tmpdir(), 'lrl-local-operation-index-'),
  );
  try {
    writeRepositoryFixture(root);
    const symbols = readSymbolsAfterBuild(root);
    const surface = symbols.preWriteLocalOperationIndex;
    const entries = surface?.byOwnerFile?.['src/repository.ts'] ?? [];
    const byName = Object.fromEntries(
      entries.map((entry) => [entry.name, entry]),
    );

    assert(
      'LO1. symbols advertises nested local operation index support',
      symbols.meta?.supports?.nestedLocalOperationIndex === true &&
        surface?.schemaVersion === 'pre-write-local-operations.v1' &&
        surface?.status === 'complete',
      JSON.stringify(
        {
          supports: symbols.meta?.supports,
          surface,
        },
        null,
        2,
      ),
    );
    assert(
      'LO2. read/query nested operations are indexed with container identity',
      byName.getWorld?.identity ===
        'src/repository.ts::createRepository#getWorld' &&
        byName.getWorld?.containerName === 'createRepository' &&
        byName.getWorld?.containerKind === 'function-declaration' &&
        byName.getWorld?.matchedField === 'preWriteLocalOperationIndex' &&
        byName.getWorld?.operationFamily === 'read-query' &&
        byName.getWorld?.domainTokens?.includes('world') &&
        byName.getWorld?.eligibleForDeadExportRanking === false &&
        byName.getWorld?.eligibleForSafeFix === false,
      JSON.stringify(entries, null, 2),
    );
    assert(
      'LO3. const arrow read/query operations are indexed with a closed container kind',
      byName.listLibraryDocs?.identity ===
        'src/repository.ts::createRepository#listLibraryDocs' &&
        byName.listLibraryDocs?.containerKind === 'function-declaration' &&
        byName.listLibraryDocs?.domainTokens?.includes('library') &&
        byName.listLibraryDocs?.domainTokens?.includes('docs'),
      JSON.stringify(entries, null, 2),
    );
    assert(
      'LO4. mutation and generic helpers are excluded from the v1 local operation surface',
      !byName.deleteWorld && !byName.normalizeInput,
      JSON.stringify(entries, null, 2),
    );
    assert(
      'LO5. local operations do not contaminate export defIndex or classMethodIndex',
      symbols.defIndex?.['src/repository.ts']?.createRepository &&
        !symbols.defIndex?.['src/repository.ts']?.getWorld &&
        !symbols.defIndex?.['src/repository.ts']?.listLibraryDocs &&
        !symbols.classMethodIndex?.['src/repository.ts']?.getWorld,
      JSON.stringify(
        {
          defIndex: symbols.defIndex?.['src/repository.ts'],
          classMethodIndex: symbols.classMethodIndex?.['src/repository.ts'],
        },
        null,
        2,
      ),
    );

    const lookup = lookupName('searchWorld', {
      symbols,
      canonicalClaims: [],
      intentDeclaration: {
        name: 'searchWorld',
        kind: 'function',
        why: 'search world data from the repository',
        ownerFile: 'src/repository.ts',
      },
    });
    assert(
      'LO6. artifact-only P1 does not leak local operations into formal lookup lanes',
      !lookup.nearNames?.some((entry) => entry.name === 'getWorld') &&
        !lookup.semanticHints?.some((entry) => entry.name === 'getWorld'),
      JSON.stringify(
        {
          nearNames: lookup.nearNames,
          semanticHints: lookup.semanticHints,
        },
        null,
        2,
      ),
    );
    const localPolicy = lookup.localOperationSiblingPolicy;
    const localPromotion = localPolicy?.promoted?.find(
      (entry) => entry.name === 'getWorld',
    );
    assert(
      'LO7. local operations surface as a separate review-evidence policy',
      localPolicy?.policyId === 'prewrite-local-operation-sibling' &&
        localPolicy?.policyVersion === 'prewrite-local-operation-sibling-v1' &&
        localPolicy?.evaluatedCandidateCount >= 1 &&
        localPromotion?.matchedField === 'preWriteLocalOperationIndex' &&
        localPromotion?.surfaceKind === 'nested-local-operation' &&
        localPromotion?.containerName === 'createRepository' &&
        localPromotion?.operationFamily === 'read-query' &&
        localPromotion?.sharedDomainTokens?.includes('world') &&
        localPromotion?.supportingReasons?.includes(
          'local-operation-same-file-domain-overlap',
        ) &&
        localPromotion?.locality?.sameFile === true &&
        localPromotion?.eligibleForDeadExportRanking === false &&
        localPromotion?.eligibleForSafeFix === false,
      JSON.stringify(localPolicy, null, 2),
    );
    assert(
      'LO8. local operation policy does not feed the service-operation cue policy',
      !lookup.serviceOperationSiblingPolicy?.promoted?.some(
        (entry) => entry.name === 'getWorld',
      ) &&
        !lookup.serviceOperationSiblingPolicy?.muted?.some(
          (entry) => entry.name === 'getWorld',
        ),
      JSON.stringify(lookup.serviceOperationSiblingPolicy, null, 2),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
