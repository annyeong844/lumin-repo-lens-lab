// Tests for `collectTypeIdentities` + `renderTypeOwnership` — P3-1 Step 2.
//
// Pinning rules from docs/history/phases/p3/p3-1.md v2 §5.3:
//   - Identity format `ownerFile::exportedName` everywhere.
//   - Duplicates keyed by name; fan-in attributed to owner not barrel.
//   - `export { X as Y }` alias hop: terminal identity is `src/y.ts::X`,
//     NOT `src/index.ts::Y`.
//   - Star re-export ambiguity / 8-hop depth → `[확인 불가]`.
//   - Contamination routing: severely-contaminated → severely label; all-
//     contaminated group → ANY_COLLISION.
//   - Markdown cells use escapeMdCell + codeCell.

import {
  collectTypeIdentities,
  renderTypeOwnership,
} from '../_lib/canon-draft-types.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Fixture builder ──────────────────────────────────────
//
// Hand-crafted `symbols.json` shape matching the actual producer's
// output (tested against real data in Step 4 integration tests).

function makeSymbols({
  defIndex = {},
  fanInByIdentity = {},
  fanInByIdentitySpace = {},
  reExportsByFile = {},
} = {}) {
  return {
    meta: {
      tool: 'build-symbol-graph.mjs',
      generated: '2026-04-21T00:00:00Z',
      root: '/fake',
      supports: { identityFanIn: true, identityFanInSpace: true, reExportRecords: 'file-level' },
    },
    defIndex,
    fanInByIdentity,
    fanInByIdentitySpace,
    reExportsByFile,
  };
}

function typeDef(name, kind, line) {
  return { [name]: { name, kind, line } };
}

function makeShapeIndex(facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    if (!groupsByHash[fact.hash]) groupsByHash[fact.hash] = [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  return {
    schemaVersion: 'shape-index.v1',
    meta: { complete },
    facts,
    groupsByHash,
    diagnostics: [],
  };
}

// ── I1. Single type owner, fanIn 5 → single-owner-strong ─────

{
  const symbols = makeSymbols({
    defIndex: {
      'src/types.ts': typeDef('User', 'TSInterfaceDeclaration', 10),
    },
    fanInByIdentity: { 'src/types.ts::User': 5 },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });

  assert('I1a. single type owner aggregated to 1 identity',
    r.typeDefsByIdentity.size === 1);
  assert('I1b. identity key is ownerFile::exportedName',
    r.typeDefsByIdentity.has('src/types.ts::User'));
  assert('I1c. identitiesByName keyed by name, array of identities',
    r.identitiesByName.get('User')?.length === 1);

  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I1d. Markdown contains single-owner-strong for fanIn=5',
    md.includes('single-owner-strong'));
  assert('I1e. Markdown cell uses codeCell(`src/types.ts::User`)',
    md.includes('`src/types.ts::User`'));
}

// ── I1f. Type/value/broad fan-in space is surfaced without changing total fan-in ─

