// Tests for _lib/mode-dispatch.mjs — P1-1 step 5.1.
//
// Pinning rules from docs/history/phases/p1/p1-1.md §4.1 + §5.1:
//   - Pure function: only reads userText + cwdMeta.
//   - mode === 'none' carries nonTriggerReason classifying WHY.
//   - Non-trigger precedence: no-repo-context > prose-rewrite >
//     comment-typo-fix > pure-inspection > guard-only.
//   - Compound guard + verb fires pre-write (matches mode-contract.md §3.5).
//   - "README 다듬어줘" → mode:'none', nonTriggerReason:'prose-rewrite'.
//     NEVER "fire + no-op in calling code".
//   - Repo context = any of {hasPackageJson, hasTsconfig, hasSrcTree}.

import { dispatchMode } from '../_lib/mode-dispatch.mjs';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const MODE_CONTRACT = readFileSync(path.join(ROOT, 'canonical/mode-contract.md'), 'utf8');
const MODE_DISPATCH = readFileSync(path.join(ROOT, '_lib/mode-dispatch.mjs'), 'utf8');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// Repo context defaults — realistic "this is a TS/JS repo" state.
const repoCtx = { hasPackageJson: true, hasTsconfig: true, hasSrcTree: true };

function fencedBlock(section) {
  const rx = new RegExp(`### ${section.replace('.', '\\.')}[\\s\\S]*?\`\`\`\\r?\\n([\\s\\S]*?)\`\`\``);
  const match = MODE_CONTRACT.match(rx);
  return match?.[1] ?? '';
}

function contractTerms(section) {
  const terms = [];
  for (const rawLine of fencedBlock(section).trim().split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) continue;
    if (line.includes('"') || line.includes(',')) {
      terms.push(...[...line.matchAll(/"([^"]+)"|[^\s,]+/g)]
        .map((m) => m[1] ?? m[0])
        .filter(Boolean));
    } else {
      terms.push(...line.split(/\s{2,}/).filter(Boolean));
    }
  }
  return terms;
}

function dispatchTerms(constName) {
  const rx = new RegExp(`const ${constName} = Object\\.freeze\\(\\[([\\s\\S]*?)\\]\\);`);
  const body = MODE_DISPATCH.match(rx)?.[1] ?? '';
  return [...body.matchAll(/'([^']+)'/g)].map((m) => m[1]);
}

function assertSameList(label, actual, expected) {
  assert(label, JSON.stringify(actual) === JSON.stringify(expected),
    `actual=${JSON.stringify(actual)}\nexpected=${JSON.stringify(expected)}`);
}

// ═══ Canonical mirror checks ═══
//
// The dispatcher intentionally mirrors canonical/mode-contract.md §3.
// These checks turn that hand mirror into a drift-locked contract.

assertSameList('M1. Korean verb list mirrors canonical §3.1',
  dispatchTerms('KOREAN_VERBS'), contractTerms('3.1 Korean'));
assertSameList('M2. English verb list mirrors canonical §3.2',
  dispatchTerms('ENGLISH_VERBS'), contractTerms('3.2 English'));
assertSameList('M3. Korean guard list mirrors canonical §3.4',
  dispatchTerms('KOREAN_GUARDS'), contractTerms('3.4 Guards').filter((x) => /[가-힣]/.test(x)));
assertSameList('M4. English guard list mirrors canonical §3.4',
  dispatchTerms('ENGLISH_GUARDS'), contractTerms('3.4 Guards').filter((x) => !/[가-힣]/.test(x)));

// ═══ Guard-alone cases (non-trigger) ═══

{
  const r = dispatchMode('찾아줘', repoCtx);
  assert('T1. "찾아줘" alone → mode:none',
    r.mode === 'none', `mode=${r.mode}`);
  assert('T1b. nonTriggerReason is guard-only',
    r.nonTriggerReason === 'guard-only', `reason=${r.nonTriggerReason}`);
  assert('T1c. matched guards include 찾아줘',
    r.matchedGuards?.includes('찾아줘'));
}

{
  const r = dispatchMode('explain this', repoCtx);
  assert('T2. "explain this" → mode:none',
    r.mode === 'none');
  assert('T2b. nonTriggerReason is guard-only OR pure-inspection',
    r.nonTriggerReason === 'guard-only' || r.nonTriggerReason === 'pure-inspection');
}

{
  const r = dispatchMode('보여줘', repoCtx);
  assert('T3. "보여줘" alone → mode:none',
    r.mode === 'none');
}

// ═══ Write verb alone (trigger) ═══

{
  const r = dispatchMode('만들어줘', repoCtx);
  assert('T4. "만들어줘" alone → mode:pre-write',
    r.mode === 'pre-write');
  assert('T4b. matched verbs include 만들어줘',
    r.matchedVerbs?.includes('만들어줘'));
  assert('T4c. compoundGuardPlusVerb is false (no guard)',
    r.compoundGuardPlusVerb === false);
}

{
  const r = dispatchMode('implement a new function', repoCtx);
  assert('T5. "implement ..." → mode:pre-write',
    r.mode === 'pre-write');
}

// ═══ Compound guard + verb (trigger, compound flag set) ═══

{
  const r = dispatchMode('기존 helper 찾아서 연결해줘', repoCtx);
  assert('T6. "찾아서 + 연결해줘" → mode:pre-write (verb wins)',
    r.mode === 'pre-write');
  assert('T6b. compoundGuardPlusVerb is true',
    r.compoundGuardPlusVerb === true);
  assert('T6c. both guard and verb matched',
    r.matchedGuards?.length > 0 && r.matchedVerbs?.length > 0);
}

