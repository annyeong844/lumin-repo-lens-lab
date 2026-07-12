#!/usr/bin/env node
// post-write.mjs — P2-1 CLI entry for the post-write delta.
//
// Flags per docs/history/phases/p2/p2-1.md v3 §4.6:
//   --root <path>                repository root (required)
//   --output <dir>               artifact dir (source of after-inventory)
//   --pre-write-advisory <file>  advisory JSON (required)
//   --delta-out <dir>            where delta artifact lands (defaults to --output)
//   --no-fresh-audit             skip the Rust after-snapshot refresh
//   --no-incremental / --cache-root / --clear-incremental-cache
//                                control the shared strict Rust per-file cache
//   --include-tests / --no-include-tests / --production / --exclude
//                                forwarded to Rust source discovery
//
// Exit codes:
//   0 — delta computed (may contain silent-new; acknowledgment is a caller concern)
//   1 — fatal error (missing flag, unreadable advisory, malformed JSON)
//
// CLI generates deltaInvocationId locally and injects into computeDelta —
// keeps computeDelta pure (docs/history/phases/p2/p2-1.md v3 §4.1 purity contract).

import { readFileSync, existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { parseCliArgs } from './_lib/cli.mjs';
import { loadIfExists } from './_lib/artifacts.mjs';
import { computeDelta } from './_lib/post-write-delta.mjs';
import { computeFileDelta, repoRelativeFileList } from './_lib/post-write-file-delta.mjs';
import { collectFiles } from './_lib/collect-files.mjs';
import { buildRustPostWriteInventory } from './_lib/post-write-rust-inventory.mjs';
import { renderMarkdown } from './_lib/post-write-render.mjs';
import { writeDelta, generateDeltaInvocationId } from './_lib/post-write-artifact.mjs';

function die(msg, code = 1) {
  process.stderr.write(`[post-write] ${msg}\n`);
  process.exit(code);
}

// ── Parse args ───────────────────────────────────────────────

const args = parseCliArgs({
  'pre-write-advisory': { type: 'string' },
  'delta-out': { type: 'string' },
  'no-fresh-audit': { type: 'boolean', default: false },
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
});

const advisoryFlag = args.raw?.['pre-write-advisory'];
if (!advisoryFlag) die('--pre-write-advisory <file> is required');

const ROOT = args.root;
const OUTPUT = args.output;
const DELTA_OUT = args.raw?.['delta-out']
  ? path.resolve(args.raw['delta-out'])
  : OUTPUT;
if (DELTA_OUT !== OUTPUT) mkdirSync(DELTA_OUT, { recursive: true });

const noFreshAudit = args.raw?.['no-fresh-audit'] === true;

// ── Load advisory ───────────────────────────────────────────

const advisoryPath = path.resolve(advisoryFlag);
if (!existsSync(advisoryPath)) die(`advisory file not found: ${advisoryPath}`);

let preWriteAdvisory;
try { preWriteAdvisory = JSON.parse(readFileSync(advisoryPath, 'utf8')); }
catch (e) { die(`advisory parse failed: ${e.message}`); }

function typeEscapeDeltaNotApplicable(advisory) {
  return advisory?.capabilities?.postWriteTypeEscapes === 'not-applicable' ||
    advisory?.capabilities?.language === 'rust' ||
    advisory?.intent?.language === 'rust' ||
    Boolean(advisory?.rustPreWrite);
}

const skipTypeEscapeDelta = typeEscapeDeltaNotApplicable(preWriteAdvisory);
const deltaInvocationId = generateDeltaInvocationId();

// ── Rust-owned after-snapshot ──────────────────────────────

let freshInventory = null;
if (!noFreshAudit && !skipTypeEscapeDelta) {
  process.stderr.write('[post-write] running lumin-audit-core for after-snapshot\n');
  try {
    freshInventory = buildRustPostWriteInventory({
      root: ROOT,
      output: OUTPUT,
      deltaInvocationId,
      includeTests: args.includeTests,
      exclude: args.exclude,
      noIncremental: args.raw?.['no-incremental'] === true,
      cacheRoot: args.raw?.['cache-root'],
      clearIncrementalCache: args.raw?.['clear-incremental-cache'] === true,
    });
  } catch (e) {
    die(`Rust after-inventory failed: ${e?.message?.slice(0, 400) ?? 'unknown'}`);
  }
}

// ── Load after-inventory ────────────────────────────────────

const afterInventory = skipTypeEscapeDelta
  ? null
  : noFreshAudit
    ? loadIfExists(OUTPUT, 'any-inventory.json', { tag: 'post-write' })
    : freshInventory?.inventory ?? null;

// ── Load before-inventory via advisory.preWrite.anyInventoryPath ─

function uniqueTruthy(values) {
  const out = [];
  const seen = new Set();
  for (const value of values) {
    if (!value) continue;
    const resolved = path.resolve(String(value));
    if (seen.has(resolved)) continue;
    seen.add(resolved);
    out.push(resolved);
  }
  return out;
}

let beforeInventory = null;
const beforeRelPath = preWriteAdvisory?.preWrite?.anyInventoryPath;
if (!skipTypeEscapeDelta && beforeRelPath) {
  const beforeDirs = uniqueTruthy([
    path.dirname(advisoryPath),
    preWriteAdvisory?.scanRange?.output,
    OUTPUT,
  ]);
  for (const dir of beforeDirs) {
    beforeInventory = loadIfExists(dir, beforeRelPath, { tag: 'post-write' });
    if (beforeInventory) break;
  }
}

// ── Compute delta ───────────────────────────────────────────
//
// deltaInvocationId is generated HERE (CLI-level) and injected into
// computeDelta, which keeps computeDelta pure per §4.1.

let afterFiles = null;
let afterFileScanFailure = null;
if (freshInventory) {
  afterFiles = freshInventory.files;
} else {
  try {
    afterFiles = repoRelativeFileList(ROOT, collectFiles(ROOT, {
      includeTests: args.includeTests,
      exclude: args.exclude,
    }));
  } catch (e) {
    afterFileScanFailure = e?.message?.slice(0, 400) ?? 'unknown';
  }
}

const fileDelta = computeFileDelta({
  root: ROOT,
  plannedFiles: preWriteAdvisory?.intent?.files ?? [],
  beforeFiles: preWriteAdvisory?.preWrite?.fileInventory?.status === 'available'
    ? preWriteAdvisory.preWrite.fileInventory.files
    : null,
  afterFiles,
  afterScanFailure: afterFileScanFailure,
});

const delta = {
  ...computeDelta({
    preWriteAdvisory,
    beforeInventory,
    afterInventory,
    deltaInvocationId,
  }),
  schemaVersion: 'lumin-post-write-delta.v1',
  fileDelta,
};

// ── Write artifact + emit Markdown ──────────────────────────

writeDelta(DELTA_OUT, delta);
const md = renderMarkdown(delta);
process.stdout.write(md);

process.exit(0);
