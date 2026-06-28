# Package Script Runtime Entry Surface

> **Role:** maintainer-facing design spec for package script commands that
> launch source files directly and must therefore seed entry-surface and
> module-reachability evidence.
> **Status:** DONE. P1 runtime extractor, argv leak guard, and P2 scoped
> unsupported-script diagnostics are implemented and covered by Node/Vitest
> entry-surface tests.
> **Last updated:** 2026-05-24.

## Problem

`entry-surface.json` already records package-script entry evidence, but the
current script extractor only recognizes a narrow build-tool subset:

- `tsup <source-file>`
- `rollup --input <source-file>`
- `esbuild <source-file>`

This is enough for build entrypoints, but not for runtime package scripts such
as:

```json
{
  "scripts": {
    "start": "tsx src/server.ts",
    "dev": "tsx watch src/server.ts",
    "cli": "node dist/cli.js"
  }
}
```

When a runtime script target is not recorded in `entryFiles`, later
`module-reachability.json` starts BFS from an incomplete entry seed set. A file
such as `src/server.ts` can then appear in `unreachableFiles` even though the
package declares it as the application entry through `package.json#scripts`.

This is an entry-surface gap, not a general graph limitation.

The P1 implementation recognizes simple `tsx`, `ts-node`, `node`, and `bun`
runtime script commands as reviewable entry-surface evidence while keeping
unknown wrappers unsupported.

Beta.56 public install verification confirmed the P1 runtime path through
`audit-repo.mjs --profile full`: `tsx src/server.ts`, `tsx watch
src/server.ts`, and `node src/main.ts` enter `entry-surface.json` with
`source: "package.scripts"` and runtime tool evidence. `npm run` wrappers stay
unsupported, and later positional arguments such as `src/config.ts` in
`node src/main.ts src/config.ts` remain script argv rather than entry files.
P2 now records unsupported wrappers in
`entry-surface.json.unsupportedScriptEntrypoints[]` without adding concrete
entry files.

## Verified Current Shape

Current code path:

- `_lib/public-surface.mjs` tokenizes package script strings.
- `collectScriptEntrypointFiles()` reads `package.json#scripts` and string
  literals from files under `scripts/`.
- `extractScriptEntrypoints()` delegates to build-tool extractors and the
  runtime extractor for supported `tsx`, `ts-node`, `node`, and `bun` direct
  file invocations.
- `_lib/entry-surface.mjs` merges `scriptEntrypointFiles` into `entryFiles`.
- `_lib/module-reachability.mjs` uses `entrySurface.entryFiles` as BFS seeds.

Therefore a missed script entry is enough to create false unreachable evidence.

## Goals

1. Recognize simple runtime script commands that directly name a JS/TS source or
   compiled JS entrypoint.
2. Preserve structured evidence showing which package script produced the entry
   seed.
3. Feed recognized runtime script entries into `scriptEntrypointFiles` and the
   `entryFiles` union.
4. Ensure `module-reachability.json` no longer reports those script-launched
   files as unreachable.
5. Keep the feature conservative: unsupported shell patterns must remain
   unmodeled rather than guessed.

## Non-Goals

- Do not execute package scripts.
- Do not build a full shell parser.
- Do not recursively follow `npm run`, `pnpm run`, `yarn`, or workspace filter
  commands in the first slice.
- Do not infer arbitrary framework entries from script names alone.
- Do not implement `unused-deps` in this slice. That requires a separate
  producer and policy for CLI-only deps, framework runtime deps, peer deps,
  optional deps, dev deps, and `@types/*` packages.
- Do not use script entry evidence as direct dead-export, `SAFE_FIX`, or
  deletion proof. It is reachability evidence only.

## Supported V1 Runtime Commands

The first implementation should only accept commands where a tokenizer-state
pass can identify a direct file token without shell ambiguity.

Supported command families:

