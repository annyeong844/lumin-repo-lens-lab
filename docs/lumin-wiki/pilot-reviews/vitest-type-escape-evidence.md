# Vitest Type Escape Evidence Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:** `tests/test-extract-ts-escapes.mjs`,
> `tests/test-any-inventory.mjs`.

---

## Purpose

This review decides whether two type-escape evidence suites can move as one
narrow Vitest mirror batch. It does not add Vitest suites.

The batch is acceptable because both suites protect the same evidence lane:
TypeScript/JavaScript type-escape facts must be detected, normalized,
identified, and serialized without turning parse errors, test-scope policy, or
shell-path handling into silent evidence loss.

The future mirror must remain behavior-preserving. It must not change
type-escape extraction, producer scan scope, artifact naming, pre-write
inventory hooks, incremental cache identity, deadness/ranking, or action-safety
promotion.

## Reviewed Evidence

| Suite                               | Preserved Node Command                   | Proposed Focused Vitest Command          | Surface Under Review                         |
| ----------------------------------- | ---------------------------------------- | ---------------------------------------- | -------------------------------------------- |
| `tests/test-extract-ts-escapes.mjs` | `node tests/test-extract-ts-escapes.mjs` | `npm run test:vitest:extract-ts-escapes` | direct type-escape extraction and identities |
| `tests/test-any-inventory.mjs`      | `node tests/test-any-inventory.mjs`      | `npm run test:vitest:any-inventory`      | producer artifact shape and scan policy      |

Current Node evidence checked for this review:

```text
node tests/test-extract-ts-escapes.mjs # 52 passed, 0 failed
node tests/test-any-inventory.mjs      # 36 passed, 0 failed
```

Goal lane: Lane F/H boundary, evidence extraction plus producer artifact shape.
This review covers type-escape fact extraction and the `any-inventory.json`
artifact only. It does not cover incremental cache reuse, pre-write inventory
hook stamping, broader pre-write advisory behavior, deadness/ranking, resolver
behavior, or performance optimization.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The shared invariant is that an empty or missing type-escape record is proof
only when parsing and scan-scope metadata say the relevant file was processed
cleanly. The future Vitest batch may share setup-only temporary repo helpers,
but each escape kind, parse-error behavior, occurrence key, and artifact field
must remain visible as named `it(...)` cases.

## Protected Invariants

The future Vitest mirrors must preserve these contracts:

- all canonical escape kinds emit at least one fact:
  `explicit-any`, `as-any`, `angle-any`, `as-unknown-as-T`,
  `rest-any-args`, `index-sig-any`, `generic-default-any`, `ts-ignore`,
  `ts-expect-error`, `no-explicit-any-disable`, and `jsdoc-any`;
- specific escape kinds win over generic `explicit-any` when one syntax form
  could otherwise be double-counted;
- `codeShape` preserves the evidence slice while `normalizedCodeShape`
  collapses whitespace outside string literals and preserves whitespace inside
  string literals;
- `occurrenceKey` is stable across line shifts, distinct across files, and
  keeps the `sha256:<64-hex>` shape;
- `insideExportedIdentity` follows exported type alias, interface, function,
  exported const, aliased export, default export, nested exported parent, local
  helper, and top-level non-export forms;
- parse errors produce a structured marker and no type-escape facts from the
  errored file;
- `any-inventory.json.meta.supports.typeEscapes === true`;
- `any-inventory.json.meta.supports.escapeKinds` preserves canonical order;
- clean inventory runs keep `meta.complete === true`;
- parse-error inventory runs list `meta.filesWithParseErrors`, set
  `meta.complete === false`, suppress facts from the bad file, and still keep
  facts from clean files;
- default inventory scan scope includes tests and reports
  `meta.scope === "TS/JS including tests"`;
- `--production` excludes test files and reports
  `meta.scope === "TS/JS production files"`;
- shell-sensitive paths with spaces and `$` characters still produce inventory
  artifacts;
- type-escape facts carry `file`, `line`, `escapeKind`, `codeShape`,
  `normalizedCodeShape`, `insideExportedIdentity`, and `occurrenceKey`;
- `--artifact-name` writes only the requested invocation-specific artifact and
  does not also write shared `any-inventory.json`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- dropping one canonical escape kind must fail;
- double-emitting generic `explicit-any` for a more specific escape kind must
  fail;
- normalizing away whitespace inside string literals must fail;
- using line number as part of the stable occurrence identity must fail;
- collapsing occurrence identity across two files must fail;
- assigning a local helper or top-level expression to an exported identity must
  fail;
- losing exported-name identity for `export { foo as bar }` must fail;
- emitting facts from a parse-error file must fail;
- marking an inventory with parse errors as complete must fail;
- treating default scan scope as production-only must fail;
- ignoring `--production` test exclusion must fail;
- breaking shell-safe paths must fail;
- writing both a custom artifact and shared `any-inventory.json` for
  `--artifact-name` must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- Temporary fixture helpers may write files, run subprocesses, read JSON, and
  clean directories only.
- Shared helpers must not decide escape-kind classification, occurrence-key
  identity, exported-identity ownership, parse-error completeness, scan-scope
  policy, artifact naming, deadness ranking, resolver behavior, or
  action-safety promotion.
- The mirror must not absorb `tests/test-any-inventory-incremental.mjs`;
  incremental cache identity needs a separate review.
- The mirror must not absorb `tests/test-pre-write-inventory-hook.mjs`; hook
  snapshot stamping is already covered by its own review page.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/extract-ts-escapes.test.mjs`,
2. `tests/any-inventory.test.mjs`,
3. `npm run test:vitest:extract-ts-escapes`,
4. `npm run test:vitest:any-inventory`,
5. candidate-board updates moving the two suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current Node assertion as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.
