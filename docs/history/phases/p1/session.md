# docs/history/phases/p1/session.md — Phase 1 pre-write gate implementation

> **Phase:** P1 — pre-write mode.
> **Role:** implementation sequencing for the pre-write gate protocol defined in `canonical/pre-write-gate.md`. This file is subordinate to canonical; when they disagree, canonical wins.
> **Status:** session plan, v1 (advisory-only, label-free).
> **Last updated:** 2026-04-20

---

## 1. Boot (read before starting)

The canonical spine files below are the input-side invariants for this phase. Skip any and the implementation will drift.

| File | Why this phase needs it |
|---|---|
| `canonical/invariants.md` | Iron Law (no structural claim without machine evidence); four failure modes; optimization target ("before writing new code, make Claude aware of what already exists"); §9 `any` sub-invariants. This phase IS the optimization target. |
| `canonical/mode-contract.md` | Trigger dispatch (§2 + §3). Tells this phase when to run, when NOT to run (guards alone = no trigger; guard+verb = trigger), and §5 deliverable contract (summary + artifact + citation trail). |
| `canonical/pre-write-gate.md` | Full protocol. §3 is the five-step sequence this session implements. §5 is the output format to emit. §7 is the confidence-and-scan-range rule every claim-bearing line must obey (section headers / blank lines / non-claim prose are exempt). §8 is the canonical/ directory interaction — recognized canonical owner tables are the **first source** of "already exists" claims, not just a drift target. |
| `canonical/fact-model.md` | §3 fact types consumed (type-owner, helper-owner, topology-edge, boundary-rule, resolver-confidence, blind-zone, type-escape). §4 confidence-downgrade rules. Every lookup emits results that cite one of these. |
| `canonical/identity-and-alias.md` | §2 identity rule (`ownerFile::exportedName`); §4 import alias preservation (`importRecord`); §6 chain resolution algorithm (mixed-file aware, ambiguity-preserving for star re-exports); §9 resolver-confidence downgrade. Name lookup correctness depends on this. |
| `canonical/any-contamination.md` | §3 contamination tiers; §4 `anyContamination` annotation shape; §6 Stage 1 + §9 "Pre-write gate interaction" (demotion rule for reuse candidates); §11 honesty requirements (raw measurements, not just labels). |

**Reading order per `canonical/index.md` §3** ("I am implementing P1 v1 — label-free advisory"): the six files above. `classification-gates.md` is deliberately NOT in this list — P1 v1 surfaces raw fan-in and contamination measurements, not classification labels. When the advisory starts emitting labels (P1 v2), the canonical file list gains `classification-gates.md`.

## 2. Scope

### 2.1 In scope (P1 v1)

- New mode `pre-write`, dispatched per `canonical/mode-contract.md` §2.1.
- Five-step protocol (ground state lookup → intent extraction → per-intent lookup → advisory emission → Claude writes). Full flow wired; each step working end-to-end on warm-cache + cold-cache repos.
- Four intent-item lookup paths: name candidate, shape candidate, file candidate, dependency candidate. Plus the fifth intent bullet: planned type escapes.
- Advisory renders with grounded citations per `canonical/invariants.md` §1 Iron Law on every **claim-bearing** line (section headers, blank lines, and non-claim prose are exempt — see §9 reviewer checklist).
- **Label-specific** any-contamination rendering per `canonical/pre-write-gate.md` §3 + `canonical/any-contamination.md` §6 Stage 1 / §9. Blanket demotion is FORBIDDEN:
  - clean (no annotation) → `[grounded, ...]`.
  - `unknown-surface` only → `[grounded structural, semantic caution: unknown-surface]`. Advisory treats `unknown` as safe boundary, NOT contamination.
  - `has-any` only (no escalation) → `[grounded structural, any signal present, semantic caution]`. Raw measurement surfaced.
  - `any-contaminated` or `severely-any-contaminated` → `[degraded, any-contaminated, confidence: low]`. Raw measurements surfaced.
- Resolver-confidence degradation per `canonical/fact-model.md` §4 + `canonical/identity-and-alias.md` §9 — per-identity demotion when a finding's file shape matches an unresolved specifier.
- Fan-in is **identity-keyed only**. `ownerFile::exportedName` fan-in is the only grounded source. Name-keyed sources (`symbols.topSymbolFanIn[name]`) are NEVER rendered as grounded fan-in per `canonical/identity-and-alias.md` §3 — if only name-keyed data is available, fan-in is reported as `null` with `fanInConfidence: "unavailable"` and the advisory emits `[확인 불가, reason: identity fan-in unavailable]`.
- **Capability discovery** before lookup. The `symbols.json` producer may or may not support `anyContamination` annotations. P1 reads `symbols.json.meta.supports.anyContamination` before interpreting absence-of-annotation as "clean" — see §5.0.
- Failure semantics per `canonical/mode-contract.md` §6 — script failure emits `[확인 불가, mode: pre-write, reason: ...]`, never silent success.

### 2.1.1 `plannedTypeEscapes` schema (tightened)

Non-empty items MUST be structured objects, not free-form strings. Keys mirror `canonical/fact-model.md` §3.9 `type-escape` so the post-write delta (P2) can compare planned vs observed 1:1.