| Runtime family | Accepted examples                                    | Notes                                                                                              |
| -------------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `tsx`          | `tsx src/server.ts`, `tsx watch src/server.ts`       | Skip flags and known mode words such as `watch`; accept JS/TS source extensions.                   |
| `ts-node`      | `ts-node src/server.ts`, `ts-node-esm src/server.ts` | Accept direct JS/TS source extensions.                                                             |
| `node`         | `node src/server.js`, `node dist/cli.js`             | Accept JS-family extensions. Existing output-to-source mapping may map `dist/*.js` back to source. |
| `bun`          | `bun src/server.ts`, `bun run src/server.ts`         | Accept only direct file invocation, not `bun run <script-name>` recursion.                         |

Accepted target tokens must:

- be package-relative (`src/server.ts`, `./src/server.ts`, `dist/cli.js`),
- have a JS-family extension (`.js`, `.jsx`, `.mjs`, `.cjs`, `.ts`, `.tsx`,
  `.mts`, `.cts`),
- not be declaration files,
- occur before the next command separator (`&&`, `||`, `;`), and
- be the first accepted runtime target for that runtime invocation; later
  positional JS/TS tokens are script arguments, not additional entry files, and
- resolve through the existing `addEntry()` / `mapOutputToSource()` path.

## Explicitly Unsupported V1 Patterns

These must not create concrete entry files in the first slice:

- `npm run start`, `pnpm run dev`, `yarn start`
- `pnpm --filter app start`
- shell variables or command substitutions that determine the target file
- globbed file targets
- non-literal targets built by a wrapper script
- `node -e`, `node -p`, `tsx -e`
- commands where the file target is hidden behind an unknown wrapper

Unsupported runtime-script diagnostics are advisory and must not lower absence
confidence globally.

## Artifact Contract

For a package:

```json
{
  "name": "server-app",
  "scripts": {
    "start": "tsx src/server.ts"
  }
}
```

`entry-surface.json` should include:

```json
{
  "scriptEntrypointFiles": ["src/server.ts"],
  "entryFiles": ["src/server.ts"],
  "evidenceByFile": {
    "src/server.ts": [
      {
        "source": "package.scripts",
        "packageName": "server-app",
        "scriptName": "start",
        "tool": "tsx",
        "target": "./src/server.ts",
        "resolvedFile": "src/server.ts",
        "packageDir": "."
      }
    ]
  }
}
```

Exact field names should follow the existing script-entry evidence convention.
New fields such as `runtime: true` or `commandFamily: "runtime"` are allowed if
they help readers distinguish build-tool entries from runtime entries.

Unsupported wrappers that are intentionally not followed are recorded in a
separate advisory lane:

```json
{
  "unsupportedScriptEntrypointCount": 1,
  "unsupportedScriptEntrypointSampleLimit": 50,
  "unsupportedScriptEntrypoints": [
    {
      "source": "package.scripts",
      "packageName": "server-app",
      "scriptName": "start",
      "command": "npm run server",
      "reason": "package-script-recursion-unsupported",
      "tool": "npm",
      "targetScript": "server",
      "confidence": "advisory",
      "packageDir": "."
    }
  ]
}
```

This lane is diagnostic evidence only. It must not add entries to
`scriptEntrypointFiles`, `entryFiles`, or `evidenceByFile`. The raw count and
sample limit keep large repos debuggable without turning script diagnostics into
an unbounded artifact.

## Reachability Contract

Given:

```ts
// src/server.ts
import { app } from "./app";
app.listen();
```

and:

```json
{
  "scripts": {
    "start": "tsx src/server.ts"
  }
}
```

`module-reachability.json` must treat `src/server.ts` as an entry seed.

Expected result:

- `runtimeReachableFiles` includes `src/server.ts`.
- `reachableFiles` includes `src/server.ts`.
- `unreachableFiles` does not include `src/server.ts`.
- Files only reachable from `src/server.ts` through runtime imports are
  reachable as usual.

## Safety Invariants

1. Runtime script entries are entry-surface evidence, not proof that exports in
   that file are safe to remove.
2. Missing or unsupported script commands must not create fake entry files.
3. A supported command must not create phantom extension variants for targets
   that do not exist, unless the existing script-entry behavior already records
   a mapped source target.
4. The extractor must stop at command separators and must not let a later build
   command contaminate an earlier runtime command.
