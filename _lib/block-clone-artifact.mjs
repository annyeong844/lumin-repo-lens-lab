// _lib/block-clone-artifact.mjs - JS/TS block-clone tokenization boundary.
//
// JavaScript owns source parsing and normalized token production. The Rust
// block_clones owner constructs suffix arrays, repeated-region groups, noise
// policy, and the final review-only artifact.

import path from "node:path";

import { computeLineStarts, lineOf } from "./line-offset.mjs";
import { parseOxcOrThrow } from "./parse-oxc.mjs";
import { detectGeneratedFileEvidence } from "./shape-hash.mjs";

export const BLOCK_CLONE_SCHEMA_VERSION = "block-clones.v1";
export const BLOCK_CLONE_POLICY_VERSION = "block-clone-review-policy-v1";
export const BLOCK_CLONE_NORMALIZATION_POLICY_ID =
  "block-clone-normalization-v1";
export const BLOCK_CLONE_THRESHOLD_POLICY_ID =
  "block-clone-threshold-policy-v2";
export const BLOCK_CLONE_NOISE_POLICY_ID = "block-clone-noise-policy-v1";

export const DEFAULT_BLOCK_CLONE_THRESHOLDS = Object.freeze({
  minTokens: 50,
  minLines: 5,
  minOccurrences: 2,
  maxInstancesPerGroup: 20,
  maxCandidateGroups: 1000,
  maxReviewGroups: 100,
  maxMutedGroups: 100,
  maxTokensPerFile: 200000,
});

const SKIP_KEYS = new Set([
  "start",
  "end",
  "loc",
  "range",
  "parent",
  "typeAnnotation",
  "returnType",
  "typeParameters",
  "declare",
  "accessibility",
  "optional",
]);

const FUNCTION_TYPES = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
]);

const BUNDLED_PATH_RE =
  /(?:^|\/)(?:public\/vendor|.*(?:\.bundle|\.min)\.(?:js|jsx|mjs|cjs))$/;

function slashPath(value) {
  return String(value ?? "").replace(/\\/g, "/");
}

function nonNegativeInteger(value, fallback) {
  const number = Number(value);
  if (!Number.isFinite(number) || number < 0) return fallback;
  return Math.floor(number);
}

function maxTokensPerFile(thresholds) {
  return nonNegativeInteger(
    thresholds?.maxTokensPerFile,
    DEFAULT_BLOCK_CLONE_THRESHOLDS.maxTokensPerFile,
  );
}

function relPath(root, filePath) {
  return slashPath(path.relative(root, filePath));
}

function isGeneratedOrBundled(relFile, src) {
  const generated = detectGeneratedFileEvidence(relFile, src);
  if (generated) return generated;
  if (BUNDLED_PATH_RE.test(slashPath(relFile))) {
    return {
      kind: "bundled-build-artifact",
      source: "path",
      evidence: "path:bundled-build-artifact",
    };
  }
  return null;
}

function literalToken(node) {
  if (!node) return null;
  if (node.type === "StringLiteral" || typeof node.value === "string")
    return "LIT:STRING";
  if (node.type === "NumericLiteral" || typeof node.value === "number")
    return "LIT:NUMBER";
  if (node.type === "BooleanLiteral" || typeof node.value === "boolean")
    return "LIT:BOOLEAN";
  if (node.type === "NullLiteral" || node.value === null) return "LIT:NULL";
  if (node.type === "RegExpLiteral") return "LIT:REGEXP";
  return null;
}

function isFunctionNode(node) {
  return FUNCTION_TYPES.has(node?.type);
}

function functionName(node, parent) {
  if (node?.id?.name) return node.id.name;
  if (
    parent?.type === "VariableDeclarator" &&
    parent.id?.type === "Identifier"
  ) {
    return parent.id.name;
  }
  if (parent?.type === "Property" && parent.key?.type === "Identifier") {
    return parent.key.name;
  }
  return null;
}

