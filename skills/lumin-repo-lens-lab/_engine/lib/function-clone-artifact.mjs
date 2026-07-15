// _lib/function-clone-artifact.mjs - JS/TS function-clone fact extraction.
//
// JavaScript owns OXC parsing and normalized per-function facts. Rust owns
// grouping, near-candidate scoring, policy metadata, and final artifact
// projection. Function-clone evidence remains review-only and does not prove
// semantic equivalence.

import { createHash } from 'node:crypto';

import { parseOxcOrThrow } from './parse-oxc.mjs';
import { computeLineStarts, lineOf } from './line-offset.mjs';
import { detectGeneratedFileEvidence } from './shape-hash.mjs';
import { functionSignatureFromFunctionNode } from './function-signature-hash.mjs';

const SKIP_KEYS = new Set([
  'start',
  'end',
  'loc',
  'range',
  'parent',
  'typeAnnotation',
  'returnType',
  'typeParameters',
  'declare',
  'accessibility',
  'shorthand',
]);

function sourceSlice(src, node) {
  if (!node || typeof node.start !== 'number' || typeof node.end !== 'number') return '';
  return src.slice(node.start, node.end);
}

function stableValue(value) {
  if (Array.isArray(value)) return value.map(stableValue);
  if (!value || typeof value !== 'object') return value;
  const out = {};
  for (const key of Object.keys(value).sort()) out[key] = stableValue(value[key]);
  return out;
}

function stableJson(value) {
  return JSON.stringify(stableValue(value));
}

function hash(value) {
  return 'sha256:' + createHash('sha256').update(stableJson(value)).digest('hex');
}

function compactSource(src) {
  return String(src ?? '').replace(/\s+/g, ' ').trim();
}

function exactBodyHash(src) {
  return 'sha256:' + createHash('sha256').update(compactSource(src)).digest('hex');
}

function isFunctionLike(node) {
  return node?.type === 'FunctionDeclaration' ||
    node?.type === 'FunctionExpression' ||
    node?.type === 'ArrowFunctionExpression';
}

function isFunctionishVariableDeclarator(declarator) {
  return declarator?.id?.type === 'Identifier' && isFunctionLike(declarator.init);
}

function exportedAliases(program) {
  const aliases = new Map();
  function add(localName, exportedName) {
    if (!localName || !exportedName) return;
    if (!aliases.has(localName)) aliases.set(localName, new Set());
    aliases.get(localName).add(exportedName);
  }

  for (const stmt of program?.body ?? []) {
    if (stmt?.type === 'ExportDefaultDeclaration' && stmt.declaration?.type === 'Identifier') {
      add(stmt.declaration.name, 'default');
      continue;
    }

    if (stmt?.type === 'ExportNamedDeclaration' && !stmt.source && !stmt.declaration) {
      for (const spec of stmt.specifiers ?? []) {
        if (spec?.type !== 'ExportSpecifier') continue;
        add(spec.local?.name, spec.exported?.name ?? spec.local?.name);
      }
    }
  }
  return aliases;
}

