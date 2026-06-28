// Tests for _lib/extract-ts-escapes.mjs — P2-0 step 2.
//
// Pinning rules from docs/history/phases/p2/p2-0.md §5.3:
//   - All 11 escapeKinds emit at least one fact on their canonical form.
//   - Precedence: specific beats generic (rest-any-args > explicit-any, etc).
//   - occurrenceKey is stable across line-shift; distinct across files.
//   - insideExportedIdentity follows the 10-row export-form table.
//   - normalizedCodeShape preserves whitespace inside string literals.

import { extractTypeEscapes, normalizeCodeShape } from '../_lib/extract-ts-escapes.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function extract(src, filePath = '/fake/test.ts') {
  return extractTypeEscapes(src, filePath);
}

// ── Helpers ──────────────────────────────────────────────

function kinds(result) {
  return (result.typeEscapes ?? []).map((e) => e.escapeKind);
}

function byKind(result, kind) {
  return (result.typeEscapes ?? []).filter((e) => e.escapeKind === kind);
}

// ═══ T1. explicit-any in type alias ═══

{
  const r = extract(`export type X = any;\n`);
  const hits = byKind(r, 'explicit-any');
  assert('T1. type alias `any` → explicit-any', hits.length === 1);
  assert('T1b. file stored', hits[0]?.file === '/fake/test.ts');
  assert('T1c. line is 1-based', hits[0]?.line === 1);
  assert('T1d. exported type alias gets insideExportedIdentity',
    hits[0]?.insideExportedIdentity === '/fake/test.ts::X',
    `identity=${hits[0]?.insideExportedIdentity}`);
}

// ═══ T2. explicit-any in interface field ═══

{
  const r = extract(`export interface Foo { a: any; b: string }\n`);
  const hits = byKind(r, 'explicit-any');
  assert('T2. interface field `any` → explicit-any', hits.length === 1);
  assert('T2b. exported interface gets insideExportedIdentity',
    hits[0]?.insideExportedIdentity === '/fake/test.ts::Foo',
    `identity=${hits[0]?.insideExportedIdentity}`);
}

// ═══ T3. as-any ═══

{
  const r = extract(`const x = (foo as any).bar;\n`);
  assert('T3. `foo as any` → as-any',
    byKind(r, 'as-any').length === 1);
}

// ═══ T4. angle-any ═══

{
  const r = extract(`const x = <any>foo;\n`, '/fake/t.ts');
  assert('T4. `<any>foo` → angle-any',
    byKind(r, 'angle-any').length === 1);
}

// ═══ T5. as-unknown-as-T ═══

{
  const r = extract(`const x = foo as unknown as Bar;\n`);
  assert('T5. chained `as unknown as T` → as-unknown-as-T',
    byKind(r, 'as-unknown-as-T').length === 1);
  // Precedence: must NOT also emit as-any.
  assert('T5b. as-unknown-as-T does NOT also emit as-any',
    byKind(r, 'as-any').length === 0);
}

// ═══ T6. rest-any-args ═══

{
  const r = extract(`export function f(...args: any[]) {}\n`);
  assert('T6. `...args: any[]` → rest-any-args',
    byKind(r, 'rest-any-args').length === 1);
  // Precedence: must NOT also emit explicit-any for the inner any.
  assert('T6b. rest-any-args does NOT also emit explicit-any',
    byKind(r, 'explicit-any').length === 0);
}

// ═══ T7. index-sig-any ═══

{
  const r = extract(`type Dict = { [key: string]: any };\n`);
  assert('T7. index signature `any` → index-sig-any',
    byKind(r, 'index-sig-any').length === 1);
  assert('T7b. index-sig-any does NOT also emit explicit-any',
    byKind(r, 'explicit-any').length === 0);
}

// ═══ T8. generic-default-any ═══

{
  const r = extract(`type Box<T = any> = { value: T };\n`);
  assert('T8. generic default `any` → generic-default-any',
    byKind(r, 'generic-default-any').length === 1);
  assert('T8b. generic-default-any does NOT also emit explicit-any',
    byKind(r, 'explicit-any').length === 0);
}

// ═══ T9. ts-ignore ═══

{
  const r = extract(`// @ts-ignore reason text\nconst x = 1;\n`);
  const hits = byKind(r, 'ts-ignore');
  assert('T9. `@ts-ignore` comment → ts-ignore fact',
    hits.length === 1);
  assert('T9b. full comment text preserved in codeShape',
    hits[0]?.codeShape.includes('@ts-ignore') &&
    hits[0]?.codeShape.includes('reason text'),
    `codeShape=${hits[0]?.codeShape}`);
}

// ═══ T10. ts-expect-error ═══

{
  const r = extract(`// @ts-expect-error upstream type bug\nconst x = 1;\n`);
  const hits = byKind(r, 'ts-expect-error');
  assert('T10. `@ts-expect-error` comment → ts-expect-error fact',
    hits.length === 1);
  assert('T10b. full comment text preserved in codeShape',
    hits[0]?.codeShape.includes('upstream type bug'));
}