function isPropertyNameIdentifier(parent, key) {
  if (!parent) return false;
  if (
    parent.type === "MemberExpression" &&
    key === "property" &&
    parent.computed !== true
  ) {
    return true;
  }
  if (
    (parent.type === "Property" ||
      parent.type === "MethodDefinition" ||
      parent.type === "PropertyDefinition" ||
      parent.type === "AccessorProperty") &&
    key === "key" &&
    parent.computed !== true
  ) {
    return true;
  }
  return false;
}

function collectBindingIdentifiers(node, out = []) {
  if (!node || typeof node !== "object") return out;
  if (node.type === "Identifier" && node.name) {
    out.push(node);
    return out;
  }
  if (node.type === "AssignmentPattern") {
    collectBindingIdentifiers(node.left, out);
    return out;
  }
  if (node.type === "RestElement") {
    collectBindingIdentifiers(node.argument, out);
    return out;
  }
  if (node.type === "ArrayPattern") {
    for (const element of node.elements ?? []) {
      collectBindingIdentifiers(element, out);
    }
    return out;
  }
  if (node.type === "ObjectPattern") {
    for (const property of node.properties ?? []) {
      if (!property) continue;
      if (property.type === "RestElement") {
        collectBindingIdentifiers(property.argument, out);
      } else if (property.type === "Property") {
        collectBindingIdentifiers(property.value, out);
      }
    }
    return out;
  }
  if (node.type === "TSParameterProperty") {
    collectBindingIdentifiers(node.parameter, out);
  }
  return out;
}

function createScope(parent = null) {
  return {
    parent,
    nextSlot: 0,
    names: new Map(),
  };
}

function declare(scope, name) {
  if (!name) return null;
  if (!scope.names.has(name)) {
    scope.names.set(name, `$${scope.nextSlot++}`);
  }
  return scope.names.get(name);
}

function resolve(scope, name) {
  for (let current = scope; current; current = current.parent) {
    if (current.names.has(name)) return current.names.get(name);
  }
  return null;
}

function statementLineCount(node, lineStarts) {
  if (!node || typeof node.start !== "number" || typeof node.end !== "number")
    return 0;
  return Math.max(
    1,
    lineOf(lineStarts, node.end) - lineOf(lineStarts, node.start) + 1,
  );
}