function topLevelFunctions(program) {
  const out = [];
  const aliases = exportedAliases(program);

  function addEntry({
    fn,
    localName,
    exportedName,
    declarationKind,
    visibility = 'exported',
    exported = true,
  }) {
    if (!fn || !exportedName) return;
    out.push({
      fn,
      localName: localName ?? exportedName,
      exportedName,
      declarationKind,
      visibility,
      exported,
    });
  }

  function addFileLocal({ fn, localName, declarationKind }) {
    if (!fn || !localName) return;
    addEntry({
      fn,
      localName,
      exportedName: localName,
      declarationKind,
      visibility: 'file-local',
      exported: false,
    });
  }

  for (const stmt of program?.body ?? []) {
    if (stmt?.type === 'ExportNamedDeclaration' && stmt.declaration) {
      const d = stmt.declaration;
      if (d.type === 'FunctionDeclaration') {
        addEntry({
          fn: d,
          localName: d.id?.name,
          exportedName: d.id?.name,
          declarationKind: d.type,
        });
        continue;
      }
      if (d.type === 'VariableDeclaration') {
        for (const decl of d.declarations ?? []) {
          if (!isFunctionishVariableDeclarator(decl)) continue;
          addEntry({
            fn: decl.init,
            localName: decl.id.name,
            exportedName: decl.id.name,
            declarationKind: d.kind ?? 'VariableDeclaration',
          });
        }
        continue;
      }
    }

    if (stmt?.type === 'ExportDefaultDeclaration' && isFunctionLike(stmt.declaration)) {
      addEntry({
        fn: stmt.declaration,
        localName: stmt.declaration.id?.name ?? 'default',
        exportedName: 'default',
        declarationKind: stmt.declaration.type,
      });
      continue;
    }

    if (stmt?.type === 'FunctionDeclaration') {
      const localName = stmt.id?.name;
      const exportedNames = aliases.get(localName);
      if (exportedNames) {
        for (const exportedName of exportedNames) {
          addEntry({ fn: stmt, localName, exportedName, declarationKind: stmt.type });
        }
      } else {
        addFileLocal({ fn: stmt, localName, declarationKind: stmt.type });
      }
      continue;
    }

    if (stmt?.type === 'VariableDeclaration') {
      for (const decl of stmt.declarations ?? []) {
        if (!isFunctionishVariableDeclarator(decl)) continue;
        const exportedNames = aliases.get(decl.id.name);
        if (exportedNames) {
          for (const exportedName of exportedNames) {
            addEntry({
              fn: decl.init,
              localName: decl.id.name,
              exportedName,
              declarationKind: stmt.kind ?? 'VariableDeclaration',
            });
          }
        } else {
          addFileLocal({
            fn: decl.init,
            localName: decl.id.name,
            declarationKind: stmt.kind ?? 'VariableDeclaration',
          });
        }
      }
    }
  }

  return out;
}

function shouldPreserveIdentifier(node, parent, key) {
  if (!node?.name) return false;
  if (parent?.type === 'MemberExpression' && key === 'property' && parent.computed !== true) return true;
  if ((parent?.type === 'Property' || parent?.type === 'MethodDefinition' ||
       parent?.type === 'PropertyDefinition' || parent?.type === 'AccessorProperty') &&
      key === 'key' && parent.computed !== true) return true;
  return false;
}

function normalizeLiteral(node, { preserveLiteralValues }) {
  const value = node?.value;
  if (preserveLiteralValues) {
    return {
      type: node.type,
      kind: value === null ? 'null' : typeof value,
      value: typeof value === 'bigint' ? value.toString() : value,
      regex: node.regex ?? undefined,
    };
  }
  return {
    type: node.type,
    kind: value === null ? 'null' : typeof value,
    regex: node.regex ? 'regex' : undefined,
  };
}

function normalizeTemplateElement(node, { preserveLiteralValues }) {
  if (preserveLiteralValues) {
    return {
      type: node.type,
      value: node.value?.raw ?? node.value?.cooked ?? '',
      tail: node.tail === true,
    };
  }
  return {
    type: node.type,
    kind: 'template-part',
    tail: node.tail === true,
  };
}

function normalizeNode(node, options = {}, parent = null, key = null) {
  if (Array.isArray(node)) return node.map((entry) => normalizeNode(entry, options, parent, key));
  if (!node || typeof node !== 'object') return node;

  if (node.type === 'Identifier') {
    return {
      type: 'Identifier',
      name: shouldPreserveIdentifier(node, parent, key) ? node.name : '$ID',
    };
  }
  if (node.type === 'PrivateIdentifier') return { type: 'PrivateIdentifier', name: '#ID' };
  if (node.type === 'ThisExpression') return { type: 'ThisExpression' };
  if (node.type === 'Super') return { type: 'Super' };
  if (node.type === 'Literal') return normalizeLiteral(node, options);
  if (node.type === 'TemplateElement') return normalizeTemplateElement(node, options);

  const out = { type: node.type };
  for (const k of Object.keys(node).sort()) {
    if (k === 'type' || SKIP_KEYS.has(k)) continue;
    const value = node[k];
    if (typeof value === 'function' || value === undefined) continue;
    out[k] = normalizeNode(value, options, node, k);
  }
  return out;
}

function functionBodyNode(fn) {
  return fn?.body ?? null;
}

function bodyStatementCount(fn) {
  const body = functionBodyNode(fn);
  if (!body) return 0;
  if (Array.isArray(body.body)) return body.body.length;
  return 1;
}

