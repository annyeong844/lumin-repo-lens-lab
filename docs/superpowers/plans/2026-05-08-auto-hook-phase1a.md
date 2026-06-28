# Auto Hook Phase 1A Implementation Plan

> **For agentic workers:** Implement this plan task-by-task with TDD. Subagents are optional only when the human explicitly asks for parallel workers. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first auto-hook substrate: path safety, hook id safety, a hook manifest, and a doctor command, without enabling analysis or reminders yet.

**Architecture:** Keep Phase 1A as infrastructure only. Hook analysis lanes remain unimplemented; this slice creates reusable safety helpers and a diagnostic doctor so later `PreToolUse` / `PostToolBatch` scripts can share the same root, path, and id contracts. No hook blocks user work; all future hook commands must exit 0 by contract.

**Tech Stack:** Node.js ESM, built-in `node:test`-style direct scripts, existing npm lint/check scripts, JSON hook manifest.

---

## File Structure

- Create `_lib/hook-path-safety.mjs`
  - Owns workspace/package/audit-root discovery and repo-relative path safety.
  - Exposes `resolveWorkspaceRoot`, `resolvePackageRoot`, `resolveAuditRoot`, `safeRepoPathForToolInput`, `safeRepoRelForRead`, `safeRepoPathSyntactic`, and `getToolTargetPath`.
- Create `_lib/hook-id-safety.mjs`
  - Owns safe hook/session/tool-use ids.
  - Exposes `isSafeId`, `safeSessionId`, `safeToolUseId`, and content-bearing fallback hashing.
- Create `hooks/hooks.json`
  - Declares a hook manifest in the shape expected by the auto-hook design.
  - The manifest stays empty in this slice (`{"hooks": {}}`) so installing the package does not enable any automatic behavior.
- Create `scripts/hook-doctor.mjs`
  - Checks whether the current repo has hook-capable root discovery, required files, and safe JSON manifest shape.
  - Prints deterministic human-readable status and exits non-zero only for local diagnostic failures.
- Create `tests/test-hook-path-safety.mjs`
  - Covers root discovery and path safety.
- Create `tests/test-hook-id-safety.mjs`
  - Covers session/tool id validation and deterministic fallback ids.
- Create `tests/test-hook-doctor.mjs`
  - Covers hook manifest discovery and doctor output.
- Modify `scripts/update-test-doc.mjs`
  - Adds descriptions for the three new hook tests.
- Modify `scripts/build-plugin-package.mjs`
  - Stages `hooks/` into the plugin-root package so the manifest ships with the package shell.

## Task 1: Path Safety Helpers

**Files:**
- Create: `_lib/hook-path-safety.mjs`
- Test: `tests/test-hook-path-safety.mjs`

- [ ] **Step 1: Write failing path safety tests**

Create `tests/test-hook-path-safety.mjs` with assertions for:

```js
resolveWorkspaceRoot(nestedCwd) === fixtureRoot
resolvePackageRoot(nestedCwd) === nearestPackageRoot
resolveAuditRoot(nestedCwd) === path.join(fixtureRoot, '.audit')
getToolTargetPath('Edit', { file_path: 'src/a.ts' }) === 'src/a.ts'
safeRepoPathForToolInput(nestedCwd, 'src/a.ts').ok === true
safeRepoPathForToolInput(nestedCwd, '../outside.ts').ok === false
safeRepoPathSyntactic('src/a.ts').ok === true
safeRepoPathSyntactic('../outside.ts').ok === false
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-path-safety.mjs
```

Expected: fails because `_lib/hook-path-safety.mjs` does not exist.

- [ ] **Step 3: Implement minimal helper**