{
  const symbols = makeSymbols({
    defIndex: {
      'src/types.ts': typeDef('User', 'TSInterfaceDeclaration', 10),
    },
    fanInByIdentity: { 'src/types.ts::User': 5 },
    fanInByIdentitySpace: {
      'src/types.ts::User': { value: 1, type: 4, broad: 0 },
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const def = r.typeDefsByIdentity.get('src/types.ts::User');
  assert('I1f. aggregate preserves fanInSpace for type owner',
    def?.fanIn === 5 &&
      def?.fanInSpace?.value === 1 &&
      def?.fanInSpace?.type === 4 &&
      def?.fanInSpace?.broad === 0,
    JSON.stringify(def));

  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I1g. Markdown renders value/type/broad fan-in space beside total fan-in',
    md.includes('| 5 | value 1, type 4, broad 0 |'),
    md);
}

// ── I2. Two owners same name, both high fanIn → DUPLICATE_STRONG ─

{
  const symbols = makeSymbols({
    defIndex: {
      'src/a.ts': typeDef('Result', 'TSTypeAliasDeclaration', 5),
      'src/b.ts': typeDef('Result', 'TSTypeAliasDeclaration', 5),
    },
    fanInByIdentity: {
      'src/a.ts::Result': 18,
      'src/b.ts::Result': 3,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I2a. two owners aggregated into name group of 2',
    r.identitiesByName.get('Result')?.length === 2);
  assert('I2b. both identities keyed by ownerFile::exportedName',
    r.typeDefsByIdentity.has('src/a.ts::Result') &&
    r.typeDefsByIdentity.has('src/b.ts::Result'));
  assert('I2c. Result + high fanIn → DUPLICATE_STRONG (Rule 1 wins over Rule 2)',
    md.includes('DUPLICATE_STRONG'));
}

// ── I3. Two owners same name Props (low-info) + low fanIn → LOCAL_COMMON_NAME ─

{
  const symbols = makeSymbols({
    defIndex: {
      'src/a.ts': typeDef('Props', 'TSInterfaceDeclaration', 5),
      'src/b.ts': typeDef('Props', 'TSInterfaceDeclaration', 5),
    },
    fanInByIdentity: {
      'src/a.ts::Props': 1,
      'src/b.ts::Props': 2,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I3. Props + max-fanIn<3 → LOCAL_COMMON_NAME',
    md.includes('LOCAL_COMMON_NAME'));
}

// ── I4. Cross-file duplicate → identity cells distinct, not collapsed on name ─

{
  const symbols = makeSymbols({
    defIndex: {
      'apps/admin/types.ts': typeDef('User', 'TSInterfaceDeclaration', 3),
      'apps/blog/types.ts':  typeDef('User', 'TSInterfaceDeclaration', 3),
    },
    fanInByIdentity: {
      'apps/admin/types.ts::User': 5,
      'apps/blog/types.ts::User':  5,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });

  assert('I4a. both identities preserved (apps/admin + apps/blog)',
    r.typeDefsByIdentity.size === 2 &&
    r.typeDefsByIdentity.has('apps/admin/types.ts::User') &&
    r.typeDefsByIdentity.has('apps/blog/types.ts::User'));

  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I4b. Markdown has both identity cells',
    md.includes('apps/admin/types.ts::User') &&
    md.includes('apps/blog/types.ts::User'));
  assert('I4c. name group does NOT collapse identities',
    r.identitiesByName.get('User').length === 2);
}

// ── I5. `typeName !== exportedName` hypothetical — identity uses exportedName ─

{
  // Construct a def with typeName explicitly different from the map key.
  // P3-1 MUST emit identity keyed on the map key (which IS exportedName
  // by the symbols.json defIndex convention — nested file→name key is
  // the exported name). Even if a future producer attaches an alternate
  // typeName field on the def, that field MUST NOT enter identity.
  const symbols = makeSymbols({
    defIndex: {
      'src/weird.ts': {
        PublicName: {
          name: 'PublicName',
          kind: 'TSTypeAliasDeclaration',
          line: 1,
          typeName: 'InternalLocal',  // hypothetical display alias
        },
      },
    },
    fanInByIdentity: { 'src/weird.ts::PublicName': 2 },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });

  assert('I5a. identity uses defIndex map key (exportedName), NOT internal typeName',
    r.typeDefsByIdentity.has('src/weird.ts::PublicName') &&
    !r.typeDefsByIdentity.has('src/weird.ts::InternalLocal'));

  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I5b. Markdown identity cell uses PublicName (exportedName), not InternalLocal',
    md.includes('src/weird.ts::PublicName') && !md.includes('src/weird.ts::InternalLocal'));
}

// ── I6. Re-export chain — owner retains identity, barrel in reExportedThrough ─
//
// `src/y.ts` defines `X`. `src/index.ts` re-exports it (with or without
// alias). Fan-in attributed to `src/y.ts::X`. Barrel `src/index.ts`
// tracked via `reExportedThrough`, NEVER as the final identity.

{
  const symbols = makeSymbols({
    defIndex: {
      'src/y.ts': typeDef('X', 'TSInterfaceDeclaration', 1),
    },
    fanInByIdentity: { 'src/y.ts::X': 3 },
    reExportsByFile: {
      'src/index.ts': [{ source: './y', line: 1 }],
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });

  assert('I6a. terminal identity is the owner, not the barrel',
    r.typeDefsByIdentity.has('src/y.ts::X') &&
    !r.typeDefsByIdentity.has('src/index.ts::X'));

  const uses = r.typeUsesByIdentity.get('src/y.ts::X');
  assert('I6b. reExportedThrough records the barrel',
    uses && Array.from(uses.reExportedThrough).includes('src/index.ts'));
}

// ── I7. Contamination — severely-any-contaminated routing ──

{
  const symbols = makeSymbols({
    defIndex: {
      'src/big.ts': typeDef('BigBlob', 'TSInterfaceDeclaration', 1),
    },
    fanInByIdentity: { 'src/big.ts::BigBlob': 100 },
  });
  // Simulate anyContamination by injecting into the def entry.
  symbols.defIndex['src/big.ts'].BigBlob.anyContamination = {
    label: 'severely-any-contaminated',
    labels: ['any-contaminated', 'severely-any-contaminated'],
    measurements: { totalFields: 3, anyFields: 3, unknownFields: 0, anyFieldRatio: 1, indexSignatureAny: false },
  };
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I7. severely-any-contaminated overrides high fanIn → severely-any-contaminated label',
    md.includes('severely-any-contaminated'));
}

// ── I8. Filter to exported TYPE declarations only ─

{
  const symbols = makeSymbols({
    defIndex: {
      'src/mixed.ts': {
        MyType: { name: 'MyType', kind: 'TSTypeAliasDeclaration', line: 1 },
        myFunc: { name: 'myFunc', kind: 'FunctionDeclaration', line: 10 },
        myConst: { name: 'myConst', kind: 'VariableDeclaration', line: 20 },
      },
    },
    fanInByIdentity: {
      'src/mixed.ts::MyType': 2,
      'src/mixed.ts::myFunc': 2,
      'src/mixed.ts::myConst': 2,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });

  assert('I8a. only TSTypeAliasDeclaration / TSInterfaceDeclaration / TSEnumDeclaration / TSModuleDeclaration included',
    r.typeDefsByIdentity.size === 1 && r.typeDefsByIdentity.has('src/mixed.ts::MyType'));
  assert('I8b. function declarations excluded',
    !r.typeDefsByIdentity.has('src/mixed.ts::myFunc'));
  assert('I8c. variable declarations excluded',
    !r.typeDefsByIdentity.has('src/mixed.ts::myConst'));
}

// ── I9. Markdown rendering — escape / codeCell / non-overwrite format ─

{
  const symbols = makeSymbols({
    defIndex: {
      'src/with|pipe.ts': typeDef('Weird|Name', 'TSTypeAliasDeclaration', 1),
    },
    fanInByIdentity: { 'src/with|pipe.ts::Weird|Name': 2 },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I9a. pipe characters in file/name escaped in Markdown',
    md.includes('\\|') || md.includes('`src/with|pipe.ts::Weird|Name`'));

  // Table header present
  assert('I9b. Markdown renders table header with Name | Identity | Owner | Fan-in | Status',
    md.includes('Name') && md.includes('Identity') && md.includes('Fan-in') && md.includes('Status'));
  // Meta block present
  assert('I9c. Markdown renders scope field in meta',
    md.includes('TS/JS including tests'));
}

// ── I10. Empty input → empty table + header preserved ──

{
  const symbols = makeSymbols({ defIndex: {}, fanInByIdentity: {} });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });
  assert('I10a. empty aggregation still emits valid Markdown',
    md.startsWith('# Type ownership draft'));
  assert('I10b. empty aggregation has no data rows',
    r.typeDefsByIdentity.size === 0);
}

// ── I11. Shape evidence enriches duplicate groups without changing labels ──

{
  const hash = 'sha256:' + 'a'.repeat(64);
  const symbols = makeSymbols({
    defIndex: {
      'src/a.ts': typeDef('Result', 'TSTypeAliasDeclaration', 5),
      'src/b.ts': typeDef('Result', 'TSTypeAliasDeclaration', 7),
    },
    fanInByIdentity: {
      'src/a.ts::Result': 18,
      'src/b.ts::Result': 3,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    shapeIndex: makeShapeIndex([
      { identity: 'src/a.ts::Result', hash },
      { identity: 'src/b.ts::Result', hash },
    ]),
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });

  assert('I11a. duplicate label semantics remain fan-in based',
    md.includes('DUPLICATE_STRONG'));
  assert('I11b. same hash duplicate group emits same-shape evidence note',
    md.includes('## Shape evidence') &&
    md.includes('same-shape evidence') &&
    md.includes(hash));
  assert('I11c. shape evidence note preserves both identities',
    md.includes('src/a.ts::Result') && md.includes('src/b.ts::Result'));
}

// ── I12. Different hashes are advisory different-shape evidence ──

{
  const hashA = 'sha256:' + 'a'.repeat(64);
  const hashB = 'sha256:' + 'b'.repeat(64);
  const symbols = makeSymbols({
    defIndex: {
      'src/a.ts': typeDef('Config', 'TSInterfaceDeclaration', 5),
      'src/b.ts': typeDef('Config', 'TSInterfaceDeclaration', 7),
    },
    fanInByIdentity: {
      'src/a.ts::Config': 2,
      'src/b.ts::Config': 1,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    shapeIndex: makeShapeIndex([
      { identity: 'src/a.ts::Config', hash: hashA },
      { identity: 'src/b.ts::Config', hash: hashB },
    ]),
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });

  assert('I12a. different hashes emit different-shape evidence',
    md.includes('different-shape evidence') &&
    md.includes(hashA) &&
    md.includes(hashB));
  assert('I12b. advisory evidence does not replace existing duplicate labels',
    md.includes('DUPLICATE_REVIEW') || md.includes('LOCAL_COMMON_NAME'));
}

// ── I13. Incomplete/partial shape index degrades missing-fact claims ──

{
  const hash = 'sha256:' + 'c'.repeat(64);
  const symbols = makeSymbols({
    defIndex: {
      'src/a.ts': typeDef('Model', 'TSInterfaceDeclaration', 5),
      'src/b.ts': typeDef('Model', 'TSInterfaceDeclaration', 7),
    },
    fanInByIdentity: {
      'src/a.ts::Model': 4,
      'src/b.ts::Model': 1,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    shapeIndex: makeShapeIndex([
      { identity: 'src/a.ts::Model', hash },
    ], { complete: false }),
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });

  assert('I13a. incomplete shape-index emits degraded evidence caveat',
    md.includes('shape evidence degraded') && md.includes('incomplete'));
  assert('I13b. partial duplicate group names missing identity explicitly',
    md.includes('shape evidence partial') &&
    md.includes('Missing shape facts') &&
    md.includes('src/b.ts::Model'));
}

// ── I14. Generated-only duplicate shape groups are summarized, not expanded ──

{
  const hashA = 'sha256:' + 'd'.repeat(64);
  const hashB = 'sha256:' + 'e'.repeat(64);
  const symbols = makeSymbols({
    defIndex: {
      'apps/a/src/routeTree.gen.ts': typeDef('FileRoutesById', 'TSInterfaceDeclaration', 5),
      'apps/b/src/routeTree.gen.ts': typeDef('FileRoutesById', 'TSInterfaceDeclaration', 7),
    },
    fanInByIdentity: {
      'apps/a/src/routeTree.gen.ts::FileRoutesById': 5,
      'apps/b/src/routeTree.gen.ts::FileRoutesById': 3,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    shapeIndex: makeShapeIndex([
      {
        identity: 'apps/a/src/routeTree.gen.ts::FileRoutesById',
        hash: hashA,
        generatedFile: { kind: 'generated-file', source: 'path', evidence: 'path:routeTree.gen' },
      },
      {
        identity: 'apps/b/src/routeTree.gen.ts::FileRoutesById',
        hash: hashB,
        generatedFile: { kind: 'generated-file', source: 'path', evidence: 'path:routeTree.gen' },
      },
    ]),
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });

  assert('I14a. generated-only group still keeps table labels',
    md.includes('DUPLICATE_STRONG') &&
    md.includes('apps/a/src/routeTree.gen.ts::FileRoutesById') &&
    md.includes('apps/b/src/routeTree.gen.ts::FileRoutesById'));
  assert('I14b. generated-only shape evidence is summarized',
    md.includes('generated-shape evidence summarized') &&
    !md.includes('different-shape evidence: `FileRoutesById`'),
    md);
}

// ── I15. Malformed generated evidence cannot silently hide shape notes ──

{
  const hashA = 'sha256:' + 'f'.repeat(64);
  const hashB = 'sha256:' + '1'.repeat(64);
  const symbols = makeSymbols({
    defIndex: {
      'src/a.ts': typeDef('WidgetProps', 'TSInterfaceDeclaration', 5),
      'src/b.ts': typeDef('WidgetProps', 'TSInterfaceDeclaration', 7),
    },
    fanInByIdentity: {
      'src/a.ts::WidgetProps': 5,
      'src/b.ts::WidgetProps': 3,
    },
  });
  const r = collectTypeIdentities({ symbols, root: '/fake' });
  const md = renderTypeOwnership({
    typeDefsByIdentity: r.typeDefsByIdentity,
    identitiesByName: r.identitiesByName,
    typeUsesByIdentity: r.typeUsesByIdentity,
    diagnostics: r.diagnostics,
    shapeIndex: makeShapeIndex([
      {
        identity: 'src/a.ts::WidgetProps',
        hash: hashA,
        generatedFile: true,
      },
      {
        identity: 'src/b.ts::WidgetProps',
        hash: hashB,
        generatedFile: true,
      },
    ]),
    meta: { scope: 'TS/JS including tests', source: 'symbols.json' },
  });

  assert('I15. malformed generatedFile metadata fails closed',
    md.includes('shape evidence unavailable') &&
    md.includes('malformed-generated-file-evidence') &&
    !md.includes('generated-shape evidence summarized'),
    md);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