export function tokenizeBlockCloneSource({
  root,
  filePath,
  src,
  thresholds = DEFAULT_BLOCK_CLONE_THRESHOLDS,
}) {
  const tokenLimit = maxTokensPerFile(thresholds);
  const relFile = relPath(root, filePath);
  const skipped = isGeneratedOrBundled(relFile, src);
  if (skipped) {
    return {
      relFile,
      tokens: [],
      skipped: {
        file: relFile,
        reason: skipped.kind,
        evidence: skipped.evidence,
      },
      diagnostics: [],
    };
  }

  let parsed;
  try {
    parsed = parseOxcOrThrow(filePath, src);
  } catch (error) {
    return {
      relFile,
      tokens: [],
      skipped: null,
      diagnostics: [
        {
          file: relFile,
          kind: "parse-error",
          message: error?.message ?? String(error),
        },
      ],
    };
  }

  const lineStarts = computeLineStarts(src);
  const tokens = [];
  let scope = createScope();
  let container = null;
  let skippedTokenCount = 0;

  function emit(value, node) {
    if (
      !value ||
      !node ||
      typeof node.start !== "number" ||
      typeof node.end !== "number"
    )
      return;
    tokens.push({
      value,
      file: relFile,
      start: node.start,
      end: node.end,
      line: lineOf(lineStarts, node.start),
      endLine: lineOf(lineStarts, Math.max(node.start, node.end - 1)),
      container,
    });
  }

  function emitBindingPattern(pattern) {
    if (!pattern || typeof pattern !== "object") return;
    if (pattern.type === "Identifier") {
      emit(`BIND:${declare(scope, pattern.name)}`, pattern);
      return;
    }
    if (pattern.type === "AssignmentPattern") {
      emit(`NODE:${pattern.type}`, pattern);
      emitBindingPattern(pattern.left);
      walk(pattern.right, pattern, "right");
      emit(`END:${pattern.type}`, pattern);
      return;
    }
    if (pattern.type === "RestElement") {
      emit(`NODE:${pattern.type}`, pattern);
      emitBindingPattern(pattern.argument);
      emit(`END:${pattern.type}`, pattern);
      return;
    }
    if (pattern.type === "ArrayPattern") {
      emit(`NODE:${pattern.type}`, pattern);
      for (const element of pattern.elements ?? []) emitBindingPattern(element);
      emit(`END:${pattern.type}`, pattern);
      return;
    }
    if (pattern.type === "ObjectPattern") {
      emit(`NODE:${pattern.type}`, pattern);
      for (const property of pattern.properties ?? []) {
        if (!property) continue;
        if (property.type === "RestElement") {
          emitBindingPattern(property);
          continue;
        }
        if (property.type === "Property") {
          emit(`NODE:${property.type}`, property);
          if (property.computed) {
            walk(property.key, property, "key");
          } else if (property.key?.type === "Identifier") {
            emit(`PROP:${property.key.name}`, property.key);
          } else if (property.key) {
            walk(property.key, property, "key");
          }
          emitBindingPattern(property.value);
          emit(`END:${property.type}`, property);
        }
      }
      emit(`END:${pattern.type}`, pattern);
      return;
    }
    walk(pattern);
  }

  function walk(node, parent = null, key = null) {
    if (!node || typeof node !== "object") return;
    if (Array.isArray(node)) {
      for (const item of node) walk(item, parent, key);
      return;
    }
    if (!node.type) return;

    if (node.type === "ImportDeclaration") {
      skippedTokenCount += statementLineCount(node, lineStarts);
      return;
    }

    if (isFunctionNode(node)) {
      emit(`NODE:${node.type}`, node);
      const previousScope = scope;
      const previousContainer = container;
      scope = createScope(previousScope);
      const name = functionName(node, parent);
      container = {
        kind: "function",
        name,
      };
      for (const param of node.params ?? []) {
        for (const binding of collectBindingIdentifiers(param))
          declare(scope, binding.name);
      }
      if (node.body) walk(node.body, node, "body");
      scope = previousScope;
      container = previousContainer;
      emit(`END:${node.type}`, node);
      return;
    }

    if (node.type === "VariableDeclarator") {
      emit(`NODE:${node.type}`, node);
      emitBindingPattern(node.id);
      if (node.init) walk(node.init, node, "init");
      emit(`END:${node.type}`, node);
      return;
    }

    if (node.type === "Identifier") {
      if (isPropertyNameIdentifier(parent, key)) {
        emit(`PROP:${node.name}`, node);
        return;
      }
      if (
        (parent?.type === "VariableDeclarator" && key === "id") ||
        ((parent?.type === "FunctionDeclaration" ||
          parent?.type === "FunctionExpression") &&
          key === "id")
      ) {
        emit(`BIND:${declare(scope, node.name)}`, node);
        return;
      }
      const slot = resolve(scope, node.name);
      emit(slot ? `REF:${slot}` : `GLOBAL:${node.name}`, node);
      return;
    }

    const literal = literalToken(node);
    if (literal) {
      emit(literal, node);
      return;
    }

    emit(`NODE:${node.type}`, node);
    for (const childKey of Object.keys(node)) {
      if (SKIP_KEYS.has(childKey) || childKey === "type") continue;
      walk(node[childKey], node, childKey);
    }
    emit(`END:${node.type}`, node);
  }

  walk(parsed.program);

  return {
    relFile,
    tokens,
    skipped: null,
    diagnostics: [],
    skippedTokenCount,
    tokenLimitExceeded: tokens.length > tokenLimit,
  };
}
