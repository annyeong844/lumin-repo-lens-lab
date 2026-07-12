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
import { collectFiles } from './_lib/collect-files.mjs';
import { JS_FAMILY_LANGS } from './_lib/lang.mjs';
import { relPath } from './_lib/paths.mjs';
import { parseOxcOrThrow } from './_lib/parse-oxc.mjs';
import { computeLineStarts, lineOf } from './_lib/line-offset.mjs';
import { classifyFileRole } from './_lib/test-paths.mjs';

const cli = parseCliArgs();
const { root: ROOT, output: OUT, verbose } = cli;

const topology = loadIfExists(OUT, 'topology.json', { tag: 'checklist-facts' });
const deadClassify = loadIfExists(OUT, 'dead-classify.json', { tag: 'checklist-facts' });
const fixPlan = loadIfExists(OUT, 'fix-plan.json', { tag: 'checklist-facts' });
const barrels = loadIfExists(OUT, 'barrels.json', { tag: 'checklist-facts' });
const triage = loadIfExists(OUT, 'triage.json', { tag: 'checklist-facts' });
const shapeIndex = loadIfExists(OUT, 'shape-index.json', { tag: 'checklist-facts' });
const functionClones = loadIfExists(OUT, 'function-clones.json', { tag: 'checklist-facts' });

const files = collectFiles(ROOT, {
  languages: JS_FAMILY_LANGS,
  includeTests: cli.includeTests,
  exclude: cli.exclude,
});

if (verbose) console.error(`[checklist-facts] scanning ${files.length} files`);

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

function collectFunctionSizeFacts() {
  const entries = [];
  let parseErrors = 0;

  for (const file of files) {
    let src, result;
    try {
      src = readFileSync(file, 'utf8');
      result = parseOxcOrThrow(file, src);
    } catch {
      parseErrors++;
      continue;
    }
    const lineStarts = computeLineStarts(src);
    const relativeFile = relPath(ROOT, file);
    const fileRole = classifyFileRole(relativeFile);

    walkAst(result.program, (node, parent, parentKey) => {
      if (!isFunctionNode(node)) return;
      const startLine = lineOf(lineStarts, node.start ?? 0);
      const endLine = lineOf(lineStarts, node.end ?? 0);
      entries.push({
        file: relativeFile,
        line: startLine,
        name: getFnName(node, parent, parentKey),
        loc: Math.max(1, endLine - startLine + 1),
        fileRole,
      });
    });
  }

  return { entries, parseErrors };
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

function collectSilentCatchFacts() {
  const sites = [];
  const documentedSites = [];
  const anonymousSites = [];
  const nonEmptyAnonymousSites = [];
  const unusedParamSites = [];
  let parseErrors = 0;

  for (const file of files) {
    let src, result;
    try {
      src = readFileSync(file, 'utf8');
      result = parseOxcOrThrow(file, src);
    } catch {
      parseErrors++;
      continue;
    }
    const lineStarts = computeLineStarts(src);
    const comments = result.comments ?? [];
    const relativeFile = relPath(ROOT, file);
    const fileRole = classifyFileRole(relativeFile);

    walkAst(result.program, (node) => {
      if (node.type !== 'CatchClause' || !node.body || !Array.isArray(node.body.body)) return;
      const site = {
        file: relativeFile,
        line: lineOf(lineStarts, node.start ?? 0),
        fileRole,
        bodyStatementCount: node.body.body.length,
      };
      const hasParam = Boolean(node.param);
      const paramName = catchParamName(node);
      if (!hasParam) anonymousSites.push(site);
      if (node.body.body.length === 0) {
        if (catchBodyHasComment(node, comments)) documentedSites.push(site);
        else sites.push(site);
      } else if (!hasParam) {
        nonEmptyAnonymousSites.push(site);
      } else if (paramName && !catchBodyReferencesParam(node.body, paramName)) {
        unusedParamSites.push({ ...site, paramName });
      }
    });
  }

  return {
    analysis: 'oxc-ast-catch-clause',
    parseErrors,
    sites,
    documentedSites,
    anonymousSites,
    nonEmptyAnonymousSites,
    unusedParamSites,
  };
}

const request = {
  schemaVersion: 'lumin-checklist-facts-producer-request.v1',
  generated: new Date().toISOString(),
  root: ROOT,
  filesScanned: files.length,
  inputs: {
    topology,
    deadClassify,
    fixPlan,
    barrels,
    triage,
    shapeIndex,
    functionClones,
  },
  astFacts: {
    functionSize: collectFunctionSizeFacts(),
    silentCatch: collectSilentCatchFacts(),
  },
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