```json
{
  "plannedTypeEscapes": [
    {
      "escapeKind": "as-unknown-as-T",
      "locationHint": "src/vendor/wrapper.ts::adaptResponse",
      "codeShape": "response as unknown as ThirdPartyShape",
      "reason": "upstream SDK lacks type exports",
      "alternativeConsidered": "unknown + decoder, rejected because runtime validation library is not yet approved"
    }
  ]
}
```

Validation rules (enforced in §5.2):

- `escapeKind` MUST be one of the 10 values in `canonical/fact-model.md` §3.9: `explicit-any`, `as-any`, `angle-any`, `as-unknown-as-T`, `rest-any-args`, `index-sig-any`, `generic-default-any`, `ts-ignore`, `ts-expect-error`, `no-explicit-any-disable`.
- `reason` is REQUIRED. Empty reason is a defect — the intent-side half of the three-stage defense is "Claude declared WHY" (any-contamination.md §6 Stage 1).
- `locationHint` is REQUIRED. If the specific identity is not yet known, the literal string `"unknown"` is acceptable; missing field is not.
- `codeShape` is OPTIONAL but recommended; enables a stronger planned-vs-observed match in P2.
- `alternativeConsidered` is OPTIONAL; documents the reason is load-bearing.

Empty `plannedTypeEscapes: []` is the default and is the explicit declaration "zero escapes planned". Post-write will treat any observed escape as a silent introduction per any-contamination.md §6 Stage 2.

### 2.2 Out of scope (deferred)

- **Shape-hash lookup.** P4 feature. P1 v1 emits `[확인 불가, shape index not yet implemented]` for shape-candidate rows per `canonical/pre-write-gate.md` §5 output-format example.
- **Classification labels in advisory output.** P1 v1 surfaces raw fan-in + contamination measurements, not `single-owner-strong` / `DUPLICATE_STRONG` / etc. Label surfacing is P1 v2 (requires adding `classification-gates.md` to the boot list and introducing `_lib/canon-draft.mjs` classification functions).
- **`any-inventory.json` artifact + post-write delta.** P2 feature per `canonical/any-contamination.md` §6 Stage 2. P1 consumes any `type-escape` facts that exist but does NOT produce the artifact itself.
- **Canon-draft emission.** P3 feature. P1 READS canonical/ when present (per `pre-write-gate.md` §8) but does not write to `canonical/` or `canonical-draft/`.
- **Drift formal definition.** P5 feature per `canonical/canon-drift.md` (skeleton). P1 emits one-line drift warnings as `pre-write-gate.md` §8 describes, without a formal drift-analysis pass.
- **Multi-repo / cross-workspace resolution.** Single repo, single `<root>`. Workspace-package handling inherits from existing `_lib/paths.mjs`; no new cross-repo logic.

## 3. Dependencies

### 3.1 Canonical spine (normative)

Six files listed in §1. Every behavior here must be traceable to a line in one of them. Any implementation choice that doesn't have a canonical anchor is either a defect or a canonical gap — investigate which.

### 3.2 Existing code (reused — no new copies)

- `_lib/cli.mjs::parseCliArgs` — mode flag, `--root`, `--output`, `--intent` (new, see §5.2).
- `_lib/artifacts.mjs::loadIfExists(dir, name, {tag})` — read `symbols.json`, `topology.json`, `triage.json`, `checklist-facts.json`, `any-inventory.json` (when present). Silent on absence, taggable for error-message clarity.
- `_lib/resolver-core.mjs::makeResolver` — path resolution for dependency-candidate lookup.
- `_lib/alias-map.mjs::extractStringTarget`, `mapOutputToSource` — barrel / re-export walking.
- `_lib/finding-provenance.mjs::specifierCouldMatchFile` — per-identity resolver-confidence demotion.
- `_lib/classify-facts.mjs::countFileReferencesAst` — if the advisory needs a quick file-internal usage hint (e.g. "reusing `formatDate` from `./date.ts` — it has 8 call sites there already"). Post-FP-41-fix this is JSX-aware.
- `_lib/test-paths.mjs::isTestLikePath` — tag test-file candidates so the advisory can flag them (reusing a test-only helper from prod code is a smell).
- `_lib/vocab.mjs` — `EVIDENCE.*`, `TAINT.*` constants. No new strings-as-vocab.

### 3.3 Input artifacts (consumed when present; cold-cache falls back to fresh scripts per `pre-write-gate.md` §4)

| Artifact | Use | Fallback |
|---|---|---|
| `<output>/symbols.json` | `defIndex`, `reExportsByFile`, `unresolvedInternalSpecifiers`, `filesWithParseErrors`, `topSymbolFanIn` | Run `build-symbol-graph.mjs` on demand (warm cache on second run) |
| `<output>/topology.json` | File-node existence, cross-submodule edge direction | Run `measure-topology.mjs` on demand |
| `<output>/triage.json` | `boundary-rule` facts (ESLint `no-restricted-imports`, workspace deps) | Run `triage-repo.mjs` on demand |
| `<output>/checklist-facts.json` | A5 decoupling ratio + A6 cycles + blind-zone annotations | Not needed for P1 v1 — skip if absent |
| `<output>/any-inventory.json` | `type-escape` facts for the planned-vs-observed hint | Not needed for P1 v1 — skip if absent (P2 producer) |
| `<root>/package.json` | Dependency-candidate lookup (`already imported from node_modules`) | Always present; read directly |
| `<root>/canonical/*.md` | Drift-warning source per `pre-write-gate.md` §8 | Absent is normal; advisory emits without drift warnings |

