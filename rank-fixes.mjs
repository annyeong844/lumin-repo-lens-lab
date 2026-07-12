#!/usr/bin/env node
// Build fix-plan.json from already-produced audit artifacts.
//
// JS owns artifact loading/writing and package.json public deep-import risk
// discovery. lumin-audit-core owns finding flattening, tier classification,
// support evidence projection, sorting, summary, and safeFixGroups.

import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { parseCliArgs } from './_lib/cli.mjs';
import { loadIfExists as loadArtifact } from './_lib/artifacts.mjs';
import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';
import {
  findNearestPackageInfo,
  getPublicDeepImportRisk,
} from './_lib/package-exports.mjs';

const RANK_FIXES_REQUEST_SCHEMA_VERSION = 'lumin-rank-fixes-producer-request.v1';

const { root, output } = parseCliArgs();
const ROOT = path.resolve(root);
const OUT = path.resolve(output);

const loadIfExists = (name) => loadArtifact(OUT, name, { tag: 'rank-fixes' });

const deadClassify = loadIfExists('dead-classify.json');
if (!deadClassify) {
  console.error('[rank-fixes] dead-classify.json is required. Run classify-dead-exports.mjs first.');
  process.exit(1);
}

const runtimeEvidence = loadIfExists('runtime-evidence.json');
const staleness = loadIfExists('staleness.json');
const symbols = loadIfExists('symbols.json');
const exportActionSafety = loadIfExists('export-action-safety.json');
const callGraph = loadIfExists('call-graph.json');
const entrySurface = loadIfExists('entry-surface.json');
const moduleReachability = loadIfExists('module-reachability.json');

function normalizeRel(file) {
  return String(file ?? '').replace(/\\/g, '/').replace(/^\.\//, '');
}

function addFindingFiles(files, list) {
  for (const item of list ?? []) {
    if (typeof item?.file === 'string' && item.file.length > 0) {
      files.add(normalizeRel(item.file));
    }
  }
}

function collectDeadClassifyFiles(artifact) {
  const files = new Set();
  addFindingFiles(files, artifact?.proposal_C_remove_symbol);
  addFindingFiles(files, artifact?.proposal_A_demote_to_internal);
  addFindingFiles(files, artifact?.proposal_B_review);
  addFindingFiles(files, artifact?.proposal_remove_export_specifier);
  addFindingFiles(files, artifact?.proposal_DEGRADED_unprocessed);
  addFindingFiles(files, artifact?.excludedCandidates);
  return [...files].sort();
}

function publicDeepImportRiskForFile(file) {
  const packageInfo = findNearestPackageInfo(ROOT, file);
  if (!packageInfo?.packageJson) {
    return { risk: false, reason: 'package-json-absent', relFileFromPkgRoot: file };
  }
  return getPublicDeepImportRisk(packageInfo.packageJson, packageInfo.relFileFromPkgRoot);
}

function buildPublicDeepImportRiskByFile(deadClassifyArtifact) {
  const entries = {};
  for (const file of collectDeadClassifyFiles(deadClassifyArtifact)) {
    entries[file] = publicDeepImportRiskForFile(file);
  }
  if (Object.keys(entries).length === 0) {
    entries.__lumin_empty_fix_plan__ = { risk: false, reason: 'empty-fix-plan' };
  }
  return entries;
}

const inputs = {
  'dead-classify.json': true,
  'runtime-evidence.json': !!runtimeEvidence,
  'staleness.json': !!staleness,
  'symbols.json': !!symbols,
  'export-action-safety.json': !!exportActionSafety,
  'call-graph.json': !!callGraph,
  'entry-surface.json': !!entrySurface,
  'module-reachability.json': !!moduleReachability,
};

const artifact = runAuditCoreJsonResultFile(
  ['rank-fixes-artifact', '--input', '-'],
  'rank-fixes-artifact',
  {
    input: JSON.stringify({
      schemaVersion: RANK_FIXES_REQUEST_SCHEMA_VERSION,
      root: ROOT,
      generated: new Date().toISOString(),
      artifacts: {
        deadClassify,
        runtimeEvidence,
        staleness,
        symbols,
        exportActionSafety,
        callGraph,
        entrySurface,
        moduleReachability,
      },
      publicDeepImportRiskByFile: buildPublicDeepImportRiskByFile(deadClassify),
    }),
  }
);

const outPath = path.join(OUT, 'fix-plan.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));

console.log('\n══════ fix-plan ranking ══════');
console.log(`  SAFE_FIX    : ${artifact.summary?.SAFE_FIX ?? 0}  (auto-fix candidates)`);
console.log(`  REVIEW_FIX  : ${artifact.summary?.REVIEW_FIX ?? 0}  (human review recommended)`);
console.log(`  DEGRADED    : ${artifact.summary?.DEGRADED ?? 0}  (evidence insufficient — not a hard warning)`);
console.log(`  MUTED       : ${artifact.summary?.MUTED ?? 0}  (policy-excluded — not a finding)`);
console.log(`  total       : ${artifact.summary?.total ?? 0}`);

const resolver = artifact.meta?.resolverBlindness;
const degradedByUnresolvedSpec = (artifact.degraded ?? []).filter((entry) =>
  typeof entry?.reason === 'string' && entry.reason.startsWith('unresolved-spec-could-match')
).length;
if (resolver && resolver.gate === 'tripped') {
  console.log(`\n  ⚠ resolver unresolvedRatio = ${(resolver.ratio * 100).toFixed(1)}%`);
  if (degradedByUnresolvedSpec > 0) {
    console.log(`    ${degradedByUnresolvedSpec} finding(s) DEGRADED by per-finding spec match — add a tsconfig path or alias to reduce.`);
  } else {
    console.log('    No finding matched an unresolved specifier locally; global ratio is informational only.');
  }
}

if ((artifact.summary?.SAFE_FIX ?? 0) > 0) {
  console.log('\n── SAFE_FIX top entries ──');
  for (const entry of (artifact.safeFixes ?? []).slice(0, 5)) {
    console.log(`  ${entry.finding.file}:${entry.finding.line}  ${entry.finding.symbol}  (${entry.reason})`);
  }
  if ((artifact.safeFixGroups ?? []).length > 0) {
    console.log('\n── SAFE_FIX grouped patterns ──');
    for (const group of artifact.safeFixGroups.slice(0, 5)) {
      const sample = group.symbols.slice(0, 4).join(', ');
      const suffix = group.symbols.length > 4 ? ', ...' : '';
      console.log(`  ${group.count}×  ${group.actionKind}  ${group.file}  (${sample}${suffix})`);
    }
  }
}

console.log(`\n[rank-fixes] saved → ${outPath}`);
