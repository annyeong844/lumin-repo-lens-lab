// Tests for _lib/pre-write-canonical-parser.mjs — P1-0 sub-step.
//
// Pinning rules from canonical/pre-write-gate.md §8 + docs/history/phases/p1/session.md §5.0.1:
//   - recognized schema (generated-canon header) → owner rows extracted
//     with correct line numbers.
//   - free-form canon markdown → { recognized: false }, no row parsing.
//   - missing file → { recognized: false, reason: "... absent" }.
//   - heuristic table parsing is forbidden — only sections titled with
//     "Single owner" or "severely-any-contaminated" produce owner rows.
//   - DUPLICATE / LOCAL_COMMON_NAME / ANY_COLLISION sections must NOT
//     produce owner rows (they're group-level, not owner-level).

import { writeFileSync, mkdtempSync, rmSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { parseCanonicalFile, findCanonicalOwnerClaim } from '../_lib/pre-write-canonical-parser.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function fx(name, content) {
  const dir = mkdtempSync(path.join(tmpdir(), `canon-parser-${name}-`));
  mkdirSync(path.join(dir, 'canonical'), { recursive: true });
  const fp = path.join(dir, 'canonical', 'type-ownership.md');
  writeFileSync(fp, content);
  return { dir, filePath: fp };
}

// ─── T1. Missing file ────────────────────────────────────
{
  const missing = path.join(tmpdir(), 'canon-parser-missing-does-not-exist', 'x.md');
  const r = parseCanonicalFile(missing);
  assert('T1. missing file → recognized:false with absent reason',
    r.recognized === false && /absent/.test(r.reason),
    `reason=${r.reason}`);
}

