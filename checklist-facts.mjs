#!/usr/bin/env node
// checklist-facts.mjs - JS AST fact collector plus Rust-owned artifact projection.
//
// JS keeps the source walking and OXC AST pass because those are still JS/TS
// producer semantics. lumin-audit-core owns checklist-facts.json gates,
// citation hints, deferred-item vocabulary, and deterministic artifact shape.

import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

import { runAuditCoreJsonResultFile } from './_lib/audit-core.mjs';
import { parseCliArgs } from './_lib/cli.mjs';
import { loadIfExists } from './_lib/artifacts.mjs';
import {
  buildContextFingerprint,
  buildRepoSnapshot,
  STRICT_IDENTITY_MODE,
} from './_lib/incremental-snapshot.mjs';
import {
  clearIncrementalCache,
  getReusableFact,
  loadProducerCache,
  openIncrementalCacheStore,
  putFact,
  saveProducerCache,
  strictCacheKeyForEntry,
} from './_lib/incremental-cache-store.mjs';
import { JS_FAMILY_LANGS } from './_lib/lang.mjs';
import { parseOxcOrThrow } from './_lib/parse-oxc.mjs';
import { computeLineStarts, lineOf } from './_lib/line-offset.mjs';
import { classifyFileRole } from './_lib/test-paths.mjs';

const cli = parseCliArgs({
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
});
const { root: ROOT, output: OUT, verbose } = cli;

const PRODUCER_ID = 'checklist-facts';
const PRODUCER_VERSION = 1;
const FACT_SCHEMA_VERSION = 1;
const PARSER_IDENTITY = 'checklist-facts:oxc-function-size-silent-catch-v1';

const topology = loadIfExists(OUT, 'topology.json', { tag: 'checklist-facts' });
const deadClassify = loadIfExists(OUT, 'dead-classify.json', { tag: 'checklist-facts' });
const fixPlan = loadIfExists(OUT, 'fix-plan.json', { tag: 'checklist-facts' });
const barrels = loadIfExists(OUT, 'barrels.json', { tag: 'checklist-facts' });
const triage = loadIfExists(OUT, 'triage.json', { tag: 'checklist-facts' });
const shapeIndex = loadIfExists(OUT, 'shape-index.json', { tag: 'checklist-facts' });
const functionClones = loadIfExists(OUT, 'function-clones.json', { tag: 'checklist-facts' });

const incrementalEnabled = cli.raw?.['no-incremental'] !== true;
const contextFingerprint = buildContextFingerprint({
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  languages: JS_FAMILY_LANGS,
  producerContext: {
    producer: PRODUCER_ID,
    producerVersion: PRODUCER_VERSION,
    factSchemaVersion: FACT_SCHEMA_VERSION,
    parserIdentity: PARSER_IDENTITY,
  },
});
const snapshot = buildRepoSnapshot({
  root: ROOT,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
  languages: JS_FAMILY_LANGS,
  contextFingerprint,
  hashContents: incrementalEnabled,
});
const snapshotEntries = Object.values(snapshot.files);
const cacheStore = openIncrementalCacheStore({
  root: ROOT,
  cacheRoot: cli.raw?.['cache-root'],
});
if (cli.raw?.['clear-incremental-cache'] === true) {
  clearIncrementalCache(cacheStore);
}
const producerCacheMeta = {
  producerId: PRODUCER_ID,
  producerVersion: PRODUCER_VERSION,
  factSchemaVersion: FACT_SCHEMA_VERSION,
  parserIdentity: PARSER_IDENTITY,
  scanFingerprint: contextFingerprint,
  configFingerprint: contextFingerprint,
};
const priorCache = incrementalEnabled
  ? loadProducerCache(cacheStore, PRODUCER_ID)
  : { entries: {}, meta: { loadStatus: 'disabled' } };
const nextCache = { entries: {}, meta: { loadStatus: 'new' } };

if (verbose) console.error(`[checklist-facts] scanning ${snapshotEntries.length} files`);

function isFunctionNode(n) {
  return n.type === 'FunctionDeclaration' ||
         n.type === 'FunctionExpression' ||
         n.type === 'ArrowFunctionExpression';
}

