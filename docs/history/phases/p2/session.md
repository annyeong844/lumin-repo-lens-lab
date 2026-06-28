# docs/history/phases/p2/session.md — Phase 2 post-write delta

> **Phase:** P2 — post-write mode. Follows P1 (pre-write gate).
> **Role:** phase-level roadmap for post-write delta mode. Claude writes code → post-write compares the new state against the P1 pre-write snapshot → emits a delta artifact that makes `no silent new any` enforceable at response time.
> **Status:** phase plan, v2 (2026-04-20 reconciliation — P2-0/1/2 landed; this file now reflects post-landing behavior, not the pre-implementation draft).
> **Last updated:** 2026-04-20

> **Implementation authority.** When this file conflicts with the per-session specs on P2-0/1/2 implementation details (ambiguity routing, `computeDelta` signature, capability source, filename pattern, incomplete-inventory handling), the per-session specs WIN: `docs/history/phases/p2/p2-0.md`, `docs/history/phases/p2/p2-1.md v3`, `docs/history/phases/p2/p2-2.md v2`. This file is the strategic roadmap; it was reconciled on 2026-04-20 to match the landed behavior, but the per-session files remain the authoritative implementation references.

---

## 1. Boot (read before starting)

| File | Why this phase needs it |
|---|---|
| `canonical/invariants.md` | Iron Law; §9 sub-invariants — especially "No silent new `any`" (every newly-introduced `any` / `as any` / `as unknown as T` / `@ts-ignore` / `@ts-expect-error` / `no-explicit-any` disable MUST be acknowledged in the final response, or removed). P2 exists to make this invariant mechanically detectable. |
| `canonical/any-contamination.md` | §5 `type-escape` occurrence fact (P2 producer); §6 Stage 2 post-write delta (output format reference); §6 Stage 3 response-time rule (delta drives this); §10 producer responsibilities (`any-inventory.mjs` future producer — P2 lands this). |
| `canonical/fact-model.md` | §3.9 `type-escape.escapeKind` enumeration (10 kinds). Every P2 fact emission + comparison must cover all 10. §4 confidence rules. |
| `canonical/mode-contract.md` | §1 modes table + §2.3 post-write triggers (currently skeleton). P2 promotes §2.3 to full spec. §5 deliverable contract (summary + JSON artifact + citation trail) — P2 inherits. §6 failure semantics. |
| `canonical/pre-write-gate.md` | §5 output format (planned type escapes section). P2's "before" input is the pre-write advisory; the planned-escape list is the anchor P2 compares against. |
| `docs/history/phases/p1/session.md` | Parent P1 plan. §10 handoff contract — P2 consumes `pre-write-advisory.<invocationId>.json` as the before snapshot. |
| `docs/history/phases/p1/p1-3.md` §10 | Specific contract P1-3 left for P2: `advisory.capabilities` parity check, `advisory.intent.plannedTypeEscapes` JSON-order preservation, `advisory.drift[]` stability. |

Not required for P2 v1:
- `classification-gates.md` — P2 v1 is label-free, same as P1 v1.
- `identity-and-alias.md` — P2 operates on file/line occurrences (`type-escape`), not on identity fan-in.

## 2. Optimization target (reminder)

From `canonical/invariants.md` §6: *"Before writing new code, make Claude aware of what already exists."* P1 serves that. P2 serves the **complementary** invariant: *"No silent new `any` within the configured scan range."* A vibe-coder's session that drops five new `as any` into scanned files must not complete successfully without Claude's final response acknowledging each one.

**Scope qualifier is intentional.** `any-inventory.mjs` records `meta.scope` / `includeTests` / `exclude` / `fileCount`; every P2 claim is scoped to what the inventory actually scanned. The producer's default `includeTests: true` matches the codebase's `_lib/cli.mjs::parseCliArgs` convention; callers who want to exclude test-file `any` from the delta use `--production` (or the explicit `--no-include-tests`). The invariant wording is scan-range-qualified to stay honest — a delta cannot say "no silent new any" about files it never looked at.