## 4. Outputs

### 4.1 Primary output — the pre-write advisory

A structured report returned to Claude (the caller) before the write proceeds. Format per `canonical/pre-write-gate.md` §5. The advisory is the DELIVERABLE; it is consumed by Claude, not by the user.

### 4.2 Machine artifact (optional, for audit trail)

Written to two paths per invocation so P2's "before snapshot" can uniquely address this run:

- `<output>/pre-write-advisory.latest.json` — always overwritten; convenience pointer for the most recent invocation.
- `<output>/pre-write-advisory.<invocationId>.json` — never overwritten; `invocationId` is an ISO-timestamp-plus-random-suffix string (e.g. `2026-04-20T12-30-00Z-abc123`). P2 reads this when it needs the exact snapshot for a specific task.

Content:
- `invocationId` — string identifier, stable per invocation.
- `intentHash` — `sha256` of the normalized intent JSON; lets P2 detect intent changes between pre-write and post-write even when the artifact is re-loaded.
- `taskId` — optional; caller-supplied task identifier for correlation with user-facing systems.
- `intent` — the five-bullet intent structure Claude stated (names, shapes, files, dependencies, plannedTypeEscapes). `plannedTypeEscapes` uses the tightened schema from §2.1.1.
- `lookups[]` — one entry per intent item, with `{intentItem, lookupResult, citation, confidence, degradationReasons, nearNames?}`.
- `boundaryChecks[]` — any boundary-rule edges the intent would cross.
- `drift[]` — canonical/ vs observed AST disagreements (empty when canonical/ absent; never emitted for unrecognized canon schemas, see §5.9).
- `capabilities` — a copy of `symbols.meta.supports` used during this invocation. Lets P2 verify the post-write producer supports the same capabilities before attempting a delta.
- `failures[]` — any mode-failure markers per `mode-contract.md` §6.

The JSON is a fact artifact in the sense of `canonical/fact-model.md` (every entry carries `source`, `scope`, `confidence`, `observedAt`). Downstream (P2 post-write) reads this as the pre-write snapshot for delta.

### 4.3 Claude's citation trail (per `mode-contract.md` §5)

Every **claim-bearing** line / bullet in the rendered advisory MUST cite a field in the JSON artifact in the form `[grounded, <artifact>.json.<path> = <value>]` per invariants.md Iron Law. `[확인 불가]` / `[degraded, ...]` variants apply when the rule in §7 of `pre-write-gate.md` fires. Section headers, blank lines, horizontal rules, and explanatory non-claim prose are exempt — citation is a property of claims, not a property of every newline.

## 5. Sequence (implementation order)

Ordered so early work unblocks later work. Each step ends in a testable state. **Phases P1-0 / P1-1 / P1-2 / P1-3 are intended as separate review gates** — land each fully green before the next starts. This keeps failure modes narrow.

```
P1-0  preparatory patch    — §5.0 only
P1-1  core                  — §5.1 dispatcher, §5.2 intent, §5.3 name lookup, §5.8 render, advisory artifact
P1-2  side lookups          — §5.4 file, §5.5 dep, §5.6 shape placeholder, §5.7 planned escapes
P1-3  integration + drift   — §5.9 drift, §5.10 integration + audit-repo flag
```

### 5.0 Preparatory patch (P1-0 — separate session from P1-1)

This step is NOT "implementation of P1". It is a standalone preparatory session whose job is to make later phases possible. Shipping it separately keeps its drift surface small and its failures easy to read.

Checks:

- Every `_lib/*.mjs` file referenced in §3.2 exists.
- Every named export (`parseCliArgs`, `loadIfExists`, `makeResolver`, `extractStringTarget`, `mapOutputToSource`, `specifierCouldMatchFile`, `countFileReferencesAst`, `isTestLikePath`, `EVIDENCE`, `TAINT`) resolves when imported.
- `symbols.json` emitted by `build-symbol-graph.mjs` includes a `meta.supports` block. If not, add it in this preparatory patch — downstream capability-discovery (§5.3) depends on this field. Shape:

  ```json
  {
    "meta": {
      "schemaVersion": 3,
      "supports": {
        "anyContamination": true | false,
        "identityFanIn": true | false,
        "reExportRecords": "symbol-level" | "file-level" | "absent"
      }
    }
  }
  ```

- **Strict `supports.anyContamination: true` condition.** A producer may set `supports.anyContamination: true` ONLY when it can actually emit the canonical `{label, labels, measurements}` annotation shape on at least one identity. If the producer does not yet emit the annotation (or emits a non-conforming flat shape), `supports.anyContamination: false` is the only honest value. **Setting `true` optimistically is a production-safety defect** — it causes §5.3's "annotation absent + supports=true → clean" branch to fire on identities that were never actually measured, silently inferring `any`-clean status for potentially contaminated code. This is the single most dangerous failure mode in P1 and is specifically test-pinned.
- `symbols.json` already carries identity-keyed fan-in either on `defIndex` entries or a separate `fanInByIdentity` map. If only name-keyed fan-in exists (`topSymbolFanIn`), add an `fanInByIdentity` emission in this preparatory patch — §5.3 cannot render grounded fan-in without it.
- **`FP_BUDGET = 0` corpus gate verification.** Confirm `tests/test-corpus.mjs` contains the `FP_BUDGET = 0` constant and that the current suite run respects it. If the gate is absent, either (a) add it in this preparatory patch or (b) remove the FP_BUDGET phrase from later exit criteria. Do not leave the reference dangling.