// ─── T2. Free-form canon (no generated-canon header) ─────
{
  const { dir, filePath } = fx('freeform', `# Random canon
This is a free-form canonical document written by hand.

## Owners

- SessionId lives in src/protocol/ids.ts
- User lives in src/models/User.ts
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T2. free-form canon → recognized:false',
      r.recognized === false,
      `got recognized=${r.recognized}`);
    assert('T2b. free-form canon → ownerTables empty',
      r.ownerTables.length === 0);
    assert('T2c. free-form canon → reason mentions header',
      /header/.test(r.reason),
      `reason=${r.reason}`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ─── T3. Recognized schema with Status header ─────────────
{
  const { dir, filePath } = fx('recognized', `# canonical-draft/type-ownership.md — DRAFT

> **Role:** observed type ownership derived from AST.
> **Status:** draft, v1
> **Generated:** 2026-04-20T10:00:00Z

## 1. Summary

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Re-exported through | Status | Tags | Any / unknown signal |
|---|---|---|---|---:|---|---|---|---|
| \`SessionId\` | \`src/protocol/ids.ts\` | TSTypeAliasDeclaration | 14 | 8 | \`src/index.ts\` | ✅ | — | — |
| \`User\` | \`src/models/User.ts\` | TSInterfaceDeclaration | 3 | 5 | — | ✅ | — | — |

### 2.2 Single owner (weak / zero-internal-fan-in)

| Type | Owner | Kind | Line | Fan-in | Re-exported through | Status | Tags | Any / unknown signal | Note |
|---|---|---|---|---:|---|---|---|---|---|
| \`InternalFlag\` | \`src/engine/flag.ts\` | TSTypeAliasDeclaration | 3 | 1 | — | ⚠ weak | — | — | only \`src/engine/flag-consumer.ts\` |
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T3. recognized schema → recognized:true',
      r.recognized === true);
    assert('T3b. recognized schema → two owner tables',
      r.ownerTables.length === 2,
      `got ${r.ownerTables.length}`);
    const sessionIdClaim = findCanonicalOwnerClaim(r.ownerTables, 'SessionId');
    assert('T3c. SessionId owner is src/protocol/ids.ts',
      sessionIdClaim?.ownerFile === 'src/protocol/ids.ts',
      `got ${sessionIdClaim?.ownerFile}`);
    assert('T3d. SessionId line number non-zero',
      sessionIdClaim?.line > 0,
      `got line=${sessionIdClaim?.line}`);
    const internalFlagClaim = findCanonicalOwnerClaim(r.ownerTables, 'InternalFlag');
    assert('T3e. InternalFlag claim found (weak-table also parsed)',
      internalFlagClaim?.ownerFile === 'src/engine/flag.ts');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ─── T4. Recognized schema via Source header ──────────────
{
  const { dir, filePath } = fx('src-header', `# type-ownership.md

> **Source:** \`_lib/extract-ts.mjs\` pass (42 files scanned)

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Status |
|---|---|---|---|---:|---|
| \`Token\` | \`src/auth/token.ts\` | TSInterfaceDeclaration | 7 | 4 | ✅ |
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T4. Source header → recognized:true',
      r.recognized === true);
    const tok = findCanonicalOwnerClaim(r.ownerTables, 'Token');
    assert('T4b. Token claim via Source-header path',
      tok?.ownerFile === 'src/auth/token.ts');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ─── T5. DUPLICATE section must NOT produce owner rows ────
//
// Group-level sections (DUPLICATE_STRONG, DUPLICATE_REVIEW, LOCAL_COMMON_NAME,
// ANY_COLLISION, low-signal-type-name) are NOT owner claims. Parser skips.
{
  const { dir, filePath } = fx('dup-section', `# type-ownership.md

> **Status:** draft, v1

### 2.4 DUPLICATE_STRONG — likely shared concept, needs resolution

| Type | Files defining | Kinds | Max fan-in | Total fan-in | Tags | Suggested action |
|---|---|---|---:|---:|---|---|
| \`Result\` | \`src/a.ts:5\`, \`src/b.ts:22\` | 2× TSTypeAliasDeclaration | 18 | 21 | — | pick one |

### 2.6 LOCAL_COMMON_NAME

| Name | Locations | Count | Tags | Note |
|---|---|---:|---|---|
| \`Props\` | 14 files | 14 | — | — |
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T5. recognized but only group-level sections → no owner tables',
      r.recognized === true && r.ownerTables.length === 0,
      `ownerTables.length=${r.ownerTables.length}`);
    assert('T5b. Result is NOT treated as a single-owner claim',
      findCanonicalOwnerClaim(r.ownerTables, 'Result') === null);
    assert('T5c. Props is NOT treated as a single-owner claim',
      findCanonicalOwnerClaim(r.ownerTables, 'Props') === null);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ─── T6. severely-any-contaminated section IS owner-level ──
{
  const { dir, filePath } = fx('severe-section', `# type-ownership.md

> **Status:** draft, v1

### 2.3 severely-any-contaminated (single-owner, Rule 0)

| Type | Owner | Kind | Line | Fan-in | Tags | Any / unknown signal |
|---|---|---|---|---:|---|---|
| \`LegacyPayload\` | \`src/legacy/payload.ts\` | TSInterfaceDeclaration | 3 | 6 | — | severely-any-contaminated (anyFieldRatio 0.85) |
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T6. severely-any-contaminated section → parsed as owner table',
      r.recognized === true && r.ownerTables.length === 1);
    const legacy = findCanonicalOwnerClaim(r.ownerTables, 'LegacyPayload');
    assert('T6b. LegacyPayload owner extracted from severe table',
      legacy?.ownerFile === 'src/legacy/payload.ts');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ─── T7. Mixed file: recognized header + free-form prose ──
//
// A file with a recognized header but extra free-form prose must parse
// only its recognized owner tables; the free-form prose is ignored.
{
  const { dir, filePath } = fx('mixed', `# type-ownership.md

> **Status:** draft, v1

This is a paragraph of free-form prose that happens to mention \`SessionId\`
and could confuse a naive parser.

Some bullets:
- \`User\` lives somewhere
- \`Token\` is also here

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Status |
|---|---|---|---|---:|---|
| \`SessionId\` | \`src/protocol/ids.ts\` | TSTypeAliasDeclaration | 14 | 8 | ✅ |
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T7. mixed file → recognized:true',
      r.recognized === true);
    assert('T7b. mixed file → free-form prose NOT parsed as owner rows',
      r.ownerTables.length === 1 && r.ownerTables[0].rows.length === 1,
      `tables=${r.ownerTables.length}, rows=${r.ownerTables[0]?.rows?.length}`);
    assert('T7c. only SessionId parsed, not User/Token from prose bullets',
      findCanonicalOwnerClaim(r.ownerTables, 'SessionId') !== null &&
      findCanonicalOwnerClaim(r.ownerTables, 'User') === null &&
      findCanonicalOwnerClaim(r.ownerTables, 'Token') === null);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ─── T8. findCanonicalOwnerClaim returns null on miss ──────
assert('T8. findCanonicalOwnerClaim on empty tables → null',
  findCanonicalOwnerClaim([], 'Anything') === null);

// ─── T9. Current flat type-ownership table ─────────────────
//
// Current type ownership drafts use a flat Name/Identity/Owner table
// with display-only Fan-in space. Pre-write canonical lookup should read
// single-owner claims from that shape while keeping duplicate/common rows out
// of the owner-claim lane.
{
  const { dir, filePath } = fx('flat-current', `# Type ownership draft

Generated: 2026-05-05T00:00:00.000Z
Scope: TS/JS production files
Source: fresh-ast-pass

| Name | Identity | Owner | Fan-in | Fan-in space | Status | Tags |
|------|----------|-------|-------:|--------------|--------|------|
| \`Session\` | \`src/session.ts::Session\` | \`src/session.ts:10\` | 3 | value 2, type 1, broad 0 | single-owner-strong ✅ | |
| \`Result\` | \`src/a.ts::Result\` | \`src/a.ts:1\` | 3 | value 3, type 0, broad 0 | DUPLICATE_STRONG ❌ | |
| \`Props\` | \`src/card.ts::Props\` | \`src/card.ts:4\` | 1 | value 1, type 0, broad 0 | LOCAL_COMMON_NAME ⚠ | |
`);
  try {
    const r = parseCanonicalFile(filePath);
    assert('T9. current flat type-ownership draft → recognized:true',
      r.recognized === true,
      `recognized=${r.recognized}, reason=${r.reason}`);
    const session = findCanonicalOwnerClaim(r.ownerTables, 'Session');
    assert('T9b. flat table Session owner comes from identity owner file',
      session?.ownerFile === 'src/session.ts',
      `claim=${JSON.stringify(session)}`);
    assert('T9c. Fan-in space does not block owner claim extraction',
      session?.line > 0,
      `claim=${JSON.stringify(session)}`);
    assert('T9d. duplicate/common flat rows stay out of owner claims',
      findCanonicalOwnerClaim(r.ownerTables, 'Result') === null &&
      findCanonicalOwnerClaim(r.ownerTables, 'Props') === null,
      `tables=${JSON.stringify(r.ownerTables, null, 2)}`);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
