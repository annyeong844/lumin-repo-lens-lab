#!/usr/bin/env node
// verify.mjs — read a claude -p stream-json log and check it against an expectation entry.
//
// Usage: node lib/verify.mjs <log-file> <prompt-relative-path>
//
// Exit codes:
//   0 — all checks passed
//   1 — at least one check failed
//   2 — runtime error (file missing, malformed JSON, unknown prompt key, etc.)
//
// Output format: one ✅/❌ line per check, then a SUMMARY line.

import { readFileSync, existsSync } from 'node:fs';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const HARNESS_ROOT = resolve(__dirname, '..');

const SKILL_NAME = 'lumin-repo-lens-lab';

function fail(msg) {
  console.error(`[verify] ERROR: ${msg}`);
  process.exit(2);
}

function loadExpectation(promptRelative) {
  const expPath = join(HARNESS_ROOT, 'expectations.json');
  if (!existsSync(expPath)) fail(`expectations.json not found at ${expPath}`);
  const exp = JSON.parse(readFileSync(expPath, 'utf8'));
  const entry = exp.tests.find((t) => t.prompt === promptRelative);
  if (!entry) fail(`No expectation entry for prompt "${promptRelative}"`);
  return entry;
}

// Parse a stream-json log. Each line is a separate JSON object emitted by claude -p.
// We extract:
//   - skillInvocations: which skills were invoked via the Skill tool
//   - bashCommands: the command text from every Bash tool call
//   - finalAssistantText: the concatenation of all assistant text blocks
function parseStreamLog(path) {
  if (!existsSync(path)) fail(`log file not found at ${path}`);
  const raw = readFileSync(path, 'utf8');
  const lines = raw.split(/\r?\n/).filter((l) => l.trim());

  const skillInvocations = new Set();
  const bashCommands = [];
  const assistantTextBlocks = [];

  for (const line of lines) {
    let evt;
    try {
      evt = JSON.parse(line);
    } catch {
      continue; // tolerate non-JSON noise
    }

    // Tool use blocks live inside assistant messages
    const message = evt.message;
    if (!message) continue;
    const content = message.content;
    if (!Array.isArray(content)) continue;

    for (const block of content) {
      if (!block || typeof block !== 'object') continue;

      if (block.type === 'tool_use') {
        if (block.name === 'Skill' && block.input?.skill) {
          skillInvocations.add(block.input.skill);
        }
        if (block.name === 'Bash' && typeof block.input?.command === 'string') {
          bashCommands.push(block.input.command);
        }
      }

      if (block.type === 'text' && typeof block.text === 'string') {
        assistantTextBlocks.push(block.text);
      }
    }
  }

  return {
    skillInvocations,
    bashCommands,
    finalAssistantText: assistantTextBlocks.join('\n\n'),
  };
}

function skillTriggered(skillInvocations) {
  for (const s of skillInvocations) {
    // Match plain or namespaced form (plugin:skill-name)
    const tail = s.includes(':') ? s.split(':').pop() : s;
    if (tail === SKILL_NAME) return true;
  }
  return false;
}

// CHECK RUNNERS — each returns { name, passed, detail }

function checkSkillTriggering(expected, observed) {
  const triggered = skillTriggered(observed.skillInvocations);
  if (expected.should_trigger) {
    return {
      name: 'skill triggering',
      passed: triggered,
      detail: triggered
        ? `Skill "${SKILL_NAME}" was invoked.`
        : `Expected Skill "${SKILL_NAME}" to be invoked. Observed skill invocations: [${[...observed.skillInvocations].join(', ') || '(none)'}].`,
    };
  } else {
    return {
      name: 'skill non-triggering',
      passed: !triggered,
      detail: triggered
        ? `Skill "${SKILL_NAME}" was invoked but expected NOT to fire on this prompt. Over-triggering erodes user trust.`
        : `Skill correctly did NOT fire. Other invocations: [${[...observed.skillInvocations].join(', ') || '(none)'}].`,
    };
  }
}

function checkBashScripts(expected, observed) {
  const expectedSubs = expected.expected_script_substrings;
  if (!expectedSubs || !expectedSubs.length) return null;

  const allBash = observed.bashCommands.join('\n');
  const missing = expectedSubs.filter((s) => !allBash.includes(s));

  return {
    name: 'expected scripts ran',
    passed: missing.length === 0,
    detail:
      missing.length === 0
        ? `All expected substrings found in bash invocations: [${expectedSubs.join(', ')}]`
        : `Missing substrings: [${missing.join(', ')}]. Observed bash commands: ${
            observed.bashCommands.length === 0
              ? '(none)'
              : '\n  - ' + observed.bashCommands.slice(0, 5).join('\n  - ')
          }`,
  };
}

