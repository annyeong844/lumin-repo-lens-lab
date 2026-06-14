// PCEF P2c: module-reachability.json records entry-rooted file reachability.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function writeFixture(dir) {
  const files = {
    'package.json': JSON.stringify({
      name: 'module-reachability-fixture',
      private: true,
      exports: {
        '.': './src/index.ts',
      },
    }, null, 2),
    'src/index.ts': [
      'import type { TypeOnly } from "./types";',
      'import { run } from "./runtime";',
      'export { run };',
      'export type IndexType = TypeOnly;',
    ].join('\n'),
    'src/runtime.ts': 'import { deep } from "./deep";\nexport const run = () => deep;\n',
    'src/deep.ts': 'export const deep = 1;\n',
    'src/types.ts': 'export interface TypeOnly { value: string }\n',
    'src/isolated.ts': 'export const isolated = true;\n',
    'src/components/App.ts': 'import { Modal } from "./Modal";\nexport function App() { return Modal; }\n',
    'src/components/Modal.ts': 'import { App } from "./App";\nexport function Modal() { return App; }\n',
  };

  for (const [name, content] of Object.entries(files)) {
    const file = path.join(dir, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
}

function makeFixtureRoot() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-module-reachability-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  writeFixture(dir);
  return { dir, out };
}