**Exit criteria (P1-0):** `tests/test-pre-write-bootstrap.mjs` asserts the following:
- Every referenced `_lib/*.mjs` export is importable.
- `symbols.meta.supports` block exists on a freshly-run fixture.
- `supports.anyContamination === true` requires the fixture's `symbols.json` to show at least one identity carrying a conforming `anyContamination` annotation (non-empty `labels` array and populated `measurements`). `true` without a conforming emission is a TEST FAILURE.
- `supports.anyContamination === false` means absent annotations are NOT inferred as clean — downstream lookup must emit `[확인 불가]` for those identities (this is verified by a secondary assertion in `tests/test-pre-write-lookup-name.mjs`).
- A flat legacy `anyContamination: { label, anyFieldRatio, ... }` (without `labels` / `measurements` wrapper) is rejected by the bootstrap parser or downgraded to `[확인 불가, reason: non-conforming anyContamination schema]`.
- `FP_BUDGET` constant exists in `tests/test-corpus.mjs` and equals 0.

P1-0 ships as its own reviewable gate. Do NOT bundle its changes with §5.1–§5.10 — this preparatory patch is small but load-bearing, and mixing it with core P1 work makes regression triage harder.

### 5.0.1 Canonical owner-table parser (P1-0 sub-step)

Recognized-schema canon parser, referenced by §5.3 (name lookup) and §5.9 (drift warning). Lives in `_lib/pre-write-canonical-parser.mjs`.

- Parses `canonical/type-ownership.md` and `canonical/helper-registry.md` when present AND when the file carries the generated-canon header signature (`> **Status:** draft, v{N}` or `> **Source:** _lib/extract-ts.mjs pass ...` per `docs/spec/SPEC-canon-generator.md` §6).
- Returns `{ ownerTables: [{ file, rows: [{name, ownerFile, line}] }], recognized: true }` on recognized schemas.
- Returns `{ ownerTables: [], recognized: false, reason: '...' }` on free-form canonical files — callers then skip canonical-first lookup and emit no drift warnings (§5.9).
- MUST NOT attempt to parse non-recognized tables heuristically. "Maybe this looks like an owner table" is forbidden.

**Exit criteria:** `tests/test-pre-write-canonical-parser.mjs` covers: recognized schema → owner rows extracted with correct line numbers; free-form canon → empty result + `recognized: false`; mixed file (recognized header + some free-form prose) → header-scoped tables only parsed; missing file → empty result + `recognized: false` with reason `"canonical/<file> absent"`.

### 5.1 Wire `pre-write` mode into dispatcher (P1-1)

- `SKILL.md` — add `## Pre-write mode` subsection after existing `## Structural review mode`, mirroring the pattern. One paragraph pointing at `docs/history/phases/p1/session.md` + `canonical/pre-write-gate.md`.
- `audit-repo.mjs` — add opt-in `--pre-write` flag plus `--intent <file>` (for non-interactive testing). NOT in default `quick` / `full` profiles.
- New file `pre-write.mjs` at repo root — CLI entry. Parses `--root`, `--output`, `--intent`, `--advisory-out`.
- `_lib/mode-dispatch.mjs` (new, ~60 LOC) — pure function `dispatchMode(userText, cwdMeta) → { mode, rationale }` implementing `mode-contract.md` §2.1 + §3. Used by `pre-write.mjs` + future post-write CLI.

**Exit criteria:** `node pre-write.mjs --root <r> --intent /dev/stdin` reads an intent JSON on stdin, prints "pre-write skeleton; no lookup yet" without errors. Dispatch table tested in isolation.

### 5.2 Intent extraction (P1-1)

Two entry points. Claude's in-session use does NOT parse natural language — Claude itself produces the intent structure by applying the five-bullet template from `pre-write-gate.md` §3 Step 2. `pre-write.mjs` accepts this as JSON via `--intent <file>` (or stdin):

```json
{
  "names": ["formatTimestamp"],
  "shapes": [
    { "fields": ["year", "month", "day", "hour"] }
  ],
  "files": ["src/utils/time.ts"],
  "dependencies": ["dayjs"],
  "plannedTypeEscapes": []
}
```

- `_lib/pre-write-intent.mjs` — schema validator + normalizer. Rejects intent blocks missing any of the five keys (empty arrays are OK; missing keys are not — matches `pre-write-gate.md` §3 Step 2 "five items" requirement).

**Exit criteria:** schema validator rejects malformed intent with a precise error citing the missing key; accepts well-formed intent with empty-list default where applicable. Unit test `tests/test-pre-write-intent.mjs`.

### 5.3 Name-candidate lookup (P1-1)

