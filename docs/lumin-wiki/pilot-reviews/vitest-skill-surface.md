# Vitest Skill Surface Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-skill-surface.mjs`.

---

## Purpose

This review decides whether `tests/test-skill-surface.mjs` can move as a
narrow Lane G Vitest mirror. It does not add the Vitest suite.

The suite protects the public text surface that a user, host, or model sees
before running the analyzer: root package metadata, README guidance, split skill
files, command docs, template docs, and public/private documentation boundaries.
It is acceptable as a single-suite mirror because it is source-text and metadata
based, runs only the public CLI help path as a subprocess, and does not classify
repository code or rank analyzer findings.

This review deliberately keeps `test-skill-package.mjs` separate. That suite
builds the packaged skill output and checks generated package contents, while
this suite checks the maintainer checkout's public text contract.

## Reviewed Evidence

| Suite                          | Preserved Node Command              | Proposed Focused Vitest Command     | Surface Under Review                                     |
| ------------------------------ | ----------------------------------- | ----------------------------------- | -------------------------------------------------------- |
| `tests/test-skill-surface.mjs` | `node tests/test-skill-surface.mjs` | `npm run test:vitest:skill-surface` | README, SKILL surfaces, command docs, public doc staging |

Current Node evidence checked for this review:

```text
node tests/test-skill-surface.mjs # 35 passed, 0 failed
```

Goal lane: Lane G, public package/plugin/hooks. This review covers only the
public skill-surface text subset of that lane.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same text and metadata
contracts without changing the public package build, public publish workflow,
hook runtime, resolver behavior, deadness/ranking behavior, or performance
measurement.

## Protected Invariants

The future Vitest mirror must preserve these public skill-surface contracts:

- `package.json` exposes the recommended `lumin-repo-lens-lab` bin pointing at
  `audit-repo.mjs`;
- `package.json` names the TS/JS monorepo evidence engine and public CLI;
- offline skill-triggering and behavior checks stay present while live sweeps
  stay opt-in;
- README first-use guidance leads Claude Code users through marketplace install
  before Codex-native link install;
- README names the stable validation modes and keeps sibling root scripts
  demoted to internal engine entrypoints;
- README explains conservative evidence boundaries for function clones, exact
  shape index evidence, and artifact reading order;
- `SKILL.md`, `SKILL.write-gate.md`, and `SKILL.canon.md` stay split by audit,
  write-gate, and canon responsibilities;
- shared `<audit-repo>` and `<SKILL_ROOT>` path tokens remain duplicated only
  where independently loaded skill surfaces need them;
- the audit skill remains slim enough for progressive disclosure and points to
  references/templates rather than embedding every workflow;
- English public docs use the English `unknown` evidence label, not Korean
  epistemic placeholders;
- plugin metadata uses default component discovery and marketplace metadata;
- command docs route users through the intended skill surfaces and avoid asking
  normal chat users to hand-write intent JSON;
- `refactor-plan` stays a coaching/template surface, not an engine mode;
- full audit review pack stays an in-session reminder surface, not an external
  API runner;
- default slash command runs a baseline-aware current-workspace audit;
- structural review wording keeps the checklist gate and separates short
  chat-facing reviews from formal report output;
- product-surface, history, spec, lab, docs, and internal-engine pages keep
  public/private staging boundaries visible;
- `.gitignore` keeps generated lab artifacts out of default tracking;
- long review checklist stays repo-neutral and maintainer-only self-audit notes
  stay outside shipping templates;
- maintainer skill-triggering harness remains present but clearly separate from
  the public skill surface.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- package metadata drifting away from the recommended public CLI must fail;
- README install order regressions that bury the marketplace path must fail;
- sibling root scripts becoming the preferred user-facing interface must fail;
- write-gate/canon-only references leaking back into the audit skill surface
  must fail;
- public docs using Korean uncertainty labels or hedged unknown wording must
  fail;
- command routing that delegates to the wrong skill surface must fail;
- refactor-plan accidentally becoming a CLI/engine mode must fail;
- audit review pack wording that suggests external API execution must fail;
- hiding the dead-export false-positive screen before chat smoothing must fail;
- moving maintainer-only docs or self-audit material into shipping templates
  must fail;
- widening the skill-triggering harness into public runtime behavior must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror may read public docs, package metadata, command files, templates,
  and references from the maintainer checkout.
- The mirror may execute `audit-repo.mjs --help` because the existing suite
  protects public CLI help text.
- The mirror must not run package publishing, plugin packaging, hook runtime
  scripts, full audit pipelines, resolver fixtures, deadness/ranking fixtures,
  or performance/incremental cache fixtures.
- The mirror must not absorb `test-skill-package.mjs`,
  `test-plugin-package.mjs`, `test-publish-public-plugin.mjs`,
  `test-github-actions-ci-policy.mjs`, hook runtime suites, analyzer behavior,
  resolver behavior, generated/framework surfaces, deadness/ranking, or
  performance/incremental cache behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/skill-surface.test.mjs`,
2. `npm run test:vitest:skill-surface`,
3. candidate-board updates moving `tests/test-skill-surface.mjs` from
   `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share local text-reading helpers
inside the test file, but no shared helper should decide which docs are public
or how command routing semantics work.

Run the preserved Node command, the focused Vitest command, `npm run
test:vitest`, and the doc-script checks when changing this batch.
