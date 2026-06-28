# Public Repo CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish a lightweight GitHub Actions workflow into the existing public `annyeong844/lumin-repo-lens-lab` package repo so public package pushes verify installability and smoke behavior without spending private CI minutes.

**Architecture:** Keep the workflow source in the private maintainer repo under `public-package/.github/workflows/ci.yml`. Extend `scripts/publish-public-plugin.mjs` so public dry-run/push syncs that workflow alongside the generated plugin package and validates that the workflow exists in the public checkout.

**Tech Stack:** Node.js ESM scripts, local git fixture tests, GitHub Actions YAML, existing `skills/lumin-repo-lens-lab` package smoke script.

---

## File Structure

- Create `public-package/.github/workflows/ci.yml`
  - Public package workflow template. It must reference only files present in the public plugin package.
- Modify `scripts/publish-public-plugin.mjs`
  - Copy the public workflow template to the public checkout.
  - Validate the synced workflow exists.
- Modify `tests/test-publish-public-plugin.mjs`
  - Add assertions that dry-run and push carry the public CI workflow.
  - Assert the workflow does not reference maintainer-only paths.
- Modify `scripts/update-test-doc.mjs` only if test suite descriptions change.
  - This plan does not add a new suite, so no update is expected.

## Task 1: Lock Publisher Workflow Sync With A Failing Test

**Files:**
- Modify: `tests/test-publish-public-plugin.mjs`

- [ ] **Step 1: Add dry-run workflow assertions**

In `tests/test-publish-public-plugin.mjs`, after the existing `PPUB4` changelog assertion, add:

```js
  const workflowPath = path.join(checkout, '.github/workflows/ci.yml');
  const workflowText = readFileSync(workflowPath, 'utf8');
  assert('PPUB4b. dry-run syncs public package CI workflow',
    existsSync(workflowPath) &&
      workflowText.includes('npm ci') &&
      workflowText.includes('npm run smoke') &&
      workflowText.includes('node skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --help'),
    workflowText);
  assert('PPUB4c. public package CI workflow does not reference maintainer-only paths',
    !/\b(tests|test-harness|docs\/spec|_lib|p6-corpus)\b/.test(workflowText),
    workflowText);
```

- [ ] **Step 2: Add pushed workflow assertion**

After the existing `PPUB6` assertion, add:

```js
  const pushedWorkflow = git(['--git-dir', bare, 'show', 'main:.github/workflows/ci.yml'], ROOT);
  assert('PPUB6b. pushed public repo includes package CI workflow',
    pushedWorkflow.includes('name: Public Package CI') &&
      pushedWorkflow.includes('working-directory: skills/lumin-repo-lens-lab') &&
      pushedWorkflow.includes('npm run smoke'),
    pushedWorkflow);
```

- [ ] **Step 3: Run test to verify RED**

Run:

```bash
node tests/test-publish-public-plugin.mjs
```

Expected: FAIL on `PPUB4b` because `.github/workflows/ci.yml` is not copied to the public checkout yet.

## Task 2: Add Public Package Workflow Template

**Files:**
- Create: `public-package/.github/workflows/ci.yml`

- [ ] **Step 1: Create workflow template**

Create `public-package/.github/workflows/ci.yml` with:

```yaml
name: Public Package CI

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  package-smoke:
    name: Package Smoke (Node ${{ matrix.node }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        node: ['20.x', '22.x']
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
          cache: npm
          cache-dependency-path: skills/lumin-repo-lens-lab/package-lock.json
      - name: Install packaged skill dependencies
        working-directory: skills/lumin-repo-lens-lab
        run: npm ci
      - name: Smoke packaged audit CLI
        working-directory: skills/lumin-repo-lens-lab
        run: npm run smoke
      - name: Verify packaged CLI help
        run: node skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --help
```

- [ ] **Step 2: Run the failing test again**

Run:

```bash
node tests/test-publish-public-plugin.mjs
```

Expected: still FAIL on `PPUB4b`, because the template exists but the publisher does not copy it yet.

## Task 3: Copy And Validate Public Workflow In Publisher

**Files:**
- Modify: `scripts/publish-public-plugin.mjs`

- [ ] **Step 1: Add workflow constants**

Near the existing `PUBLIC_ROOT_DOCS`, `SYNC_DIRS`, and `DISALLOWED_ROOT_ENTRIES` constants, add:

```js
const PUBLIC_WORKFLOW_SOURCE = path.join(ROOT, 'public-package/.github/workflows/ci.yml');
const PUBLIC_WORKFLOW_DEST = '.github/workflows/ci.yml';
```

- [ ] **Step 2: Copy workflow during sync**

In `syncPublicCheckout({ checkoutDir, distDir })`, after copying root docs, add:

```js
  copyFileIfExists(PUBLIC_WORKFLOW_SOURCE, path.join(checkoutDir, PUBLIC_WORKFLOW_DEST));
```

- [ ] **Step 3: Validate workflow exists**

In `validatePackageSurface(checkoutDir, expectedVersion)`, add `PUBLIC_WORKFLOW_DEST` to the required file list:

```js
    PUBLIC_WORKFLOW_DEST,
```

The required-file block should include:

```js
  for (const rel of [
    '.claude-plugin/plugin.json',
    '.claude-plugin/marketplace.json',
    'commands/lumin-repo-lens-lab.md',
    'skills/lumin-repo-lens-lab/SKILL.md',
    'skills/lumin-repo-lens-lab-write-gate/SKILL.md',
    'skills/lumin-repo-lens-lab-canon/SKILL.md',
    'README.plugin-package.md',
    PUBLIC_WORKFLOW_DEST,
  ]) {
    if (!existsSync(path.join(checkoutDir, rel))) {
      throw new Error(`public plugin package missing required file: ${rel}`);
    }
  }
```

- [ ] **Step 4: Run publisher test to verify GREEN**

Run:

```bash
node tests/test-publish-public-plugin.mjs
```

Expected: PASS all publisher assertions, including `PPUB4b`, `PPUB4c`, and `PPUB6b`.

## Task 4: Verify Public Package Dry Run

**Files:**
- No code changes.

- [ ] **Step 1: Run public plugin dry-run**

Run:

```bash
npm run check:public-plugin
```

Expected:

- command exits 0;
- output includes either `prepared public package` or `public package already up to date`;
- no maintainer-only root entries are reported.

- [ ] **Step 2: Run local package smoke directly**

Run:

```bash
pushd skills/lumin-repo-lens-lab
npm ci
npm run smoke
node scripts/audit-repo.mjs --help
popd
```

Expected:

- `npm ci` exits 0;
- `npm run smoke` prints `[smoke-test] ok: ...`;
- `audit-repo.mjs --help` prints `lumin-repo-lens-lab public CLI`.

## Task 5: Final Maintainer Verification

**Files:**
- No code changes unless a check fails.

- [ ] **Step 1: Run focused checks**

Run:

```bash
node tests/test-publish-public-plugin.mjs
npm run check
npm run check:drift
npm run check:test-doc
npm run check:doc-script-refs
npm run lint
git diff --check
```

Expected: all commands exit 0.

- [ ] **Step 2: Commit implementation**

Run:

```bash
git add public-package/.github/workflows/ci.yml scripts/publish-public-plugin.mjs tests/test-publish-public-plugin.mjs
git commit -m "Add public package CI workflow"
```

Expected: commit succeeds.

- [ ] **Step 3: Open draft PR**

Run:

```bash
git push -u origin codex/public-repo-ci-design
```

Open a draft PR against `annyeong844/lumin_lab:main` with:

```markdown
## Summary

- adds a public package CI workflow template for `annyeong844/lumin-repo-lens-lab`
- syncs the workflow through `scripts/publish-public-plugin.mjs`
- validates workflow presence in public package dry-run/push tests

## Validation

- node tests/test-publish-public-plugin.mjs
- npm run check
- npm run check:drift
- npm run check:test-doc
- npm run check:doc-script-refs
- npm run lint
- git diff --check

No private server CI is required for this draft PR.
```

Expected: draft PR is open and private server CI remains skipped while draft.

## Self-Review

- Spec coverage: The plan implements the design's stable workflow source, public package sync, public package-only checks, no maintainer-only paths, dry-run validation, and private CI conservation.
- Red-flag scan: No deferred or unresolved implementation steps remain.
- Type consistency: Constants `PUBLIC_WORKFLOW_SOURCE` and `PUBLIC_WORKFLOW_DEST` are defined before use and consumed by sync and validation steps.