- `_lib/pre-write-lookup-name.mjs` — `lookupName(intentName, {symbols, canonicalClaims, root}) → { result, citation, confidence, fanIn, fanInConfidence, anyContamination, canonicalClaim? }`.
- Implementation:
  1. **Capability discovery first.** Read `symbols.meta.supports.anyContamination` and `symbols.meta.supports.identityFanIn`. Missing → treat as `false`; the advisory will emit `[확인 불가, reason: producer did not emit <capability>]` for the affected dimension.
  2. **Canonical-first lookup** per `canonical/pre-write-gate.md` §8 — canonical is the FIRST source of "already exists", not just a drift target. Procedure:
     - If `canonicalClaims` is non-empty (from §5.0.1 recognized-schema parser output), search for `intentName` in the parsed owner table.
     - If found, the result includes `canonicalClaim: { ownerFile, exportedName, declaredAt: 'canonical/<file>:Lnn' }` and the rendered line says `CANONICAL_EXISTS` with citation `[grounded, canonical/<file>:Lnn row for <intentName>]`.
     - Then cross-check current AST (step 3+). Outcome matrix:
       - canonical + AST both present, aligned → `CANONICAL_EXISTS` + `EXISTS` both cited; confidence of each dimension carried separately.
       - canonical present, AST absent → `CANONICAL_EXISTS` rendered; current-AST confidence = `[확인 불가, reason: canonical claims presence, AST does not observe the name in the current scan range]`.
       - canonical present, AST disagrees on owner → surface both AND emit drift warning per §5.9.
       - canonical schema unrecognized (§5.0.1 returned no owner table) → canonical is NOT used as lookup evidence; fall through to AST-only path.
  3. Search `symbols.defIndex` for any file that has a direct top-level `exportedName == intentName`. Use identity `ownerFile::exportedName`.
  4. **Identity-keyed fan-in only.** If `symbols.meta.supports.identityFanIn == true` AND `symbols.fanInByIdentity[identity]` exists, render as grounded. Otherwise `fanIn: null`, `fanInConfidence: "unavailable"`, citation `[확인 불가, reason: identity fan-in not emitted by producer; symbols.topSymbolFanIn is name-keyed and would conflate distinct identities]`. **Never read `symbols.topSymbolFanIn[intentName]` for a grounded claim** — per `canonical/identity-and-alias.md` §3 that source is name-keyed and two different identities sharing a name would silently merge.
  5. **Label-specific contamination rendering** per `canonical/pre-write-gate.md` §3 "Any-contamination demotion rule":
     - `anyContamination` absent AND `supports.anyContamination == true` → clean, `[grounded, ...]`.
     - `supports.anyContamination == false` → `[확인 불가, reason: producer did not emit anyContamination capability]` — do NOT infer clean.
     - `labels == ["unknown-surface"]` only → `[grounded structural, semantic caution: unknown-surface]`. Surface `measurements.unknownFields`.
     - `labels` includes `has-any` but NOT `any-contaminated` / `severely-any-contaminated` → `[grounded structural, any signal present, semantic caution]`. Surface raw `measurements`.
     - `labels` includes `any-contaminated` or `severely-any-contaminated` → `[degraded, any-contaminated, confidence: low]`. Surface raw `measurements` including `anyFieldRatio`, `asAnyCount`, etc.
  6. Apply per-identity resolver-confidence downgrade per `identity-and-alias.md` §9 — if any unresolved specifier's path shape matches the owner file (via `specifierCouldMatchFile`), demote one level.
  7. **Near-name hint (search hint only, not a reuse claim).** After exact match, scan `symbols.defIndex` for names with edit-distance ≤ 2 OR shared prefix ≥ 4 chars against `intentName`. Emit up to 5 nearby names as a separate `nearNames` field with citation `[degraded, fuzzy-name match; source: symbols.json.defIndex name scan — search hint only, NOT a grounded reuse claim]`. The advisory renders this under an explicit `Search hints (not reuse candidates)` sub-section, clearly separated from `Already exists` EXISTS rows. Rationale: P1's core job is surfacing "Claude about to write `formatTimestamp` when `formatDate` / `formatDateTime` already exist" — exact-match alone misses this, but a fuzzy hint must never masquerade as an EXISTS claim.
  8. If absent from scan range, return `{ result: 'NOT_OBSERVED', scanRange: 'TS/JS production files ex tests', citation: 'symbols.json.defIndex does not contain the name', nearNames: [...] }`.

**Exit criteria:** `tests/test-pre-write-lookup-name.mjs` covers: exists with identity fan-in; exists with name-keyed-only fan-in (must emit `[확인 불가]`, NOT grounded number); exists clean; exists `unknown-surface`-only (grounded structural + caution, NEVER "contaminated"); exists `has-any`-only (grounded structural + mild caution); exists `any-contaminated` (degraded); exists `severely-any-contaminated` (degraded with severe measurements); capability absent (`[확인 불가]`, not clean-inferred); not-observed with near-name hint carrying degraded citation; resolver-confidence demotion when file shape matches unresolved specifier.

### 5.4 File-candidate lookup + boundary check (P1-2)

- `_lib/pre-write-lookup-file.mjs` — `lookupFile(intentFile, {topology, triage}) → { result, boundaryRule?, citation }`.
- Implementation:
  1. Check `topology.nodes` for the exact path.
  2. If exists, surface fan-in (inbound edges) + LOC (from `checklist-facts.json` if available).
  3. If new, determine containing submodule via `_lib/paths.mjs::buildSubmoduleResolver`.
  4. Cross-reference `triage.boundaryRules` for any rule whose `from` / `to` glob matches the new file's submodule and the intent's likely consumers.

**Exit criteria:** `tests/test-pre-write-lookup-file.mjs` covers: existing file; new file with no boundary rule; new file with an allowed boundary; new file with a forbidden boundary (advisory flags this but does not block).

