// WT-15: class methods must be visible to pre-write search hints without
// becoming dead-export candidates.

import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { lookupName } from '../_lib/pre-write-lookup-name.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
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

const root = mkdtempSync(path.join(os.tmpdir(), 'lrl-class-method-prewrite-'));
const out = path.join(root, '.audit');

try {
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ private: true, type: 'module' }, null, 2));
  writeFileSync(path.join(root, 'src', 'event-dispatcher.ts'), [
    'class TaskControlEventDispatcher {',
    '  handleApiError(error: unknown) { return error; }',
    '  handleSignIn() { return "sign-in"; }',
    '  handleSignUp() { return "sign-up"; }',
    '  handleDelete(taskId: string) { return taskId; }',
    '  handleRegenerate() { return "regen"; }',
    '}',
    '',
    'export const taskControlEventDispatcher = new TaskControlEventDispatcher();',
    '',
  ].join('\n'));

  runNode(['build-symbol-graph.mjs', '--root', root, '--output', out]);
  const symbols = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  const fileMethods = symbols.classMethodIndex?.['src/event-dispatcher.ts'] ?? {};
  const handleDeleteMethods = fileMethods.handleDelete ?? [];

  assert('CM1. symbols.json advertises class method pre-write surface support',
    symbols.meta?.supports?.classMethodIndex === true,
    JSON.stringify(symbols.meta?.supports, null, 2));
  assert('CM2. class method index records handleDelete with class owner',
    handleDeleteMethods.some((m) =>
      m.className === 'TaskControlEventDispatcher' &&
      m.name === 'handleDelete' &&
      m.identity === 'src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete'),
    JSON.stringify(fileMethods, null, 2));
  assert('CM3. class methods do not get promoted into export defIndex',
    !symbols.defIndex?.['src/event-dispatcher.ts']?.handleDelete,
    JSON.stringify(symbols.defIndex?.['src/event-dispatcher.ts'], null, 2));

  const lookup = lookupName('handleBulkDelete', {
    symbols,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'handleBulkDelete',
      kind: 'function',
      why: 'extract a bulk delete event handler from class dispatch code',
    },
  });
  const near = lookup.nearNames ?? [];
  assert('CM4. pre-write nearNames can see class method handleDelete',
    near.some((n) =>
      n.name === 'handleDelete' &&
      n.ownerFile === 'src/event-dispatcher.ts' &&
      n.className === 'TaskControlEventDispatcher' &&
      n.identity === 'src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete' &&
      n.matchedField === 'classMethodIndex'),
    JSON.stringify(near, null, 2));
  assert('CM5. delete-domain method outranks unrelated handle-prefix methods',
    near[0]?.name === 'handleDelete',
    JSON.stringify(near, null, 2));
} finally {
  rmSync(root, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