// ═══ T11. no-explicit-any-disable (4 comment forms) ═══

{
  const r = extract(
    `// eslint-disable-next-line no-explicit-any\nconst a = 1;\n` +
    `// eslint-disable-next-line @typescript-eslint/no-explicit-any\nconst b = 2;\n` +
    `// eslint-disable-line @typescript-eslint/no-explicit-any\nconst c = 3;\n` +
    `/* eslint-disable @typescript-eslint/no-explicit-any */\nconst d = 4;\n`
  );
  const hits = byKind(r, 'no-explicit-any-disable');
  assert('T11. four eslint-disable forms → four facts',
    hits.length === 4,
    `hits=${hits.length}, codeShapes=${hits.map((h) => h.codeShape).join(' | ')}`);
}

// ═══ T11b. jsdoc-any ═══

{
  const r = extract(`/** @type {any} */\nconst fromJsdoc = readValue();\n`, '/fake/t.mjs');
  const hits = byKind(r, 'jsdoc-any');
  assert('T11b. `/** @type {any} */` comment → jsdoc-any fact',
    hits.length === 1,
    `hits=${hits.length}, kinds=${kinds(r).join(',')}`);
  assert('T11c. JSDoc comment text preserved in codeShape',
    hits[0]?.codeShape.includes('@type') && hits[0]?.codeShape.includes('{any}'),
    `codeShape=${hits[0]?.codeShape}`);
  assert('T11d. jsdoc-any line is the comment line',
    hits[0]?.line === 1,
    `line=${hits[0]?.line}`);
}

// ═══ T12. normalizedCodeShape — collapse whitespace OUTSIDE strings ═══

{
  const r = extract(`const x = foo   as    any ;\n`, '/fake/t.ts');
  const h = byKind(r, 'as-any')[0];
  assert('T12. normalizedCodeShape collapses outer whitespace runs',
    h?.normalizedCodeShape === 'foo as any',
    `got: ${h?.normalizedCodeShape}`);
}

// ═══ T13. normalizedCodeShape — preserve whitespace INSIDE string literals ═══

{
  const src = `const x = ("a   b" as any);\n`;
  const r = extract(src);
  const h = byKind(r, 'as-any')[0];
  // String literal interior whitespace must remain intact.
  assert('T13. normalizedCodeShape preserves whitespace inside string literals',
    /a   b/.test(h?.normalizedCodeShape ?? ''),
    `got: ${h?.normalizedCodeShape}`);
}

// ═══ T14. occurrenceKey — distinct across files ═══

{
  const a = extract(`const x = foo as any;\n`, '/a.ts');
  const b = extract(`const x = foo as any;\n`, '/b.ts');
  const ka = byKind(a, 'as-any')[0]?.occurrenceKey;
  const kb = byKind(b, 'as-any')[0]?.occurrenceKey;
  assert('T14. distinct files produce distinct occurrenceKeys',
    ka && kb && ka !== kb,
    `a=${ka}, b=${kb}`);
  assert('T14b. occurrenceKey format is "sha256:<64-hex>"',
    /^sha256:[a-f0-9]{64}$/.test(ka),
    `format: ${ka}`);
}

// ═══ T15. occurrenceKey — stable across line shift ═══

{
  const a = extract(`const x = foo as any;\n`, '/stable.ts');
  // Same code, different line (blank lines added above).
  const b = extract(`\n\n\nconst x = foo as any;\n`, '/stable.ts');
  const ka = byKind(a, 'as-any')[0]?.occurrenceKey;
  const kb = byKind(b, 'as-any')[0]?.occurrenceKey;
  assert('T15. line-shift fixture produces SAME occurrenceKey',
    ka && kb && ka === kb,
    `a=${ka}, b=${kb}`);
}

// ═══ T16. insideExportedIdentity — export function ═══