Without P2, Stage 3 (response-time acknowledgment) is wishful thinking — there is no mechanical check that the final response actually cited what was introduced. P2 produces the delta artifact that makes the check automatable within the scan range.

## 3. Scope

### 3.1 In (P2 entire phase)

- **`any-inventory.mjs`** producer — fresh AST pass that emits `type-escape` facts per `canonical/fact-model.md` §3.9. Covers ALL 10 `escapeKind` values (not the partial set `any-contamination.md` §10 originally listed). Output artifact: `any-inventory.json`.
- **`_lib/post-write-delta.mjs`** — pure function with the signature:

  ```ts
  computeDelta({
    preWriteAdvisory,    // full P1 advisory artifact (JSON)
    beforeInventory,     // any-inventory.pre.<invocationId>.json (may be null)
    afterInventory,      // any-inventory.json emitted at post-write time (may be null)
    deltaInvocationId,   // CALLER-INJECTED string — keeps computeDelta pure
                         // (same inputs → byte-identical output). post-write.mjs
                         // generates it via generateDeltaInvocationId().
  }) → DeltaResult
  ```

  Internally reads `preWriteAdvisory.intent.plannedTypeEscapes` for planned matching. **Capability source is the inventory artifact itself** — `afterInventory.meta.supports.typeEscapes` + `escapeKinds[]` + `complete` (NOT `preWriteAdvisory.capabilities`, which is P1-0's symbols-graph projection; tests pin that advisory.capabilities shape never gates P2 classification). The four inputs are strictly separated:

  ```
  preWriteAdvisory    — planned intent + invocationId (P1 artifact)
  beforeInventory     — actual type-escape occurrences at pre-write time (P2-0 snapshot)
  afterInventory      — actual type-escape occurrences at post-write time
  deltaInvocationId   — freshly-generated id, injected so the function stays pure
  ```

  Output classifications per `canonical/any-contamination.md` §6 Stage 2 + P2-1 §4.2 label table (see below).
- **Planned ↔ observed matching rule** (required for P2-1; implemented in `_lib/post-write-delta.mjs`). `plannedTypeEscapes` entries carry `{escapeKind, locationHint, codeShape?, reason, alternativeConsidered?}` (per docs/history/phases/p1/p1-1.md §2.1.1); `type-escape` facts carry `{file, line, escapeKind, codeShape, insideExportedIdentity}` (per `canonical/fact-model.md` §3.9). Matching order:

  1. **Same `escapeKind`** is required — no cross-kind matching.
  2. **Location match** — one of:
     - `planned.locationHint === observed.insideExportedIdentity` (exact identity match), OR
     - `planned.locationHint === observed.file` (exact file match), OR
     - `planned.locationHint` is a **directory-boundary** prefix of `observed.file` — i.e. `planned.locationHint.endsWith('/') && observed.file.startsWith(planned.locationHint)`. Strings that share a prefix without ending at a `/` boundary (e.g. `src/foo` vs `src/foobar.ts`) do NOT match; normalize to `src/foo/` first. Prevents false planned-matches on neighboring similar paths.
     - `planned.locationHint === "unknown"` (caller explicitly declared location unknown — matches any candidate with the same kind).
  3. **Absent-from-before preference** (when baseline trustworthy): if `baseline.status === 'available'` AND `scanRangeParity.status === 'ok'`, prefer candidates that are NOT already in `beforeInventory`. Planned escapes are about to be added — picking an absent-from-before candidate matches intent. If all candidates are pre-existing, fall through and pick deterministically (honest: planned declared code that already existed).
  4. **Code-shape tiebreak**: if multiple observed candidates still match after steps 1–3, prefer those whose `normalizedCodeShape` equals `normalizeCodeShape(planned.codeShape)`. P2-1 imports `normalizeCodeShape` from `_lib/extract-ts-escapes.mjs` (exported per the P2-1 §3.3 narrow exception) so producer and matcher agree byte-for-byte. The normalizer is token-aware — whitespace INSIDE string literals, template literals, and comments is PRESERVED; only whitespace outside those is collapsed; trailing `;` dropped.
  5. **One-to-one default**: one planned item matches AT MOST one observed occurrence. If the planned entry has an explicit `count: N` field (future schema extension), match up to N.
  6. **Ambiguity** (authoritative P2-1 v3 behavior — NOT the pre-landing draft): if ≥2 candidates remain tied after step 4, the first (by deterministic `(file, line, occurrenceKey)` sort) is marked `planned`; every other remaining candidate carries a `'ambiguous-planned-match'` diagnostic and **passes through to baseline comparison**. The remainder's final label depends on the baseline lookup — `pre-existing` (key in before), `silent-new` (key NOT in before), or `observed-unbaselined` (baseline missing). **Remainders are NEVER hard-labeled `silent-new`** — the ambiguity diagnostic is orthogonal to whether the occurrence is new. Entry examples:

  ```json
  // Remainder found in before → pre-existing + diagnostic (NO Stage 3 ack required)
  { "label": "pre-existing", "diagnostics": ["ambiguous-planned-match"], ... }

  // Remainder NOT in before → silent-new + diagnostic (Stage 3 ack required)
  { "label": "silent-new",   "diagnostics": ["ambiguous-planned-match"], ... }

  // Baseline missing → observed-unbaselined + diagnostic (NO Stage 3 ack)
  { "label": "observed-unbaselined", "diagnostics": ["ambiguous-planned-match"], ... }
  ```

  Matching is performed before the baseline-state classification: planned-status wins, then baseline-comparison labels the remaining observed occurrences (including the ambiguity remainders).

- **Occurrence key — stable beyond line numbers.** `type-escape` facts carry a `line` field for display + fallback, but P2 matching against planned list and before-inventory uses a stable `occurrenceKey` hashed from structural attributes:

  ```
  occurrenceKey = sha256(
    file + '|' + escapeKind + '|' +
    normalizedCodeShape + '|' +
    (insideExportedIdentity ?? '<top-level>')
  )
  ```

  A formatter run that shifts lines but leaves code shape intact does NOT flip occurrences from `pre-existing` to `silent-new`. `line` remains in the artifact for citation and human reading.

  **Schema promotion required.** `occurrenceKey` and `normalizedCodeShape` are NEW fields beyond `canonical/fact-model.md §3.9`'s current `type-escape` shape (`file, line, escapeKind, codeShape, insideExportedIdentity`). Before P2-0 implementation lands, `canonical/fact-model.md` §3.9 must be amended to add these two fields (motivated reviewer P0; not a silent extension). Until the canonical amendment lands, P2-0 producer emission is the forcing function: the bootstrap test fails on canonical ↔ emission schema drift.

- **Escape-comment text preservation.** `ts-ignore` / `ts-expect-error` / `no-explicit-any-disable` facts carry their FULL comment text as `codeShape` (e.g. `"@ts-expect-error upstream type bug"`) — not just the directive keyword. Legitimate uses (type-system limits in fixtures) are distinguishable from silent drop-ins. Applies to all comment-style escape kinds.

- **`_lib/post-write-render.mjs`** — Markdown + JSON renderer for the delta, matching the format in `canonical/any-contamination.md` §6 Stage 2.
- **`post-write.mjs`** CLI entry — reads `pre-write-advisory.<invocationId>.json` (or `.latest.json`), runs `any-inventory.mjs` on current tree, computes delta, writes artifact + Markdown.
- **`audit-repo.mjs --post-write`** opt-in flag — mirrors `--pre-write` integration.
- **Release-blocking integration test** — multi-source fixture with intentional planned + silent escapes; delta + response-time check.

### 3.2 Out (deferred)

- Shape-hash duplicate detection (P4).
- Partner-export graph / framework sentinel for FP-41-style over-escalation (P2 sub-spec, separate landing after P2 core — see §7).
- Classification labels in the delta output (P2 v2).
- Canon-draft emission (P3).
- Scope-aware shadowing in identifier references (known v1.10.0 gap, deferred).

## 4. Three-phase breakdown

Like P1, P2 splits into separate review gates.

```
P2-0  preparatory patch   — any-inventory.mjs producer, type-escape fact emission
P2-1  core delta engine   — _lib/post-write-delta.mjs, render, post-write.mjs CLI
P2-2  integration + audit — audit-repo.mjs --post-write, end-to-end integration test
```

### 4.1 P2-0 — `any-inventory.mjs` producer

Standalone preparatory session. Not P2 core — its job is to make the downstream phases possible.

Deliverables:
- `any-inventory.mjs` (top-level CLI script) — walks TS/JS production files via `_lib/extract-ts.mjs` (existing) + a new `_lib/extract-ts-escapes.mjs` helper that emits one `type-escape` fact per occurrence. Covers the full 10-kind enumeration from `canonical/fact-model.md` §3.9.
- Artifact: `<output>/any-inventory.json` — `{ meta, typeEscapes: [...] }`. Required `meta` shape:

  ```json
  {
    "meta": {
      "tool": "any-inventory.mjs",
      "generated": "<ISO>",
      "complete": true,
      "scope": "TS/JS production files",
      "includeTests": true,
      "exclude": ["node_modules", "dist"],
      "fileCount": 123,
      "supports": {
        "typeEscapes": true,
        "escapeKinds": [
          "explicit-any", "as-any", "angle-any", "as-unknown-as-T",
          "rest-any-args", "index-sig-any", "generic-default-any",
          "ts-ignore", "ts-expect-error", "no-explicit-any-disable"
        ]
      }
    },
    "typeEscapes": [ /* one entry per occurrence, per fact-model.md §3.9 */ ]
  }
  ```

  - **`supports.typeEscapes === true`** is the producer's explicit promise that it can emit every `escapeKind` listed. Downstream P2-1 delta uses this for capability parity (NOT `anyContamination` — that was the P1 per-identity annotation; P2's facts are occurrence-level).
  - **`supports.escapeKinds[]`** MUST equal the 10-kind set from `canonical/fact-model.md` §3.9. The bootstrap test parses the canonical markdown and fails on drift.
  - **`meta.scope` / `includeTests` / `exclude`** describe the scan range. **Default `includeTests: true`** (codebase convention; producer defaults matches `_lib/cli.mjs::parseCliArgs`); `--production` opts out. P2-1 delta handling of mismatch (authoritative): when before/after scope differs, `scanRangeParity.status = 'mismatch'` and **baseline comparison is invalidated** — unmatched occurrences degrade to `observed-unbaselined`, `removed` is NOT computed, `requiredAcknowledgements` returns empty. **Planned matching (advisory.intent × afterInventory) still runs** — the mismatch does not invalidate intent-side evidence. This is narrower than the pre-landing draft wording ("downgrades confidence repo-wide"); the landed behavior preserves planned labels.

