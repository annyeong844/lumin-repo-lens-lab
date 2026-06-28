# Framework Policy Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace broad framework path muting in dead-export classification with a package-scoped policy matrix that only mutes when both framework activation evidence and a specific protected convention match are present. Weak or ambiguous framework evidence must remain review-visible and be summarized through aggregate counters.

**Architecture:** Add a pure framework policy module, a separate source-fact collector for Hono route registrations, a thin I/O wrapper in existing policy utilities, and integrate the decision into `classify-dead-exports.mjs` before ordinary dead-export tiering. Preserve existing artifact compatibility by keeping current mute reason names where possible while adding a structured `summary.frameworkPolicy` object for new counters.

**Tech Stack:** Node.js ESM (`.mjs`), existing `oxc-parser` wrapper, current symbol/classify/rank pipeline, fixture-driven tests under `tests/`.

---

## Task 1: Add Pure Policy Matrix Tests

**Files:**
- Create `tests/test-framework-policy-matrix.mjs`

**Steps:**
- [ ] Import the new pure API from `_lib/framework-policy-matrix.mjs`.
- [ ] Build tests around explicit package records, known files, and optional `frameworkFacts`.
- [ ] Cover positive and negative decisions without reading from disk.
- [ ] Assert action, reason, framework, and aggregate counter intent.

**Required test cases:**
- [ ] Root Next evidence does not activate a nested package that has its own `package.json`.
- [ ] Nested package with its own Next evidence protects `src/app/**/page.*` and `src/pages/**`.
- [ ] Next root or `src` level `middleware.*` / `proxy.*` is protected only when located alongside `app`, `src/app`, `pages`, or `src/pages`.
- [ ] `app/foo/middleware.ts` stays visible.
- [ ] `instrumentation.*` protects `register` and `onRequestError`.
- [ ] `instrumentation-client.*` returns `review-hint`, not `mute`, until fixtures prove safe muting.
- [ ] Nuxt `@nuxt/opencollective` is a rejected signal and never activates Nuxt/Nitro muting.
- [ ] Nuxt nested composables stay visible without re-export or scan config evidence.
- [ ] SvelteKit route exports protect `load`, `actions`, HTTP verbs, and page options with `entries` limited to dynamic route modules.
- [ ] Astro endpoint exports protect HTTP methods and `getStaticPaths`; arbitrary default exports are not Astro config defaults.
- [ ] React Router exports outside the initial protected set return `review-hint`.
- [ ] Hono path shape alone does not mute; a matching `honoRouteRegistrations.handlerRefs[]` fact can mute the referenced handler.
- [ ] NestJS paths and decorators do not receive framework-policy mute.

**API shape to test:**

```js
import {
  ACTION_MUTE,
  ACTION_NONE,
  ACTION_REVIEW_HINT,
  createFrameworkPolicyContext,
  createFrameworkPolicyCounters,
  classifyFrameworkPolicy,
  recordFrameworkPolicyDecision,
} from '../_lib/framework-policy-matrix.mjs';

const context = createFrameworkPolicyContext({
  root: fixtureRoot,
  packageRecords: [
    {
      root: fixtureRoot,
      relRoot: '.',
      packageJson: { dependencies: { next: '15.0.0' } },
    },
  ],
  files: ['src/app/page.tsx', 'src/proxy.ts'],
  frameworkFacts: { honoRouteRegistrations: [] },
});

const decision = classifyFrameworkPolicy(context, {
  file: 'src/app/page.tsx',
  exportName: 'default',
});
```

**Verification command:**

```powershell
node tests/test-framework-policy-matrix.mjs
```

**Expected initial output:**

```text
Cannot find module '../_lib/framework-policy-matrix.mjs'
```

**Expected final output:**

```text
PASS
```

## Task 2: Implement `_lib/framework-policy-matrix.mjs`

**Files:**
- Create `_lib/framework-policy-matrix.mjs`

