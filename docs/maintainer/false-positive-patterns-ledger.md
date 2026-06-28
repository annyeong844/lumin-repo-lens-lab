# False Positive Patterns

Known detection failure modes. Consult before emitting findings. Update if a new pattern is discovered this session.

Format per entry: **Pattern** / **Symptom** / **Mitigation** / **Example**.

---

## FP-01 — Bundler-consumed config file default export

**Pattern:** Config files (`vite.config.*`, `rollup.config.*`, `webpack.config.*`, `next.config.*`, etc.) export a default consumed by the bundler, not imported by TypeScript code.

**Symptom:** `classify-dead-exports.mjs` flags the default as Category C (completely dead). Symbol graph finds no consumer.

**Mitigation:** Exclude `*.config.*` files from dead classification. If included, label as `[degraded, confidence: low, FP-01]`.

**Tool configs to always exclude:**
`vite.config.*`, `vitest.config.*`, `webpack.config.*`, `rollup.config.*`, `astro.config.*`, `next.config.*`, `playwright.config.*`, `jest.config.*`, `tsup.config.*`, `esbuild.config.*`

---

## FP-02 — `.d.ts` declared constants

**Pattern:** Constants in `.d.ts` are ambient declarations, generated, or runtime-resolved via module augmentation.

**Symptom:** Dead classifier flags const in `.d.ts` as Category C.

**Mitigation:** Exclude `.d.ts` from dead classification by default.

---

## FP-03 — Node `#prefix` subpath imports misclassified as external

**Pattern:** `package.json` `imports` field maps `#alias/*` to internal paths. Default resolver doesn't read `imports`, treats these as npm packages.

**Symptom:** Internal dependencies appear as "external" in topology. Downstream: some exports appear dead because real consumers were hidden.

**Mitigation:** `_lib/alias-map.mjs::buildAliasMap` already handles this. If implementing from scratch, read `pkgJson.imports` and register hash-wildcard mappings.

**Example:** `#web/*` → `./dist/web/*.js` → resolver maps to `./src/web/*.ts`.

---

## FP-04 — React JSX legacy imports

**Pattern:** `.tsx` file with `import React from 'react'` where React is used only by JSX (no explicit `React.X` calls). Legacy JSX transform requires React in scope; new `react-jsx` transform does not.

**Symptom:** Semi-dead imports finder flags React (imported, 0 direct identifier usage).

**Mitigation:** Check `tsconfig.json` `compilerOptions.jsx`:
- `"react"` / `"react-jsxdev"` → React import needed, NOT dead
- `"react-jsx"` → React import IS dead, removable

Emit finding as `[degraded, FP-04]` until tsconfig confirmed.

---

## FP-05 — Pass-through re-export chain

**Pattern:** `A.ts` has `export { X } from './b'`. `C.ts` does `import { X } from './a'`. Resolver links C→A but X's actual definition is in B.

**Symptom:** B's export appears dead even though it has transitive consumers.

**Mitigation:** When building consumer map, follow re-export chains to original definition. Register C as B's consumer, not A's.

---

## FP-06 — Type + predicate partner pattern

**Pattern:** Module exports both `interface Foo` and `function isFoo(x): x is Foo`. External consumers import `isFoo` only; type is inferred via narrowing.

**Symptom:** Type appears in dead Category A (export removable) but predicate is the public API.

**Mitigation:** For each Category A type candidate, check if same file exports `is{Name}`, `assert{Name}`, `parse{Name}`, `to{Name}`, `from{Name}`. If yes, flag as predicate partner — likely intentional.

---

## FP-07 — Test-support helpers outside production scope

**Pattern:** Helpers in `test-support/`, `tests/helpers/`, `__mocks__/` used only by `*.test.ts` files. Dead classifier on production-only scope misses test consumers.

**Symptom:** Test helper flagged as Category C in production-only run.

**Mitigation:** Either scan includes tests, or explicitly separate "production dead" vs "test-support dead" in report. Default: include tests in consumer scan.

---

## FP-08 — Python dynamic method call

**Pattern:** Python is dynamically typed. `obj.method()` cannot be resolved statically without full type inference.

**Symptom:** If attempting Python method call resolution, hit-rate near 0.

**Mitigation:** For Python audits, do NOT claim method call counts. Only identifier-based direct calls. Method-level findings must be `[blind]` with explicit note.

---

## FP-09 — Auto-generated files

**Pattern:** Files from codegen (protobuf, openapi-typescript, graphql-codegen) have mechanical structure with many "unused" exports that are intentionally exhaustive for API surface.

**Symptom:** Dense dead candidates in single file or directory.

**Mitigation:** Exclude glob patterns like `**/__generated__/**`, `**/generated/**`, `**/*.gen.ts`, `**/*.generated.ts` before running dead classifier.

---

## FP-10 — `isolatedModules` type re-export elision

**Pattern:** With `isolatedModules: true`, type-only re-exports require `export type { X }`. Older code using `export { X }` for types may confuse type-aware parsers.

**Symptom:** Type appears as value re-export when it should be type-only.

**Mitigation:** Both forms counted; distinguish in output. Not usually harmful but worth noting.

---

## FP-11 — Discipline scanner self-reference

**Pattern:** `measure-discipline.mjs` stores its own detection patterns as string/regex literals (`/@ts-ignore/g`, `{ name: ':any', ... }`). `emit-sarif.mjs` and comment headers also mention those tokens in prose. When the scanner runs on the grounded-audit repo itself (self-audit / dogfooding), each pattern definition gets counted as an occurrence of the thing it's supposed to detect.

**Symptom:** On self-scan, discipline totals inflate: `:any`, `as any`, `@ts-ignore`, `@ts-nocheck`, `eslint-disable` all report 3–6 hits despite the skill being pure `.mjs` JavaScript with zero TypeScript. All hits are in `measure-discipline.mjs`, `emit-sarif.mjs` rule descriptions, and header/doc comments.

**Mitigation:** Exclude the skill's own scripts (`measure-discipline.mjs`, `emit-sarif.mjs`, `references/false-positive-patterns.md`) when dogfooding, or label hits in those files as `[degraded, FP-11]`. Do NOT change scanner behavior to ignore its own code — that would mask real patterns if the scanner is embedded in a host project. The FP is specific to self-audit context.

