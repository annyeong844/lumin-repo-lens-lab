// Tests for _lib/shape-hash.mjs - P4-1 pure shape normalization/hash core.
//
// These tests do NOT create shape-index.json yet. They pin the structural
// evidence layer that a later P4 producer will consume.

import {
  detectGeneratedFileEvidence,
  extractShapeHashFactsFromSource,
  groupShapeFactsByHash,
  normalizeTypeText,
} from '../_lib/shape-hash.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function extract(src, file = 'src/types.ts') {
  return extractShapeHashFactsFromSource(src, file, {
    observedAt: '2026-04-22T00:00:00.000Z',
  });
}

function factByName(result, name) {
  return result.facts.find((f) => f.exportedName === name);
}

// T1. Field order and declaration spelling do not change object shape.
{
  const r = extract(`
    export interface UserA {
      b: number;
      a: string;
    }
    export type UserB = {
      a: string;
      b: number;
    };
  `);
  const a = factByName(r, 'UserA');
  const b = factByName(r, 'UserB');
  assert('S1a. two supported exported shapes extracted',
    r.facts.length === 2,
    `facts=${JSON.stringify(r.facts)} diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('S1b. same fields in different order -> same hash',
    a?.hash && a.hash === b?.hash,
    `a=${a?.hash} b=${b?.hash}`);
  assert('S1c. fields sorted by name in fact payload',
    JSON.stringify(a?.fields.map((f) => f.name)) === JSON.stringify(['a', 'b']));
}

// T2. Changing a field type changes the hash.
{
  const r = extract(`
    export type A = { id: string };
    export type B = { id: number };
  `);
  assert('S2. field type change -> different hash',
    factByName(r, 'A')?.hash !== factByName(r, 'B')?.hash);
}

// T3. Optional and readonly modifiers are hash-bearing.
{
  const r = extract(`
    export type A = { id: string; name: string };
    export type B = { id?: string; name: string };
    export type C = { readonly id: string; name: string };
  `);
  const a = factByName(r, 'A')?.hash;
  const b = factByName(r, 'B')?.hash;
  const c = factByName(r, 'C')?.hash;
  assert('S3a. optional vs required -> different hash', a !== b);
  assert('S3b. readonly vs mutable -> different hash', a !== c);
}

// T4. Type text normalization is punctuation-space stable and literal-safe.
{
  assert('S4a. punctuation spacing normalized outside literals',
    normalizeTypeText('Array < string | number >') === 'Array<string|number>');
  assert('S4b. string literal interior spacing preserved',
    normalizeTypeText('"a | b" | string') === '"a | b"|string');
}

// T5. The same union type with different spacing hashes the same.
{
  const r = extract(`
    export type A = { value: "a | b" | string };
    export type B = { value:"a | b"|string };
  `);
  assert('S5. semantically same type text spacing -> same hash',
    factByName(r, 'A')?.hash === factByName(r, 'B')?.hash);
}

// T6. Unsupported mapped/generic shapes emit diagnostics, not fake facts.
{
  const r = extract(`
    export type Mapped<T> = { [K in keyof T]: T[K] };
  `);
  assert('S6a. unsupported mapped/generic shape emits no fact',
    r.facts.length === 0,
    JSON.stringify(r.facts));
  assert('S6b. unsupported reason is explicit',
    r.diagnostics.some((d) => d.code === 'unsupported-type-parameters'),
    JSON.stringify(r.diagnostics));
}

// T7. Computed members are rejected rather than fuzzily matched.
{
  const r = extract(`
    export type Weird = { [key: string]: string };
  `);
  assert('S7a. index/computed shape emits no fact',
    r.facts.length === 0);
  assert('S7b. unsupported member diagnostic present',
    r.diagnostics.some((d) => d.code === 'unsupported-member-kind'),
    JSON.stringify(r.diagnostics));
}

// T8. Fact shape carries canonical metadata and sha256 hash format.
{
  const r = extract(`export interface User { id: string }\n`);
  const f = factByName(r, 'User');
  assert('S8a. fact kind is shape-hash',
    f?.kind === 'shape-hash');
  assert('S8b. hash is sha256:<64 hex>',
    /^sha256:[a-f0-9]{64}$/.test(f?.hash ?? ''),
    f?.hash);
  assert('S8c. canonical metadata present',
    f?.source === 'fresh-ast-pass' &&
    f?.scope === 'TS/JS production files, exported types only' &&
    f?.confidence === 'high' &&
    f?.observedAt === '2026-04-22T00:00:00.000Z');
  assert('S8d. identity uses ownerFile::exportedName',
    f?.identity === 'src/types.ts::User' &&
    JSON.stringify(f?.identities) === JSON.stringify(['src/types.ts::User']));
}

// T9. Parse errors are source-level diagnostics with no high-confidence facts.
{
  const r = extract(`export interface Broken { id: string `);
  assert('S9a. parse error emits no facts',
    r.facts.length === 0);
  assert('S9b. parse-error diagnostic present',
    r.diagnostics.some((d) => d.code === 'parse-error'),
    JSON.stringify(r.diagnostics));
}

// T10. Export specifier aliases become exported identity names.
{
  const r = extract(`
    interface LocalUser { id: string }
    export { LocalUser as PublicUser };
  `);
  const f = factByName(r, 'PublicUser');
  assert('S10a. local declaration exported by specifier is extracted',
    r.facts.length === 1,
    JSON.stringify(r));
  assert('S10b. identity uses exported alias, not local name',
    f?.identity === 'src/types.ts::PublicUser',
    JSON.stringify(f));
}

// T11. Grouping by hash produces deterministic identity lists.
{
  const r = extract(`
    export type B = { x: string };
    export type A = { x: string };
  `);
  const groups = groupShapeFactsByHash(r.facts);
  const group = Object.values(groups)[0] ?? [];
  assert('S11. groupsByHash identity lists are sorted',
    JSON.stringify(group) === JSON.stringify(['src/types.ts::A', 'src/types.ts::B']),
    JSON.stringify(groups));
}

// T12. TS declaration merging is unsupported rather than partial-hashed.
{
  const r = extract(`
    export interface Foo { a: string }
    export interface Foo { b: number }
  `);
  assert('S12a. declaration-merged identity emits no partial shape facts',
    r.facts.length === 0,
    JSON.stringify(r.facts));
  assert('S12b. declaration-merge-unsupported diagnostic is explicit',
    r.diagnostics.some((d) =>
      d.code === 'declaration-merge-unsupported' &&
      d.identity === 'src/types.ts::Foo'),
    JSON.stringify(r.diagnostics));
}

// T13. Generated-file evidence is tagged on facts, not used as a fake match.
{
  const r = extract(`export interface FileRoutesById { id: string }\n`, 'src/routeTree.gen.ts');
  const f = factByName(r, 'FileRoutesById');
  assert('S13a. routeTree.gen.ts fact carries generated-file evidence',
    f?.generatedFile?.kind === 'generated-file' &&
    f.generatedFile.source === 'path',
    JSON.stringify(f));
  assert('S13b. generated header detector works without path convention',
    detectGeneratedFileEvidence('src/manual.ts', '// @generated by tool\nexport interface A { id: string }')?.source === 'header');
}

// T14. Literal unions are hashable, order-insensitive shape facts.
{
  const r = extract(`
    export type StatusA = "open" | 'closed' | null | undefined | true | 1 | 1n;
    export type StatusB = 1n | true | undefined | null | \`closed\` | "open" | 1;
    export type StatusC = "open" | "pending";
  `);
  const a = factByName(r, 'StatusA');
  const b = factByName(r, 'StatusB');
  const c = factByName(r, 'StatusC');
  assert('S14a. literal union aliases are extracted',
    a?.shapeKind === 'literal-union' && b?.shapeKind === 'literal-union',
    JSON.stringify(r));
  assert('S14b. same literal union in different order -> same hash',
    a?.hash && a.hash === b?.hash && a.hash !== c?.hash,
    `a=${a?.hash} b=${b?.hash} c=${c?.hash}`);
  assert('S14c. literal union fact carries normalized literal evidence',
    a?.literals?.some((lit) => lit.kind === 'string' && lit.value === 'open') &&
    a?.literals?.some((lit) => lit.kind === 'undefined') &&
    a?.literals?.some((lit) => lit.kind === 'bigint' && lit.value === '1'),
    JSON.stringify(a));
}

// T15. Mixed broad unions stay diagnostic-only, not fuzzy facts.
{
  const r = extract(`export type Mixed = "open" | string;\n`);
  assert('S15a. broad mixed union emits no shape fact',
    r.facts.length === 0,
    JSON.stringify(r.facts));
  assert('S15b. unsupported literal-union member diagnostic is explicit',
    r.diagnostics.some((d) =>
      d.code === 'unsupported-literal-union-member' &&
      d.identity === 'src/types.ts::Mixed'),
    JSON.stringify(r.diagnostics));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