**Steps:**
- [ ] Export action constants: `ACTION_MUTE`, `ACTION_REVIEW_HINT`, `ACTION_NONE`.
- [ ] Normalize all paths to repo-relative POSIX style.
- [ ] Assign each candidate to its nearest package root from `packageRecords`.
- [ ] Activate framework evidence only within that nearest package root.
- [ ] Require both package-scoped framework evidence and protected convention match for `ACTION_MUTE`.
- [ ] Return `ACTION_REVIEW_HINT` for intentionally visible weak matches.
- [ ] Return `ACTION_NONE` for unrelated candidates.
- [ ] Export counter helpers used by `classify-dead-exports.mjs`.

**Decision result shape:**

```js
{
  action: ACTION_MUTE,
  reason: 'frameworkSentinel_FP27',
  framework: 'next',
  ruleId: 'next-app-router-special-file',
  evidence: {
    packageRoot: '.',
    activation: ['dependency:next'],
    convention: 'src/app/**/page.*',
  },
}
```

**Counter shape:**

```json
{
  "frameworkPolicy": {
    "mutedFindings": { "next": 1, "nuxt": 0 },
    "reviewHintFindings": { "react-router": 1 },
    "rejectedSignalOccurrences": {
      "@nuxt/opencollective": { "packages": 1, "findingsAffected": 0 }
    },
    "pathShapedCandidatesKeptVisible": {
      "middleware": 1,
      "routes": 1,
      "app": 1
    }
  }
}
```

**Framework activation rules:**
- Next.js: dependency or devDependency named `next`.
- Remix / React Router framework mode: dependency or devDependency named `@remix-run/node`, `@remix-run/react`, `@react-router/dev`, or `react-router`.
- Hono: dependency or devDependency named `hono`, but muting requires route facts.
- SvelteKit: dependency or devDependency named `@sveltejs/kit`.
- Astro: dependency or devDependency named `astro`.
- Nuxt/Nitro: dependency or devDependency named `nuxt`, `nuxt3`, `nitro`, `nitropack`, or `@nuxt/kit`; explicitly reject `@nuxt/opencollective` and do not treat bare `h3` as activation.
- NestJS: dependency names may be recognized for counters, but do not produce framework-policy `mute`.

**Protected convention rules:**
- Next.js:
  - `pages/**` and `src/pages/**`.
  - `pages/api/**` and `src/pages/api/**`.
  - `app/**/page.*`, `src/app/**/page.*`, and App Router special files.
  - Root or `src` level `middleware.*` / `proxy.*` only alongside `app`, `src/app`, `pages`, or `src/pages`.
  - `proxy.*` / `middleware.*`: default export, `proxy`, and `config`.
  - `instrumentation.*`: `register` and `onRequestError`.
  - `instrumentation-client.*`: `ACTION_REVIEW_HINT` until later fixtures prove safe muting.
- Remix / React Router:
  - Legacy Remix `app/routes/**` route modules.
  - React Router route modules referenced by route-config facts when available.
  - Initial mute set: default, `loader`, `action`, `meta`, `links`, `headers`, `ErrorBoundary`.
  - Other framework exports stay review-visible.
- Hono:
  - Only route fact matches from `frameworkFacts.honoRouteRegistrations[].handlerRefs[]` may mute.
  - Directory shape such as `routes/**` is not enough.
- SvelteKit:
  - `src/routes/**/+page.*`, `+page.server.*`, `+layout.*`, `+layout.server.*`, `+server.*`, `+error.*`.
  - Export names: `load`, `actions`, HTTP methods, `prerender`, `ssr`, `csr`, `trailingSlash`, `config`.
  - `entries` only in dynamic `+page.*`, `+page.server.*`, or `+server.*`.
- Astro:
  - `src/pages/**` JS/TS endpoints and route modules.
  - Export names: HTTP methods, `ALL`, and `getStaticPaths`.
  - `astro.config.*` default export is handled by existing config policy or an Astro-specific config match; arbitrary default exports are not Astro config defaults.
- Nuxt/Nitro:
  - Nuxt/Nitro known route/plugin/server utility conventions after activation.
  - Top-level `app/composables/*.ts` and root `composables/*.ts` only.
  - Nested composables require additional evidence and remain visible in phase 1.
- NestJS:
  - Never hidden by framework policy.

**Verification command:**

```powershell
node tests/test-framework-policy-matrix.mjs
```

**Expected output:**

```text
PASS
```