### 5.5 Dependency-candidate lookup (P1-2)

- `_lib/pre-write-lookup-dep.mjs` — `lookupDependency(depName, {packageJson, symbols}) → { result, citation, existingImports }`.
- Implementation:
  1. Check `package.json.dependencies` + `devDependencies` + `peerDependencies`.
  2. If present, note any existing imports from `symbols.uses` that already consume this package — these are example consumer files, not a completeness claim.
  3. If present AND no internal import consumer observed, render as `DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS`. Do NOT call this "unused" or suggest cleanup — packages may be consumed by scripts, config files, runtime plugins, or build steps outside the import graph. Citation: `[확인 불가, scan range: import graph only; package may be used by scripts/config/runtime]`.
  4. If absent, return `NEW_PACKAGE` with install-required flag.

**Exit criteria:** `tests/test-pre-write-lookup-dep.mjs` covers: already dependency + at least one existing consumer (grounded example list); already dependency + no import consumer (renders `DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS` with `[확인 불가]` scan-range citation, NOT "unused"); new package; scoped package name handling; dependency declared in devDependencies vs peerDependencies (both count as available).

### 5.6 Shape-candidate lookup — degraded placeholder (P1-2)

P4 will ship the shape-hash fact producer. Until then:
- Return `{ result: 'UNAVAILABLE', citation: '[확인 불가, shape index not yet implemented; shape-hash pass is P4 per fact-model.md §5 producers table]' }` for every shape lookup.
- The advisory section `Watch-for` per `pre-write-gate.md` §5 renders this one line per shape item.
- DO NOT fall back to grep-based field overlap — `pre-write-gate.md` §5 explicitly forbids heuristic shape match claims. `[확인 불가]` is the correct answer here.

**Exit criteria:** shape lookups always return UNAVAILABLE with the exact citation string; `tests/test-pre-write-lookup-shape.mjs` pins this (prevents accidental re-introduction of a heuristic path).

### 5.7 Planned type escapes — echo through to output (P1-2)

- `_lib/pre-write-escapes.mjs` — `renderPlannedEscapes(intent.plannedTypeEscapes) → string`.
- No lookup; this is the Step 2 intent echo that post-write will later compare against observed escapes (P2 Stage 2). Non-empty entries MUST carry a declared reason; entries without a reason are rejected by the intent validator (§5.2).
- Empty list renders the default text from `pre-write-gate.md` §5: "0 escapes planned. Post-write will treat any observed ... as a silent introduction per any-contamination.md §6 Stage 2."

**Exit criteria:** `tests/test-pre-write-escapes.mjs` covers empty-list rendering and non-empty rendering with declared reason.

### 5.8 Advisory assembly + Markdown render (P1-1)

- `_lib/pre-write-render.mjs` — pure function `renderAdvisory(lookups, intent, failures) → { markdown, json }`.
- Markdown output exactly matches the format in `pre-write-gate.md` §5 (sections: Already exists, Already exists — any-contaminated, New code, Watch-for, Planned type escapes).
- JSON output is the §4.2 artifact shape.

**Exit criteria:** golden-file test (`tests/test-pre-write-render.mjs`) against a fixture advisory — assert section headers, citation format, and **claim-bearing citation coverage**: every rendered `EXISTS`, `CANONICAL_EXISTS`, `NOT_OBSERVED`, `DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS`, `NEW_PACKAGE`, `BOUNDARY`, drift, planned-escape, and near-name-hint line carries a citation. Section headers, blank lines, horizontal rules, and explanatory non-claim prose are exempt — citation is a property of claims, not every line.

### 5.9 Canonical/ drift warning (one-line, per §8) (P1-3)

Parser scope is deliberately narrow. Free-form canon markdown varies by author; attempting generic parsing would generate noisy false-positive drift. P1 only parses tables that match **known generated/promoted canon schemas** — specifically those emitted by (future) `generate-canon-draft.mjs` per `docs/spec/SPEC-canon-generator.md` §6 output format.

Detection:

- If the file has a header block stating `> **Status:** draft, v{N}` OR `> **Source:** `_lib/extract-ts.mjs` pass ...`, treat the registry tables as recognized-schema.
- If the file is prose or a free-form canon, emit NO drift warning. Optional: emit a single informational line `[확인 불가, reason: canonical/<file> schema not recognized; P1 drift parser supports only generated/promoted canon tables]`.

When schema is recognized:

- Parse owner claims from tables §2.1 (single-owner strong), §2.2 (single-owner weak/zero-internal), §2.3 (severely-any-contaminated single-owner), §2.4 (DUPLICATE_STRONG) per canon-draft output.
- Flag any name-lookup where canonical says X but current AST observes Y.
- Output: `CANONICAL DRIFT: canonical/type-ownership.md:Lnn lists owner as <X>; current AST observes <Y>.`

Full drift analysis is P5; this is a flag, not a resolution.

**Exit criteria:** `tests/test-pre-write-drift.mjs` covers: canonical absent (no warnings); canonical present with recognized schema, aligned (no warnings); canonical present with recognized schema, disagreement (one-line warning cited with line number); canonical present with UNrecognized schema (optional `[확인 불가]` informational line, no drift warnings emitted); multi-file canonical (type-ownership + helper-registry both parsed).

### 5.10 Integration + dogfood (P1-3)