- **Pre-write snapshot hook** — `pre-write.mjs` extended to ALSO run `any-inventory.mjs` at the same time as the name/file/dep/shape lookups, writing `any-inventory.pre.<invocationId>.json` alongside `pre-write-advisory.<invocationId>.json`.

  **Both** advisory files — `pre-write-advisory.latest.json` AND `pre-write-advisory.<invocationId>.json` — MUST carry the same `preWrite.anyInventoryPath` pointer:

  ```json
  {
    "preWrite": {
      "anyInventoryPath": "any-inventory.pre.<invocationId>.json"
    }
  }
  ```

  `post-write.mjs` reads `preWrite.anyInventoryPath` from whichever advisory file was passed via `--pre-write-advisory`. A CI flow that pins a specific `<invocationId>.json` must be able to find the matching baseline without resolving through `.latest.json` — the invocation-specific advisory is self-contained.

  This is the contract that enables full `silent-new` classification in P2-1; without it, post-write degrades to `observed-unbaselined` honestly.

- Bootstrap test — walks a fixture containing one occurrence of each of the 10 `escapeKind` values; asserts all 10 appear in the output; asserts `meta.supports.escapeKinds[]` matches canonical 1:1; asserts `meta.scope` / `includeTests` / `exclude` are populated.

Exit criteria: `any-inventory.json` emits all 10 escape kinds on a conforming fixture; producer test pins the enumeration 1:1 against `fact-model.md` §3.9; pre-write snapshot hook writes the before-inventory alongside the advisory; `update-test-doc.mjs` clean.