## Task 3: Add Hono Route Fact Collector

**Files:**
- Create `tests/test-framework-policy-facts.mjs`
- Create `_lib/framework-policy-facts.mjs`

**Steps:**
- [ ] Write tests first for simple Hono registrations.
- [ ] Use the existing OXC parse helper instead of regex-based source parsing.
- [ ] Extract handler references from `app.get`, `app.post`, `app.put`, `app.patch`, `app.delete`, `app.options`, `app.all`, `app.use`, `app.route`, and `app.mount` when the handler can be resolved to an imported or local exported identifier.
- [ ] Emit `handlerRefs[]` arrays to support multiple middleware/handler arguments.
- [ ] Keep unresolved or dynamic handlers out of facts instead of guessing.

**Fact shape:**

```js
{
  file: 'src/server.ts',
  callee: 'app.get',
  route: '/x',
  handlerRefs: [
    { file: 'src/middleware.ts', exportName: 'auth' },
    { file: 'src/handlers.ts', exportName: 'handler' },
  ],
}
```

**Test fixtures:**
- `import { auth } from './middleware'; import { handler } from './handlers'; app.get('/x', auth, handler);`
- `app.use('/x', auth);`
- `app.route('/api', apiRoutes);` where `apiRoutes` is imported from another file and exported there.
- A dynamic expression such as `app.get(path, makeHandler())` emits no guessed handler fact.

**Verification command:**

```powershell
node tests/test-framework-policy-facts.mjs
```

**Expected initial output:**

```text
Cannot find module '../_lib/framework-policy-facts.mjs'
```

**Expected final output:**

```text
PASS
```

## Task 4: Integrate Framework Policy Into Dead-Export Classification

**Files:**
- Update `_lib/classify-policies.mjs`
- Update `classify-dead-exports.mjs`
- Update `rank-fixes.mjs` only if it needs to preserve additional policy evidence.

**Steps:**
- [ ] Keep existing exported helpers for backward compatibility unless tests prove they can be removed.
- [ ] Add a wrapper that builds package records from `detectRepoMode(root)` and workspace package roots.
- [ ] Read each workspace `package.json` through the existing safe JSON helper.
- [ ] Build known file lists from symbol data and dead candidates.
- [ ] Collect Hono route facts before classifying framework candidates.
- [ ] Replace broad `isCoreSentinel` / `isNuxtNitroSentinel` muting with `classifyFrameworkPolicy`.
- [ ] If decision is `ACTION_MUTE`, record the excluded candidate with existing `policyEvidence`.
- [ ] If decision is `ACTION_REVIEW_HINT` or `ACTION_NONE`, keep candidate visible.
- [ ] Add `summary.frameworkPolicy` with aggregate counters.
- [ ] Preserve existing count fields such as `frameworkSentinel_FP27` and `nuxtNitro_FP30` for downstream compatibility.

**Integration sketch:**

```js
const frameworkPolicyContext = createFrameworkPolicyContextForRepo({
  root: ROOT,
  repoMode,
  symbolsData,
  deadProdList,
});
const frameworkPolicyCounters = createFrameworkPolicyCounters(frameworkPolicyContext);

for (const d of deadProdList) {
  const decision = classifyFrameworkPolicy(frameworkPolicyContext, {
    file: d.file,
    exportName: d.name,
    kind: d.kind,
  });
  recordFrameworkPolicyDecision(frameworkPolicyCounters, decision, d);

  if (decision.action === ACTION_MUTE) {
    if (decision.reason === 'nuxtNitro_FP30') excludedNuxtNitro++;
    else excludedFramework++;
    recordExcluded(d, decision.reason, decision.evidence);
    continue;
  }

  // Existing ordinary dead-export classification continues here.
}
```

**Verification commands:**

```powershell
node tests/test-framework-policy-matrix.mjs
node tests/test-framework-policy-facts.mjs
node tests/test-corpus.mjs
```

**Expected output:**

```text
PASS
```

## Task 5: Add Release-Blocking Corpus Fixtures

**Files:**
- Update `tests/test-corpus.mjs`