function getFnName(node, parent, _key) {
  if (node.id?.name) return node.id.name;
  if (parent?.type === 'VariableDeclarator' && parent.id?.type === 'Identifier') {
    return parent.id.name;
  }
  if (parent?.type === 'Property' && parent.key?.name) return parent.key.name;
  if ((parent?.type === 'MethodDefinition' || parent?.type === 'PropertyDefinition' ||
       parent?.type === 'AccessorProperty') && parent.key?.name) {
    return parent.key.name;
  }
  if (parent?.type === 'AssignmentExpression' && parent.left?.type === 'Identifier') {
    return parent.left.name;
  }
  return '<anonymous>';
}

function walkAst(node, visit, parent = null, parentKey = null) {
  if (!node || typeof node !== 'object') return;
  visit(node, parent, parentKey);
  for (const k of Object.keys(node)) {
    if (k === 'type' || k === 'start' || k === 'end' ||
        k === 'loc' || k === 'range' || k === 'parent') continue;
    const v = node[k];
    if (Array.isArray(v)) {
      for (const c of v) {
        if (c && typeof c === 'object' && typeof c.type === 'string') {
          walkAst(c, visit, node, k);
        }
      }
    } else if (v && typeof v === 'object' && typeof v.type === 'string') {
      walkAst(v, visit, node, k);
    }
  }
}

function catchBodyHasComment(node, comments) {
  const bodyStart = node.body?.start;
  const bodyEnd = node.body?.end;
  if (typeof bodyStart !== 'number' || typeof bodyEnd !== 'number') return false;
  return comments.some((comment) =>
    typeof comment.start === 'number' &&
    typeof comment.end === 'number' &&
    comment.start > bodyStart &&
    comment.end < bodyEnd &&
    String(comment.value ?? '').trim().length > 0);
}

function catchParamName(node) {
  return node?.param?.type === 'Identifier' ? node.param.name : null;
}

function isIdentifierReference(node, parent, key) {
  if (!node || node.type !== 'Identifier') return false;
  if (!parent) return true;

  if ((parent.type === 'VariableDeclarator' ||
       parent.type === 'FunctionDeclaration' ||
       parent.type === 'FunctionExpression' ||
       parent.type === 'ClassDeclaration' ||
       parent.type === 'ClassExpression') &&
      key === 'id') return false;
  if ((parent.type === 'FunctionDeclaration' ||
       parent.type === 'FunctionExpression' ||
       parent.type === 'ArrowFunctionExpression') &&
      key === 'params') return false;
  if ((parent.type === 'Property' || parent.type === 'MethodDefinition' ||
       parent.type === 'PropertyDefinition' || parent.type === 'AccessorProperty') &&
      key === 'key' && parent.computed !== true) return false;
  if (parent.type === 'MemberExpression' && key === 'property' &&
      parent.computed !== true) return false;
  if (parent.type === 'LabeledStatement' && key === 'label') return false;
  if (parent.type === 'BreakStatement' && key === 'label') return false;
  if (parent.type === 'ContinueStatement' && key === 'label') return false;
  if (parent.type === 'ImportSpecifier' || parent.type === 'ImportDefaultSpecifier' ||
      parent.type === 'ImportNamespaceSpecifier' || parent.type === 'ExportSpecifier') return false;

  return true;
}

function catchBodyReferencesParam(body, name) {
  if (!body || !name) return false;
  let found = false;

  walkAst(body, (node, parent, key) => {
    if (found) return;
    if (node.type === 'Identifier' && node.name === name &&
        isIdentifierReference(node, parent, key)) {
      found = true;
    }
  });
  return found;
}

function emptyFileAstFacts(parseError = false) {
  return {
    parseError,
    functionEntries: [],
    sites: [],
    documentedSites: [],
    anonymousSites: [],
    nonEmptyAnonymousSites: [],
    unusedParamSites: [],
  };
}

function validFileRole(role) {
  return role === 'production' || role === 'test' || role === 'script';
}