### 4.2 P2-1 — core delta engine

Deliverables:
- `_lib/post-write-delta.mjs` — pure `computeDelta({preWriteAdvisory, beforeInventory, afterInventory, deltaInvocationId})`. `deltaInvocationId` is caller-injected (post-write.mjs generates it and passes it in) so the function is deterministic: same inputs → byte-identical output. Classifications, single authoritative table:

  | Label | Baseline needed? | Meaning | `requiredAcknowledgements()` returns it? |
  |---|---|---|---|
  | `planned` | no | planned escape observed after write | no, already declared |
  | `planned-not-observed` | no | planned entry had no matching observed occurrence (Claude planned but didn't write it) | no; positive/neutral note |
  | `silent-new` | **yes** (baseline available) | not planned; not in `beforeInventory`; observed in `afterInventory` | **yes — Stage 3 enforcement** |
  | `pre-existing` | yes (baseline available) | in both `beforeInventory` and `afterInventory`; not planned | no |
  | `removed` | yes (baseline available) | in `beforeInventory`; absent from `afterInventory` | no |
  | `observed-unbaselined` | only when baseline is MISSING | observed in `afterInventory`; not planned; baseline absent so new-vs-existing unknown | no — Stage 3 NOT triggered, informational only |

  The rule: **Stage 3 response-time enforcement fires ONLY on `silent-new`**. `observed-unbaselined` surfaces the set so Claude or a reviewer can audit, but never prescribes acknowledgment — a repo with 80 pre-existing `as any` must not get 80 Stage 3 alerts on first P2 run with no baseline.

  `requiredAcknowledgements(delta)` MUST return exactly the `silent-new` entries; NEVER `observed-unbaselined`, `planned-not-observed`, `pre-existing`, or `removed`. This is structurally pinned by test.

  The delta artifact's meta records baseline state, capability/scan-range parity, AND inventory completeness explicitly:

  ```json
  {
    "preWriteInvocationId": "...",
    "deltaInvocationId": "...",
    "baseline": {
      "status": "available" | "missing",
      "source": "any-inventory.pre.<invocationId>.json" | null,
      "reason": "...when missing..."
    },
    "capabilityParity": {
      "status": "ok" | "mismatch" | "unchecked" | "missing",
      "mismatchDetail": "..."
    },
    "scanRangeParity": {
      "status": "ok" | "mismatch" | "baseline-missing",
      "mismatchDetail": "..."
    },
    "inventoryCompleteness": {
      "afterComplete": true,
      "beforeComplete": true,
      "filesWithParseErrors": [
        { "side": "before" | "after", "file": "...", "message": "...", "line": 12 }
      ]
    }
  }
  ```

  **`inventoryCompleteness` exists because `meta.complete === false` (parse errors) creates a blind zone** — occurrences in unparseable files are NOT in `typeEscapes[]`. The renderer enforces: the fully-clean summary `"No silent new any in the scan range."` appears ONLY when `silentNew === 0` AND all parities ok AND baseline available AND BOTH completenesses true. Otherwise a caveated summary names the confidence-limiting reason.

  Recommended pairing for real delta: `pre-write.mjs` (P1) is extended by P2-0 to also snapshot an `any-inventory.pre.<invocationId>.json` alongside the advisory. Post-write reads that file to enable full `silent-new` classification. Without it, post-write degrades honestly to `observed-unbaselined`.

- `_lib/post-write-render.mjs` — `renderMarkdown(delta)` matches `any-contamination.md` §6 Stage 2 format exactly. `renderJson(delta)` returns the artifact shape.
- `_lib/post-write-artifact.mjs` — dual-write helper `writeDelta(outputDir, delta)` writing both `post-write-delta.latest.json` AND `post-write-delta.<preWriteInvocationId>.<deltaInvocationId>.json`. Also re-exports `generateDeltaInvocationId` (= `generateInvocationId` from `_lib/pre-write-artifact.mjs`). Atomic temp+rename. Re-running post-write for the same advisory produces a NEW specific file (fresh `deltaInvocationId`) and preserves prior specific files; `latest.json` tracks the newest.
- `post-write.mjs` CLI:
  - `--root <path>` — repo root.
  - `--output <dir>` — artifact dir.
  - `--pre-write-advisory <file>` — before snapshot (latest.json OR a specific invocationId.json).
  - `--delta-out <dir>` — where to write the delta artifact (defaults to `--output`).
  - `--no-fresh-audit` — skip cold-cache spawn of `any-inventory.mjs` for after-snapshot.
  - `--include-tests` / `--no-include-tests` / `--production` / `--exclude <pattern>` — scan-range flags forwarded to `any-inventory.mjs`.
  - Generates `deltaInvocationId` via `generateDeltaInvocationId()` BEFORE calling `computeDelta` → injects it → reads advisory → runs `any-inventory.mjs` (cold-cache pattern reused from P1-3) → computes delta → writes artifact → prints Markdown.

Exit criteria: unit tests for `computeDelta` covering every label from the §4.2 table — `planned`, `planned-not-observed`, `silent-new`, `pre-existing`, `removed`, `observed-unbaselined` — plus the cross-cutting cases (baseline available vs missing, scan-range mismatch, ambiguous-planned-match diagnostics, capability parity failure). Pinning test: `requiredAcknowledgements(delta)` returns EXACTLY the `silent-new` entries — never `observed-unbaselined`, never `planned-not-observed`, never `pre-existing`, never `removed`. Render golden fixtures cover each label's rendering. CLI smoke test exercises planned + silent + removed + observed-unbaselined in distinct runs. Shell-safety pinning (space + `$` path) per P1-3 rule.

### 4.3 P2-2 — integration layer

Deliverables:
- `audit-repo.mjs --post-write --pre-write-advisory <file>` flag. Not in default profiles. Exit-code contract parallels `--pre-write`:
  - 0 — audit succeeded; post-write either ran or was not requested.
  - 1 — audit-step-failed (existing).
  - 2 — `--post-write` requested without `--pre-write-advisory`, OR `--pre-write` and `--post-write` requested together (mutually exclusive; chaining would make the baseline = post-state).
- **`manifest.postWrite` summary block** — populated after the spawn succeeds by re-reading the delta JSON so CI / caller-side Stage 3 gating doesn't need to re-open the delta:

  ```ts
  manifest.postWrite = {
    requested: boolean,
    ran: boolean,
    reason?: string,                          // when !ran
    deltaPath?: string,                       // at (--delta-out ?? --output)/post-write-delta.latest.json
    silentNew?: number,
    requiredAcknowledgementCount?: number,
    baselineStatus?: 'available' | 'missing',
    scanRangeParity?: 'ok' | 'mismatch' | 'baseline-missing',
    afterComplete?: boolean,
  }
  ```

  Honest signal: if spawn succeeds but the delta JSON fails to parse, summary fields stay ABSENT (not defaulted to "clean"). Downstream treats missing-field as "unknown".

  `post-write.mjs` non-zero exit → `manifest.postWrite.ran === false`, final exit code stays 0 (advisory semantics per P1-3 parallel; future `--strict-post-write` flag could flip this to exit 2).

- **`requiredAcknowledgements(delta)` helper** lives in `_lib/post-write-delta.mjs` (landed in P2-1, not P2-2). Returns the list of `silent-new` escapes that a caller MUST acknowledge. `manifest.postWrite.requiredAcknowledgementCount` surfaces the count at the orchestrator level. P2 does NOT rewrite or inspect Claude's responses — it produces the evidence. See §7 known risks.
- Release-blocking integration test (`tests/test-post-write-integration.mjs`) — TWO fixtures:
  - **Fixture 1 (multi-label)**: `src/adapters/a.ts` + `src/adapters/b.ts` (TWO DIFFERENT files — distinct `occurrenceKey`s; required to create two genuine planned candidates) + `src/unplanned.ts` + `src/bad.ts` (parse error). Exercises `planned` + `silent-new` + `ambiguous-planned-match` + `afterComplete=false` in one run.
  - **Fixture 2 (baseline-missing)**: pre-write with `--no-fresh-audit` → advisory has no `anyInventoryPath` → post-write degrades to `observed-unbaselined` + caveated summary + `requiredAcknowledgements.length === 0`.
  - **Scope boundary (important)**: the integration test verifies the delta artifact + manifest carry the evidence needed for Stage 3 response-time enforcement. It does NOT verify a hypothetical Claude final response. Stage 3 is a caller-side contract.
- `SKILL.md` update — `## Post-write mode` subsection + orchestrator paragraph.

Exit criteria: all three phases landed + full suite green + FP_BUDGET=0 corpus still passes + lint/dead 0.

## 5. Handoff to P3 / P4

- P3 canon-draft generator (already specced in `docs/spec/SPEC-canon-generator.md v0.2.2`) proceeds independently of P2.
- P4 shape-hash producer — P2 does NOT enable shape-hash. Shape-hash produces a separate fact kind and lives in its own phase. The P2 delta artifact will not carry shape-duplicate information.
- P5 formal drift (`check-canon.mjs`) — orthogonal to P2.

## 6. Invariants P2 must preserve from P1

- `canonical/*.md` files are NOT edited during P2 implementation unless an explicitly-motivated reviewer P0 requires it.
- P1 modules (`pre-write-*.mjs`, `_lib/pre-write-*.mjs`) are NOT refactored or semantically rewritten. P2 ADDS modules; existing P1 lookup / render / artifact behavior must remain unchanged and test-pinned (all P1 tests stay green after P2 lands). **The ONE allowed P1 touch** is a narrow append-only hook in `pre-write.mjs` (P2-0): it invokes `any-inventory.mjs` after the existing pipeline and records `advisory.preWrite.anyInventoryPath`. No pre-existing P1 code is deleted or reordered; the hook is a strict addition. Every existing P1 pinning test (name-first lookup order, CANONICAL DRIFT section exclusivity, claim-bearing citation, etc.) must remain green.
- `advisory.intent.plannedTypeEscapes` JSON order preserved by P1-1 is the contract P2 reads — must NOT mutate the input.
- **Capability parity check (authoritative P2-1 v3 source).** P2 checks the inventory artifacts themselves — `afterInventory.meta.supports.typeEscapes === true` AND `escapeKinds[]` byte-equal to `canonical/fact-model.md §3.9`. `preWriteAdvisory.capabilities` is a P1-0 symbols-graph projection and is NOT the type-escape capability source; P2-1 tests pin that classification behavior is identical whether `advisory.capabilities` is absent / null / `{}` / `{typeEscapes: false}`. If `afterInventory` is absent → `capabilityParity.status = 'missing'`, entries empty, `failures[]` records `after-inventory-missing`, no silent-new. If `afterInventory` present but capability surface unusable → `capabilityParity.status = 'mismatch'`, entries empty. If `beforeInventory` unusable (capability surface bad) → `baseline.status = 'missing'` (baseline problem, not a capability one); post-write still runs, degrades observed entries to `observed-unbaselined`. `meta.complete === false` is tolerated at the capability level and surfaces via `inventoryCompleteness` (the renderer gates the fully-clean summary on `afterComplete === true` AND `beforeComplete === true`). `anyContamination` belongs to P1's per-identity annotation path; P2's occurrence-level facts use `typeEscapes` capability, not `anyContamination`.
- `execFileSync` with argv arrays only for producer spawning. `execSync` shell strings remain forbidden (P1-3 rule).
- FP_BUDGET=0 corpus gate stays at 0. Any precision regression in P2 additions fails the gate.

## 7. Known risks + open questions

- **Baseline availability drives accuracy.** `silent-new` requires `beforeInventory`. P2-0 addresses this by extending `pre-write.mjs` to snapshot `any-inventory.pre.<invocationId>.json` alongside the advisory. When a caller runs post-write without a matching pre-write snapshot (ad-hoc invocation, CI path that skipped pre-write, lost artifact), the delta downgrades to `observed-unbaselined` — informational only, no Stage 3 trigger. A repo with 80 pre-existing `as any` does NOT get 80 silent-new alerts on first run. The baseline block in delta metadata records `status: missing` honestly.
- **Scan-range mismatch between before and after inventories.** P2-0's `meta.scope` / `includeTests` / `exclude` block makes the producer's scope explicit. P2-1 `computeDelta` refuses to compute per-occurrence delta when `beforeInventory.meta.scope !== afterInventory.meta.scope` or `includeTests` differs or `exclude[]` differs; it emits `[확인 불가, reason: scan-range mismatch]` at the repo level rather than fake precise numbers.
- **Scope-aware shadowing in escape detection.** Same gap that P1-1 documented — a local `any` inside a nested scope may over-claim. Document and defer.
- **Planned-match ambiguity (landed behavior, P2-1 v3).** Two or more observed occurrences can match one planned item. §3.1 matching rule picks the first by absent-from-before preference → code-shape tiebreak → deterministic sort. The **remainder passes through to baseline comparison** and classifies as `pre-existing` / `silent-new` / `observed-unbaselined` depending on the baseline lookup — NOT hard-labeled `silent-new`. Every remainder carries a `'ambiguous-planned-match'` diagnostic for audit. This protects the "no false silent-new claim" discipline: ambiguity is orthogonal to whether the occurrence is new. Future schema extension (`plannedTypeEscapes[].count`) can disambiguate without breaking compatibility.
- **Partner-export graph / FP-41-style over-escalation.** Separate sub-spec under P2. Drafted after P2 core ships; may turn out to be its own phase.
- **Response-time (Stage 3) enforcement — scope boundary.** P2 produces the delta and exposes `requiredAcknowledgements(delta)` to enumerate what MUST be acknowledged. P2 does NOT inspect or rewrite Claude's final response; Stage 3 enforcement (verifying the acknowledgment happened) is a caller responsibility — Claude's prompt layer, a CI gate, or a reviewer. The integration test in P2-2 verifies the delta carries the evidence, NOT that Stage 3 itself fires.
- **Line-number instability under formatters.** §3.1 matching uses `occurrenceKey` (hash over file + escapeKind + normalizedCodeShape + insideExportedIdentity), with `line` kept only for display/citation. A prettier run that reflows lines without changing code shape will NOT flip `pre-existing` to `silent-new`. If code shape itself changes (e.g. variable rename), that's a legitimate new occurrence.

## 8. Non-goals (P2 entire phase)

- No refactoring of pre-write code path.
- No rewrite of response-time enforcement logic — that is a Claude-side concern; P2 provides the evidence.
- No `check-canon.mjs` / formal drift (P5).
- No shape-hash (P4).
- No classification labels (P2 v2).
- No canonical/*.md edits unless explicitly motivated by a reviewer P0.
- No SARIF extension for post-write delta — P2 v1 stays Markdown + JSON; SARIF can follow in P2 v2 if needed.

## 9. Next session

**P2 complete as of 2026-04-20.** All three sub-phases landed:

- **P2-0** — `any-inventory.mjs` producer + canonical §3.9 amendment (`occurrenceKey`, `normalizedCodeShape`) + pre-write snapshot hook.
- **P2-1** — `_lib/post-write-delta.mjs` pure `computeDelta` with caller-injected `deltaInvocationId` + 6-label classification + `requiredAcknowledgements` + render + artifact + `post-write.mjs` CLI.
- **P2-2** — `audit-repo.mjs --post-write` orchestrator integration with `manifest.postWrite` summary fields + mutual exclusion with `--pre-write` + release-blocking integration test.

Implementation authority: `docs/history/phases/p2/p2-0.md`, `docs/history/phases/p2/p2-1.md v3`, `docs/history/phases/p2/p2-2.md v2`. This file is reconciled to match landed behavior (2026-04-20).

### What's next (outside P2 scope)

- P3 canon-draft generator (already specced in `docs/spec/SPEC-canon-generator.md v0.2.2`).
- P4 shape-hash producer — separate phase.
- P5 formal drift (`check-canon.mjs`).
- Partner-export graph / framework sentinel (FP-41 sub-spec).
- `--strict-post-write` CI-gating flag — convert `manifest.postWrite.ran === false` to exit 2.
- Classification labels in delta output (P2 v2).
- Scope-aware shadowing in identifier references (v1.10.0 gap).