{
  const r = dispatchMode('find and refactor this module', repoCtx);
  assert('T7. English "find and refactor" → mode:pre-write, compound',
    r.mode === 'pre-write' && r.compoundGuardPlusVerb === true);
}

// ═══ No repo context (non-trigger, highest precedence) ═══

{
  const r = dispatchMode('만들어줘', { hasPackageJson: false, hasTsconfig: false, hasSrcTree: false });
  assert('T8. verb + no repo context → mode:none',
    r.mode === 'none');
  assert('T8b. nonTriggerReason is no-repo-context',
    r.nonTriggerReason === 'no-repo-context');
}

{
  // Only hasPackageJson is enough to count as repo context.
  const r = dispatchMode('만들어줘', { hasPackageJson: true, hasTsconfig: false, hasSrcTree: false });
  assert('T9. verb + only package.json → mode:pre-write (context satisfied)',
    r.mode === 'pre-write');
}

// ═══ Prose rewrite (CRITICAL non-trigger, canonical §2.2) ═══
//
// "README 다듬어줘" → mode:'none', nonTriggerReason:'prose-rewrite'.
// The dispatcher MUST detect prose via file-name hint inside userText
// (README / CHANGELOG / *.md / docs/ + doc-verb). "Fire + no-op in
// calling code" is forbidden.

{
  const r = dispatchMode('README 다듬어줘', repoCtx);
  assert('T10. "README 다듬어줘" → mode:none (prose-rewrite canonical rule)',
    r.mode === 'none', `mode=${r.mode}`);
  assert('T10b. nonTriggerReason is prose-rewrite',
    r.nonTriggerReason === 'prose-rewrite',
    `reason=${r.nonTriggerReason}`);
}

{
  const r = dispatchMode('CHANGELOG 업데이트해줘', repoCtx);
  assert('T11. "CHANGELOG 업데이트" → mode:none (prose)',
    r.mode === 'none' && r.nonTriggerReason === 'prose-rewrite');
}

{
  const r = dispatchMode('docs/architecture.md 다듬어줘', repoCtx);
  assert('T12. "docs/*.md 다듬어줘" → mode:none (prose path)',
    r.mode === 'none' && r.nonTriggerReason === 'prose-rewrite');
}

{
  const r = dispatchMode('rewrite the README', repoCtx);
  assert('T13. "rewrite the README" → mode:none',
    r.mode === 'none' && r.nonTriggerReason === 'prose-rewrite');
}

// ═══ Comment typo fix (non-trigger) ═══

{
  const r = dispatchMode('주석 오타 고쳐줘', repoCtx);
  assert('T14. "주석 오타 고쳐줘" → mode:none (comment-typo-fix)',
    r.mode === 'none' && r.nonTriggerReason === 'comment-typo-fix',
    `mode=${r.mode}, reason=${r.nonTriggerReason}`);
}

{
  const r = dispatchMode('fix comment typo', repoCtx);
  assert('T15. "fix comment typo" → mode:none',
    r.mode === 'none' && r.nonTriggerReason === 'comment-typo-fix');
}

// Without the specific combination, generic "fix" still triggers.
{
  const r = dispatchMode('fix this bug', repoCtx);
  assert('T16. "fix this bug" → mode:pre-write (generic fix, not comment-typo)',
    r.mode === 'pre-write');
}

// ═══ Pure inspection (non-trigger) ═══

{
  const r = dispatchMode('이 코드 어떻게 동작해요?', repoCtx);
  assert('T17. "이 코드 어떻게 동작해요?" → mode:none (pure-inspection)',
    r.mode === 'none');
  // Either pure-inspection OR guard-only is acceptable — depends on
  // whether "어떻게 해?" is in guard list vs its own detector. Pin the
  // outcome, not the reason hierarchy for this specific phrase.
}

// ═══ Non-trigger precedence ═══

{
  // No repo context AND prose rewrite — precedence says no-repo-context wins.
  const r = dispatchMode('README 다듬어줘', { hasPackageJson: false, hasTsconfig: false, hasSrcTree: false });
  assert('T18. no-repo-context beats prose-rewrite in precedence',
    r.mode === 'none' && r.nonTriggerReason === 'no-repo-context');
}

// ═══ Return shape sanity ═══

{
  const r = dispatchMode('만들어줘', repoCtx);
  assert('T19. trigger result has string rationale',
    typeof r.rationale === 'string' && r.rationale.length > 0);
  assert('T19b. trigger result has matchedVerbs array',
    Array.isArray(r.matchedVerbs));
  assert('T19c. trigger result has matchedGuards array (possibly empty)',
    Array.isArray(r.matchedGuards));
  assert('T19d. trigger result has compoundGuardPlusVerb boolean',
    typeof r.compoundGuardPlusVerb === 'boolean');
  assert('T19e. trigger result does NOT carry nonTriggerReason',
    r.nonTriggerReason === undefined);
}

// ═══ Purity: same input → same output ═══

{
  const a = dispatchMode('만들어줘', repoCtx);
  const b = dispatchMode('만들어줘', repoCtx);
  assert('T20. pure function: deterministic mode',
    a.mode === b.mode);
  assert('T20b. pure function: deterministic rationale',
    a.rationale === b.rationale);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