function collectCallTokens(body) {
  const tokens = new Set();

  function calleeName(callee) {
    if (callee?.type === 'Identifier') return callee.name;
    if (callee?.type === 'MemberExpression') {
      const prop = callee.property;
      if (!callee.computed && prop?.type === 'Identifier') return prop.name;
      if (prop?.type === 'Literal') return String(prop.value);
    }
    if (callee?.type === 'ChainExpression') return calleeName(callee.expression);
    if (callee?.type === 'NewExpression') return calleeName(callee.callee);
    return null;
  }

  function walk(node) {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'CallExpression' || node.type === 'NewExpression') {
      const name = calleeName(node.callee);
      if (name) tokens.add(name);
    }
    for (const k of Object.keys(node)) {
      if (k === 'type' || SKIP_KEYS.has(k)) continue;
      const v = node[k];
      if (Array.isArray(v)) {
        for (const c of v) if (c && typeof c === 'object') walk(c);
      } else if (v && typeof v === 'object') {
        walk(v);
      }
    }
  }

  walk(body);
  return [...tokens].sort();
}

function buildFunctionFact({ entry, src, ownerFile, lineStarts, scope }) {
  const {
    fn,
    exportedName,
    localName,
    declarationKind,
    visibility = 'exported',
    exported = true,
  } = entry;
  const body = functionBodyNode(fn);
  if (!body) return null;

  const startLine = lineOf(lineStarts, fn.start ?? 0);
  const endLine = lineOf(lineStarts, fn.end ?? 0);
  const bodyStartLine = lineOf(lineStarts, body.start ?? fn.start ?? 0);
  const bodyEndLine = lineOf(lineStarts, body.end ?? fn.end ?? 0);
  const identity = `${ownerFile}::${exportedName}`;
  const bodySource = sourceSlice(src, body);
  const normalizedExact = normalizeNode(body, { preserveLiteralValues: true });
  const normalizedStructure = normalizeNode(body, { preserveLiteralValues: false });
  const generatedFile = detectGeneratedFileEvidence(ownerFile, src);
  const signature = functionSignatureFromFunctionNode(fn, src);

  return {
    kind: 'function-body-fingerprint',
    identity,
    exportedName,
    localName,
    visibility,
    exported,
    ownerFile,
    line: startLine,
    endLine,
    bodyLineStart: bodyStartLine,
    bodyLineEnd: bodyEndLine,
    bodyLoc: Math.max(1, bodyEndLine - bodyStartLine + 1),
    declarationKind,
    functionKind: fn.type,
    async: fn.async === true,
    generator: fn.generator === true,
    paramCount: Array.isArray(fn.params) ? fn.params.length : 0,
    statementCount: bodyStatementCount(fn),
    exactBodyHash: exactBodyHash(bodySource),
    normalizedExactHash: hash(normalizedExact),
    normalizedStructureHash: hash(normalizedStructure),
    ...(signature.ok ? {
      normalizedSignatureHash: signature.hash,
      signature: signature.signature,
      signatureParamCount: signature.normalizedSignature.params.length,
    } : {}),
    callTokens: collectCallTokens(body),
    source: 'fresh-ast-pass',
    scope,
    confidence: 'high',
    ...(generatedFile ? { generatedFile } : {}),
  };
}

function readErrorDiagnostic(file, message) {
  return {
    kind: 'function-clone-diagnostic',
    code: 'read-error',
    severity: 'error',
    file,
    message,
  };
}

function parseErrorDiagnostic(file, message) {
  return {
    kind: 'function-clone-diagnostic',
    code: 'parse-error',
    severity: 'error',
    file,
    message,
  };
}

export function functionCloneReadErrorPayload(relFile, message) {
  return {
    facts: [],
    diagnostics: [readErrorDiagnostic(relFile, `read failed: ${message}`)],
    filesWithParseErrors: [],
    filesWithReadErrors: [{ file: relFile, message }],
  };
}

export function extractFunctionCloneFilePayload({ src, relFile, scope }) {
  let parsed;
  try {
    parsed = parseOxcOrThrow(relFile, src);
  } catch (e) {
    return {
      facts: [],
      diagnostics: [parseErrorDiagnostic(relFile, e.message)],
      filesWithParseErrors: [{ file: relFile, message: e.message }],
      filesWithReadErrors: [],
    };
  }

  const lineStarts = computeLineStarts(src);
  const facts = [];
  for (const entry of topLevelFunctions(parsed.program)) {
    const fact = buildFunctionFact({
      entry,
      src,
      ownerFile: relFile,
      lineStarts,
      scope,
    });
    if (fact) facts.push(fact);
  }

  return {
    facts,
    diagnostics: [],
    filesWithParseErrors: [],
    filesWithReadErrors: [],
  };
}