**Example:** 2026-04-18 dogfood of grounded-audit on itself: `:any=5, as any=4, @ts-ignore=6` — all 15 hits verified via grep as self-references in `measure-discipline.mjs` lines 35–38 and `emit-sarif.mjs` line 60.

---

## FP-12 — Top-level `tests/` directory invisible to file walker

**Pattern:** Repos that place tests in a top-level `tests/` directory (not
`src/**/*.test.ts`) are systematically missed by `_lib/collect-files.mjs::collectFiles`.
The legacy canonical-dirs list (`src`, `lib`, `bin`, `types`, `apps`,
`packages`) did not include `tests`.

**Symptom:** Dead-export list inflates with functions imported only by tests.
`symbols.json` reports `deadInTest: 0` regardless of how many test files
actually exist. Production functions that are tested but not used in prod code
are falsely flagged as Category C.

**Mitigation:** (v0.6.1+) `collectFiles` now walks `tests`, `test`,
`__tests__`, `e2e`, `integration` by default. Keep `includeTests: false` as
the caller semantic for "filter tests from result", but discovery happens
regardless.

**Example:** 2026-04-18, cli-jaw v1.7.20 audit. `tests/events.test.ts:11`
imports `extractToolLabel`. Scanner walked only 150 files from `src/lib/bin/
types`, missing 145 files in `tests/` → `extractToolLabel` flagged
Category C false-positive.

---

## FP-13 — Repo-root entry files not descended

**Pattern:** `collectFiles` built `searchDirs` only from canonical subdirs.
If any subdir existed, root-level files like `server.ts`, `main.ts`,
`index.ts` (common entry points referenced by `package.json` `main` or
`tsconfig.json` `include`) were not walked.

**Symptom:** Exports consumed only by repo-root entry files look dead.
Common case: HTTP route registration functions (`registerXxxRoutes`)
imported by `server.ts` and called during bootstrap — all flagged as dead
because `server.ts` was invisible.

**Mitigation:** (v0.6.1+) After building `searchDirs`, `collectFiles`
additionally enumerates (non-recursively) `.ts` / `.tsx` / `.js` / `.jsx` /
`.mjs` / `.cjs` files at the repo root.

**Example:** 2026-04-18, cli-jaw v1.7.20 audit. `server.ts` (root, listed in
`tsconfig.include`) imports 11 `register*Routes` from `src/routes/*.ts`.
Not walked → all 11 flagged Category C. ~15% of dead candidates traced to
this single FP.

---

## FP-14 — Frontend asset dirs (`public/`, `app/`, `pages/`) invisible

**Pattern:** Vite / Astro / SvelteKit / Next.js projects store frontend code
in `public/`, `app/`, `pages/`, or `scripts/`. These directories were not in
the default walk list. Exports used only by frontend code appeared dead.

**Symptom:** Shared types (e.g., `HeartbeatJob` consumed by both a
server-side registry and a frontend state module) flagged as dead when
server consumers and frontend consumers are partitioned across walk
boundaries.

**Mitigation:** (v0.6.1+) `collectFiles` walks `public`, `app`, `pages`,
`scripts` by default. Future: auto-detect Vite/Astro/Svelte via config
presence and add framework-specific dirs.

**Example:** 2026-04-18, cli-jaw v1.7.20 audit. `public/js/features/heartbeat.ts`
consumes `HeartbeatJob` from server-side types. Not walked → any cross-tier
shared types inflated Category C list in frontend-heavy codebases.

---

## FP-15 — L2 `any-typed` inflated by missing dependencies

**Pattern:** `resolve-method-calls.mjs` reports `BLIND any-typed` as the
dominant bucket (often 50–80% of method calls). User may read this as "the
codebase is chaotic" when the real cause is environmental — `node_modules/`
or specifically `@types/node` isn't installed. Without `@types/node`,
`process.argv` / `fs.readFile` / `path.join` all resolve to `any`, and every
subsequent `.method()` call on them becomes an any-typed blind.

**Symptom:** L2 summary shows high `anyTyped` rate. `resolved-node_modules`
count is 0 or near-zero. `effectiveInternalRate` looks catastrophic despite
the codebase being normal TypeScript.

**Mitigation:** (v0.6.1+) `resolve-method-calls.mjs` now emits an
`envDiagnostic` in artifact meta AND prints a ⚠ warning to stderr when
`node_modules/` or `node_modules/@types/node` is missing. Users see the
diagnostic BEFORE the 30s program build. The artifact's `envDiagnostic.epistemicNote`
explicitly labels the L2 result as "unreliable until deps installed" so
downstream reporters can surface the caveat.

**Example (measured 2026-04-18, cli-jaw v1.7.20)**:

| Bucket | Before `npm install` | After `npm install` | Δ |
| --- | --- | --- | --- |
| resolved-internal | 208 (1.8%) | 208 (1.8%) | 0 |
| resolved-lib.d.ts | 3182 (26.8%) | 6345 (53.5%) | +100% (DOM/lib registrations) |
| resolved-node_modules | **0 (0.0%)** | **4595 (38.7%)** | 0 → 38.7% |
| BLIND any-typed | **8418 (70.9%)** | **440 (3.7%)** | −94.8% |
| BLIND unresolved | 58 (0.5%) | 278 (2.3%) | +2% |
| **true blind total** | **71.4%** | **6.1%** | −65.3pp (−91.5%) |
| effective internal rate (non-lib/ext) | 2.4% | 22.5% | +20pp |

Precise validation of the pattern. Diagnostic works. Once `node_modules/` and
`@types/node` are installed, the any-typed collapse is dramatic — roughly
10× reduction in true blind rate. This confirms the diagnostic is the
correct action, not a cop-out.

---

## FP-16 — Root-prefix imports (`from 'src/...'`) treated as EXTERNAL

**Pattern:** Projects that rely on build-time `baseUrl: '.'` resolution
(Claude Code, some Next.js / Vite setups) import internal modules as
`from 'src/bootstrap/state.js'` without matching tsconfig `paths` setup.
Without path info, the resolver's bare-specifier branch returned `EXTERNAL`.

**Symptom:** High unresolved-use rate (>20%). Downstream: symbols imported
this way have zero attributed consumers → flagged as dead. `symbols.json.meta.unresolvedRatio` is elevated; top unresolved specifiers share a
common first segment (`src/...`, `app/...`, `lib/...`).

