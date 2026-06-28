# Function Signature Cues Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking completed work.

**Goal:** Surface exact exported function type-signature collisions so pre-write can warn about same-contract helpers even when names and bodies differ.

**Architecture:** Add a small shared function-signature normalizer, attach signature hashes to `function-clones.json`, and let pre-write `shapes[].typeLiteral` consult those hashes when the intent is a function type. This remains review-only evidence and does not claim semantic equivalence.

**Tech Stack:** Node.js ESM, OXC AST, existing producer/test harness.

---

### Task 1: RED Tests For Signature Cues

**Files:**
- Modify: `tests/test-build-function-clone-index.mjs`
- Modify: `tests/test-pre-write-cli.mjs`

- [x] **Step 1: Add a function-clone test for identical generic signatures with different bodies**

Add a fixture with:

```ts
export function useShallow<S, U>(selector: (state: S) => U): (state: S) => U {
  return selector;
}

export function useShallowDuplicate<S, U>(selector: (state: S) => U): (state: S) => U {
  return (state) => selector(state);
}
```

Expected: no exact/structure body group is required, but `signatureGroups` contains both identities and `meta.signatureGroupCount === 1`.

- [x] **Step 2: Add a pre-write test for a different-name same-signature intent**

Use intent:

```json
{
  "names": ["composeProjection"],
  "shapes": [
    {
      "fields": [],
      "typeLiteral": "<S, U>(selector: (state: S) => U) => (state: S) => U"
    }
  ],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}
```

Expected: pre-write cold-caches `function-clones.json` and renders a grounded function-signature match for the existing `useShallow` identity.

- [x] **Step 3: Run tests and verify RED**

Run:

```bash
node tests/test-build-function-clone-index.mjs
node tests/test-pre-write-cli.mjs
```

Expected: the new assertions fail because `signatureGroups` and function-signature lookup do not exist yet.

### Task 2: Shared Signature Normalizer

**Files:**
- Create: `_lib/function-signature-hash.mjs`

- [x] **Step 1: Implement a small normalizer**

Create helpers that accept either a `FunctionDeclaration`/function-like node or a `TSFunctionType` alias annotation and produce:

```js
{
  ok: true,
  hash: "sha256:<64hex>",
  signature: "<$T0,$T1>(($T0)=>$T1):($T0)=>$T1",
  normalizedSignature: {
    schemaVersion: "function-signature.normalized.v1",
    typeParameters: [...],
    params: [...],
    returnType: ...
  }
}
```

The normalizer must ignore top-level parameter names and normalize type parameter names by position. Unsupported or untyped functions return `{ ok: false, reason }` rather than broad matching.

- [x] **Step 2: Run targeted syntax checks**

Run:

```bash
node --check _lib/function-signature-hash.mjs
```

Expected: no syntax errors.

### Task 3: Function Clone Artifact Signature Groups

**Files:**
- Modify: `_lib/function-clone-artifact.mjs`
- Modify generated mirror later through `npm run build:skill`

- [x] **Step 1: Add signature fields to function facts**

For exported top-level typed functions, add:

```js
normalizedSignatureHash
signature
signatureParamCount
```

Do not add signature hashes for untyped JS helpers.

- [x] **Step 2: Add `signatureGroups` to the artifact**

Group facts by `normalizedSignatureHash` with `minSize = 2`. Emit review-only groups with identities, owner files, exported names, lines, signature text, and reason text that says this is a same type-contract cue, not semantic equivalence.

- [x] **Step 3: Update metadata**

Add:

```js
meta.signatureGroupCount
meta.supports.normalizedFunctionSignatureHash = true
meta.supports.functionSignatureGroups = true
```

If the schema version is bumped, update the existing test expectation.

- [x] **Step 4: Verify GREEN for function-clone tests**

Run:

```bash
node tests/test-build-function-clone-index.mjs
```

Expected: all tests pass.

### Task 4: Pre-Write Function Signature Lookup

**Files:**
- Modify: `_lib/pre-write-cold-cache.mjs`
- Modify: `_lib/pre-write-lookup-shape.mjs`
- Modify: `pre-write.mjs`
- Modify: `_lib/pre-write-render.mjs` only if rendering needs a clearer label

- [x] **Step 1: Cold-cache function clones only for function typeLiteral intents**

When `intent.shapes[].typeLiteral` normalizes as a function signature, add `function-clones.json` to the cold-cache producer set. Do not cold-cache it for fields-only object shapes.

- [x] **Step 2: Load `function-clones.json` in pre-write**

Pass `{ shapeIndex, functionClones }` to `lookupShape`.

- [x] **Step 3: Add function-signature matching in `lookupShape`**

For function type literals, match against `functionClones.facts[].normalizedSignatureHash`. Return a watch-for lookup with `result: "SIGNATURE_MATCH"` and grounded citations when matches exist. Keep `UNAVAILABLE` when the artifact is missing or incomplete.

- [x] **Step 4: Verify GREEN for pre-write tests**

Run:

```bash
node tests/test-pre-write-cli.mjs
```

Expected: all tests pass, including the new different-name same-signature case.

### Task 5: Docs, Mirror, And Validation

**Files:**
- Modify: `README.md`
- Modify generated skill files through `npm run build:skill`

- [x] **Step 1: Tighten README claim**

Say full/pre-write can surface exact function signature collisions when function-clone evidence exists, while semantic equivalence still requires review.

- [x] **Step 2: Build mirror**

Run:

```bash
npm run build:skill
```

- [x] **Step 3: Run focused validation**

Run:

```bash
node tests/test-build-function-clone-index.mjs
node tests/test-pre-write-cli.mjs
npm run check:drift
node tests/test-skill-package.mjs
git diff --check
```

Expected: all pass. Do not run full local CI unless the touched code broadens beyond this slice.

### Follow-Ups Not In This Slice

- Down-rank common verb tokens such as `create` in near-name pre-write lookup so `createLogger` does not pull broad `create*` families without stronger evidence.
- Investigate an opt-in ast-grep-backed fallback for unsupported shape/function patterns. This must stay review-only and should not replace the grounded signature hash lane added here.
