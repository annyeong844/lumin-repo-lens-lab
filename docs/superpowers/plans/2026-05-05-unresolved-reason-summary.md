# Unresolved Reason Summary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a reason-grouped unresolved-internal summary to `symbols.json` so large-repo blind zones are explainable without loosening resolver or ranking safety.

**Architecture:** Reuse existing `unresolvedInternalSpecifierRecords` as the source of truth. Add a pure artifact builder in `_lib/symbol-graph-artifact.mjs` that groups records by `reason`, counts examples, preserves top `hint`/`resolverStage` metadata, and emits deterministic output. Do not change resolver behavior or ranking.

**Tech Stack:** Node.js ESM, existing fixture tests, `build-symbol-graph.mjs`, `symbols.json`.

---

### Task 1: Add Failing Artifact Test

**Files:**
- Modify: `tests/test-tsconfig-paths-scoped.mjs`

- [ ] **Step 1: Write the failing test**

Append assertions after the existing `T21` and `T22` unresolved-record checks:

```js
const summary = syms5.unresolvedInternalSummaryByReason ?? {};
assert('T23. unresolved summary counts tsconfig target misses by reason',
  summary['tsconfig-path-target-missing']?.count === 1 &&
    summary['tsconfig-path-target-missing']?.hints?.['generated-artifact-missing'] === 1 &&
    summary['tsconfig-path-target-missing']?.examples?.some((r) =>
      r.specifier === '@scope/generated-client' &&
      r.consumerFile === 'apps/web/src/consumer.ts'),
  `summary=${JSON.stringify(summary)}`);

assert('T24. unresolved summary counts workspace subpath misses by reason',
  summary['workspace-package-subpath-target-missing']?.count === 1 &&
    summary['workspace-package-subpath-target-missing']?.examples?.some((r) =>
      r.specifier === '@scope/types/thing' &&
      r.matchedPattern === '@scope/types/*'),
  `summary=${JSON.stringify(summary)}`);
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
node tests/test-tsconfig-paths-scoped.mjs
```

Expected: `T23` or `T24` fails because `unresolvedInternalSummaryByReason` is missing.

### Task 2: Build Summary From Existing Records

**Files:**
- Modify: `_lib/symbol-graph-artifact.mjs`

- [ ] **Step 1: Add deterministic summary helper**

Add this pure helper near `buildTopUnresolvedSpecifiers`:

```js
function compactUnresolvedExample(record) {
  return {
    specifier: record.specifier,
    consumerFile: record.consumerFile,
    kind: record.kind,
    ...(record.resolverStage ? { resolverStage: record.resolverStage } : {}),
    ...(record.matchedPattern ? { matchedPattern: record.matchedPattern } : {}),
    ...(record.hint ? { hint: record.hint } : {}),
    ...(Array.isArray(record.targetCandidates) && record.targetCandidates.length
      ? { targetCandidates: record.targetCandidates.slice(0, 3) }
      : {}),
  };
}

function buildUnresolvedInternalSummaryByReason(records) {
  const groups = new Map();
  for (const record of records ?? []) {
    const reason = record?.reason ?? 'unknown-internal-resolution';
    if (!groups.has(reason)) {
      groups.set(reason, {
        count: 0,
        resolverStages: new Map(),
        hints: new Map(),
        examples: [],
      });
    }
    const group = groups.get(reason);
    group.count++;
    if (record.resolverStage) {
      group.resolverStages.set(
        record.resolverStage,
        (group.resolverStages.get(record.resolverStage) ?? 0) + 1,
      );
    }
    if (record.hint) {
      group.hints.set(record.hint, (group.hints.get(record.hint) ?? 0) + 1);
    }
    if (group.examples.length < 5) {
      group.examples.push(compactUnresolvedExample(record));
    }
  }

  return Object.fromEntries([...groups.entries()]
    .sort((a, b) => b[1].count - a[1].count || a[0].localeCompare(b[0]))
    .map(([reason, group]) => [reason, {
      count: group.count,
      ...(group.resolverStages.size
        ? { resolverStages: Object.fromEntries([...group.resolverStages.entries()].sort()) }
        : {}),
      ...(group.hints.size
        ? { hints: Object.fromEntries([...group.hints.entries()].sort()) }
        : {}),
      examples: group.examples.sort((a, b) =>
        `${a.consumerFile ?? ''}|${a.specifier ?? ''}|${a.kind ?? ''}`
          .localeCompare(`${b.consumerFile ?? ''}|${b.specifier ?? ''}|${b.kind ?? ''}`)),
    }]));
}
```

- [ ] **Step 2: Attach helper output to symbols artifact**

In the returned artifact object, add:

```js
unresolvedInternalSummaryByReason:
  buildUnresolvedInternalSummaryByReason(unresolvedInternalSpecifierRecords),
```

Place it next to `unresolvedInternalSpecifierRecords` so consumers can find the detailed records and grouped summary together.

- [ ] **Step 3: Run test to verify it passes**

Run:

```bash
node tests/test-tsconfig-paths-scoped.mjs
```

Expected: all assertions pass, including `T23` and `T24`.

### Task 3: Regression And Package Validation

**Files:**
- No production changes beyond Task 2.

- [ ] **Step 1: Run focused resolver tests**

Run:

```bash
node tests/test-tsconfig-paths-scoped.mjs
node tests/test-workspace-no-exports.mjs
```

Expected: both pass.

- [ ] **Step 2: Run syntax and drift checks**

Run:

```bash
npm run check
npm run check:drift
```

Expected: both pass.

- [ ] **Step 3: Rebuild generated skill surface**

Run:

```bash
npm run build:skill
npm run build:plugin
```

Expected: generated skill mirror includes `_lib/symbol-graph-artifact.mjs` with the summary builder.

- [ ] **Step 4: Run public package dry-run**

Run:

```bash
npm run check:public-plugin
```

Expected: dry-run prepares the current package without version drift.

### Task 4: Commit

**Files:**
- Modify: `_lib/symbol-graph-artifact.mjs`
- Modify: `tests/test-tsconfig-paths-scoped.mjs`
- Modify generated mirror files from `npm run build:skill`

- [ ] **Step 1: Review diff**

Run:

```bash
git diff --stat
git diff --check
```

Expected: diff contains only the artifact summary, tests, and generated mirror.

- [ ] **Step 2: Commit**

Run:

```bash
git add _lib/symbol-graph-artifact.mjs tests/test-tsconfig-paths-scoped.mjs skills/lumin-repo-lens-lab/_engine/lib/symbol-graph-artifact.mjs docs/superpowers/plans/2026-05-05-unresolved-reason-summary.md
git commit -m "Add unresolved internal reason summary"
```