- End-to-end test `tests/test-pre-write-integration.mjs` — builds a small fixture repo in `tmpdir()`, runs `pre-write.mjs --root <fx> --intent <stub>`, asserts the advisory markdown contains expected claim-bearing lines and that each such line carries a citation. Non-claim lines (headers, prose, blank) are not matched against the citation regex.
- Dogfood pass on this skill's own repo with a hand-written intent block (e.g. "names: [`formatDate`], files: [`src/utils/date.ts`]") and manual eyeballing against the canonical output.
- `audit-repo.mjs --pre-write` opt-in flag wired end-to-end.
- `scripts/update-test-doc.mjs` — register all new test files (listed in §6).

**Exit criteria:** full suite passes including `FP_BUDGET=0` corpus gate (see §5.0 for a bootstrap check that this gate exists in the repo); dogfood run on this skill's own repo emits an advisory whose every **claim-bearing** line cites a real artifact field; `tests/README.md` regenerated clean.

## 6. Test plan summary

New test files (land with the phase indicated in §5 step headers — do NOT mix P1-0 tests into a P1-1 PR):

1. `tests/test-pre-write-bootstrap.mjs` (P1-0) — §5.0 dependency inventory; `symbols.meta.supports` capability block presence; strict `supports.anyContamination=true` requires conforming emission; legacy flat schema rejected; `FP_BUDGET` constant exists in `tests/test-corpus.mjs` and equals 0.
2. `tests/test-pre-write-canonical-parser.mjs` (P1-0) — §5.0.1 recognized-schema detection; owner rows extracted from generated-canon headers; free-form canon → empty result with `recognized: false`; missing file handled cleanly.
3. `tests/test-pre-write-intent.mjs` (P1-1) — intent schema validator, including tightened `plannedTypeEscapes` schema (10 `escapeKind` enumeration, required `reason`/`locationHint`).
4. `tests/test-pre-write-lookup-name.mjs` (P1-1) — name lookup; **canonical-first when recognized schema present** (CANONICAL_EXISTS + AST cross-check matrix); **label-specific any rendering** (clean / unknown-surface-only / has-any-only / any-contaminated / severe); identity-keyed-only fan-in (name-keyed refused); capability-absent `[확인 불가]` NOT collapsed to clean; near-name hint degraded AND rendered under explicit "Search hints (not reuse candidates)" sub-section, not under EXISTS; resolver confidence per-identity.
5. `tests/test-pre-write-lookup-file.mjs` (P1-2) — file lookup + boundary rule check.
6. `tests/test-pre-write-lookup-dep.mjs` (P1-2) — dependency lookup; `DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS` rendering with `[확인 불가]` citation (never "unused").
7. `tests/test-pre-write-lookup-shape.mjs` (P1-2) — shape lookup returns UNAVAILABLE; heuristic fallback forbidden.
8. `tests/test-pre-write-escapes.mjs` (P1-2) — planned type escapes rendering (empty-list default + non-empty with all 10 `escapeKind` values); validator rejection of missing `reason` / `locationHint` / out-of-enum `escapeKind`.
9. `tests/test-pre-write-render.mjs` (P1-1) — advisory Markdown + JSON golden file; **claim-bearing citation coverage** (every EXISTS / CANONICAL_EXISTS / NOT_OBSERVED / DEPENDENCY_AVAILABLE / NEW_PACKAGE / BOUNDARY / drift / planned-escape line cited; near-name hints rendered under the "Search hints (not reuse candidates)" sub-section, not under "Already exists"; non-claim lines exempt).
10. `tests/test-pre-write-drift.mjs` (P1-3) — canonical drift one-liner with recognized-schema detection (generated-canon tables only; free-form canon emits no drift).
11. `tests/test-pre-write-advisory-artifact.mjs` (P1-1) — `pre-write-advisory.latest.json` + `.<invocationId>.json` both written; `invocationId` stable within invocation; `intentHash` sha256 of normalized intent; `capabilities` copied from `symbols.meta.supports`.
12. `tests/test-pre-write-integration.mjs` (P1-3) — end-to-end with fixture repo.
13. `tests/test-mode-dispatch.mjs` (P1-1) — mode dispatch rule table (guard alone vs guard+verb vs verb alone — per mode-contract.md §2.1 + §3.5).

FP budget stays at 0.

## 7. Non-goals for this session

- No LLM-side changes. P1 is CLI + library; Claude's prompt behavior for invoking pre-write is a separate concern handled by `SKILL.md` + the user runtime.
- No `_lib/shape-hash.mjs` / `any-inventory.mjs` — those are P4 and P2 producers respectively.
- No classification-label emission in the advisory — that is P1 v2 + `classification-gates.md` boot addition per `canonical/index.md` §3.
- No canon-draft writing — that is P3.
- No drift resolution logic — that is P5.

## 8. Known risks + open questions

