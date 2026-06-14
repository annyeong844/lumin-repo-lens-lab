// Tests for build-function-clone-index.mjs — deterministic helper clone cues.

import { execFileSync } from 'node:child_process';
import {
  writeFileSync,
  readFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  existsSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'build-function-clone-index.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, extraArgs = []) {
  return execFileSync(NODE, [CLI, '--root', root, '--output', output, ...extraArgs], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'function-clones.json'), 'utf8'));
}

// T1. Same structure with distant helper names is surfaced as a review cue.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-fixture', type: 'module' }));
    write(fx, 'src/money-a.ts',
      `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
      `  const dollars = cents / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
      `}\n`);
    write(fx, 'src/money-b.ts',
      `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
      `  const amount = value / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);\n` +
      `}\n`);

    const stdout = run(fx, out);
    const index = readIndex(out);
    const group = index.structureGroups.find((g) =>
      g.identities.includes('src/money-a.ts::formatCurrencyCents') &&
      g.identities.includes('src/money-b.ts::renderPaymentTotal'));

    assert('FC1a. CLI writes function-clones.json',
      existsSync(path.join(out, 'function-clones.json')));
    assert('FC1b. stdout summarizes function clone run',
      stdout.includes('[function-clones]') && stdout.includes('function facts'),
      stdout);
    assert('FC1c. schemaVersion is function-clones.v3',
      index.schemaVersion === 'function-clones.v3');
    assert('FC1d. meta declares semanticEquivalence=false',
      index.meta.supports?.semanticEquivalence === false &&
      index.meta.supports?.normalizedStructureHash === true,
      JSON.stringify(index.meta.supports));
    assert('FC1e. distant names with same structure are grouped',
      !!group,
      JSON.stringify(index.structureGroups));
    assert('FC1f. group reason is review-cue only',
      group?.reason?.includes('not proof of semantic equivalence'),
      JSON.stringify(group));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T2. Exact normalized bodies and parse failures are explicit.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-exact-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-exact-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-fixture', type: 'module' }));
    write(fx, 'src/a.ts',
      `export const parseOne = (raw: string) => {\n` +
      `  const value = raw.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n`);
    write(fx, 'src/b.ts',
      `const local = (raw: string) => {\n` +
      `  const value = raw.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n` +
      `export { local as parseTwo };\n`);
    write(fx, 'src/bad.ts', `export function broken( {`);

    run(fx, out);
    const index = readIndex(out);
    const exact = index.exactBodyGroups.find((g) =>
      g.identities.includes('src/a.ts::parseOne') &&
      g.identities.includes('src/b.ts::parseTwo'));

    assert('FC2a. exact normalized body group emitted for aliased export',
      !!exact,
      JSON.stringify(index.exactBodyGroups));
    assert('FC2b. parse errors make artifact incomplete but keep good facts',
      index.meta.complete === false &&
      index.meta.filesWithParseErrors.some((e) => e.file === 'src/bad.ts') &&
      index.facts.some((f) => f.identity === 'src/a.ts::parseOne'),
      JSON.stringify(index.meta));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T3. --production excludes test helpers and records scope.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-prod-'));
  const outDefault = mkdtempSync(path.join(tmpdir(), 'fn-clone-prod-out1-'));
  const outProd = mkdtempSync(path.join(tmpdir(), 'fn-clone-prod-out2-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-fixture', type: 'module' }));
    write(fx, 'src/a.ts', `export function prod() { return 1 + 1; }\n`);
    write(fx, 'tests/a.test.ts', `export function testHelper() { return 1 + 1; }\n`);

    run(fx, outDefault);
    run(fx, outProd, ['--production']);
    const defaultIndex = readIndex(outDefault);
    const prodIndex = readIndex(outProd);

    assert('FC3a. default includes tests',
      defaultIndex.facts.some((f) => f.identity === 'tests/a.test.ts::testHelper'),
      JSON.stringify(defaultIndex.facts));
    assert('FC3b. --production excludes tests',
      !prodIndex.facts.some((f) => f.identity === 'tests/a.test.ts::testHelper') &&
      prodIndex.meta.scope === 'TS/JS production files, top-level exported and file-local functions',
      JSON.stringify(prodIndex.meta));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(outDefault, { recursive: true, force: true });
    rmSync(outProd, { recursive: true, force: true });
  }
}

// T4. Similar exported helpers with different AST structure are surfaced
// only as near review candidates, never as exact/structure clone groups.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-near-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-near-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-near-fixture', type: 'module' }));
    write(fx, 'src/date-a.ts',
      `export function formatDate(value: Date) {\n` +
      `  const formatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' });\n` +
      `  return formatter.format(value);\n` +
      `}\n`);
    write(fx, 'src/date-b.ts',
      `export function dateFormat(input: Date) {\n` +
      `  return new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' }).format(input);\n` +
      `}\n`);

    run(fx, out);
    const index = readIndex(out);
    const pair = ['src/date-a.ts::formatDate', 'src/date-b.ts::dateFormat'];
    const exact = index.exactBodyGroups.find((g) => pair.every((id) => g.identities.includes(id)));
    const structure = index.structureGroups.find((g) => pair.every((id) => g.identities.includes(id)));
    const near = index.nearFunctionCandidates?.find((g) => pair.every((id) => g.identities.includes(id)));

    assert('FC4a. near function support is declared without claiming semantic equivalence',
      index.meta.supports?.nearFunctionCandidates === true &&
      index.meta.supports?.semanticEquivalence === false,
      JSON.stringify(index.meta.supports));
    assert('FC4a2. near function thresholds are exposed as policy metadata',
      index.meta.thresholdPolicies?.some((policy) =>
        policy.policyId === 'function-clone-near-policy' &&
        policy.policyVersion === 'function-clone-near-policy-v1' &&
        policy.policyClass === 'review' &&
        policy.thresholds?.minNearScore === 0.62 &&
        policy.thresholds?.maxNearCandidates === 50),
      JSON.stringify(index.meta.thresholdPolicies, null, 2));
    assert('FC4b. structurally different date helpers are not promoted to exact/structure groups',
      !exact && !structure,
      `exact=${JSON.stringify(index.exactBodyGroups)}\nstructure=${JSON.stringify(index.structureGroups)}`);
    assert('FC4c. structurally different date helpers are surfaced as near review candidates',
      !!near &&
      index.meta.nearFunctionCandidateCount === 1 &&
      near.kind === 'near-function-candidate' &&
      near.risk === 'review-only' &&
      near.sharedCallTokens.includes('DateTimeFormat') &&
      near.nameTokenJaccard >= 0.5,
      JSON.stringify(index.nearFunctionCandidates));
    assert('FC4d. near candidate text refuses automatic semantic merge claims',
      /not proof of semantic equivalence/.test(near?.reason ?? '') &&
      /source review required/.test(near?.reason ?? ''),
      JSON.stringify(near));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T5. Identical exported function type signatures are surfaced even when
// bodies and names differ enough to avoid body/near clone lanes.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-signature-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-signature-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-signature-fixture', type: 'module' }));
    write(fx, 'src/shallow.ts',
      `export function useShallow<S, U>(selector: (state: S) => U): (state: S) => U {\n` +
      `  return selector;\n` +
      `}\n\n` +
      `export function composeProjection<S, U>(selector: (state: S) => U): (state: S) => U {\n` +
      `  return (state) => selector(state);\n` +
      `}\n`);

    run(fx, out);
    const index = readIndex(out);
    const pair = ['src/shallow.ts::useShallow', 'src/shallow.ts::composeProjection'];
    const exact = index.exactBodyGroups.find((g) => pair.every((id) => g.identities.includes(id)));
    const structure = index.structureGroups.find((g) => pair.every((id) => g.identities.includes(id)));
    const near = index.nearFunctionCandidates?.find((g) => pair.every((id) => g.identities.includes(id)));
    const signature = index.signatureGroups?.find((g) => pair.every((id) => g.identities.includes(id)));

    assert('FC5a. same-signature helpers are not body clone groups',
      !exact && !structure,
      `exact=${JSON.stringify(index.exactBodyGroups)}\nstructure=${JSON.stringify(index.structureGroups)}`);
    assert('FC5b. same-signature helpers do not need near body cues',
      !near,
      JSON.stringify(index.nearFunctionCandidates));
    assert('FC5c. same function signature is surfaced as a review cue',
      !!signature &&
      index.meta.signatureGroupCount === 1 &&
      index.meta.supports?.functionSignatureGroups === true &&
      signature.risk === 'review-only' &&
      /not proof of semantic equivalence/.test(signature.reason ?? ''),
      JSON.stringify({ meta: index.meta, signatureGroups: index.signatureGroups }));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T6. Exact body clone grouping must not drop identical facts solely because
// the functions are small. Near/structure clone grouping can stay size-gated,
// but exact body hashes are already a precise grouping key.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-small-exact-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-small-exact-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-small-exact-fixture', type: 'module' }));
    write(fx, 'src/a.ts', `export function answerOne() { return 42; }\n`);
    write(fx, 'src/b.ts', `export function answerTwo() { return 42; }\n`);

    run(fx, out);
    const index = readIndex(out);
    const pair = ['src/a.ts::answerOne', 'src/b.ts::answerTwo'];
    const sameHashFacts = index.facts.filter((fact) =>
      pair.includes(fact.identity) &&
      fact.normalizedExactHash === index.facts.find((f) => f.identity === pair[0])?.normalizedExactHash);
    const exact = index.exactBodyGroups.find((g) => pair.every((id) => g.identities.includes(id)));

    assert('FC6a. fixture records two facts with the same normalized exact hash',
      sameHashFacts.length === 2,
      JSON.stringify(index.facts));
    assert('FC6b. exactBodyGroups includes small exact-body clones',
      !!exact,
      JSON.stringify(index.exactBodyGroups));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T7. File-local top-level helpers must be indexed for pre-write function
// signature cues. The exported API wrappers are intentionally unannotated so
// only the local helpers can satisfy the signature lookup.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-local-helper-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-local-helper-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-local-helper-fixture', type: 'module' }));
    write(fx, 'src/user-a.ts',
      `function normalizeUserName(raw: string): string {\n` +
      `  return raw.trim().toLowerCase();\n` +
      `}\n\n` +
      `export function callA(raw: string) {\n` +
      `  return normalizeUserName(raw);\n` +
      `}\n`);
    write(fx, 'src/user-b.ts',
      `const cleanUserName = (value: string): string => {\n` +
      `  return value.trim().toLowerCase();\n` +
      `};\n\n` +
      `export function callB(raw: string) {\n` +
      `  return cleanUserName(raw);\n` +
      `}\n`);

    run(fx, out);
    const index = readIndex(out);
    const localPair = ['src/user-a.ts::normalizeUserName', 'src/user-b.ts::cleanUserName'];
    const localFacts = index.facts.filter((fact) => localPair.includes(fact.identity));
    const signature = index.signatureGroups?.find((g) =>
      localPair.every((id) => g.identities.includes(id)));

    assert('FC7a. file-local top-level helpers are recorded as function facts',
      localFacts.length === 2 &&
        localFacts.every((fact) =>
          fact.visibility === 'file-local' &&
          fact.exported === false &&
          fact.normalizedSignatureHash),
      JSON.stringify(index.facts, null, 2));
    assert('FC7b. exported wrappers without explicit return types do not create signature facts',
      !index.facts.some((fact) =>
        ['src/user-a.ts::callA', 'src/user-b.ts::callB'].includes(fact.identity) &&
        fact.normalizedSignatureHash),
      JSON.stringify(index.facts, null, 2));
    assert('FC7c. file-local helper signatures are grouped as review cues',
      !!signature &&
        signature.risk === 'review-only' &&
        signature.visibilities?.includes('file-local'),
      JSON.stringify(index.signatureGroups, null, 2));
    assert('FC7d. meta advertises file-local top-level function support',
      index.meta.supports?.fileLocalTopLevelFunctions === true &&
        /file-local/.test(index.meta.scope),
      JSON.stringify(index.meta, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T8. Identifier-backed default exports are still exported public facts.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-default-alias-'));
  const out = mkdtempSync(path.join(tmpdir(), 'fn-clone-default-alias-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-default-alias-fixture', type: 'module' }));
    write(fx, 'src/default-fn.ts',
      `function normalizePayload(raw: string): string {\n` +
      `  return raw.trim().toLowerCase();\n` +
      `}\n\n` +
      `export default normalizePayload;\n`);
    write(fx, 'src/default-const.ts',
      `const serializePayload = (raw: string): string => {\n` +
      `  return raw.trim().toLowerCase();\n` +
      `};\n\n` +
      `export default serializePayload;\n`);

    run(fx, out);
    const index = readIndex(out);
    const defaultFn = index.facts.find((fact) => fact.identity === 'src/default-fn.ts::default');
    const defaultConst = index.facts.find((fact) => fact.identity === 'src/default-const.ts::default');

    assert('FC8a. function identifier default export is recorded as exported default',
      defaultFn?.visibility === 'exported' &&
        defaultFn.exported === true &&
        defaultFn.exportedName === 'default' &&
        defaultFn.localName === 'normalizePayload' &&
        defaultFn.normalizedSignatureHash,
      JSON.stringify(index.facts, null, 2));
    assert('FC8b. const-arrow identifier default export is recorded as exported default',
      defaultConst?.visibility === 'exported' &&
        defaultConst.exported === true &&
        defaultConst.exportedName === 'default' &&
        defaultConst.localName === 'serializePayload' &&
        defaultConst.normalizedSignatureHash,
      JSON.stringify(index.facts, null, 2));
    assert('FC8c. default-exported identifiers are not duplicated as file-local helpers',
      !index.facts.some((fact) =>
        fact.identity === 'src/default-fn.ts::normalizePayload' ||
        fact.identity === 'src/default-const.ts::serializePayload'),
      JSON.stringify(index.facts, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T9. The producer creates its output directory when run standalone.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'fn-clone-missing-output-'));
  const parent = mkdtempSync(path.join(tmpdir(), 'fn-clone-missing-output-parent-'));
  const out = path.join(parent, 'nested', 'audit-artifacts');
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fn-clone-missing-output-fixture', type: 'module' }));
    write(fx, 'src/a.ts', `export function ready(): boolean { return true; }\n`);

    run(fx, out, ['--no-incremental']);
    const index = readIndex(out);

    assert('FC9a. standalone producer creates a missing output directory',
      existsSync(path.join(out, 'function-clones.json')) &&
        index.facts.some((fact) => fact.identity === 'src/a.ts::ready'),
      JSON.stringify(index, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(parent, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