function validSite(site, relativeFile, requireParam = false) {
  return site && typeof site === 'object' &&
    site.file === relativeFile &&
    Number.isSafeInteger(site.line) && site.line >= 1 &&
    validFileRole(site.fileRole) &&
    Number.isSafeInteger(site.bodyStatementCount) && site.bodyStatementCount >= 0 &&
    (!requireParam || (typeof site.paramName === 'string' && site.paramName.length > 0));
}

function validFileAstFacts(payload, relativeFile) {
  if (!payload || typeof payload !== 'object' || typeof payload.parseError !== 'boolean') {
    return false;
  }
  const arrays = [
    'functionEntries',
    'sites',
    'documentedSites',
    'anonymousSites',
    'nonEmptyAnonymousSites',
    'unusedParamSites',
  ];
  if (!arrays.every((field) => Array.isArray(payload[field]))) return false;
  if (payload.parseError) return arrays.every((field) => payload[field].length === 0);

  return payload.functionEntries.every((entry) =>
    entry && typeof entry === 'object' &&
    entry.file === relativeFile &&
    Number.isSafeInteger(entry.line) && entry.line >= 1 &&
    typeof entry.name === 'string' &&
    Number.isSafeInteger(entry.loc) && entry.loc >= 1 &&
    validFileRole(entry.fileRole)) &&
    payload.sites.every((site) => validSite(site, relativeFile)) &&
    payload.documentedSites.every((site) => validSite(site, relativeFile)) &&
    payload.anonymousSites.every((site) => validSite(site, relativeFile)) &&
    payload.nonEmptyAnonymousSites.every((site) => validSite(site, relativeFile)) &&
    payload.unusedParamSites.every((site) => validSite(site, relativeFile, true));
}

function collectFileAstFacts(entry) {
  if (!entry.readable) return emptyFileAstFacts(true);

  let src, result;
  try {
    src = readFileSync(entry.absPath, 'utf8');
    result = parseOxcOrThrow(entry.absPath, src);
  } catch {
    return emptyFileAstFacts(true);
  }

  const payload = emptyFileAstFacts();
  const lineStarts = computeLineStarts(src);
  const relativeFile = entry.relPath;
  const fileRole = classifyFileRole(relativeFile);

  walkAst(result.program, (node, parent, parentKey) => {
    if (isFunctionNode(node)) {
      const startLine = lineOf(lineStarts, node.start ?? 0);
      const endLine = lineOf(lineStarts, node.end ?? 0);
      payload.functionEntries.push({
        file: relativeFile,
        line: startLine,
        name: getFnName(node, parent, parentKey),
        loc: Math.max(1, endLine - startLine + 1),
        fileRole,
      });
    }

    if (node.type !== 'CatchClause' || !node.body || !Array.isArray(node.body.body)) {
      return;
    }
    const site = {
      file: relativeFile,
      line: lineOf(lineStarts, node.start ?? 0),
      fileRole,
      bodyStatementCount: node.body.body.length,
    };
    const hasParam = Boolean(node.param);
    const paramName = catchParamName(node);
    if (!hasParam) payload.anonymousSites.push(site);
    if (node.body.body.length === 0) {
      if (catchBodyHasComment(node, result.comments ?? [])) {
        payload.documentedSites.push(site);
      } else {
        payload.sites.push(site);
      }
    } else if (!hasParam) {
      payload.nonEmptyAnonymousSites.push(site);
    } else if (paramName && !catchBodyReferencesParam(node.body, paramName)) {
      payload.unusedParamSites.push({ ...site, paramName });
    }
  });

  return payload;
}