- **Canonical/ drift flag false positives.** A canonical entry might be deliberately stale during a migration. The advisory flags it; Claude decides. Same shape as `pre-write-gate.md` §6 — the gate is advisory, not blocking.
- **Intent self-declaration gaming.** Nothing prevents Claude from declaring an empty `plannedTypeEscapes` list then writing `as any`. P1 emits the planned list honestly; P2 post-write delta catches silent additions. This is an expected layer boundary, not a P1 defect.
- **Resolver-confidence demotion scope.** The per-identity demotion in §5.3 uses `specifierCouldMatchFile`. Depth-limited to specifiers actually observed in the scan; deeply-aliased cases may under-demote. Documented as an honest limit, not a silent gap.
- **Latency budget.** `pre-write-gate.md` §4 states < 5s warm, < 30s cold. The cold path runs `build-symbol-graph.mjs` + `measure-topology.mjs` + `triage-repo.mjs`. If cold latency overshoots on large repos, add a `--no-fresh-audit` flag that falls back to `[확인 불가, reason: artifacts missing and cold-cache build skipped]` rather than blocking.

## 9. What a reviewer should check when this session closes

- Every lookup function cites a canonical rule (file + section) in its header comment.
- **Every claim-bearing line/bullet carries a citation.** Section headers, blank lines, horizontal rules, and pure explanatory prose are exempt. The integration test enforces citation coverage for: every `EXISTS` / `NOT_OBSERVED` / `LIKELY EXISTS` row; every `DEPENDENCY_AVAILABLE*` / `NEW_PACKAGE` row; every `BOUNDARY allowed` / `BOUNDARY forbidden` row; every drift warning; every `Planned type escapes` item (empty-list and non-empty both cited). Non-claim text is NOT matched against a citation regex.
- **Label-specific any handling in advisory, not blanket demotion.** Pinning tests present for each of: clean (grounded), `unknown-surface`-only (grounded structural + caution), `has-any`-only (grounded structural + mild caution), `any-contaminated` / `severely-any-contaminated` (degraded). No test or code path may render `unknown-surface` as "contaminated" or `has-any`-only as `[degraded, any-contaminated]`.
- **Fan-in is identity-keyed only.** Pinning test proves `symbols.topSymbolFanIn[name]` is never read as a grounded fan-in source. Name-keyed fan-in data → `fanIn: null`, `fanInConfidence: "unavailable"`, `[확인 불가]` citation.
- **Capability-discovery honesty.** Pinning test proves `symbols.meta.supports.anyContamination == false` does NOT collapse to "clean"; it emits `[확인 불가, reason: producer did not emit anyContamination capability]` per lookup row instead. Second pinning test proves `supports.anyContamination: true` is REJECTED by the bootstrap check unless the producer can actually emit a conforming `{label, labels, measurements}` annotation on at least one identity — optimistic `true` is a bootstrap-test FAILURE.
- **Canonical-first lookup (recognized schemas only).** Pinning test proves: when `canonical/type-ownership.md` has the generated-canon header, its owner rows are used as the FIRST source for name lookup (emits CANONICAL_EXISTS with the `canonical/<file>:Lnn` citation). When the canonical file is free-form prose with no recognized header, canonical is NOT consulted for lookup evidence (falls through to AST-only). The AST cross-check matrix (aligned / AST-absent / drift / unrecognized) is each exercised by a distinct fixture.
- **Near-name hint is a search hint, not a claim.** Pinning test proves `nearNames` rows render under a `Search hints (not reuse candidates)` sub-section heading, not under `Already exists`; the citation text contains "search hint only" verbatim; no near-name row can syntactically match the `EXISTS` rendering template.
- **FP_BUDGET=0 gate exists.** `tests/test-pre-write-bootstrap.mjs` reads `tests/test-corpus.mjs` and asserts the `FP_BUDGET` constant is declared and equals 0. Removes the possibility that later exit-criteria references are dangling.
- **`plannedTypeEscapes` schema conformance.** Validator rejects entries with missing `reason` or `locationHint`, or with `escapeKind` outside the §3.9 enumeration. Pinning test covers all 10 valid `escapeKind` values.
- Mode dispatch unit test covers every example in `mode-contract.md` §3.1 / §3.2 / §3.4 / §3.5 — guard alone, verb alone, guard+verb, pure inspection.
- Shape lookup cannot return anything except `UNAVAILABLE` in this phase; the pinning test is present.
- **`pre-write-advisory.json` invocation policy.** Pinning test proves both `pre-write-advisory.latest.json` AND `pre-write-advisory.<invocationId>.json` are written; `invocationId` is stable within an invocation; `intentHash` is sha256 of normalized intent.
- **Drift parser scope.** Pinning test proves drift parser emits NO warnings on free-form canonical/*.md files; only recognized generated-canon schemas (per `docs/spec/SPEC-canon-generator.md` §6) are parsed.
- `FP_BUDGET = 0` gate remains; full suite green.
- `canonical/pre-write-gate.md` edits during this session MUST be explicitly motivated by a reviewer P0 (e.g. the label-specific demotion clarification landed here). Implementation-driven silent tweaks are not allowed. If the session discovers a canonical gap, surface it as a separate spec amendment with reasoning.

## 10. Hand-off to P2 (post-write)

After P1 ships, P2 consumes the `pre-write-advisory.json` artifact as the "before" snapshot. P2's job is the Stage-2 delta per `any-contamination.md` §6 Stage 2 — compare planned vs observed type escapes, plus shape-hash duplicate detection, plus boundary violations introduced by the write. P1's advisory JSON is the contract; P1 must not break the `intent` / `lookups[]` shape without updating P2's consumer.

Expected P2 boot additions: `classification-gates.md` (if labels surface), `canonical/lifecycle.md` (skeleton → promoted when P2 starts). P1 does not need either.