**Mitigation:** (v0.6.2+) The resolver's bare-specifier branch now attempts
root-prefix resolution: if `spec` starts with a segment that exists as a
directory under `root`, try resolving `<root>/<spec>` with standard
extension / index lookups. If found → valid internal edge; if not → still
returns `EXTERNAL` (legacy behavior preserved for actual npm packages).

**Example (measured 2026-04-18, claudecodesrc/src):** 6,655 unresolved uses
out of 24,137 (27.6%) — 925+ import statements across 301 files. Functions
`markPostCompaction`, `setLastMainRequestId`, `getMeter`, `getLoggerProvider`,
`addToTotalDurationState`, etc. all falsely flagged Class C. With the fix,
most of these resolve correctly to their actual definitions.

---

## FP-17 — Canonical-dirs filter hides non-standard layouts

**Pattern:** Legacy v0.6.1 `collectFiles` built `searchDirs` only from a
hardcoded canonical list (`src`, `lib`, `bin`, `types`, `apps`, `packages`,
tests dirs, frontend dirs). When ONE canonical dir existed under a
target root (e.g., `types/`), the fallback "walk root if no canonical match"
silently disabled. Result: repos with non-canonical subdirs (`client/`,
`server/`, `shared/`, `commands/`, `services/`, `components/`, `assistant/`,
`coordinator/`, …) were catastrophically under-scanned.

**Symptom:** Target reports drastically fewer files than the disk actually
contains. Dead-export list incomplete because most of the codebase wasn't
parsed. No warning emitted — failure is silent.

**Mitigation:** (v0.6.2+) `collectFiles` now walks **all non-pruned
top-level subdirs**. Prune list expanded: `node_modules`, `.git`, `coverage`,
`.next`, `.svelte-kit`, `.astro`, `.turbo`, `.cache`, `.nuxt`, `.output`,
`out`, `target`, `venv`, `__pycache__`; plus `dist*`, `build*`, and
dotdirs (except canonical `.` roots). Canonical list kept as "marker set"
for debug but no longer gates scan inclusion.

**Example (measured 2026-04-18, claudecodesrc/src):** Target has 1,884
`.ts/.tsx` files. Pre-fix scanner found **29** (only `types/` + root-level
== 1.5% visibility). Post-fix: 1,902 files scanned (matches disk).

---

## FP-18 — Dynamic `await import(...)` expressions not tracked as edges

**Pattern:** Modern codebases use dynamic `import()` for lazy loading,
code splitting, and plugin systems. These are parsed as `CallExpression`
with `ImportKeyword` callee — NOT `ImportDeclaration`. The AST walker
that builds import edges only looks at `ImportDeclaration` nodes.

**Symptom:** Modules consumed only via `await import('./x.js')` appear
dead. Large chunks of lazy-loaded code (routes, commands, plugins) are
systematically under-counted.

**Mitigation:** (v0.6.6+) `build-symbol-graph.mjs:extractDefinitionsAndUses`
gained `collectDynamicImports(node)` which recursively walks the entire
AST (not just `program.body`) and emits a synthetic use record for every
`ImportExpression` whose source is a string literal. The use is tagged
`{ kind: 'dynamic', name: '*', dynamic: true }` and the main consumer loop
treats `kind: 'dynamic'` identically to `kind: 'namespace'` — the target
file enters `namespaceUsers`, its symbols become `namespaceShadowed` and
drop out of `trulyDead`. Non-literal specs are not exactly resolvable.
When a template import has a static directory prefix such as
`` import(`./commands/${name}.js`) ``, the producer emits
`symbols.dynamicImportOpacity[].targetDir`, and
`classify-dead-exports.mjs` materializes exports under that directory as
`MUTED` with `dynamicImportOpacity_FP18` evidence.

**Example (measured 2026-04-19, synthetic repo):** 4-file probe where
`src/target.ts` exports `foo`, `bar`, `Baz`; `src/loader.ts` does
`await import('./target.js')` and `() => import('./target.js')`.
- Pre-fix (v0.6.5): `target.ts`'s 3 symbols all in `trulyDead` (0
  static consumers, dynamic edges invisible).
- Post-fix (v0.6.6): `deadTotal: 7 → trulyDead: 4` — 3 target.ts symbols
  moved to `namespaceShadowed` bucket. `totalUsesResolved: 3` (1 static +
  2 dynamic edges resolved).

Real-repo impact unmeasured on claudecodesrc but prior reading found 207
files in `commands/` using the lazy-loader pattern `load: () => import(...)`;
`components/App.tsx`'s three dynamic consumers (`replLauncher.tsx`,
`dialogLaunchers.tsx`, `ink/ink.tsx`) previously invisible now resolve.

---

## FP-19 — JSX runtime imports (`React`, hooks) flagged semi-dead

**Pattern:** `.tsx` files with `import React from 'react'` and
`import { useCallback, useMemo, useRef } from 'react'` never explicitly
call `React.*` or `useCallback()` directly in the AST — the JSX-runtime
compile step consumes them implicitly (classic-runtime JSX emit) or they're
invoked inside JSX-embedded expressions the extractor doesn't fully walk.

**Symptom:** Call-graph's `semiDeadList` dominated by `React` + hook
imports from React. Real semi-dead cases buried under this noise.

**Mitigation:** (v0.6.6+) `build-call-graph.mjs` gained
`REACT_FRAMEWORK_NAMES` set (~27 names: `React`, `Fragment`, all standard
hooks, `forwardRef`/`memo`/`lazy`/`Suspense`, `createContext`, etc.)
plus `isReactRuntimeImport(filePath, localName, source)` predicate.
Filter placed AFTER the `calledNames` / `hits >= 2` checks so
`reactSkipCount` measures TRUE rescues (symbols that would have been
flagged without FP-19), not noise that was never at risk. Applies only
to `.tsx` / `.jsx` files importing from `react`, `react-dom`, or their
subpath exports. Surfaced as `summary.semiDeadReactFiltered: N` in
`call-graph.json` and prose line "(FP-19 React JSX runtime 제외: N)"
in console output.

