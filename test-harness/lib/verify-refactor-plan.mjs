#!/usr/bin/env node
// Verifies refactor-plan chat output against the humane SHORT/FULL contract.
// This is a maintainer harness: it checks sample model outputs, not audit JSON.

import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const FORBIDDEN_TONE = /\b(bad|broken|trash|terrible|failed)\b/i;
const RAW_JSON_BLOCK = /```(?:json|jsonc)\b|^\s*\{\s*$/m;
const EVIDENCE_ANCHOR = /\b[a-z0-9-]+\.json\b|\[(?:grounded|degraded|확인 불가|unknown)\b/i;
const CODING_AGENT_PROMPT = /\bAsk the coding agent:/i;

const SHORT_SECTION_PATTERNS = [
  ['working-section', /(?:^|\n)\s*(?:#{1,3}\s*)?(?:\*\*)?What (?:Already Works|Is Already Working)(?:\*\*)?/i],
  ['next-slice-section', /(?:^|\n)\s*(?:#{1,3}\s*)?(?:\*\*)?Next (?:Refactor )?Slice(?:\*\*)?/i],
  ['verification-section', /(?:^|\n)\s*(?:#{1,3}\s*)?(?:\*\*)?How (?:We )?Verify(?:\*\*)?/i],
  ['after-that-section', /(?:^|\n)\s*(?:#{1,3}\s*)?(?:\*\*)?After That(?:\*\*)?/i],
];

const FULL_SECTION_PATTERNS = [
  ['working-section', /what is already working/i],
  ['goal-section', /goal in plain language/i],
  ['evidence-section', /evidence snapshot/i],
  ['phase-map-section', /phase map/i],
  ['slice-spec-section', /phase 1 slice spec/i],
  ['quick-audit-section', /phase 1 quick-audit scope/i],
  ['verification-section', /acceptance and verification/i],
  ['risks-section', /risks and leave-alone list/i],
  ['closeout-section', /closeout loop/i],
];

function countMatches(text, regex) {
  return [...text.matchAll(new RegExp(regex.source, `${regex.flags.replace('g', '')}g`))].length;
}

function add(errors, code, detail) {
  errors.push({ code, detail });
}

export function verifyRefactorPlan(text, options = {}) {
  const mode = options.mode ?? 'short';
  const expectCodeChange = Boolean(options.expectCodeChange);
  const errors = [];

  if (typeof text !== 'string' || text.trim() === '') {
    add(errors, 'empty-output', 'refactor-plan output is empty');
    return { ok: false, errors };
  }

  if (FORBIDDEN_TONE.test(text)) {
    add(errors, 'discouraging-tone', 'output uses forbidden discouraging wording');
  }

  if (RAW_JSON_BLOCK.test(text)) {
    add(errors, 'raw-json-in-chat', 'chat-facing plan should not include raw JSON unless explicitly requested');
  }

  if (!EVIDENCE_ANCHOR.test(text)) {
    add(errors, 'missing-evidence-anchor', 'plan should cite at least one artifact or claim label');
  }

  if (expectCodeChange && !/\bpre-write\b/i.test(text)) {
    add(errors, 'missing-prewrite-handoff', 'code-changing plan must include a pre-write handoff');
  }

  if (expectCodeChange && !CODING_AGENT_PROMPT.test(text)) {
    add(errors, 'missing-coding-agent-prompt', 'code-changing plan must include a copy/paste coding-agent prompt');
  }

  if (mode === 'short') {
    for (const [code, pattern] of SHORT_SECTION_PATTERNS) {
      if (!pattern.test(text)) add(errors, code, `missing SHORT section matching ${pattern}`);
    }
    const sliceCount = countMatches(text, SHORT_SECTION_PATTERNS[1][1]);
    if (sliceCount > 1) {
      add(errors, 'multiple-next-slices', `SHORT mode should pick one next slice by default; found ${sliceCount}`);
    }
  } else if (mode === 'full') {
    for (const [code, pattern] of FULL_SECTION_PATTERNS) {
      if (!pattern.test(text)) add(errors, code, `missing FULL section matching ${pattern}`);
    }
  } else {
    add(errors, 'unknown-mode', `unknown mode: ${mode}`);
  }

  return { ok: errors.length === 0, errors };
}

function usage() {
  return [
    'usage: node test-harness/lib/verify-refactor-plan.mjs [--mode short|full] [--expect-code-change] <markdown-file>',
    '',
    'Checks refactor-plan output shape without running an audit.',
  ].join('\n');
}

function parseArgs(argv) {
  const out = { mode: 'short', expectCodeChange: false, file: null };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--mode') {
      out.mode = argv[++i];
    } else if (arg === '--expect-code-change') {
      out.expectCodeChange = true;
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
    console.error(`[verify-refactor-plan] ${e.message}`);
    console.error(usage());
    return 2;
  }
  if (args.help) {
    console.log(usage());
    return 0;
  }
  if (!args.file) {
    console.error('[verify-refactor-plan] missing markdown file');
    console.error(usage());
    return 2;
  }
  if (!existsSync(args.file)) {
    console.error(`[verify-refactor-plan] file not found: ${args.file}`);
    return 2;
  }

  const result = verifyRefactorPlan(readFileSync(args.file, 'utf8'), args);
  if (result.ok) {
    console.log('[verify-refactor-plan] OK');
    return 0;
  }

  console.error('[verify-refactor-plan] FAIL');
  for (const error of result.errors) {
    console.error(`- ${error.code}: ${error.detail}`);
  }
  return 1;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  process.exit(main(process.argv.slice(2)));
}
