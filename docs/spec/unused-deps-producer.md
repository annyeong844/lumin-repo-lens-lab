# Unused Dependencies Producer

> **Role:** maintainer-facing design spec for review-only dependency hygiene
> evidence derived from package manifests and observed external consumers.
> **Status:** DONE for the review-only dependency hygiene surface.
> **Last updated:** 2026-05-24.

## Problem

Lumin Repo Lens already collects enough data to answer part of the dependency
hygiene question:

- `symbols.json.dependencyImportConsumers[]` records external package import
  consumers.
- `entry-surface.json` now records runtime package-script entries such as
  `tsx src/server.ts` and `node src/main.ts`.
- `framework-resource-surfaces.json` records framework/resource capability
  packs that can explain convention-driven dependency use.
- package manifests declare `dependencies`, `devDependencies`,
  `peerDependencies`, and `optionalDependencies`.

What is missing is a producer that compares declared dependencies against
observed consumers and surfaces the result as a bounded review artifact.

Without that producer, Lumin can explain graph reachability but cannot expose
simple "declared but not observed" dependency hygiene evidence. The gap is not
that the engine cannot analyze dependencies; the gap is that no
`unused-deps.json` artifact exists.

This spec intentionally follows the package-script runtime entry work. A script
entry such as `tsx src/server.ts` must be recognized before dependency hygiene
can be trusted, because otherwise entry reachability and CLI-only dependency
classification both start from an incomplete baseline.

## Goals

1. Emit `unused-deps.json` as review-only dependency hygiene evidence.
2. Compare package manifest declarations against observed external imports and
   package-script/tool evidence.
3. Keep dependency statuses explicit: used, muted, review-unused, or
   confidence-limited.
4. Preserve package scope in monorepos; a dependency declared in one package
   must be evaluated against consumers in that package scope.
5. Surface manifest artifact presence and weak summary/review-pack wording
   through a separate wording review because dependency language can imply
   action.
6. Avoid automated package removal, lockfile edits, or `SAFE_FIX` claims.

## Current Implementation State

The review-only dependency hygiene surface is complete for the current scope.
P1 landed in beta.57 as an artifact-only implementation:

- `build-unused-deps.mjs` writes `unused-deps.json`.
- `_lib/unused-deps-artifact.mjs` owns package identity normalization,
  package-script tool evidence, nearest-package ownership, workspace-internal
  muting, and dependency classification.
- `audit-repo.mjs` runs the producer after `build-symbol-graph.mjs`.
- `manifest.json` records `unused-deps.json` as a produced artifact.
- `manifest.json.unusedDependencies` mirrors shallow status, counts, reasons,
  and capped `topReviewUnused[]` package-name examples for navigation.
- `tests/test-unused-deps-producer.mjs` and
  `tests/unused-deps-producer.test.mjs` pin the Node/Vitest mirror contract.

The beta.57 installed-package verification confirmed the P1 boundary:
`unused-deps.json` was emitted with `unused-deps.v1` and
`unused-deps-review-policy-v1`; `review-unused` dependencies did not create
package edits, fix-plan entries, SARIF warnings, `SAFE_FIX` entries, or
Markdown removal wording.

P2/P3 landed through [`unused-deps-review-surface.md`](unused-deps-review-surface.md):
`audit-summary.latest.md` and `audit-review-pack.latest.md` render
dependency hygiene as weak review counts with artifact paths only, while keeping
package-name examples in JSON evidence. Beta.58 installed-package verification
confirmed manifest mirror fields, summary/review-pack wording, package-name
absence from Markdown, strong-wording absence, and no leakage into package
edits, fix-plan, action-safety, dead-classify, SARIF, `SAFE_FIX`, `EXISTS`, or
`SAFE_CUE` lanes.

Future dependency-specific configuration, lockfile semantics, and broader
corpus calibration are separate follow-up work. They should reopen this area
only with corpus evidence that the review-only artifact is too noisy or too
quiet.

## Non-Goals

- Do not edit `package.json`, lockfiles, or package manager metadata.
- Do not run `npm uninstall`, `pnpm remove`, or equivalent package-manager
  commands.
- Do not treat a dependency with no observed static import as proven removable.
- Do not execute package scripts to discover dynamic dependency use.
- Do not implement package manager specific lockfile graph semantics in the
  first slice.
- Do not infer all framework plugin ecosystems. Start with explicit script and
  capability-pack evidence.
- Do not create SARIF warnings or fix-plan entries in the first slice.