function runProducer(extraArgs = []) {
  const { dir, out } = makeFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-entry-surface.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });

    execFileSync(process.execPath, [
      path.join(ROOT, 'build-module-reachability.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
      ...extraArgs,
    ], { encoding: 'utf8' });

    return JSON.parse(readFileSync(path.join(out, 'module-reachability.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function runAuditRepo(profile = 'quick') {
  const { dir, out } = makeFixtureRoot();
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'audit-repo.mjs'),
      '--root', dir,
      '--output', out,
      '--profile', profile,
      '--production',
    ], { encoding: 'utf8' });
    return {
      manifest: JSON.parse(readFileSync(path.join(out, 'manifest.json'), 'utf8')),
      reachability: JSON.parse(readFileSync(path.join(out, 'module-reachability.json'), 'utf8')),
      summaryMd: readFileSync(path.join(out, 'audit-summary.latest.md'), 'utf8'),
      reviewPackMd: profile === 'quick'
        ? ''
        : readFileSync(path.join(out, 'audit-review-pack.latest.md'), 'utf8'),
    };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

const artifact = runProducer();
const runtimeReachable = new Set(artifact.runtimeReachableFiles ?? []);
const typeReachable = new Set(artifact.typeReachableFiles ?? []);
const reachable = new Set(artifact.reachableFiles ?? []);
const unreachable = new Set(artifact.unreachableFiles ?? []);
const boundedOut = new Set(artifact.boundedOutFiles ?? []);

assert('E1. artifact meta names build-module-reachability.mjs',
  artifact.meta?.tool === 'build-module-reachability.mjs',
  JSON.stringify(artifact.meta));
assert('E2. artifact meta points to entry-surface.json',
  artifact.meta?.entrySurfaceFile === 'entry-surface.json',
  JSON.stringify(artifact.meta));
assert('E3. runtime BFS reaches entry and transitive value dependency',
  runtimeReachable.has('src/index.ts') &&
    runtimeReachable.has('src/runtime.ts') &&
    runtimeReachable.has('src/deep.ts'),
  JSON.stringify(artifact.runtimeReachableFiles));
assert('E4. runtime BFS excludes type-only dependency',
  !runtimeReachable.has('src/types.ts'),
  JSON.stringify(artifact.runtimeReachableFiles));
assert('E5. type reachability includes type-only dependency',
  typeReachable.has('src/types.ts'),
  JSON.stringify(artifact.typeReachableFiles));
assert('E6. reachableFiles is the union of runtime and type reachability',
  reachable.has('src/index.ts') &&
    reachable.has('src/runtime.ts') &&
    reachable.has('src/deep.ts') &&
    reachable.has('src/types.ts'),
  JSON.stringify(artifact.reachableFiles));
assert('E7. isolated file is unreachable when BFS is complete',
  unreachable.has('src/isolated.ts') && !boundedOut.has('src/isolated.ts'),
  JSON.stringify({ unreachable: artifact.unreachableFiles, boundedOut: artifact.boundedOutFiles }));
assert('E8. clean run does not mark bounded out files',
  artifact.meta?.boundedOutReason === null && artifact.summary?.boundedOut === 0,
  JSON.stringify({ meta: artifact.meta, summary: artifact.summary }));
assert('E9. submodule completeness is copied from entry-surface',
  artifact.meta?.completenessBySubmodule?.src === 'high',
  JSON.stringify(artifact.meta?.completenessBySubmodule));
assert('E10. artifact declares unreachable SCC support',
  artifact.meta?.supports?.unreachableStronglyConnectedComponents === true,
  JSON.stringify(artifact.meta?.supports));
assert('E11. entry-unreachable runtime SCC is recorded as review evidence',
  (artifact.unreachableStronglyConnectedComponents ?? []).some((component) =>
    Array.isArray(component?.files) &&
    component.files.join('|') === 'src/components/App.ts|src/components/Modal.ts' &&
    component.kind === 'entry-unreachable-scc' &&
    component.graph === 'runtime'),
  JSON.stringify(artifact.unreachableStronglyConnectedComponents));
assert('E12. unreachable SCC summary counts groups and files',
  artifact.summary?.unreachableStronglyConnectedComponents === 1 &&
    artifact.summary?.unreachableStronglyConnectedFiles === 2,
  JSON.stringify(artifact.summary));

const capped = runProducer(['--max-files-visited', '1']);
assert('E13. emergency cap records boundedOutReason',
  capped.meta?.boundedOutReason === 'max-files-visited',
  JSON.stringify(capped.meta));
assert('E14. cap sends unvisited files to boundedOut, not unreachable',
  (capped.boundedOutFiles ?? []).includes('src/isolated.ts') &&
    !(capped.unreachableFiles ?? []).includes('src/isolated.ts'),
  JSON.stringify({ boundedOut: capped.boundedOutFiles, unreachable: capped.unreachableFiles }));

const audit = runAuditRepo();
assert('E15. audit-repo quick profile runs build-module-reachability.mjs',
  audit.manifest.commandsRun?.some((step) =>
    step.step === 'build-module-reachability.mjs' && step.status === 'ok'),
  JSON.stringify(audit.manifest.commandsRun));
assert('E16. audit-repo artifactsProduced lists module-reachability.json',
  audit.manifest.artifactsProduced?.includes('module-reachability.json'),
  JSON.stringify(audit.manifest.artifactsProduced));
assert('E17. pipeline reachability keeps isolated file unreachable',
  audit.reachability.unreachableFiles?.includes('src/isolated.ts'),
  JSON.stringify(audit.reachability.unreachableFiles));
assert('E18. audit summary surfaces unreachable SCCs as review evidence',
  audit.summaryMd.includes('Unreachable SCCs: 1 group, 2 files') &&
    audit.summaryMd.includes('module-reachability.json.unreachableStronglyConnectedComponents') &&
    audit.summaryMd.includes('before treating intra-cycle imports as liveness'),
  audit.summaryMd);

const fullAudit = runAuditRepo('full');
assert('E19. audit review pack mirrors unreachable SCC review cue in dead-surface lane',
  fullAudit.reviewPackMd.includes('Unreachable SCCs: 1 group, 2 files') &&
    fullAudit.reviewPackMd.includes('module-reachability.json.unreachableStronglyConnectedComponents') &&
    fullAudit.reviewPackMd.includes('dead-file-group review evidence, not export SAFE_FIX'),
  fullAudit.reviewPackMd);

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