**Example (2026-04-19, synthetic App.tsx):** File with
`import React from 'react'; import { useState, useCallback, useMemo,
useEffect } from 'react';` plus realistic hook usage inside JSX. Pre-fix
mental model: `React` default flagged (hits=1, only in import line).
Post-fix: `semiDead: 0, semiDeadReactFiltered: 1` — precisely the
`React` default rescued, hooks already resolved via `hits >= 2` text
heuristic. Real-repo impact (claudecodesrc pending): ledger noted 13/15
first entries were React/useCallback noise; with the filter, those
drop out of the output.

---

## FP-20 — Exported type aliases used only intra-file (PERCEPTION FP)

**Pattern:** Components export their `Props` type for potential external
composition: `export type SelectOptionProps = { ... }`. If no other file
imports the type, dead-export detection correctly flags it as having no
external consumer. The classifier puts it in Class A (demote to internal)
since it's used within the defining file.

**Symptom:** Users see 500+ "dead" type aliases in Class A and think the
tool is wrong.

**Mitigation:** This is NOT a code bug — the classifier is correct. Class A
= "export is unnecessary, demote to internal" is the right action, not
"delete". The FP is perceptual: Class A should be framed as "exposure
reduction opportunity", not "dead code". Consider renaming Class A
label in templates/report-template.md to clarify intent.

**Example (2026-04-18, claudecodesrc/src):** 1,310 Class A verdicts
including `QueryEngineConfig`, `SelectOptionProps`, `LogSelectorProps`.
All are exported `type Props = ...` patterns. Demoting them is correct
refactor guidance; deleting them would break the declaring component.

---

## FP-21 — `buildAliasMap` crashes on nested conditional exports

**Pattern:** `package.json.exports` can nest conditions:
`{ ".": { node: { import: { types, default }, require: ... } } }`.
Legacy code did `target.import ?? target.default ?? target.types` and called
`.replace()` on the result — but for the nested shape, `target.import` is
undefined and `target.default` is itself an object, so `.replace` threw
`TypeError: t.replace is not a function`.

**Symptom:** Symbol-graph / classify / call-graph all abort with uncaught
TypeError. Audit pipeline gets blocked on any unjs-style library (consola,
ofetch often do this via unbuild).

**Mitigation:** (v0.6.2+) `_lib/alias-map.mjs::extractStringTarget` —
recursive helper walks `import → default → node → require → types` keys,
unwrapping nested objects and arrays until it finds a string. Handles all
ESM/CJS/Node subpath-export conditional shapes in practice.

**Example (2026-04-18, unjs/consola):** `package.json.exports['.']` is
`{ node: { import: {...}, require: {...} } }`. Pre-fix: pipeline crashed
at first symbol-graph invocation. Post-fix: resolves cleanly to the
string target.

---

## FP-22 — Config files flagged Class C "completely dead"

**Pattern:** Build/tooling config files (`*.config.ts`, `eslint.config.mjs`,
`vitest.config.ts`, `build.config.ts`, etc.) are consumed by CLI tools
(eslint, vitest, vite, webpack, rollup, unbuild, tsup, esbuild, jest,
playwright, tailwind, postcss) **by filename convention**, not via TS
import. They appear to have zero consumers in the symbol graph.

**Symptom:** Class C (completely dead) inflated by config file exports.
On ofetch: 2/2 Class C entries were eslint + vitest configs. On consola:
2/4 Class C were eslint + build configs. Precision on Class C drops below
50% on small popular libraries — the tool shows more noise than signal.

**Mitigation:** (v0.6.3+) `classify-dead-exports.mjs` masks ~18 well-known
config filename patterns BEFORE classification. Masked count surfaced in
output as `[FP-22 excluded] config files: N`.

**Example (2026-04-18):**
- ofetch: 2 eslint + vitest configs excluded → C: 2 → 0 (100% reduction).
- consola: 2 eslint + build configs excluded → C: 4 → 2 (50% reduction).

---

## FP-23 — Package public-API terminals flagged Class A "demote"

**Pattern:** A file that is the terminal target of a
`package.json.exports` chain IS the package's public API. External
consumers (other npm packages depending on this library) import from
it — those imports are outside the scanned set, so symbol-graph counts
zero consumers, and classifier demotes to Class A ("remove export").

**Symptom:** Library `src/index.ts` etc. entries repeatedly flagged Class A.
Developer removes `export`, breaks every external consumer. Catastrophic
guidance.

**Mitigation:** (v0.6.3+) `classify-dead-exports.mjs` consults
`buildAliasMap()` from resolver; files that appear as `exact.path` in any
alias entry are flagged as public-API terminals and EXCLUDED from
classification entirely. Surfaced as `[FP-23 excluded] public API: N`.

**Example (2026-04-18):** ofetch's 3 Class A entries (`ofetch`,
`createConsola`, default from `src/index.ts`) all are `package.json.exports['.']`
targets. Excluded post-fix. consola: 5/11 Class A entries similarly excluded.

---

## FP-24 — Cross-file imports within `examples/` under-resolved

**Pattern:** `examples/basic.ts` imports `{ reporterDemo } from "./utils"`
(consola). `examples/utils/index.ts` exports `reporterDemo`. Symbol-graph
walks both files (after FP-14 fix) but counts `reporterDemo`'s internal
use as 0. Classifier flags it Class C "completely dead, 0 internal uses".

**Symptom:** Cross-file relative imports within `examples/` subtree appear
unresolved even when the target file IS scanned. Might indicate path
normalization issue or that examples/ is tagged as a separate partition
and its edges aren't attributed.

**Mitigation:** (investigate, not yet fixed) Needs tracing: verify
`resolveSpecifier` from build-symbol-graph.mjs correctly resolves relative
imports when both fromFile and target are under `examples/`. Possible
root cause: my FP-16 "root-prefix" heuristic may short-circuit relative
resolution under certain conditions.

**Example (2026-04-18, unjs/consola):** `examples/basic.ts:??` has
`import { reporterDemo } from "./utils"`. `examples/utils/index.ts:4`
exports `reporterDemo`. Flagged Class C despite being imported.

---

## FP-25 — Transitive barrel re-export chains hide consumers

**Pattern:** Monorepos with deep barrel-file chains (`packages/foo/src/index.ts`
→ `packages/foo/src/api/index.ts` → `packages/foo/src/api/v4.ts` →
`packages/foo/src/api/v4/schemas.ts`) expose public APIs through layered
re-exports. The symbol graph's direct-consumer map misses symbols that
are only reached via the chain: `schemas.ts` exports a function → barrel
at each level just re-exports → external package `packages/bar/src/foo.ts`
imports from the topmost barrel. Classifier sees "only consumed by barrels"
and labels Class A (demote) or even C (dead).

