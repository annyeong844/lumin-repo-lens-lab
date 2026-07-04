#!/usr/bin/env node
// build-entry-surface.mjs - PCEF P2 entry file surface artifact.

import path from 'node:path';

import { atomicWrite } from './_lib/atomic-write.mjs';
import { loadIfExists } from './_lib/artifacts.mjs';
import { parseCliArgs } from './_lib/cli.mjs';
import { detectRepoMode } from './_lib/repo-mode.mjs';
import { collectEntrySurfaceFacts } from './_lib/entry-surface.mjs';
import { projectEntrySurfaceArtifact } from './_lib/entry-surface-artifact.mjs';

const cli = parseCliArgs({});
const ROOT = cli.root;
const OUTPUT = cli.output;

const symbolsData = loadIfExists(OUTPUT, 'symbols.json', { tag: 'build-entry-surface' });
if (!symbolsData) {
  console.error('[entry-surface] symbols.json is required. Run build-symbol-graph.mjs first.');
  process.exit(1);
}

const facts = collectEntrySurfaceFacts({
  root: ROOT,
  repoMode: detectRepoMode(ROOT),
  symbolsData,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
});
const artifact = projectEntrySurfaceArtifact(facts);

const outPath = path.join(OUTPUT, 'entry-surface.json');
atomicWrite(outPath, JSON.stringify(artifact, null, 2));

console.log('\n══════ entry surface ══════');
console.log(`  public API     : ${artifact.publicApiFiles.length}`);
console.log(`  scripts        : ${artifact.scriptEntrypointFiles.length}`);
console.log(`  HTML modules   : ${artifact.htmlEntrypointFiles.length}`);
console.log(`  framework      : ${artifact.frameworkEntrypointFiles.length}`);
console.log(`  config         : ${artifact.configEntrypointFiles.length}`);
console.log(`  wrote          : ${outPath}`);
