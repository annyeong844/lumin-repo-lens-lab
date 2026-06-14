#!/usr/bin/env node
// lint-prompts.mjs — offline check that prompts/ and expectations.json stay in sync.
//
// Run before every PR. No claude CLI required, no API cost.
//
// Checks:
//   1. expectations.json is valid JSON and has the expected top-level shape
//   2. Every prompt file under prompts/ has a matching entry in expectations.json
//   3. Every expectation entry references an existing prompt file
//   4. expected_mode values are valid (against canonical/mode-contract.md modes)
//   5. expected_citation_pattern compiles as a regex
//   6. No prompt file is empty
//   7. Negative tests don't accidentally carry positive-only fields

import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const HARNESS_ROOT = resolve(__dirname, '..');

const VALID_MODES = new Set([
  'audit',
  'pre-write',
  'post-write',
  'structural-review',
  'canon-draft',
  'check-canon',
  'none',
]);

const POSITIVE_ONLY_FIELDS = [
  'expected_mode',
  'expected_script_substrings',
  'expected_artifacts_referenced',
  'expected_response_substrings',
  'expected_citation_pattern',
  'expected_to_mention_count',
];

function listPromptFiles(dir, prefix = '') {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    const rel = prefix ? `${prefix}/${entry}` : entry;
    if (statSync(full).isDirectory()) {
      out.push(...listPromptFiles(full, rel));
    } else if (entry.endsWith('.txt')) {
      out.push(rel);
    }
  }
  return out;
}

function main() {
  const errors = [];
  const warnings = [];

  // 1. Load expectations
  const expPath = join(HARNESS_ROOT, 'expectations.json');
  if (!existsSync(expPath)) {
    console.error('FATAL: expectations.json not found');
    process.exit(2);
  }
  let exp;
  try {
    exp = JSON.parse(readFileSync(expPath, 'utf8'));
  } catch (e) {
    console.error(`FATAL: expectations.json is not valid JSON: ${e.message}`);
    process.exit(2);
  }
  if (!exp.tests || !Array.isArray(exp.tests)) {
    errors.push('expectations.json must have a top-level "tests" array');
  }

  // 2 & 3. Cross-reference prompts ↔ expectations
  const promptsDir = join(HARNESS_ROOT, 'prompts');
  if (!existsSync(promptsDir)) {
    console.error('FATAL: prompts/ directory not found');
    process.exit(2);
  }
  const promptFiles = new Set(listPromptFiles(promptsDir));
  const expectedPrompts = new Set();

  for (const entry of exp.tests || []) {
    if (!entry.prompt) {
      errors.push('Expectation entry missing "prompt" field');
      continue;
    }
    expectedPrompts.add(entry.prompt);

    // 3. expectation references an existing prompt
    if (!promptFiles.has(entry.prompt)) {
      errors.push(`Expectation references non-existent prompt: ${entry.prompt}`);
    }

    // 4. Mode validity (positive only)
    if (entry.should_trigger === true) {
      if (!entry.expected_mode) {
        errors.push(`Positive test "${entry.prompt}" missing expected_mode`);
      } else if (!VALID_MODES.has(entry.expected_mode)) {
        errors.push(
          `Positive test "${entry.prompt}" has invalid expected_mode: "${entry.expected_mode}". Valid modes: ${[...VALID_MODES].join(', ')}`,
        );
      }
    }

    // 5. Citation regex compiles (positive only — negative tests don't have one)
    if (entry.expected_citation_pattern) {
      try {
        new RegExp(entry.expected_citation_pattern);
      } catch (e) {
        errors.push(
          `Bad regex in "${entry.prompt}".expected_citation_pattern: ${e.message}`,
        );
      }
    }

    // 7. Negative tests should not carry positive-only fields
    if (entry.should_trigger === false) {
      for (const field of POSITIVE_ONLY_FIELDS) {
        if (entry[field] !== undefined) {
          warnings.push(
            `Negative test "${entry.prompt}" carries positive-only field "${field}". Probably a copy-paste mistake; ignored at runtime.`,
          );
        }
      }
    }
  }

  // 2. Every prompt file has an expectation
  for (const p of promptFiles) {
    if (!expectedPrompts.has(p)) {
      errors.push(`Prompt file has no expectation: ${p}`);
    }
  }

  // 6. No empty prompts
  for (const p of promptFiles) {
    const full = join(promptsDir, p);
    const content = readFileSync(full, 'utf8').trim();
    if (!content) {
      errors.push(`Prompt file is empty: ${p}`);
    }
  }

  // OUTPUT
  console.log(`\n=== Lint Report ===`);
  console.log(`Prompts found: ${promptFiles.size}`);
  console.log(`Expectations: ${exp.tests?.length ?? 0}`);

  if (warnings.length) {
    console.log(`\nWarnings (${warnings.length}):`);
    for (const w of warnings) console.log(`  ⚠️  ${w}`);
  }
  if (errors.length) {
    console.log(`\nErrors (${errors.length}):`);
    for (const e of errors) console.log(`  ❌ ${e}`);
    console.log('\nFAIL');
    process.exit(1);
  }

  console.log('\nOK — prompts and expectations are in sync.');
  process.exit(0);
}

main();