**Steps:**
- [ ] Add a case for root Next dependency plus nested non-Next workspace package; nested `app/page.tsx` remains review-visible.
- [ ] Add a case for nested workspace package with its own Next dependency; `src/app/page.tsx`, `src/pages/index.tsx`, and root/src `proxy.ts` are muted.
- [ ] Add a negative case for `app/foo/middleware.ts`.
- [ ] Extend the existing Nuxt/Nitro scope case so `@nuxt/opencollective` and bare `h3` remain rejected signals.
- [ ] Add a Nuxt nested composable fixture that remains review-visible unless additional evidence exists.
- [ ] Add a NestJS middleware/helper fixture proving `middleware/` and `plugins/` paths are not muted by Nuxt/Nitro or NestJS framework policy.
- [ ] Add Hono positive and negative fixtures.
- [ ] Add SvelteKit route export fixtures.
- [ ] Add Astro endpoint export fixtures.
- [ ] Add React Router review-hint fixture if phase 1 counters expose it in `dead-classify.json`.

**Assertions:**
- `SAFE_FIX` remains zero for framework magic symbols.
- Known framework magic exports are either `MUTED` with the correct reason or kept visible as `REVIEW_FIX` / `DEGRADED`.
- Broad path-only candidates never become `MUTED`.
- `dead-classify.json.summary.frameworkPolicy` counters use stable units:
  - `mutedFindings` counts findings.
  - `reviewHintFindings` counts findings.
  - `rejectedSignalOccurrences` counts package-level rejected signals and affected findings separately.
  - `pathShapedCandidatesKeptVisible` counts visible findings.

**Verification command:**

```powershell
node tests/test-corpus.mjs
```

**Expected output:**

```text
PASS
```

## Task 6: Update Generated Package and User-Facing Docs

**Files:**
- Update `CHANGELOG.md`
- Update `tests/README.md` through `npm run update-test-doc` if corpus test inventory changes.
- Run `npm run build:skill` and `npm run build:plugin` if source changes affect packaged skill/plugin mirrors.

**Steps:**
- [ ] Add a changelog entry describing the framework-policy safety matrix, package-scope enforcement, base framework convention coverage, and visible review-hint behavior.
- [ ] Update generated test docs if the test runner requires it.
- [ ] Rebuild skill and plugin package mirrors.
- [ ] Confirm the generated public package still excludes the Codex-only skill from Claude Code plugin dist.

**Verification commands:**

```powershell
npm run update-test-doc
npm run build:skill
npm run build:plugin
git diff --check
```

**Expected output:**

```text
no whitespace errors
```

## Task 7: Full Verification

**Files:**
- No direct edits.

**Steps:**
- [ ] Run focused tests first.
- [ ] Run repo-wide checks.
- [ ] Run a quick self-audit if time permits and compare key regression metrics.

**Verification commands:**

```powershell
node tests/test-framework-policy-matrix.mjs
node tests/test-framework-policy-facts.mjs
node tests/test-corpus.mjs
npm run ci
```

**Expected output:**

```text
all checks pass
```

## Task 8: Commit and PR

**Files:**
- No direct edits.

**Steps:**
- [ ] Review `git status -sb`.
- [ ] Review `git diff --stat`.
- [ ] Ensure untracked `.audit-*` output directories are not staged.
- [ ] Commit with a message like `Implement package-scoped framework policy safety`.
- [ ] Push the feature branch.
- [ ] Open a draft PR against `main`.

**Verification commands:**

```powershell
git status -sb
git diff --stat
```

**Expected output:**

```text
only intentional source, test, doc, and generated package files are modified
```

## Self-Review Checklist

- [ ] Every `MUTED` framework decision requires both package-scoped activation evidence and a specific protected convention match.
- [ ] Path shape alone cannot mute.
- [ ] Evidence from the repo root does not leak into a nested workspace package with its own `package.json`.
- [ ] NestJS middleware/helper exports are not hidden by framework policy.
- [ ] Nuxt/Nitro activation rejects `@nuxt/opencollective` and bare `h3`.
- [ ] Hono positive muting depends on source facts, not directory names.
- [ ] Phase 1 review hints are aggregate counters only.
- [ ] Existing artifact consumers keep their current reason names and summary fields.
- [ ] Tests cover both false-positive and false-mute prevention.
