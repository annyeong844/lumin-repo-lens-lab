// PCEF P3 bounded member-call resolution.
//
// Depth-1 member calls on imported exported objects can support call-graph
// evidence only when the target object surface is mechanically known. Unknown
// or deeper member calls must be counted as bounded-out so ranking does not
// overstate "no observed callers".

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

function runCallGraph(files) {
  const root = mkdtempSync(path.join(tmpdir(), 'lrl-call-bounded-root-'));
  const out = mkdtempSync(path.join(tmpdir(), 'lrl-call-bounded-out-'));
  try {
    for (const [rel, text] of Object.entries(files)) write(root, rel, text);
    execFileSync('node', ['build-call-graph.mjs', '--root', root, '--output', out], {
      cwd: DIR,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    return JSON.parse(readFileSync(path.join(out, 'call-graph.json'), 'utf8'));
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

{
  const artifact = runCallGraph({
    'src/lib.ts': [
      'export function actualRun() { return 1; }',
      'export default { run: actualRun };',
      '',
    ].join('\n'),
    'src/consumer.ts': [
      'import api from "./lib";',
      'api.run();',
      '',
    ].join('\n'),
  });

  assert('B1. default exported object member call maps to referenced function',
    artifact.callFanInByIdentity?.['src/lib.ts::actualRun'] === 1 &&
      artifact.meta?.supports?.boundedMemberCallResolution === true,
    JSON.stringify({
      fanIn: artifact.callFanInByIdentity,
      supports: artifact.meta?.supports,
    }));
}

{
  const artifact = runCallGraph({
    'src/lib.ts': [
      'export function actualRun() { return 1; }',
      'export const count = 1;',
      'export const tools = { run: actualRun, inline() { return 2; }, value: 1, count };',
      '',
    ].join('\n'),
    'src/consumer.ts': [
      'import { tools } from "./lib";',
      'tools.run();',
      'tools.inline();',
      'tools.value();',
      'tools.count();',
      '',
    ].join('\n'),
  });

  assert('B2. named exported object member calls map function properties only',
    artifact.callFanInByIdentity?.['src/lib.ts::actualRun'] === 1 &&
      artifact.callFanInByIdentity?.['src/lib.ts::inline'] === 1 &&
      artifact.callFanInByIdentity?.['src/lib.ts::value'] === undefined &&
      artifact.callFanInByIdentity?.['src/lib.ts::count'] === 0,
    JSON.stringify(artifact.callFanInByIdentity));
  assert('B3. non-function imported object member calls are bounded out',
    artifact.boundedOutMemberCallsByFile?.['src/consumer.ts'] === 2 &&
      artifact.memberCallsByFile?.['src/consumer.ts'] === 4,
    JSON.stringify({
      bounded: artifact.boundedOutMemberCallsByFile,
      total: artifact.memberCallsByFile,
    }));
}

{
  const artifact = runCallGraph({
    'src/lib.ts': [
      'export function actualRun() { return 1; }',
      'export default { run: actualRun };',
      '',
    ].join('\n'),
    'src/consumer.ts': [
      'import api from "./lib";',
      'api.run.deep();',
      '',
    ].join('\n'),
  });

  assert('B4. depth-2 imported object member calls are bounded out',
    artifact.callFanInByIdentity?.['src/lib.ts::actualRun'] === 0 &&
      artifact.boundedOutMemberCallsByFile?.['src/consumer.ts'] === 1 &&
      artifact.memberCallsByFile?.['src/consumer.ts'] === 1,
    JSON.stringify({
      fanIn: artifact.callFanInByIdentity,
      bounded: artifact.boundedOutMemberCallsByFile,
      total: artifact.memberCallsByFile,
    }));
}

console.log(`\n[test-call-graph-bounded] passed=${passed} failed=${failed}`);
if (failed) process.exit(1);