function collectAstFacts() {
  const functionEntries = [];
  const sites = [];
  const documentedSites = [];
  const anonymousSites = [];
  const nonEmptyAnonymousSites = [];
  const unusedParamSites = [];
  let parseErrors = 0;
  let changedFiles = 0;
  let reusedFiles = 0;
  let invalidatedFiles = 0;
  const currentStrictKeys = new Set();

  const appendPayload = (payload) => {
    if (payload.parseError) parseErrors++;
    functionEntries.push(...payload.functionEntries);
    sites.push(...payload.sites);
    documentedSites.push(...payload.documentedSites);
    anonymousSites.push(...payload.anonymousSites);
    nonEmptyAnonymousSites.push(...payload.nonEmptyAnonymousSites);
    unusedParamSites.push(...payload.unusedParamSites);
  };

  for (const entry of snapshotEntries) {
    currentStrictKeys.add(strictCacheKeyForEntry(entry));
    const reuse = incrementalEnabled
      ? getReusableFact(priorCache, { snapshotEntry: entry, producerMeta: producerCacheMeta })
      : { status: 'miss', reason: 'disabled-by-flag' };

    if (reuse.status === 'hit' && validFileAstFacts(reuse.payload, entry.relPath)) {
      reusedFiles++;
      appendPayload(reuse.payload);
      putFact(nextCache, {
        snapshotEntry: entry,
        producerMeta: producerCacheMeta,
        payload: reuse.payload,
      });
      continue;
    }

    if (
      reuse.reason !== 'missing-entry' &&
      reuse.reason !== 'disabled-by-flag' &&
      reuse.reason !== 'current-file-unreadable'
    ) {
      invalidatedFiles++;
    } else if (reuse.status === 'hit') {
      invalidatedFiles++;
    }
    changedFiles++;
    const payload = collectFileAstFacts(entry);
    appendPayload(payload);
    if (incrementalEnabled && entry.readable) {
      putFact(nextCache, {
        snapshotEntry: entry,
        producerMeta: producerCacheMeta,
        payload,
      });
    }
  }

  const droppedFiles = Object.keys(priorCache.entries ?? {})
    .filter((key) => !currentStrictKeys.has(key)).length;
  if (incrementalEnabled) {
    saveProducerCache(cacheStore, PRODUCER_ID, nextCache);
  }

  return {
    astFacts: {
      functionSize: {
        entries: functionEntries,
        parseErrors,
      },
      silentCatch: {
        analysis: 'oxc-ast-catch-clause',
        parseErrors,
        sites,
        documentedSites,
        anonymousSites,
        nonEmptyAnonymousSites,
        unusedParamSites,
      },
    },
    incremental: {
      enabled: incrementalEnabled,
      identityMode: incrementalEnabled ? STRICT_IDENTITY_MODE : null,
      cacheVersion: 1,
      cacheRoot: incrementalEnabled ? cacheStore.cacheRoot : null,
      changedFiles,
      reusedFiles,
      droppedFiles,
      invalidatedFiles,
      reason: incrementalEnabled ? null : 'disabled-by-flag',
    },
  };
}

const collected = collectAstFacts();

const request = {
  schemaVersion: 'lumin-checklist-facts-producer-request.v1',
  generated: new Date().toISOString(),
  root: ROOT,
  filesScanned: snapshotEntries.length,
  inputs: {
    topology,
    deadClassify,
    fixPlan,
    barrels,
    triage,
    shapeIndex,
    functionClones,
  },
  astFacts: collected.astFacts,
  incremental: collected.incremental,
};

const artifact = runAuditCoreJsonResultFile(
  ['checklist-facts-artifact', '--input', '-'],
  'checklist-facts',
  { input: JSON.stringify(request) },
);

const outPath = path.join(OUT, 'checklist-facts.json');
writeFileSync(outPath, JSON.stringify(artifact, null, 2));

console.log('\n══════ checklist-facts ══════');
const gateOf = (s) => s?.gate ?? 'unknown';
const gateIcon = (g) => g === 'fix' ? '❌' : g === 'watch' ? '⚠' : g === 'ok' ? '✅' : '·';
for (const [key, val] of Object.entries(artifact)) {
  if (key === 'meta' || key === '_not_computed') continue;
  const g = gateOf(val);
  console.log(`  ${gateIcon(g)}  ${key.padEnd(28)} gate=${g}`);
}
console.log(`\n[checklist-facts] saved → ${outPath}`);
console.log(`[checklist-facts] ${artifact._not_computed.length} items deferred to the checklist walker.`);
if (incrementalEnabled) {
  console.log(
    `[checklist-facts] incremental: ${collected.incremental.changedFiles} changed, ` +
    `${collected.incremental.reusedFiles} reused, ${collected.incremental.droppedFiles} dropped, ` +
    `${collected.incremental.invalidatedFiles} invalidated`,
  );
}
