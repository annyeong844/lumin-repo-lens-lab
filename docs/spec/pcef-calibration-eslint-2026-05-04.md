# PCEF Calibration Note: ESLint

> **Role:** maintainer-facing calibration note for Proof-Carrying Export Fix.
> **Status:** observation, not a public product claim.
> **Engine:** `lumin-repo-lens-lab` `0.9.0-beta.13`, source commit `488001c`.
> **Target:** unpacked `eslint@10.3.0` archive at
> `C:\Users\endof\Downloads\eslint-main`.
> **Date:** 2026-05-04.

This note records what the current engine produced before further PCEF work.
It is a comparison anchor for later P0/P1 changes, not a release gate by
itself.

## Commands

```powershell
node audit-repo.mjs --root "C:\Users\endof\Downloads\eslint-main" `
  --output "C:\Users\endof\Downloads\eslint-main\.audit" `
  --profile full

node audit-repo.mjs --root "C:\Users\endof\Downloads\eslint-main" `
  --output "C:\Users\endof\Downloads\eslint-main\.audit-production" `
  --profile full --production
```

## Full Scan

- Scan range: 1477 JS/TS files, tests included.
- Confidence: parse errors 17, unresolved internal 33 / 2697
  (`unresolvedInternalRatio = 0.0122`).
- Blind zones: 1 parser precision-gap.
- Parse-error files were intentional broken fixtures under `tests/fixtures/**`,
  such as `tests/fixtures/cli/syntax-error.js` and
  `tests/fixtures/config-file/js/.eslintrc.broken.js`.
- Dead-export tiers:
  - `SAFE_FIX = 2`
  - `REVIEW_FIX = 0`
  - `DEGRADED = 0`
  - `MUTED = 13`
  - `safeFixGroups = 1`

## Production Scan

- Scan range: 479 JS/TS production files.
- Confidence: parse errors 0, unresolved internal 5 / 1395
  (`unresolvedInternalRatio = 0.0036`).
- Blind zones: none.
- Dead-export tiers:
  - `SAFE_FIX = 2`
  - `REVIEW_FIX = 0`
  - `DEGRADED = 0`
  - `MUTED = 14`
  - `safeFixGroups = 1`
- Entry-surface completeness was `high` for all production submodules.
- Module reachability found 403 reachable files, 87 unreachable files, and no
  bounded-out files.

## Observed Safe Fix Group

Both safe fixes came from one declaration-file group:

```text
packages/eslint-config-eslint/types/nodejs.d.ts
  line 3: cjsConfigs
  line 5: esmConfigs
```

The source file was:

```ts
import type { Linter } from "eslint";

export declare const cjsConfigs: Linter.Config[];

export declare const esmConfigs: Linter.Config[];
```

The engine selected `delete_value_declaration` for both findings with
`proofComplete = true`, no `actionBlockers`, and no
`strongerActionBlockers`. In the production scan the reason included
`entry-unreachable`.

Context check:

- `packages/eslint-config-eslint/package.json` exposes `types/index.d.ts`,
  `types/base.d.ts`, `types/cjs.d.ts`, and `types/formatting.d.ts`.
- It does not expose `types/nodejs.d.ts` through `exports` or `typesVersions`.
- Runtime `nodejs.js` exports `cjsConfigs` and `esmConfigs`, and runtime
  consumers import those values from `./nodejs`.
- No checked source reference to `types/nodejs.d.ts` was found.

This makes the deadness decision plausible, but it exposes an action-safety
follow-up below.

## Follow-Up: Post-Delete Import Integrity

If both exported declarations are deleted, the remaining import
`import type { Linter } from "eslint";` becomes unnecessary unless another
grouped edit still uses it. PCEF must therefore treat import cleanup as part of
delete-action proof:

- evaluate imported binding use after `actionGroupId` dedupe,
- remove import specifiers or whole import declarations when the import is only
  used by deleted ranges and cleanup is safe,
- preserve module syntax with `export {};` if import cleanup would turn the
  file into a script or ambient declaration context,
- block the stronger delete action when cleanup cannot be proven, while still
  allowing a weaker demotion action when one exists.

This is not a deadness false positive. It is an action-proof completeness
obligation for future fixer work.

## Calibration Use

Use this case after PCEF P1 changes to check:

- the same two findings remain grouped,
- public API declarations in `lib/types/*.d.ts` remain `MUTED` via
  `publicApi_FP23`,
- production scan remains blind-zone free,
- delete actions either carry import-cleanup/module-marker proof or fall back to
  a weaker safe action.
