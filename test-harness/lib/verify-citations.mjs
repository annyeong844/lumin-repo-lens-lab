#!/usr/bin/env node
// Verifies Rule 1 grounded citations in saved model output.
//
// This is a maintainer harness, not an audit producer. It checks whether
// labels like:
//   [grounded, topology.json.summary.sccCount = 0]
// are falsifiable against JSON artifacts in an audit output directory.

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function stableStringify(value) {
  if (Array.isArray(value)) return `[${value.map(stableStringify).join(',')}]`;
  if (value && typeof value === 'object') {
    return `{${Object.keys(value).sort().map((k) => `${JSON.stringify(k)}:${stableStringify(value[k])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

function valuePreview(value) {
  const s = stableStringify(value);
  return s.length > 160 ? `${s.slice(0, 157)}...` : s;
}

function scanGroundedCitations(text) {
  const citations = [];
  for (let i = 0; i < text.length; i++) {
    if (!text.startsWith('[grounded', i)) continue;
    let depth = 0;
    let quote = null;
    let escaped = false;
    for (let j = i; j < text.length; j++) {
      const ch = text[j];
      if (quote) {
        if (escaped) {
          escaped = false;
        } else if (ch === '\\') {
          escaped = true;
        } else if (ch === quote) {
          quote = null;
        }
        continue;
      }
      if (ch === '"' || ch === "'") {
        quote = ch;
      } else if (ch === '[') {
        depth++;
      } else if (ch === ']') {
        depth--;
        if (depth === 0) {
          citations.push({
            raw: text.slice(i, j + 1),
            body: text.slice(i + 1, j),
            index: i,
          });
          i = j;
          break;
        }
      }
    }
  }
  return citations;
}

function splitTopLevel(text, delimiter) {
  let quote = null;
  let escaped = false;
  let braceDepth = 0;
  let bracketDepth = 0;
  let parenDepth = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (ch === '\\') {
        escaped = true;
      } else if (ch === quote) {
        quote = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'") quote = ch;
    else if (ch === '{') braceDepth++;
    else if (ch === '}') braceDepth--;
    else if (ch === '[') bracketDepth++;
    else if (ch === ']') bracketDepth--;
    else if (ch === '(') parenDepth++;
    else if (ch === ')') parenDepth--;
    else if (ch === delimiter && braceDepth === 0 && bracketDepth === 0 && parenDepth === 0) {
      return [text.slice(0, i).trim(), text.slice(i + 1).trim()];
    }
  }
  return [text.trim(), ''];
}

function normalizeRelaxedJsonLiteral(text) {
  return text
    .replace(/'/g, '"')
    .replace(/([{,]\s*)([A-Za-z_$][\w$-]*)(\s*:)/g, '$1"$2"$3');
}

function parseExpectedValue(raw) {
  const text = raw.trim();
  if (!text) return { ok: false, reason: 'empty expected value' };
  if (/^(N|X|\.\.\.|<.+>)$/.test(text) || /\bN\b/.test(text)) {
    return { ok: false, reason: 'placeholder value is not mechanically checkable' };
  }
  if (/^-?\d+(?:\.\d+)?$/.test(text)) return { ok: true, value: Number(text) };
  if (text === 'true') return { ok: true, value: true };
  if (text === 'false') return { ok: true, value: false };
  if (text === 'null') return { ok: true, value: null };
  if ((text.startsWith('"') && text.endsWith('"')) || (text.startsWith("'") && text.endsWith("'"))) {
    try {
      return { ok: true, value: JSON.parse(text.replace(/^'|'$/g, '"')) };
    } catch {
      return { ok: false, reason: 'quoted value is not parseable' };
    }
  }
  if ((text.startsWith('{') && text.endsWith('}')) || (text.startsWith('[') && text.endsWith(']'))) {
    try {
      return { ok: true, value: JSON.parse(normalizeRelaxedJsonLiteral(text)) };
    } catch (e) {
      return { ok: false, reason: `object/array value is not parseable: ${e.message}` };
    }
  }
  if (/^[A-Za-z0-9_.:/@-]+$/.test(text)) return { ok: true, value: text };
  return { ok: false, reason: `unsupported expected value: ${text}` };
}

function parsePathTokens(fieldPath) {
  const tokens = [];
  let i = 0;
  while (i < fieldPath.length) {
    if (fieldPath[i] === '.') {
      i++;
      const start = i;
      while (i < fieldPath.length && fieldPath[i] !== '.' && fieldPath[i] !== '[') i++;
      if (i === start) throw new Error(`empty path segment in ${fieldPath}`);
      tokens.push(fieldPath.slice(start, i));
    } else if (fieldPath[i] === '[') {
      i++;
      while (/\s/.test(fieldPath[i] ?? '')) i++;
      if (fieldPath[i] === '"' || fieldPath[i] === "'") {
        const quote = fieldPath[i++];
        let value = '';
        let escaped = false;
        while (i < fieldPath.length) {
          const ch = fieldPath[i++];
          if (escaped) {
            value += ch;
            escaped = false;
          } else if (ch === '\\') {
            escaped = true;
          } else if (ch === quote) {
            break;
          } else {
            value += ch;
          }
        }
        while (/\s/.test(fieldPath[i] ?? '')) i++;
        if (fieldPath[i] !== ']') throw new Error(`unterminated bracket segment in ${fieldPath}`);
        i++;
        tokens.push(value);
      } else {
        const start = i;
        while (i < fieldPath.length && fieldPath[i] !== ']') i++;
        if (fieldPath[i] !== ']') throw new Error(`unterminated bracket segment in ${fieldPath}`);
        const raw = fieldPath.slice(start, i).trim();
        i++;
        if (!/^\d+$/.test(raw)) throw new Error(`unsupported bracket segment [${raw}] in ${fieldPath}`);
        tokens.push(Number(raw));
      }
    } else if (/\s/.test(fieldPath[i])) {
      i++;
    } else {
      throw new Error(`path must start with "." or "[": ${fieldPath.slice(i)}`);
    }
  }
  return tokens;
}

function getAtPath(root, tokens) {
  let cur = root;
  for (const token of tokens) {
    if (token === 'length') {
      if (cur == null || typeof cur.length !== 'number') {
        return { ok: false, reason: 'length requested on non-array/non-string value' };
      }
      cur = cur.length;
    } else if (cur != null && Object.hasOwn(cur, token)) {
      cur = cur[token];
    } else {
      return { ok: false, reason: `missing path segment ${JSON.stringify(token)}` };
    }
  }
  return { ok: true, value: cur };
}

function parseGroundedAssignment(citationBody) {
  const body = citationBody.replace(/^grounded(?:\s+structural)?\s*,\s*/i, '').trim();
  const [firstClause, rest] = splitTopLevel(body, ',');
  const [lhs, rhs] = splitTopLevel(firstClause, '=');
  if (!rhs) return { ok: false, reason: 'grounded citation has no top-level "=" assignment', body };
  const artifactMatch = lhs.match(/(?:^|\s)([A-Za-z0-9_.-]+\.json)(.*)$/);
  if (!artifactMatch) {
    return { ok: false, reason: 'grounded citation does not name a .json artifact path', body };
  }
  return {
    ok: true,
    artifact: artifactMatch[1],
    fieldPath: (artifactMatch[2] ?? '').trim(),
    expectedRaw: rhs,
    trailing: rest,
    body,
  };
}

function readJsonCandidate(artifact, artifactsDir, rootDir, cache) {
  const candidates = [
    path.join(artifactsDir, artifact),
    rootDir ? path.join(rootDir, artifact) : null,
  ].filter(Boolean);
  const filePath = candidates.find((p) => existsSync(p));
  if (!filePath) return { ok: false, reason: `artifact not found: ${artifact}` };
  if (cache.has(filePath)) return cache.get(filePath);
  try {
    const parsed = { ok: true, filePath, value: JSON.parse(readFileSync(filePath, 'utf8')) };
    cache.set(filePath, parsed);
    return parsed;
  } catch (e) {
    const failed = { ok: false, reason: `failed to parse ${filePath}: ${e.message}` };
    cache.set(filePath, failed);
    return failed;
  }
}

function valuesEqual(a, b) {
  return stableStringify(a) === stableStringify(b);
}

export function verifyGroundedCitations(text, options = {}) {
  const artifactsDir = path.resolve(options.artifactsDir ?? '.audit');
  const rootDir = options.rootDir ? path.resolve(options.rootDir) : null;
  const cache = new Map();
  const failures = [];
  const warnings = [];
  const citations = scanGroundedCitations(text);
  let checked = 0;

  for (const citation of citations) {
    const assignment = parseGroundedAssignment(citation.body);
    if (!assignment.ok) {
      failures.push({ code: 'unfalsifiable-grounded-citation', citation: citation.raw, detail: assignment.reason });
      continue;
    }
    const artifact = readJsonCandidate(assignment.artifact, artifactsDir, rootDir, cache);
    if (!artifact.ok) {
      failures.push({ code: 'artifact-unavailable', citation: citation.raw, detail: artifact.reason });
      continue;
    }
    let tokens;
    try {
      tokens = parsePathTokens(assignment.fieldPath);
    } catch (e) {
      failures.push({ code: 'path-parse-error', citation: citation.raw, detail: e.message });
      continue;
    }
    const actual = getAtPath(artifact.value, tokens);
    if (!actual.ok) {
      failures.push({ code: 'artifact-path-missing', citation: citation.raw, detail: actual.reason });
      continue;
    }
    const expected = parseExpectedValue(assignment.expectedRaw);
    if (!expected.ok) {
      failures.push({ code: 'expected-value-uncheckable', citation: citation.raw, detail: expected.reason });
      continue;
    }
    checked++;
    if (!valuesEqual(actual.value, expected.value)) {
      failures.push({
        code: 'value-mismatch',
        citation: citation.raw,
        detail: `expected ${valuePreview(expected.value)}, artifact has ${valuePreview(actual.value)}`,
      });
    }
    if (assignment.trailing) {
      warnings.push({
        code: 'trailing-unverified-clause',
        citation: citation.raw,
        detail: `verified first assignment only; trailing clause was: ${assignment.trailing}`,
      });
    }
  }

  return {
    ok: failures.length === 0,
    citationsFound: citations.length,
    checked,
    failures,
    warnings,
  };
}

function usage() {
  return [
    'usage: node test-harness/lib/verify-citations.mjs --artifacts <dir> [--root <repo>] <markdown-file|->',
    '',
    'Verifies `[grounded, artifact.json.path = value]` labels against JSON artifacts.',
    'Use `-` to read Markdown from stdin.',
  ].join('\n');
}

function parseArgs(argv) {
  const out = { artifactsDir: '.audit', rootDir: null, file: null };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--artifacts' || arg === '--output') {
      out.artifactsDir = argv[++i];
    } else if (arg === '--root') {
      out.rootDir = argv[++i];
    } else if (arg === '--help' || arg === '-h') {
      out.help = true;
    } else if (!out.file) {
      out.file = arg;
    } else {
      throw new Error(`unexpected argument: ${arg}`);
    }
  }
  return out;
}

function main(argv) {
  let args;
  try {
    args = parseArgs(argv);
  } catch (e) {
    console.error(`[verify-citations] ${e.message}`);
    console.error(usage());
    return 2;
  }
  if (args.help) {
    console.log(usage());
    return 0;
  }
  if (!args.file) {
    console.error('[verify-citations] missing markdown file');
    console.error(usage());
    return 2;
  }
  if (args.file !== '-' && !existsSync(args.file)) {
    console.error(`[verify-citations] file not found: ${args.file}`);
    return 2;
  }

  const text = args.file === '-'
    ? readFileSync(0, 'utf8')
    : readFileSync(args.file, 'utf8');
  const result = verifyGroundedCitations(text, args);
  if (result.ok) {
    console.log(`[verify-citations] OK — checked ${result.checked}/${result.citationsFound} grounded citation(s)`);
    for (const warning of result.warnings) {
      console.warn(`- warn ${warning.code}: ${warning.detail}`);
    }
    return 0;
  }

  console.error(`[verify-citations] FAIL — checked ${result.checked}/${result.citationsFound} grounded citation(s)`);
  for (const failure of result.failures) {
    console.error(`- ${failure.code}: ${failure.detail}`);
    console.error(`  ${failure.citation}`);
  }
  for (const warning of result.warnings) {
    console.warn(`- warn ${warning.code}: ${warning.detail}`);
  }
  return 1;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2)));
}