## Inputs

The first implementation should consume existing artifacts and manifests:

| Input                                      | Purpose                                                                       |
| ------------------------------------------ | ----------------------------------------------------------------------------- |
| `symbols.json.dependencyImportConsumers[]` | External import consumers observed during symbol graph extraction.            |
| `symbols.json.uses.external`               | Coarse external import count for summary sanity checks.                       |
| `entry-surface.json.evidenceByFile[]`      | Runtime package script evidence and tool names.                               |
| `framework-resource-surfaces.json`         | Framework/resource capability packs that can explain convention dependencies. |
| workspace package manifests                | Declared dependency fields and package scope.                                 |
| root package scripts                       | CLI/tool dependency evidence not visible as imports.                          |

The producer may read package manifests directly when needed, but it should not
invent a package boundary that `repo-mode` did not recognize. If workspace
boundary evidence is incomplete, the artifact must say so.

## Configuration And Scope Constraints

The first producer slice should not invent a new configuration system.

Current scan controls are coarse and CLI-driven:

- `--root` selects the checkout or subdirectory being audited.
- `--exclude <pattern>` is repeatable and filters scan paths.
- `--production` / `--include-tests=false` remove test-like files from the
  scan range.

There is no dedicated `unused-deps` config file, dependency allowlist, package
manager profile, or per-workspace ignore policy yet. Large monorepos should
therefore be evaluated through deliberate scan roots or explicit excludes until
corpus data justifies a more specific config surface.

This constraint is part of the evidence contract:

- `unused-deps.json` must record the effective scan root, excludes, and
  include-test policy used for the run.
- A declaration outside the selected scan range must not be reported as unused
  just because the consumer package was not scanned.
- If the producer cannot tell whether a package scope was fully scanned, it
  should emit `confidence-limited` with `workspace-boundary-incomplete` or
  `scan-scope-incomplete`.
- P1 should prefer fewer claims over a broad config surface that cannot yet be
  tested.

Future config work may add explicit dependency allowlists, package-manager
profiles, and per-workspace policy files, but those belong in a later slice
after artifact-only corpus runs show which exceptions recur.

## Dependency Identity

Observed external specifiers must normalize to package names before comparison:

| Specifier            | Package identity                   |
| -------------------- | ---------------------------------- |
| `react`              | `react`                            |
| `react/jsx-runtime`  | `react`                            |
| `@scope/pkg/subpath` | `@scope/pkg`                       |
| `node:fs`            | builtin, not a package dependency  |
| `./local`            | internal, not a package dependency |

Declaration fields must stay distinct:

```json
{
  "name": "react",
  "field": "dependencies",
  "range": "^19.0.0",
  "packageDir": "apps/web"
}
```

The same package name in `dependencies` and `devDependencies` is a manifest
shape issue, not two independent unused candidates.

## Artifact Shape

`unused-deps.json` should be deterministic and shallow-reader friendly:

```json
{
  "schemaVersion": "unused-deps.v1",
  "policyVersion": "unused-deps-review-policy-v1",
  "status": "complete",
  "root": "/repo",
  "summary": {
    "packageCount": 2,
    "declaredDependencyCount": 12,
    "usedCount": 7,
    "reviewUnusedCount": 2,
    "mutedCount": 3,
    "confidenceLimitedCount": 0,
    "byReason": {
      "no-observed-consumer": 2,
      "package-script-tool": 1,
      "peer-contract": 1,
      "ambient-types": 1
    }
  },
  "packages": [
    {
      "packageDir": ".",
      "packageName": "app",
      "manifestPath": "package.json",
      "status": "complete",
      "dependencies": [
        {
          "name": "tsx",
          "field": "devDependencies",
          "range": "^4.0.0",
          "status": "muted",
          "reason": "package-script-tool",
          "confidence": "grounded",
          "evidence": [
            {
              "kind": "package-script",
              "scriptName": "start",
              "tool": "tsx",
              "command": "tsx src/server.ts"
            }
          ]
        },
        {
          "name": "left-pad",
          "field": "dependencies",
          "range": "^1.3.0",
          "status": "review-unused",
          "reason": "no-observed-consumer",
          "confidence": "review",
          "evidence": []
        }
      ]
    }
  ]
}
```

Empty `dependencies[]` is proof only when the package entry has
`status: "complete"`. If required inputs are absent, use `status:
"unavailable"` or `status: "confidence-limited"` and record a reason.

## Status Semantics

