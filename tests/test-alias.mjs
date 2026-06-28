// Tests for Issue 6: `export { local as publicName }` classification.
// Baseline bug: classifier says "정의 자체 제거 가능" for aliased dead exports,
// but the actual definition under the exported name doesn't exist — the local
// symbol might be still in use within the file.
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const DIR = path.resolve(__dirname, '..');
const FX = '/tmp/alias-fx';
const OUT = '/tmp/alias-test';

// ── Build fixture hermetically (no external dependencies) ──
rmSync(FX, { recursive: true, force: true });
mkdirSync(path.join(FX, 'src'), { recursive: true });
writeFileSync(path.join(FX, 'package.json'), '{"name":"alias-fx","type":"module"}');
// Aliased, local still in use elsewhere in file
writeFileSync(path.join(FX, 'src/aliased_local_used.ts'),
  'function foo() { return 42; }\n' +
  'export function localConsumer() { return foo() + 1; }\n' +
  'export { foo as publicThing };\n');
// Aliased, local also unused
writeFileSync(path.join(FX, 'src/aliased_local_dead.ts'),
  'function bar() { return 42; }\n' +
  'export { bar as publicThing };\n');
// Non-aliased re-export (control)
writeFileSync(path.join(FX, 'src/non_aliased.ts'),
  'function helper() { return 1; }\n' +
  'export { helper };\n');
// Consumer keeps at least one import alive so FP-23 doesn't collapse everything
writeFileSync(path.join(FX, 'src/consumer.ts'),
  "import { localConsumer } from './aliased_local_used';\n" +
  'export const _keepAlive = localConsumer;\n');

let passed = 0, failed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function run(cmd) {
  try {
    return execSync(cmd, { cwd: DIR, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (e) {
    return (e.stdout || '') + (e.stderr || '');
  }
}

// v1.8.5: in development or on a bare checkout, `oxc-parser` may be
// missing. Without this check, the artifact read below fails with an
// opaque ENOENT and the test appears to fail for mysterious reasons.
// Fail fast and surface the underlying error instead.
function runChecked(cmd) {
  try {
    return execSync(cmd, { cwd: DIR, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (e) {
    console.error(`[${path.basename(process.argv[1])}] pipeline step failed:`);
    console.error(`  cmd: ${cmd}`);
    if (e.stdout) console.error(`  stdout: ${String(e.stdout).slice(0, 500)}`);
    if (e.stderr) console.error(`  stderr: ${String(e.stderr).slice(0, 500)}`);
    console.error(`\nHint: if this is "Cannot find package 'oxc-parser'", run \`npm install\` first.`);
    process.exit(1);
  }
}

// Run symbol graph + classify
rmSync(OUT, { recursive: true, force: true });
runChecked(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`);
runChecked(`node classify-dead-exports.mjs --root ${FX} --output ${OUT}`);

const symbols = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
const classify = JSON.parse(readFileSync(path.join(OUT, 'dead-classify.json'), 'utf8'));

// ── T1: symbol graph records localName for aliased ExportSpecifier ─
{
  const aliased = symbols.deadProdList.find(
    (d) => d.file === 'src/aliased_local_used.ts' && d.symbol === 'publicThing',
  );
  assert(
    'T1. symbols.deadProdList entry for aliased export has localName field',
    aliased && aliased.localName === 'foo',
    `got: ${JSON.stringify(aliased)}`,
  );
}

// ── T2: non-aliased ExportSpecifier does NOT set localName (or sets equal) ─
{
  const nonAliased = symbols.deadProdList.find(
    (d) => d.file === 'src/non_aliased.ts' && d.symbol === 'helper',
  );
  assert(
    'T2. symbols.deadProdList entry for non-aliased has no distinct localName',
    nonAliased && (!nonAliased.localName || nonAliased.localName === 'helper'),
    `got: ${JSON.stringify(nonAliased)}`,
  );
}

// ── T3: C-class proposal for aliased-local-used must NOT say "정의 자체 제거" ─
{
  const cProposal = classify.proposal_C_remove_symbol || [];
  const aliasedDead = cProposal.find(
    (c) => c.file === 'src/aliased_local_used.ts' && c.symbol === 'publicThing',
  );
  if (!aliasedDead) {
    // Not necessarily in C — might be routed elsewhere. Check A and any new bucket.
    const aProposal = classify.proposal_A_demote_to_internal || [];
    const inA = aProposal.find(
      (c) => c.file === 'src/aliased_local_used.ts' && c.symbol === 'publicThing',
    );
    const specProposal = classify.proposal_remove_export_specifier || [];
    const inSpec = specProposal.find(
      (c) => c.file === 'src/aliased_local_used.ts' && c.symbol === 'publicThing',
    );
    assert(
      'T3. aliased_local_used publicThing routed to specifier-aware bucket (not C with definition-removal action)',
      Boolean(inSpec) || Boolean(inA),
      `not found in A or specifier bucket. classify keys: ${Object.keys(classify)}`,
    );
  } else {
    // If still in C, the action text must reflect specifier semantics, not "definition removal"
    assert(
      'T3. if in C, action text must NOT claim "정의 자체 제거"',
      !/정의 자체 제거/.test(aliasedDead.action),
      `got action: "${aliasedDead.action}"`,
    );
  }
}

// ── T4: proposal must surface localName so user knows what to check ─
{
  const allProposals = [
    ...(classify.proposal_C_remove_symbol || []),
    ...(classify.proposal_A_demote_to_internal || []),
    ...(classify.proposal_remove_export_specifier || []),
  ];
  const aliased = allProposals.find(
    (c) => c.file === 'src/aliased_local_used.ts' && c.symbol === 'publicThing',
  );
  assert(
    'T4. aliased export proposal carries localName = "foo"',
    aliased && aliased.localName === 'foo',
    `got: ${JSON.stringify(aliased)}`,
  );
}

// ── T5: when local is dead too, proposal must note that (informational) ─
{
  const allProposals = [
    ...(classify.proposal_C_remove_symbol || []),
    ...(classify.proposal_A_demote_to_internal || []),
    ...(classify.proposal_remove_export_specifier || []),
  ];
  const aliasedBothDead = allProposals.find(
    (c) => c.file === 'src/aliased_local_dead.ts' && c.symbol === 'publicThing',
  );
  assert(
    'T5. aliased_local_dead publicThing proposal includes local-also-dead signal',
    aliasedBothDead && (aliasedBothDead.localAlsoDead === true || aliasedBothDead.localInternalUses === 0),
    `got: ${JSON.stringify(aliasedBothDead)}`,
  );
}

// ── T6: occurrence count for aliased kind is based on LOCAL name, not exported ─
// In aliased_local_used.ts, 'internal' appears 2 times (def + call); 'publicThing' appears 1 time (export line).
// Post-fix: fileInternalUses should count via 'internal' → should be > 0.
{
  const allProposals = [
    ...(classify.proposal_C_remove_symbol || []),
    ...(classify.proposal_A_demote_to_internal || []),
    ...(classify.proposal_remove_export_specifier || []),
  ];
  const aliased = allProposals.find(
    (c) => c.file === 'src/aliased_local_used.ts' && c.symbol === 'publicThing',
  );
  assert(
    'T6. aliased_local_used: internal use count reflects LOCAL name (>0)',
    aliased && (aliased.localInternalUses ?? aliased.fileInternalUses ?? 0) > 0,
    `got: ${JSON.stringify(aliased)}`,
  );
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