function checkArtifactsReferenced(expected, observed) {
  const refs = expected.expected_artifacts_referenced;
  if (!refs || !refs.length) return null;

  const text = observed.finalAssistantText;
  const missing = refs.filter((r) => !text.includes(r));

  return {
    name: 'artifacts referenced in response',
    passed: missing.length === 0,
    detail:
      missing.length === 0
        ? `All expected artifacts mentioned: [${refs.join(', ')}]`
        : `Missing artifact mentions: [${missing.join(', ')}]. The response should cite the JSON artifact name backing each numerical claim per fact-model.md §2.`,
  };
}

function checkResponseSubstrings(expected, observed) {
  const subs = expected.expected_response_substrings;
  if (!subs || !subs.length) return null;

  const text = observed.finalAssistantText;
  const missing = subs.filter((s) => !text.includes(s));

  return {
    name: 'response contains expected substrings',
    passed: missing.length === 0,
    detail:
      missing.length === 0
        ? `All expected substrings present: [${subs.join(', ')}]`
        : `Missing substrings: [${missing.join(', ')}].`,
  };
}

function checkCitationPattern(expected, observed) {
  const pattern = expected.expected_citation_pattern;
  if (!pattern) return null;

  let re;
  try {
    re = new RegExp(pattern);
  } catch (e) {
    return {
      name: 'citation pattern compiles',
      passed: false,
      detail: `Bad regex in expectations.json: ${e.message}`,
    };
  }

  const text = observed.finalAssistantText;
  const passed = re.test(text);

  return {
    name: 'citation discipline',
    passed,
    detail: passed
      ? `Found citation matching /${pattern}/ in response.`
      : `No citation matching /${pattern}/ found in response. Iron Law violation: structural claims must carry [grounded, ...] / [degraded, ...] / [확인 불가, ...] labels per canonical/invariants.md §1.`,
  };
}

function checkBareCountAntipattern(expected, observed) {
  // Only meaningful when the test expects a count.
  if (!expected.expected_to_mention_count) return null;

  const text = observed.finalAssistantText;

  // Find numeric mentions like "3 dead" / "3건" / "3 exports" — anywhere there's
  // a count adjacent to a structural noun. Then check whether each line carrying
  // such a mention also carries a citation in the same line.
  //
  // Note on regex: \b doesn't fire between an ASCII digit and a Hangul char
  // (Korean characters aren't word chars), so "3건" needs no left boundary.
  // We use a left lookbehind to avoid matching mid-decimal like "3.14건" and a
  // right \b only on the English nouns to avoid matching "dead" in "deadline".
  const lines = text.split(/\r?\n/);
  const numericNounRe =
    /(?<![\d.])(\d+)\s*(?:(건|개)|((?:dead\s+(?:export|code)s?|exports?|cycles?|god\s+modules?|files?))\b)/i;
  const citationRe = /\[(grounded|degraded|확인\s*불가|unknown)[^\]]*\]/;

  const violations = [];
  for (const line of lines) {
    const m = line.match(numericNounRe);
    if (!m) continue;
    if (!citationRe.test(line)) {
      violations.push(line.trim().slice(0, 140));
    }
  }

  return {
    name: 'no bare counts (Iron Law)',
    passed: violations.length === 0,
    detail:
      violations.length === 0
        ? 'Every numeric structural mention carries a citation on the same line.'
        : `Bare counts without citations:\n  - ${violations.slice(0, 3).join('\n  - ')}`,
  };
}

// MAIN

function main() {
  const [, , logFile, promptRel] = process.argv;
  if (!logFile || !promptRel) {
    console.error('Usage: node lib/verify.mjs <log-file> <prompt-relative-path>');
    process.exit(2);
  }

  const expected = loadExpectation(promptRel);
  const observed = parseStreamLog(logFile);

  const checks = [
    checkSkillTriggering(expected, observed),
    checkBashScripts(expected, observed),
    checkArtifactsReferenced(expected, observed),
    checkResponseSubstrings(expected, observed),
    checkCitationPattern(expected, observed),
    checkBareCountAntipattern(expected, observed),
  ].filter(Boolean);

  let passedCount = 0;
  let failedCount = 0;

  console.log(`\n=== Verification: ${promptRel} ===`);
  console.log(`Expected: should_trigger=${expected.should_trigger}, mode=${expected.expected_mode || '—'}`);
  if (expected.expected_status === 'known_gap') {
    console.log('NOTE: This test is documented as a known gap (see expectations.json _why field).');
  }
  console.log('');

  for (const check of checks) {
    const sigil = check.passed ? '✅' : '❌';
    console.log(`${sigil} ${check.name}: ${check.detail}`);
    check.passed ? passedCount++ : failedCount++;
  }

  console.log('');
  console.log(`SUMMARY: ${passedCount} passed, ${failedCount} failed`);
  process.exit(failedCount === 0 ? 0 : 1);
}

main();