| Status               | Meaning                                                                              |
| -------------------- | ------------------------------------------------------------------------------------ |
| `used`               | A static import/require consumer was observed in the package scope.                  |
| `muted`              | No direct static import was observed, but explicit evidence explains the dependency. |
| `review-unused`      | No consumer or accepted explanation was observed; human review needed.               |
| `confidence-limited` | Inputs are missing or incomplete; absence must not be interpreted.                   |
| `unavailable`        | The producer could not evaluate this package or repository.                          |

`review-unused` is not a removal instruction. It means the dependency deserves
review in the current scan range.

## Muting And Explanation Reasons

The first policy should support a small reason set:

| Reason                          | Applies when                                                                                    |
| ------------------------------- | ----------------------------------------------------------------------------------------------- |
| `external-import-consumer`      | The package identity appears in `dependencyImportConsumers[]`.                                  |
| `package-script-tool`           | A package script directly invokes the dependency as a tool.                                     |
| `framework-runtime`             | A framework/resource capability pack explains the package, e.g. Next/Storybook/Strapi evidence. |
| `peer-contract`                 | The dependency is declared as a peer contract; static import absence is not enough.             |
| `optional-runtime`              | The dependency is optional/platform/runtime selected.                                           |
| `ambient-types`                 | An `@types/*` dependency may provide ambient declarations.                                      |
| `workspace-internal`            | The declaration points at an internal workspace package.                                        |
| `no-observed-consumer`          | No accepted consumer or explanation was found.                                                  |
| `input-artifact-missing`        | Required evidence artifact is missing.                                                          |
| `workspace-boundary-incomplete` | Package scope could not be evaluated safely.                                                    |
| `scan-scope-incomplete`         | The selected root/excludes mean a package or consumer scope was only partially scanned.         |

Each muted or confidence-limited dependency should carry the evidence that
caused the classification. Do not silently hide entries.

## Package Script Tool Evidence

The producer should reuse tokenizer-state script parsing rules from the
entry-surface package-script work where practical.

Examples that can explain a dependency:

```json
{
  "scripts": {
    "start": "tsx src/server.ts",
    "dev": "vite --host 0.0.0.0",
    "build": "next build",
    "lint": "eslint ."
  }
}
```

Expected explanations:

- `tsx` can be muted by `package-script-tool`.
- `vite`, `next`, and `eslint` can be muted by `package-script-tool` when the
  command token names the declared dependency.
- `npm run start` does not recursively explain dependencies in V1.

Runtime entry extraction and dependency tool extraction are related but not the
same claim. Runtime entry evidence seeds reachability. Tool evidence explains a
declared dependency.

## Peer, Optional, And Type Dependencies

The first slice must be conservative:

- `peerDependencies` default to `muted` with `peer-contract` unless corpus
  evidence proves a better classification.
- `optionalDependencies` default to `muted` with `optional-runtime`.
- `@types/*` dependencies default to `muted` with `ambient-types` when no
  stronger consumer evidence exists.
- A normal package plus matching `@types/*` package should keep separate
  records but may cross-reference each other through evidence.

These defaults avoid turning type/runtime contracts into false unused claims.

## Manifest And Markdown Surface

`manifest.json` should mirror only summary fields:

```json
{
  "unusedDependencies": {
    "artifact": "unused-deps.json",
    "schemaVersion": "unused-deps.v1",
    "policyVersion": "unused-deps-review-policy-v1",
    "status": "complete",
    "packageCount": 2,
    "reviewUnusedCount": 2,
    "mutedCount": 3,
    "confidenceLimitedCount": 0,
    "topExamples": [
      {
        "packageDir": ".",
        "name": "left-pad",
        "field": "dependencies",
        "reason": "no-observed-consumer"
      }
    ]
  }
}
```

`audit-summary.latest.md` and review-pack Lane 3 should include a review-only
line:

```text
Dependency hygiene: 2 review-unused dependencies, 3 muted explanations.
Read manifest.json.unusedDependencies and unused-deps.json before changing
package manifests.
```

The wording must avoid `safe`, `remove`, `delete`, `uninstall`, and `fix`
claims in the first slice.

## Safety Invariants

1. `unused-deps.json` never creates `SAFE_FIX`, `EXISTS`, or fix-plan entries.
2. `review-unused` never means safe to remove.
3. Missing artifacts produce `confidence-limited` or `unavailable`, not
   zero-unused proof.
4. Package boundaries must be explicit; root-level observations do not prove
   nested workspace package usage unless the package scope matches.