5. Evidence must name the script that produced the entry so reviewers can audit
   the command manually.
6. The first slice must keep `unused-deps` out of scope.

## Fixtures

### Positive: tsx Runtime Entry

Fixture:

```json
{
  "name": "script-runtime-entry-fixture",
  "private": true,
  "scripts": {
    "start": "tsx src/server.ts"
  }
}
```

Files:

```ts
// src/server.ts
import { app } from "./app";
app.listen();

// src/app.ts
export const app = { listen() {} };

// src/isolated.ts
export const isolated = true;
```

Assertions:

- `entry-surface.json.scriptEntrypointFiles` contains `src/server.ts`.
- `entry-surface.json.evidenceByFile["src/server.ts"]` includes
  `source: "package.scripts"` and `scriptName: "start"`.
- `module-reachability.json.unreachableFiles` does not contain
  `src/server.ts`.
- `module-reachability.json.unreachableFiles` still contains `src/isolated.ts`.

### Positive: node Compiled Entry Mapping

Fixture:

```json
{
  "name": "script-node-entry-fixture",
  "private": true,
  "scripts": {
    "cli": "node dist/cli.js"
  }
}
```

Files:

```ts
// src/cli.ts
export function main() {}
```

Assertion:

- If existing output-to-source mapping maps `dist/cli.js` to `src/cli.ts`, the
  script entry points at `src/cli.ts`.
- If it cannot be mapped, the first slice may leave it unsupported rather than
  inventing a target.

### Negative: Unknown Wrapper

Fixture:

```json
{
  "scripts": {
    "start": "custom-runner src/server.ts"
  }
}
```

Assertion:

- `src/server.ts` is not added from this script in V1.

### Negative: Script Recursion

Fixture:

```json
{
  "scripts": {
    "start": "npm run server",
    "server": "tsx src/server.ts"
  }
}
```

Assertion:

- V1 does not resolve `npm run server` recursively.
- A later slice may add recursion with cycle and workspace safeguards.

### Negative: Script Arguments Are Not Entries

Fixture:

```json
{
  "scripts": {
    "main": "node src/main.ts src/config.ts"
  }
}
```

Assertion:

- `src/main.ts` is entry evidence for the runtime script.
- `src/config.ts` is not entry evidence. It is argv passed to
  `src/main.ts`, so module reachability must still be allowed to report it
  unreachable when nothing imports it.

## Implementation Slices

### P1: Runtime Script Extractor

- Add a runtime-command extractor beside `extractTsupEntrypoints`,
  `extractRollupEntrypoints`, and `extractEsbuildEntrypoints`.
- Wire it into `extractScriptEntrypoints()`.
- Add focused tests to the entry-surface suite.
- Add a module-reachability assertion proving `tsx src/server.ts` is no longer
  unreachable.
- Implemented and verified in beta.56. `tests/test-entry-surface-artifact.mjs`
  / `tests/entry-surface-artifact.test.mjs` pin runtime script entry seeding,
  unknown wrapper rejection, and the argv leak guard.

### P2: Unsupported Script Diagnostics

- Implemented as `entry-surface.json.unsupportedScriptEntrypoints[]`.
- Records unsupported package-script recursion, such as `npm run server`,
  without recursively following the script.
- Records unknown wrappers that visibly receive source-like JS/TS targets, such
  as `custom-runner src/server.ts`.
- Keeps diagnostics scoped to the affected package/script and does not create
  concrete entry evidence.
- Covered by `tests/test-entry-surface-artifact.mjs` and
  `tests/entry-surface-artifact.test.mjs`.

### Related Follow-Up: Unused Dependencies Producer

- Already split into `docs/spec/unused-deps-producer.md` / WT-25.
- Inputs: package manifests, resolved external import specs, framework/resource
  capability packs, package manager metadata where available.
- Output: review-only dependency hygiene evidence.
- Not blocking this package-script runtime entry surface.

## Future Extensions Outside DONE Scope

- Should `bun run src/server.ts` be supported in P1 or delayed until Bun command
  semantics are modeled more fully?
- Should `node --loader tsx src/server.ts` be treated as `node` or `tsx`
  evidence?