**Symptom:** Large monorepos show inflated Class A numbers concentrated in
leaf files of deep barrel trees. Typical zod/unjs layout:
`packages/<pkg>/src/<module>/<sub>/<file>.ts` where the "consumer" column
lists only sibling `index.ts` barrels.

**Mitigation:** (v0.6.4+) `build-symbol-graph.mjs` emits
`reExportsByFile` metadata: `{ fromFile: [relPath, relPath, ...] }` giving
all re-export target files per barrel. `classify-dead-exports.mjs` walks
this map transitively: if symbol `X` is re-exported by a barrel that is
imported by file `Y`, attribute `Y` as a transitive consumer of `X`.
Breadth-first walk, cycle-guarded, bounded by re-export-only chains (direct
imports break the chain).

**Example (2026-04-18, colinhacks/zod v3):** pre-fix, 88 symbols in
`packages/zod/src/v4/**` were flagged Class A because their only listed
consumers were intra-package barrels. Post-fix, those 88 symbols now have
attributed external-package consumers (`packages/zod-mini`, `packages/docs`)
and the Class A count for `packages/zod` drops from 6 to 2.

---

## FP-26 — pnpm monorepo with `pnpm-workspace.yaml` invisible to `detectRepoMode`

**Pattern:** pnpm declares its workspace roots in a separate
`pnpm-workspace.yaml` file, not in `package.json.workspaces`. Legacy
`detectRepoMode` read only `pkgJson.workspaces` → any pnpm monorepo was
silently classified as `single-package`. Downstream, `buildAliasMap` built
no cross-package aliases, and cross-package imports (`import { z } from
"zod/v4"`) were treated as EXTERNAL npm packages.

**Symptom:** `meta.repoMode === "single-package"` on a clearly multi-package
repo. `workspaceDirs.length === 0`. `aliasMap` size 0 or near-zero despite
rich `package.json.exports` fields in each subpackage. Inter-package symbol
traffic invisible → the entire monorepo's public API surface flagged
Class A / Class C in bulk.