Implement only the functions covered by the test. Use `git rev-parse --show-toplevel` when available, then ancestor walk for `.git`, `pnpm-workspace.yaml`, `package.json#workspaces`, and nearest `package.json`.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-path-safety.mjs
```

Expected: all path safety assertions pass.

## Task 2: Hook ID Safety

**Files:**
- Create: `_lib/hook-id-safety.mjs`
- Test: `tests/test-hook-id-safety.mjs`

- [ ] **Step 1: Write failing id safety tests**

Create `tests/test-hook-id-safety.mjs` with assertions for:

```js
isSafeId('abc_123-XYZ') === true
isSafeId('../bad') === false
safeSessionId({ session_id: 'sid_123' }) === 'sid_123'
safeSessionId({ transcript_path: '/tmp/transcript.jsonl' }).startsWith('sid_') === true
safeToolUseId({ tool_use_id: 'tool_123' }) === 'tool_123'
safeToolUseId(payloadA) === safeToolUseId(payloadB)
safeToolUseId(payloadA) does not include raw Write.content
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-id-safety.mjs
```

Expected: fails because `_lib/hook-id-safety.mjs` does not exist.

- [ ] **Step 3: Implement minimal id helper**

Use `sha256` for fallback ids. Content-bearing fields such as `content`, `old_string`, and `new_string` must contribute only `{sha256, byteLength}`, never raw content.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-id-safety.mjs
```

Expected: all id safety assertions pass.

## Task 3: Hook Manifest And Doctor

**Files:**
- Create: `hooks/hooks.json`
- Create: `scripts/hook-doctor.mjs`
- Test: `tests/test-hook-doctor.mjs`

- [ ] **Step 1: Write failing doctor tests**

Create `tests/test-hook-doctor.mjs` with assertions that:

```js
hooks/hooks.json exists and parses as JSON
doctor output includes "hook doctor"
doctor output includes "workspaceRoot"
doctor output includes "hooks/hooks.json"
doctor exits 0 in the repo root
```

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-hook-doctor.mjs
```

Expected: fails because the manifest or doctor script does not exist.

- [ ] **Step 3: Add hook manifest and doctor**

Add `hooks/hooks.json` with no active hook events (`{"hooks": {}}`). Add `scripts/hook-doctor.mjs` that reads root information through `_lib/hook-path-safety.mjs`, validates the manifest, prints a concise report, and exits 0 when checks pass.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-hook-doctor.mjs
```

Expected: all doctor assertions pass.

## Task 4: Test Docs And Verification

**Files:**
- Modify: `scripts/update-test-doc.mjs`
- Generated/checked: `tests/README.md` if the repo expects generated test docs to change.

- [ ] **Step 1: Add test descriptions**

Add:

```js
'test-hook-path-safety.mjs': 'auto-hook Phase 1A path/root safety helpers',
'test-hook-id-safety.mjs': 'auto-hook Phase 1A session/tool id safety helpers',
'test-hook-doctor.mjs': 'auto-hook Phase 1A hook manifest and doctor smoke test',
```

- [ ] **Step 2: Run targeted validation**

Run:

```bash
node tests/test-hook-path-safety.mjs
node tests/test-hook-id-safety.mjs
node tests/test-hook-doctor.mjs
npm run check
npm run lint
```

Expected: all commands exit 0.

## Task 5: Plugin Package Staging

**Files:**
- Modify: `scripts/build-plugin-package.mjs`
- Test: `tests/test-plugin-package.mjs`

- [ ] **Step 1: Write failing plugin package test**

Add a plugin-root packaging assertion that `hooks/hooks.json` exists in the staged plugin output.

- [ ] **Step 2: Verify RED**

Run:

```bash
node tests/test-plugin-package.mjs
```

Expected: fails because `build-plugin-package.mjs` does not copy `hooks/`.

- [ ] **Step 3: Copy hooks into plugin root package**

Have `build-plugin-package.mjs` copy `hooks/` when the directory exists.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
node tests/test-plugin-package.mjs
```

Expected: the plugin package includes `hooks/hooks.json`.

- [ ] **Step 5: Commit**

Run:

```bash
git add _lib/hook-path-safety.mjs _lib/hook-id-safety.mjs hooks/hooks.json scripts/hook-doctor.mjs scripts/build-plugin-package.mjs tests/test-hook-path-safety.mjs tests/test-hook-id-safety.mjs tests/test-hook-doctor.mjs tests/test-plugin-package.mjs scripts/update-test-doc.mjs tests/README.md docs/superpowers/plans/2026-05-08-auto-hook-phase1a.md
git commit -m "Add auto hook phase 1A substrate"
```

## Self-Review

- Scope is intentionally limited to substrate and diagnostics.
- No analysis lane, reminders, ACK loop, or blocking behavior is implemented in Phase 1A.
- Every runtime helper added here is directly covered by a targeted test.
- Hook manifest exists for shape validation, not for automatic user-facing enforcement.
