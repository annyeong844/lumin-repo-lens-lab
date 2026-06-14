// PCEF P1 export-action-safety producer.
//
// These tests pin the contract that SAFE_FIX is backed by a concrete safe
// edit action. Deadness alone is not enough: deletion needs local-use and
// side-effect proof, while demotion can preserve runtime behavior.

import { execFileSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, text) {
  const p = path.join(root, rel);
  mkdirSync(path.dirname(p), { recursive: true });
  writeFileSync(p, text);
}

function runFixture(files, proposals, buckets = {}) {
  const root = mkdtempSync(path.join(tmpdir(), 'pcef-action-root-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pcef-action-out-'));
  try {
    for (const [rel, text] of Object.entries(files)) write(root, rel, text);
    writeFileSync(path.join(out, 'dead-classify.json'), JSON.stringify({
      proposal_C_remove_symbol: buckets.C ?? proposals,
      proposal_A_demote_to_internal: buckets.A ?? [],
      proposal_B_review: buckets.B ?? [],
      proposal_remove_export_specifier: [],
    }, null, 2));
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      defIndex: {},
      fanInByIdentity: {},
    }, null, 2));

    execFileSync('node', ['export-action-safety.mjs', '--root', root, '--output', out], {
      cwd: DIR,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    return JSON.parse(readFileSync(path.join(out, 'export-action-safety.json'), 'utf8'));
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

const common = {
  line: 1,
  kind: 'VariableDeclaration',
  bucket: 'C',
};

{
  const artifact = runFixture({
    'src/token.ts': 'export const token = registerTelemetry();\n',
  }, [{ ...common, file: 'src/token.ts', symbol: 'token' }]);
  const action = artifact.findings[0].safeAction;
  assert('A1. side-effect initializer selects demote action',
    action?.kind === 'demote_export_declaration',
    JSON.stringify(action));
  assert('A1b. side-effect initializer has no selected-action blockers',
    Array.isArray(action?.actionBlockers) && action.actionBlockers.length === 0,
    JSON.stringify(action));
  assert('A1c. side-effect initializer only blocks stronger delete action',
    action?.strongerActionBlockers?.includes('side-effect-initializer'),
    JSON.stringify(action));
}

{
  const artifact = runFixture({
    'src/size.ts': 'export const SIZE = 12;\nexport const buttonSize = SIZE + 4;\n',
  }, [{ ...common, file: 'src/size.ts', symbol: 'SIZE' }]);
  const action = artifact.findings[0].safeAction;
  assert('A2. local value refs preserve binding via demote',
    action?.kind === 'demote_export_declaration',
    JSON.stringify(action));
  assert('A2b. local value refs block stronger delete only',
    action?.strongerActionBlockers?.includes('local-refs-present'),
    JSON.stringify(action));
}

{
  const artifact = runFixture({
    'src/types.ts': 'export type Options = { debug: boolean };\nconst defaults: Options = { debug: false };\n',
  }, [{ ...common, file: 'src/types.ts', symbol: 'Options', kind: 'TSTypeAliasDeclaration' }]);
  const action = artifact.findings[0].safeAction;
  assert('A3. local type refs preserve type binding via demote',
    action?.kind === 'demote_export_declaration',
    JSON.stringify(action));
  assert('A3b. local type refs block type deletion only',
    action?.strongerActionBlockers?.includes('local-refs-present'),
    JSON.stringify(action));
}

{
  const artifact = runFixture({
    'src/dead-type.ts': 'export interface InternalOptions { debug: boolean }\n',
  }, [{ ...common, file: 'src/dead-type.ts', symbol: 'InternalOptions', kind: 'TSInterfaceDeclaration' }]);
  const action = artifact.findings[0].safeAction;
  assert('A4. unreferenced interface can delete type declaration',
    action?.kind === 'delete_type_declaration',
    JSON.stringify(action));
}

{
  const proposal = {
    ...common,
    bucket: 'B',
    file: 'src/public-types.ts',
    symbol: 'Internal',
    kind: 'TSTypeAliasDeclaration',
    declarationExportDependency: true,
    declarationExportRefs: { count: 1, lines: [2] },
    fileInternalRefs: { typeRefs: 1, valueRefs: 0 },
  };
  const artifact = runFixture({
    'src/public-types.ts':
      'export type Internal = string;\n' +
      'export interface PublicThing { value: Internal }\n',
  }, [], { C: [], B: [proposal] });
  const action = artifact.findings[0].safeAction;
  assert('A4b. B bucket local type declaration dependency gets demote action',
    action?.kind === 'demote_export_declaration',
    JSON.stringify(artifact.findings[0]));
  assert('A4c. B bucket declaration dependency blocks stronger delete only',
    action?.strongerActionBlockers?.includes('local-refs-present'),
    JSON.stringify(action));
}

{
  const artifact = runFixture({
    'src/multi.ts': 'export const a = 1, b = 2;\n',
  }, [{ ...common, file: 'src/multi.ts', symbol: 'a' }]);
  const action = artifact.findings[0].safeAction;
  assert('A5. partial multi-declarator has no safe action in v1',
    action === null,
    JSON.stringify(artifact.findings[0]));
  assert('A5b. partial multi-declarator records action blocker',
    artifact.findings[0].actionBlockers?.includes('partial-multi-declarator'),
    JSON.stringify(artifact.findings[0]));
}

{
  const artifact = runFixture({
    'src/reexport.ts': 'export { value } from "./source";\n',
  }, [{ ...common, file: 'src/reexport.ts', symbol: 'value', kind: 'ExportSpecifier' }]);
  assert('A6. re-export-from-source remains review in v1',
    artifact.findings[0].safeAction === null &&
    artifact.findings[0].actionBlockers?.includes('re-export-from-source'),
    JSON.stringify(artifact.findings[0]));
}

{
  const artifact = runFixture({
    'src/only.ts': 'export const only = 1;\n',
  }, [{ ...common, file: 'src/only.ts', symbol: 'only' }]);
  const action = artifact.findings[0].safeAction;
  assert('A7. last export safe action includes module marker patch',
    action?.requiresModuleMarker === true &&
    action?.edits?.some((e) => e.kind === 'insert' && e.text.includes('export {};')),
    JSON.stringify(action));
}

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}
console.log(`\n${passed} passed, 0 failed`);