**Mitigation:** (v0.6.5+) `_lib/repo-mode.mjs` gains
`parsePnpmWorkspaceYaml(yamlText)`: minimal indent-aware parser matching
the `packages:` list convention (`packages:\n  - 'packages/*'\n  -
'!packages/deprecated'`). `detectRepoMode` first checks
`package.json.workspaces`, then falls back to reading
`pnpm-workspace.yaml`. Negated patterns (`!...`) are skipped as exclusions
(consistent with pnpm semantics — they still aren't walked, matching
pnpm's own behavior of excluding negated globs from workspace resolution).

**Example (2026-04-18, colinhacks/zod v3):**

| Field | Before | After |
| --- | --- | --- |
| `detectRepoMode` | `single-package` | `monorepo` |
| `workspaceDirs.length` | 0 | 7 |
| `aliasMap` entries | 0 | 12 |
| cross-package import resolution | EXTERNAL for all | resolved for all 7 |

---

## FP-27 — Framework sentinel files (`app/page.tsx`, `+page.svelte`) flagged dead

**Pattern:** Next.js App Router (`app/page.tsx`, `app/layout.tsx`,
`app/loading.tsx`, `app/error.tsx`, `app/route.ts`, etc.), Pages Router
(`pages/foo.tsx`, `pages/api/bar.ts`), SvelteKit (`+page.svelte`,
`+layout.ts`, `+page.server.ts`), and similar conventions rely on
**filename-based file-system routing**. These files are never explicitly
imported by user code; the framework runtime discovers and mounts them by
path. Symbol graph sees 0 consumers → classifier places them in Class C
"completely dead".

**Symptom:** Docs-site packages, example-app packages, or any
`packages/*/app/**` or `packages/*/pages/**` tree in a monorepo shows
dense Class C findings. Typical volume: 20–40 entries per framework-aware
package.

**Mitigation:** (v0.6.5+) `classify-dead-exports.mjs` gains
`isFrameworkSentinel(relPath)`:
- `pages/**` anywhere in path → sentinel.
- `app/**` with basename stem in `FRAMEWORK_SENTINEL_BASENAMES`
  (`page`, `layout`, `loading`, `error`, `not-found`, `route`, `default`,
  `template`, `global-error`, `sitemap`, `robots`, `manifest`, `icon`,
  `apple-icon`, `opengraph-image`, `twitter-image`, `head`) → sentinel.
- SvelteKit `+<stem>.(svelte|ts|tsx|js)` at any depth → sentinel.
- Astro endpoint convention `+<name>.ts` → sentinel.

Regex `(?:^|\/)app\/` (not `^(?:[^/]+\/)?app\/`) so that `packages/docs/
app/page.tsx` and similar nested paths in monorepos still match. Sentinel
files are EXCLUDED from Class C classification; surfaced as `[FP-27
excluded] framework sentinels: N`.

**Example (2026-04-18, colinhacks/zod v3):** `packages/docs/app/**`
contained 23 Next.js App Router sentinel files (`page.tsx`, `layout.tsx`,
`loading.tsx`, `not-found.tsx` across ~6 routes). Pre-fix: all 23 in
Class C. Post-fix: 23 excluded, `excludedMeta.frameworkSentinels: 23`.
Class C dropped from 77 → 52.

---

## FP-28 — Custom `@source` conditional exports bypassed in favor of compiled `import`

**Pattern:** A library that ships both source and compiled output can
declare a custom source-pointing condition in `package.json.exports`:
```json
"exports": {
  ".": {
    "@zod/source": "./src/index.ts",
    "import": "./index.js",
    "types": "./index.d.ts"
  }
}
```
Legacy `extractStringTarget` preferred `import → default → node → require
→ types` (compiled-output priority). It returned `./index.js` (compiled
JS dist) even when the audit tool needs the source `./src/index.ts` for
AST parsing / consumer attribution.

**Symptom:** For repos with `@<pkg>/source`-style conditions, alias-map
targets point at compiled `dist/*.js` files. Dead-export classifier
attributes cross-package consumer edges to dist files, which it can't
parse. Effect: the entire monorepo's inter-package consumer graph is
silently disconnected — most Class A / Class C verdicts in the mono-repo
are noise.

**Mitigation:** (v0.6.5+) `extractStringTarget` checks condition keys for
source-indicating substrings FIRST: any key containing `source`, `src`,
`development`, or `develop` gets preference ahead of the standard
`import/default/...` list. Falls back to the standard order for ordinary
exports. Preserves compatibility (no source-condition = same behavior as
before).

**Example (2026-04-18, colinhacks/zod v3):** `packages/zod/package.json`
uses `@zod/source: "./src/index.ts"` alongside `import: "./index.js"`.
Pre-fix: alias-map target = `packages/zod/index.js` (non-existent at
scan time; dist files aren't emitted in a fresh clone). Post-fix: target
= `packages/zod/src/index.ts`, resolver successfully maps imports from
`packages/zod-mini` / `packages/docs` to actual source definitions.
This single fix is upstream of FP-25's effective impact — without the
`@zod/source` preference, the transitive barrel walk would chase
phantom dist files.

---

## FP-29 — pnpm `packages/**` and negated patterns ignored by workspace expander

**Pattern:** pnpm-workspace.yaml routinely uses:
```yaml
packages:
  - docs
  - packages/**
  - '!packages/deprecated'
  - playground
  - test/fixtures/*
```
Legacy `detectRepoMode` (v0.6.5) handled only `/*` (single-star, immediate
children) and literal names. `packages/**` (recursive) fell into the
"literal" branch and was treated as a directory named `packages/**` —
which doesn't exist, so silently skipped. Negated patterns (`!foo`) were
stripped by `parsePnpmWorkspaceYaml` without being applied as exclusions.

**Symptom:** On pnpm monorepos using `packages/**` convention (nuxt,
many unjs repos, most modern ecosystem packages), `workspaceDirs` is
drastically incomplete — only `docs`, `playground`, literal-name dirs,
and `/*`-matched test fixtures populate. `aliasMap` contains only the
workspaces that DO get detected (often a single `@org/docs/*`), and
cross-package imports for the missing packages are treated as EXTERNAL.
Cascade: `unresolvedRatio > 70%`, dead-export list inflated with
cross-package-consumed symbols. Silent — no warning.

**Mitigation:** (v0.6.7+) `detectRepoMode` partitions patterns into
includes vs excludes, expands with three branches:
- `foo/**` → recursive walk via `walkForPkgs(foo)`, collect every subdir
  (any depth, excluding `node_modules` and dotdirs) that has a
  `package.json`.
- `foo/*` → immediate children only (legacy behavior preserved).
- `foo` → literal path.
Then applies exclusions: any collected dir whose absolute path exactly
matches or is prefixed by a negated pattern is filtered out.
`parsePnpmWorkspaceYaml` now preserves `!`-prefixed patterns
(previously stripped) so the exclusion phase sees them.

**Example (2026-04-19, nuxt/nuxt):**

| Measure | v0.6.6 | v0.6.7 | Δ |
| --- | --- | --- | --- |
| `workspaceDirs` | 12 | 20 | +67% (+8 real packages) |
| `aliasMap` entries | 1 | 29 | 29× |
| unresolved-ratio | 71% | 60% | −11pp |
| dead prod candidates | 386 | 376 | −2.6% (initial) |
| negation patterns applied | 0 | 2 (`!packages/nuxi`, `!packages/test-utils`) | ✓ |

---

## FP-30 — Nitro / Nuxt framework-internal runtime auto-registered files

**Pattern:** Nuxt 3 and Nitro use filesystem-convention routing for
runtime artifacts that the framework itself discovers and registers at
build/start time. Inside the Nuxt monorepo itself, this translates to:
- `packages/nitro-server/src/runtime/handlers/*.ts` — h3 event handlers
- `packages/nitro-server/src/runtime/middleware/*.ts` — Nitro middleware
- `packages/nitro-server/src/runtime/plugins/*.ts` — Nitro plugins
- `packages/nitro-server/src/runtime/utils/*.ts` — shared runtime helpers
- `packages/nuxt/src/app/plugins/*.ts` — built-in Nuxt plugins
- `packages/nuxt/src/app/middleware/*.ts` — built-in Nuxt middleware
- `packages/nuxt/src/app/entry.ts`, `entry-spa.ts` — framework entry points

In user-level Nuxt apps, the same convention applies at project root:
- `plugins/*.ts`, `middleware/*.ts`, `server/api/**`, `server/middleware/**`,
  `server/plugins/**`, `server/routes/**`, `composables/**`, `components/**`

None of these are explicitly imported by user code; the framework's
virtual-module system (`#imports`, `#app`, `#nitro`) wires them.

**Symptom:** Nuxt audit shows ~43 Class C entries matching these
conventions — all `export default` shapes that are zero-consumer by
static analysis but framework-registered.

**Mitigation:** (v0.6.8+) `classify-dead-exports.mjs` gained
`detectNuxtNitro(rootPkgJson, workspaceDirs)` which scans root + all
workspace `package.json` files for matching deps/devDeps/peerDeps
(`nuxt`, `nitropack`, `nitro`, `h3`, `@nuxt/*`, `@nitro/*`) or a matching
`name` field. Gate result (`isNuxtNitro: boolean`) unlocks
`isNuxtNitroSentinel(relPath)` which matches:
- `**/server/(api|middleware|plugins|routes)/**` — h3 convention
- `**/runtime/(handlers|middleware|plugins|utils|server-assets)/**`
- `**/app/(plugins|middleware)/**` + `**/app/entry(-spa)?.*`
- `**/(plugins|middleware|composables)/*` one level deep (user-app
  convention)
- `**/components/runtime/**`

`excludedNuxtNitro` counter surfaced in summary.excluded.nuxtNitro_FP30
+ console line "[FP-30 excluded] Nuxt/Nitro filesystem-routed files".
Falls back to no-op when detection fails — zero overmatch risk outside
Nuxt/Nitro ecosystem.

**Example (2026-04-19, nuxt/nuxt):**

| Stage | v0.6.7 | v0.6.8 | Δ |
| --- | --- | --- | --- |
| Classifier total | 154 | 72 | −53% |
| Class C | 53 | 15 | −72% |
| Class A | 78 | 37 | −53% |
| Class B | 23 | 20 | −13% |
| FP-30 excluded | 0 | 82 | (new) |

Combined with FP-22/23/27/31: 137 sentinels total excluded on nuxt
(14+32+9+82), vs. 0 pre-v0.6.3. Grounded-audit's own self-audit
unaffected (no Nuxt/Nitro deps → gate returns false).

---

## FP-31 — Test fixture directories inflate production dead list

**Pattern:** Monorepos often include fixture packages (`test/fixtures/*`,
`packages/*/test/layer-fixture/**`, `packages/*/test/components-fixture/**`)
that are valid pnpm workspaces (have `package.json`) and intentionally
contain minimal code exercising specific framework behaviors. Files
inside these directories define exports that are "consumed" only via
being loaded by the test harness under a specific framework-runtime
scenario — invisible to static analysis.

Legacy test-path filter in `build-symbol-graph.mjs` caught only
`.test.<ext>` suffix and `/test-support/` path. Missed:
- Package-internal `test/` subdirectories
- `*-fixture/` convention dirs (`components-fixture/`, `layer-fixture/`,
  `layers-fixture/`, `package-fixture/`)
- Workspace-level `test/fixtures/*` packages (activated by FP-29 fix)
- `/fixtures/` dir name generally

**Symptom:** Dead production list contains 30–50% fixture-code entries
after FP-29 expands workspace discovery. User sees "N fixtures flagged"
and loses trust in the tool. On nuxt: 115/292 = 39% of classifier
findings were fixtures.

**Mitigation:** (v0.6.7+ / extended v0.6.8) `build-symbol-graph.mjs:isTestPath(f)`
checks path segments. Initial set: `test`, `tests`, `test-support`,
`fixtures`, `mocks`, plus any segment ending in `-fixture` / `-fixtures`.
v0.6.8 extensions surfaced by vite audit:
- `playground` / `playgrounds` — strong dev-test convention in
  Vite/unjs/Rspress/Astro ecosystems.
- `__<anything>__` generalized (any double-underscore-wrapped segment) —
  catches `__tests__`, `__mocks__`, `__snapshots__`, `__fixtures__`,
  `__tests_dts__` (vite-specific type-test fixtures), and similar.
Applied via `deadInTest` filter before classification.

**Example (2026-04-19, nuxt/nuxt):**

| Stage | Pre-FP-31 | Post-FP-31 | Δ |
| --- | --- | --- | --- |
| deadInProd (from symbols.json) | 376 | 209 | −44% |
| Classifier total | 292 | 154 | −47% |
| Class C | 184 | 53 | −71% |
| Class A | 85 | 78 | −8% |
| Class B | 23 | 23 | 0 |

**Example (2026-04-19, vitejs/vite):** Vite's `playground/` is a workspace
of ~50 test playgrounds (react, vue, alias, css, ssr, etc.). Before
extension: 322/387 (83%) of classified findings were playground fixtures.

| Stage | Pre-v0.6.8 FP-31 | Post-v0.6.8 FP-31 | Δ |
| --- | --- | --- | --- |
| deadInProd | 574 | 151 | −74% |
| Classifier total | 387 | 60 | −85% |
| Class C | 261 | 15 | −94% |

---

## FP-33 — `package.json` with UTF-8 BOM crashes JSON parsing

**Pattern:** Windows-edited `package.json` files occasionally ship with a
UTF-8 BOM (`0xEF 0xBB 0xBF` / `\uFEFF` prefix). `JSON.parse()` in Node.js
does NOT strip BOM — it throws `SyntaxError: Unexpected token '﻿'`.
Several of our scripts read external `package.json` files (resolver.mjs
for workspace detection, build-symbol-graph for barrel detection,
classify-dead-exports for Nuxt/Nitro detection, triage-repo for overview).

**Symptom:** Audit pipeline aborts with uncaught SyntaxError on the first
BOM-prefixed package.json. Surface error message points at the JSON
parser, not the offending file. Reproduces on any monorepo where a
workspace member has a BOM-prefixed package.json — not controllable by
the audit tool. On vitejs/vite, one of the create-vite templates
ships with BOM.

**Mitigation:** (v0.6.8+) All external-JSON-read sites use
`JSON.parse(readFileSync(p, 'utf8').replace(/^\uFEFF/, ''))`. Our own
artifact reads (symbols.json, cache files) aren't affected (we emit
without BOM), so only external-data readers were patched. Sites fixed:
- `_lib/repo-mode.mjs` (root pkg — `detectRepoMode`)
- `_lib/alias-map.mjs` (workspace pkg in `buildAliasMap`)
- `build-symbol-graph.mjs:455` (barrel-file package.json)
- `classify-dead-exports.mjs:99` (Nuxt/Nitro detection)
- `triage-repo.mjs:41` (repo overview)

**Example (2026-04-19, vitejs/vite):** Pipeline crashed at
`buildAliasMap`. Trace: `SyntaxError: Unexpected token '﻿', "﻿{ \"name\"..."`.
Post-fix: full audit completes in one shot (1,418 files parsed).

## FP-36 — Monorepo-local tsconfig `paths` ignored (scope-unaware resolver)

**Pattern:** In multi-app monorepos, each app/package often defines its
own `tsconfig.json` with a local paths alias such as:

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  }
}
```

Inside `apps/agents`, `@/components/auth-control` must resolve to
`apps/agents/components/auth-control.tsx`. Inside `apps/admin`, the
same specifier must resolve to `apps/admin/components/auth-control.tsx`.

Before v1.9.7, the resolver had no `tsconfig paths` support at all —
it only read `package.json` exports/imports + root-prefix fallbacks.
Every `@/*` specifier fell through to the `EXTERNAL` sentinel, which
at the call site collapsed to `null` and inflated `unresolvedUses`.
Consequence: exports with real internal consumers appeared consumer-less
and got classified as Tier C "dead export."

**Symptom:** Elevated Tier C count on any monorepo using per-app path
aliases. Observed on duyet/monorepo (2026-04): 218 of 397 Tier C
findings were actually consumed via app-local `@/*` aliases. 73.2%
FP rate driven by this single resolver blind spot. The same audit on
single-app repos (e.g., cli-jaw) showed a healthy ~22% effective
rate — the bug hid on flat repos and only manifested in multi-tsconfig
trees.

**Critical design note:** `@/*` is NOT a global alias. It is a
**per-fromFile local alias**. A flat alias map of the form

```
@/* → apps/agents/*
@/* → apps/admin/*
```

cannot work: whichever entry is inserted last wins globally, and the
resolver returns the wrong file for at least one app. The resolver
must be **scope-aware** — for each importing file, apply the nearest
applicable tsconfig whose scope directory contains the importer.

**Mitigation (v1.9.7):**

1. New module `_lib/tsconfig-paths.mjs` walks the repo and records
   every `compilerOptions.paths` entry with its scope directory
   (directory of the owning tsconfig.json) and `baseUrl`. Handles
   `extends` chains and JSON-with-comments.
2. `_lib/alias-map.mjs` attaches the flat array as
   `aliasMap.scopedTsconfigPaths`.
3. `_lib/resolver-core.mjs` consumes it BEFORE `package.json`
   exports lookup. Filters to entries whose `scopeDir` contains
   `fromFile`, sorts nearest-scope-first then longest-prefix-first,
   tries each matching target in order.
4. New sentinel `UNRESOLVED_INTERNAL` distinguishes "local alias
   matched but target file missing" (scanner blind spot) from
   `EXTERNAL` (legitimate npm package).
5. `build-symbol-graph` splits `unresolvedUses` into three counters:
   `uses.resolvedInternal` / `uses.external` / `uses.unresolvedInternal`.
   External packages no longer inflate the resolver-blindness ratio.
6. `symbols.json` emits `topUnresolvedSpecifiers[]` with prefix,
   count, example, and `likelyCause` heuristic — when the prefix
   matches `@/`, `~/`, `#/`, or a `@scope/` pattern, we tell the user
   "check per-app tsconfig.json" explicitly.
7. `rank-fixes` and `emit-sarif` now gate on `unresolvedInternalRatio`,
   not the legacy conflated total.

**Evidence:** `tests/test-tsconfig-paths-scoped.mjs` verifies the
invariant with a two-app fixture. T3 (same specifier, different
importers, different target files) is the structural guard — a flat
alias map cannot satisfy it. Empirically verified that reverting the
scoped-paths block makes T1–T4 and T6 all fail (5 of 7 assertions).

**Iron Law implication:** Tier C is raw evidence, not a claim.

```
Tier C = no consumer found in the constructed graph
       ≠ unused / truly dead
```

When `unresolvedInternalRatio` is high or top unresolved prefixes
match local alias conventions, Claude must downgrade the claim
rather than treat Tier C as truth. v1.9.5's ranking layer gates
on this via SAFE_FIX → DEGRADED demotion; Claude's review layer
should do the same when reading fix-plan.json.



When a new FP pattern is discovered, add here with incremented FP-NN number and source case description.

### Template for new entries

```
## FP-NN — <pattern name>

**Pattern:** <what the detector encounters>
**Symptom:** <how it surfaces in artifacts>
**Mitigation:** <how to handle or exclude>
**Example:** <one concrete case>
```

### Previous sessions

- **Geulbat 2026-04-18**: Seeded FP-01 through FP-10. `.d.ts` const case (FP-02) and vite.config default (FP-01) confirmed by grep cross-check. FP-03 (`#prefix`) was hit in Geulbat daemon package but not corrected in original run.

## Checking against this ledger

Before emitting any dead / semi-dead finding, check:

1. Does the file match a bundler config pattern? → FP-01
2. Is it a `.d.ts` ambient declaration? → FP-02
3. Is the "missing consumer" actually behind `#prefix` imports? → FP-03
4. Is it a React import in `.tsx`? → FP-04
5. Does the symbol flow through a re-export chain? → FP-05
6. Does a partner predicate function exist? → FP-06
7. Does a test file consume it (production-only scope)? → FP-07
8. Is this Python method call resolution? → FP-08
9. Is the file auto-generated? → FP-09
10. Is `isolatedModules` in play? → FP-10
11. Are these discipline patterns inside the scanner's own source? → FP-11
12. Does the target repo put tests in a top-level `tests/` directory? → FP-12
13. Are repo-root entry files (`server.ts`, `main.ts`, `index.ts`) wiring the app? → FP-13
14. Does the repo have frontend code in `public/` / `app/` / `pages/`? → FP-14
15. Is L2 method resolution dominated by `any-typed`? Check if `node_modules/@types/` exists. → FP-15
16. Is the target repo using `from 'src/...'` root-prefix imports without tsconfig paths? → FP-16
17. Does the target path have unconventional top-level subdirs (`client`, `server`, `shared`)? → FP-17
18. Does the code use `await import(...)` for lazy loading? → FP-18
19. Are `React` / `useCallback` flagged semi-dead in `.tsx` files? → FP-19
20. Is a Class A (demote) verdict a bug or perceptual? → FP-20
21. Did `buildAliasMap` crash on nested conditional exports? → FP-21
22. Are `*.config.ts` / `eslint.config.mjs` / `vitest.config.*` flagged Class C? → FP-22
23. Are `package.json.exports` target files (public API) flagged Class A/C? → FP-23
24. Are exports within `examples/` cross-file imports under-resolved? → FP-24
25. Are deep barrel re-export chains hiding transitive consumers in monorepos? → FP-25
26. Is the repo a pnpm monorepo (`pnpm-workspace.yaml` only, no `workspaces` in package.json)? → FP-26
27. Are `app/page.tsx`, `pages/foo.tsx`, SvelteKit `+page.svelte` flagged as dead? → FP-27
28. Does the repo use custom source-pointing conditional exports (e.g., `@zod/source`)? → FP-28
29. Is the pnpm-workspace.yaml using `packages/**` (recursive) or negated patterns? → FP-29
30. Are `server/api/`, `runtime/handlers/`, `plugins/` files with `export default` flagged dead in a Nuxt/Nitro repo? → FP-30
31. Are test fixture directories (`fixtures/`, `*-fixture/`, `test/`, `playground/`, `__*__`) showing as production dead? → FP-31
32. Does the audit crash with `SyntaxError: Unexpected token '﻿'` on a package.json? → FP-33 (BOM)
33. Is this a multi-app monorepo where each `apps/*` has its own `tsconfig.json` with `paths: { "@/*": [...] }`? → FP-36 (scope-unaware resolver)

If any matches: add `false_positive_flag: true` and `fp_ledger_refs: [FP-NN]` to the finding, and emit `[degraded]` or exclude entirely depending on severity.
