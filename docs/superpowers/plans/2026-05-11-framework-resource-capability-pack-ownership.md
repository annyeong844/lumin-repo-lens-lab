# Framework Resource Capability Pack Ownership Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable capability-pack ownership metadata to framework/resource surface diagnostics.

**Architecture:** Keep `framework-resource-surfaces.json` classifier-only and evidence-only. Each surface lane carries a `capabilityPack` string, and the artifact summary exposes `byCapabilityPack` so WT-21 consumers can reason about support ownership without inferring it from `lane`, `reason`, or `framework`.

**Tech Stack:** Node.js ESM, built-in `assert`, existing framework/resource surface classifier and producer tests.

---

### Task 1: Pin Capability Pack Fields In Tests

**Files:**
- Modify: `tests/test-framework-resource-surfaces.mjs`
- Modify: `tests/test-build-framework-resource-surfaces.mjs`

- [ ] **Step 1: Write failing assertions**

Add assertions that the existing fixture surfaces include:

```js
assert.equal(story.surfaceLanes[0].capabilityPack, 'framework.storybook');
assert.equal(strapi.surfaceLanes[0].capabilityPack, 'framework.strapi');
assert.equal(artifact.summary.byCapabilityPack['framework.storybook'], 1);
assert.equal(artifact.summary.byCapabilityPack['framework.strapi'], 1);
assert.equal(artifact.summary.byCapabilityPack['surface.bundled-build-artifact'], 3);
assert.equal(artifact.summary.byCapabilityPack['surface.generated-declaration'], 1);
assert.equal(artifact.summary.byCapabilityPack['surface.scaffold-template'], 1);
assert.equal(artifact.summary.byCapabilityPack['surface.codemod-resource'], 1);
```

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```bash
node tests/test-framework-resource-surfaces.mjs
node tests/test-build-framework-resource-surfaces.mjs
```

Expected: fail because `capabilityPack` and `summary.byCapabilityPack` are absent.

### Task 2: Add Capability Pack Ownership To Classifier

**Files:**
- Modify: `_lib/framework-resource-surfaces.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/lib/framework-resource-surfaces.mjs`

- [ ] **Step 1: Implement minimal classifier fields**

Update `surfaceLane` to accept `capabilityPack`, and set these pack IDs in lane constructors:

```js
framework.storybook
framework.strapi
surface.generated-declaration
surface.bundled-build-artifact
surface.scaffold-template
surface.codemod-resource
```

- [ ] **Step 2: Add summary pivot**

Update `buildSummary` to increment and emit:

```js
byCapabilityPack: sortedObject(byCapabilityPack)
```

- [ ] **Step 3: Run focused tests and verify GREEN**

Run:

```bash
node tests/test-framework-resource-surfaces.mjs
node tests/test-build-framework-resource-surfaces.mjs
```

Expected: both pass.

### Task 3: Record WT-21 Tracker State

**Files:**
- Modify: `docs/spec/lumin-work-tracker.md`

- [ ] **Step 1: Update WT-21 current state**

Add a sentence noting that framework/resource surface diagnostics carry stable `capabilityPack` ownership and `summary.byCapabilityPack`.

- [ ] **Step 2: Run formatting/checks**

Run:

```bash
npm run check:drift
npm run check:test-doc
npm run lint
```

Expected: all pass.

### Task 4: Final Verification

**Files:**
- No additional edits unless verification reveals a regression.

- [ ] **Step 1: Run full targeted verification**

Run:

```bash
node tests/test-framework-resource-surfaces.mjs
node tests/test-build-framework-resource-surfaces.mjs
node tests/test-audit-repo.mjs
npm run check
```

Expected: all pass.

- [ ] **Step 2: Commit**

Commit with:

```bash
git add _lib/framework-resource-surfaces.mjs skills/lumin-repo-lens-lab/_engine/lib/framework-resource-surfaces.mjs tests/test-framework-resource-surfaces.mjs tests/test-build-framework-resource-surfaces.mjs docs/spec/lumin-work-tracker.md docs/superpowers/plans/2026-05-11-framework-resource-capability-pack-ownership.md
git commit -m "Add framework resource capability pack ownership [skip ci]"
```