5. Scan-range boundaries must be explicit; unscanned sibling packages must not
   make a declaration look unused.
6. Package-script tool evidence explains dependency declarations but must not
   create runtime reachability by itself.
7. Unsupported script wrappers do not explain dependencies unless a later
   policy explicitly models them.
8. Peer, optional, and ambient type packages stay muted in V1 unless a stronger
   reviewed policy exists.
9. Framework/resource capability packs may mute or confidence-limit dependency
   hygiene evidence, but they must not create positive import consumers.
10. The producer must preserve raw declared fields and evidence paths so a human
    can audit the claim.
11. Public package verification is required before marking this user-visible
    behavior `DONE`.

## Implementation Slices

### P0: Spec And Tracker

- Add this spec.
- Add a WT-25 tracker note.
- Do not change analyzer behavior.
- Status: complete.

### P1: Artifact-Only Producer

- Add `build-unused-deps.mjs`.
- Read package manifests and `symbols.json.dependencyImportConsumers[]`.
- Emit `unused-deps.json` with `used`, `review-unused`, and
  `confidence-limited` statuses.
- Keep script/framework/peer/optional/type handling minimal and conservative.
- Record scan root, excludes, and include-test policy in the artifact.
- No manifest or Markdown rendering yet.
- Status: complete in beta.57.

### P2: Script Tool And Framework Explanations

- Add package-script tool token explanations.
- Add framework/resource capability-pack explanations for known grounded
  frameworks.
- Keep unsupported wrappers visible as confidence limits when they mention
  declared tools.
- Status: complete for package-script tool evidence and conservative review
  classification; future framework additions are capability-pack follow-ups.

### P3: Manifest And Review-Pack Surface

- Mirror summary fields into `manifest.json.unusedDependencies`.
- Add audit-summary and review-pack wording.
- Keep all wording review-only.
- Status: complete in beta.58.

### Future: Corpus Calibration And Configuration

- Run on at least:
  - one app with runtime package scripts,
  - one framework app,
  - one library with peer dependencies,
  - one monorepo with nested package manifests.
- Treat this as future tuning, not a blocker for the current review-only
  surface. Configuration should be added only after corpus data shows a concrete
  false-positive or false-negative pattern.
- Record false review-unused cases separately from true unused findings.
- Do not add package removal advice until calibration supports it.

## Acceptance Fixtures

### Static Import Consumer

`dependencies.react` plus `import React from "react"` should classify `react`
as `used` with `external-import-consumer` evidence.

### Package Script Tool

`devDependencies.tsx` plus `"start": "tsx src/server.ts"` should classify
`tsx` as `muted` with `package-script-tool` evidence.

### Runtime Entry Baseline

The same fixture should keep `src/server.ts` reachable through
`entry-surface.json`; dependency hygiene should not compensate for missing
entry evidence.

### Review-Unused Candidate

`dependencies.left-pad` with no observed import, script, peer, optional,
framework, or type explanation should classify as `review-unused` with
`no-observed-consumer`.

### Argv Leak Guard

`"main": "node src/main.ts src/config.ts"` must not let `src/config.ts` become
entry evidence, and must not use `src/config.ts` as dependency evidence.

### Peer Contract

`peerDependencies.react` with no static import should be `muted` with
`peer-contract`, not `review-unused`.

### Ambient Types

`devDependencies["@types/node"]` should be `muted` with `ambient-types` in V1.

### Workspace Scope

A dependency declared in `apps/web/package.json` must be evaluated against
consumers inside `apps/web`, not unrelated root or sibling package imports.

### Scoped Root Guard

Running with `--root apps/web` must not report dependencies from an unscanned
root or sibling package as unused. Running from repo root with
`--exclude apps/server` must either omit `apps/server` declarations or mark the
affected package scope `confidence-limited`.

## Open Questions

- Should `devDependencies` without script/config evidence be classified as
  `review-unused` or a weaker `dev-unobserved` status?
- Should lockfile metadata be read in P1 or delayed until package-manager
  specific support exists?
- Should Lumin eventually support an `unused-deps` allowlist/config file, or is
  `--root` plus `--exclude` enough for the product shape?
- Which framework capability packs are mature enough to explain dependencies
  without creating false mutes?
- Should `bin` declarations in dependencies be indexed from installed
  `node_modules`, or should V1 only trust package script command names?
- Should future package-manager commands recurse through `npm run`, `pnpm run`,
  and `yarn` script aliases, and how should cycles be reported?