{
  const r = extract(`export function fetchUser() { return x as any; }\n`);
  const h = byKind(r, 'as-any')[0];
  assert('T16. export function → <file>::fetchUser',
    h?.insideExportedIdentity === '/fake/test.ts::fetchUser',
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T17. insideExportedIdentity — export const arrow ═══

{
  const r = extract(`export const fetchUser = () => x as any;\n`);
  const h = byKind(r, 'as-any')[0];
  assert('T17. export const arrow → <file>::fetchUser',
    h?.insideExportedIdentity === '/fake/test.ts::fetchUser',
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T18. insideExportedIdentity — export { foo as bar } ═══

{
  // Local foo is exported as bar. Matching uses the EXPORTED name.
  const r = extract(
    `function foo() { return x as any; }\n` +
    `export { foo as bar };\n`
  );
  const h = byKind(r, 'as-any')[0];
  assert('T18. export { foo as bar } → <file>::bar (exported name, not local)',
    h?.insideExportedIdentity === '/fake/test.ts::bar',
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T19. insideExportedIdentity — export default named ═══

{
  const r = extract(`export default function fetchUser() { return x as any; }\n`);
  const h = byKind(r, 'as-any')[0];
  assert('T19. export default named → <file>::default',
    h?.insideExportedIdentity === '/fake/test.ts::default',
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T20. insideExportedIdentity — export default anonymous arrow ═══

{
  const r = extract(`export default () => (x as any);\n`);
  const h = byKind(r, 'as-any')[0];
  assert('T20. export default arrow → <file>::default',
    h?.insideExportedIdentity === '/fake/test.ts::default',
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T21. insideExportedIdentity — local helper → null ═══

{
  const r = extract(`function unused() { return x as any; }\n`);
  const h = byKind(r, 'as-any')[0];
  assert('T21. non-exported local helper → null',
    h?.insideExportedIdentity === null,
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T22. insideExportedIdentity — top-level outside any function ═══

{
  const r = extract(`const x = foo as any;\n`);
  const h = byKind(r, 'as-any')[0];
  assert('T22. top-level outside any function/class → null',
    h?.insideExportedIdentity === null,
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T23. insideExportedIdentity — nested non-exported inside exported parent ═══

{
  const r = extract(
    `export function outer() {\n` +
    `  function inner() { return x as any; }\n` +
    `  return inner();\n` +
    `}\n`
  );
  const h = byKind(r, 'as-any')[0];
  assert('T23. nested non-exported inside exported parent → exported parent wins',
    h?.insideExportedIdentity === '/fake/test.ts::outer',
    `got: ${h?.insideExportedIdentity}`);
}

// ═══ T24. Parse error returns structured marker ═══

{
  const r = extract(`const x = ;;;broken syntax\n`);
  assert('T24. parse error returns { parseError, typeEscapes: [] }',
    typeof r.parseError === 'string' && r.parseError.length > 0,
    `r=${JSON.stringify(r).slice(0, 200)}`);
  assert('T24b. no typeEscapes on parse error',
    !Array.isArray(r.typeEscapes) || r.typeEscapes.length === 0);
}

// ═══ T25. All 11 escape kinds covered ═══

const uniqKinds = [
  'explicit-any', 'as-any', 'angle-any', 'as-unknown-as-T',
  'rest-any-args', 'index-sig-any', 'generic-default-any',
  'ts-ignore', 'ts-expect-error', 'no-explicit-any-disable',
  'jsdoc-any',
];
{
  // One-shot fixture with at least one of each kind.
  const src =
    `type A = any;\n` +                                   // explicit-any
    `const b = (x as any);\n` +                           // as-any
    `const c = (<any>x);\n` +                             // angle-any
    `const d = (x as unknown as Foo);\n` +                // as-unknown-as-T
    `function e(...args: any[]) {}\n` +                   // rest-any-args
    `type F = { [k: string]: any };\n` +                  // index-sig-any
    `type G<T = any> = T;\n` +                            // generic-default-any
    `// @ts-ignore reason\nconst h = 1;\n` +               // ts-ignore
    `// @ts-expect-error reason\nconst i = 1;\n` +         // ts-expect-error
    `// eslint-disable-next-line no-explicit-any\nconst j = 1;\n` +  // no-explicit-any-disable
    `/** @type {any} */\nconst k = readValue();\n`;       // jsdoc-any
  const r = extract(src);
  const seen = new Set(kinds(r));
  for (const k of uniqKinds) {
    assert(`T25. full-coverage fixture emits ${k}`,
      seen.has(k),
      `seen=${[...seen].join(',')}`);
  }
}

// ═══ T26. normalizeCodeShape is exported and preserves string-literal whitespace (P2-1 pre-step) ═══

{
  // String-literal interior whitespace MUST be preserved so P2-1's
  // planned matching normalization stays in lockstep with P2-0's
  // occurrence emission.
  const raw = `foo   as   "a   b"   as   any`;
  const normalized = normalizeCodeShape(raw);
  assert('T26. normalizeCodeShape preserves string-literal interior whitespace',
    normalized === `foo as "a   b" as any`,
    `got: ${JSON.stringify(normalized)}`);

  // Reference-equality pin: the exported function is the SAME function
  // extractTypeEscapes uses internally. An extraction run on a matching
  // source must produce a normalizedCodeShape byte-identical to what the
  // exported helper computes on the same code slice — this would fail if
  // a future refactor forked the normalizer into a duplicate regex impl.
  const src = `const x = foo   as   "a   b"   as   any;\n`;
  const r = extract(src);
  const h = byKind(r, 'as-any')[0];
  assert('T26b. extractTypeEscapes emits normalizedCodeShape matching the exported helper',
    h?.normalizedCodeShape === normalizeCodeShape(h?.codeShape ?? ''),
    `emitted=${JSON.stringify(h?.normalizedCodeShape)} expected=${JSON.stringify(normalizeCodeShape(h?.codeShape ?? ''))}`);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
