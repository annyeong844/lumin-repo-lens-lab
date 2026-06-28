// PCEF P3 canonical definition ids.
//
// These tests pin the cross-producer identity contract: symbols.json owns the
// canonical definition id for an exported identity, and call-graph.json must
// resolve call fan-in through the same id even when the exported name is an
// alias for a local declaration.

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

function runFixture(files) {
  const root = mkdtempSync(path.join(tmpdir(), 'pcef-defid-root-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pcef-defid-out-'));
  try {
    for (const [rel, text] of Object.entries(files)) write(root, rel, text);
    execFileSync('node', ['build-symbol-graph.mjs', '--root', root, '--output', out], {
      cwd: DIR,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    execFileSync('node', ['build-call-graph.mjs', '--root', root, '--output', out], {
      cwd: DIR,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    writeFileSync(path.join(out, 'dead-classify.json'), JSON.stringify({
      proposal_C_remove_symbol: [{
        file: 'src/lib.ts',
        symbol: 'publicApi',
        localName: 'impl',
        line: 4,
        kind: 'ExportSpecifier',
      }],
      proposal_A_demote_to_internal: [],
      proposal_B_review: [],
      proposal_remove_export_specifier: [],
    }, null, 2));
    execFileSync('node', ['export-action-safety.mjs', '--root', root, '--output', out], {
      cwd: DIR,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    return {
      symbols: JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8')),
      calls: JSON.parse(readFileSync(path.join(out, 'call-graph.json'), 'utf8')),
      actionSafety: JSON.parse(readFileSync(path.join(out, 'export-action-safety.json'), 'utf8')),
    };
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

{
  const { symbols, calls, actionSafety } = runFixture({
    'src/lib.ts': [
      'function impl() {',
      '  return 1;',
      '}',
      'export { impl as publicApi };',
      '',
    ].join('\n'),
    'src/consumer.ts': [
      'import { publicApi } from "./lib";',
      'publicApi();',
      '',
    ].join('\n'),
  });

  const symbolDefId = symbols.defIndex?.['src/lib.ts']?.publicApi?.definitionId;
  const aliasDefId = calls.exportAliasMap?.['src/lib.ts::publicApi'];
  const actionDefId = actionSafety.findings?.[0]?.safeAction?.target?.definitionId;

  assert('D1. symbols.json emits canonical definitionId for export alias',
    typeof symbolDefId === 'string' &&
    /^src\/lib\.ts#FunctionDeclaration:\d+-\d+$/.test(symbolDefId),
    JSON.stringify(symbols.defIndex?.['src/lib.ts']?.publicApi));
  assert('D2. call graph exportAliasMap uses the same definitionId',
    typeof aliasDefId === 'string' && aliasDefId === symbolDefId,
    JSON.stringify({ symbolDefId, aliasDefId, exportAliasMap: calls.exportAliasMap }));
  assert('D3. callFanInByDefinitionId counts calls through aliased export',
    calls.callFanInByDefinitionId?.[symbolDefId] === 1,
    JSON.stringify(calls.callFanInByDefinitionId));
  assert('D4. callFanInByIdentity also counts the exported identity',
    calls.callFanInByIdentity?.['src/lib.ts::publicApi'] === 1,
    JSON.stringify(calls.callFanInByIdentity));
  assert('D5. export-action-safety target uses the same definitionId',
    actionDefId === symbolDefId,
    JSON.stringify({ symbolDefId, actionDefId, finding: actionSafety.findings?.[0] }));
}

console.log(`\n[test-definition-id-canonical] passed=${passed} failed=${failed}`);
if (failed) process.exit(1);
