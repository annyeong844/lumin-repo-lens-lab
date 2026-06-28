# docs/history/phases/p5/session.md — Phase 5 canon drift detector (`check-canon.mjs`)

> **Phase:** P5 — canon drift detector. Follows P3 (canon draft generator). P4 shape-hash is a parallel/independent phase flagged but not started; P5 does not depend on P4 for initial scope (rename inference v1 is manual).
> **Role:** phase-level roadmap for the LAST piece of the stickiness loop — compare promoted `canonical/*.md` against fresh canon draft and report drift so the human/LLM can decide whether to re-promote or change the code.
> **Status:** phase plan, v2 (2026-04-21 reviewer P0 × 5 + P1 × 4 absorbed: exit-code matrix unified (standalone strict, orchestrator advisory + `--strict-check-canon`); parser strictness policy (strict on recognized schema, skip unknown as `unrecognized-canon-schema`); drift categories source-specific as canonical with generic `family` tag; missing-input semantics split per invocation mode; `canonical/canon-drift.md` kept as single file including parser contract; `canon-drift.json` minimal shape pinned in P5-0; never read prior `canonical-draft/*.md` files; `--canon-draft + --check-canon` independent; rename inference v1 = manual (confirmed)).
> **Last updated:** 2026-04-21

> **P-phase naming decision (2026-04-21):** `docs/history/phases/p3/session.md` §4.5 deferred the number assignment; `docs/spec/SPEC-canon-generator.md` §4.2 row "5" + fact-model §7 "P5 activates" + multiple "Phase 5" references across the spec already use **P5**. This phase takes that name. `Post-P3` is a colloquial synonym in earlier docs; P5 is the canonical handle going forward.

---

> **Implementation authority.** This file sequences P5 into reviewable sub-phases; it does NOT re-derive design for underlying canon shapes. When this file conflicts with parent design on per-source drift semantics (e.g., what counts as "drift" for topology vs naming), the relevant canonical file wins. Hierarchy:
>   1. `canonical/*.md` (spine — invariants win).
>   2. `canonical/classification-gates.md` §9/§10/§11/§12 (drift = deviation from canonical label rules).
>   3. `canonical/fact-model.md` §7 (drift skeleton — this phase activates and fleshes it out).
>   4. `docs/spec/SPEC-canon-generator.md` v0.2.2 (Phase 1-4 shapes; §5.4 "existing-canon observational header" is the handoff point).
>   5. `docs/history/phases/p3/p3-1.md` v2, `docs/history/phases/p3/p3-2.md` v2, `docs/history/phases/p3/p3-3.md` v3, `docs/history/phases/p3/p3-4.md` v2 (4 draft shapes — drift targets).
>   6. This file + `docs/history/phases/p5/p5-<N>.md` (P5 sub-phase sequencing).

## 1. Boot (read before starting)

| File | Why P5 needs it |
|---|---|
| `canonical/fact-model.md` §7 | Drift skeleton: *"a canonical declaration exists (promoted to `canonical/*.md`) that disagrees with a current fact of the same kind."* P5 activates this skeleton and gives formal drift semantics per fact kind. |
| `canonical/classification-gates.md` §9 / §10 / §11 / §12 | The 4 label sets P3 emits. A drift-aware consumer must recognize every label; drift includes label changes (e.g., `single-owner-strong` → `zero-internal-fan-in` across runs). |
| `canonical/identity-and-alias.md` §2 / §6 | Identity format `ownerFile::exportedName`. The canon parser MUST extract this same identity from Markdown tables. |
| `canonical/invariants.md` | Canon promotion is a human decision; P5 NEVER writes to `canonical/`. P5 is a REPORT, not a mutator. |
| `docs/spec/SPEC-canon-generator.md` §5.4 + §6 | Phase 1 output format. P5's canon parser reverses the renderer: parses the emitted Markdown table back into structured records keyed on identity. |
| `docs/history/phases/p3/p3-1.md` v2 §4.1, `docs/history/phases/p3/p3-2.md` v2 §4.1, `docs/history/phases/p3/p3-3.md` v3 §4.1, `docs/history/phases/p3/p3-4.md` v2 §4.1 | Per-source canon draft Markdown shape. Each sub-phase's `renderTypeOwnership` / `renderHelperRegistry` / `renderTopology` / `renderNaming` is what we parse back. |
| `_lib/canon-draft-{utils,types,helpers,topology,naming}.mjs` | The renderers whose output P5 parses. If a renderer changes shape, the P5 parser must change in the same commit. |

Not required for P5 v1:
- `p4` (shape-hash) — flagged but not started. v1 rename inference is manual; shape-hash would enable auto-match in v2+.
- `canonical/canon-drift.md` — the fact-model §7 skeleton mentions this file; P5 v1 creates it (or promotes the skeleton in-place).
- `_lib/extract-ts.mjs` — P5 doesn't do AST work. Canon-draft already did the AST pass; P5 reads its output.
- Runtime / test data.

## 2. Optimization target

The stickiness loop from `docs/spec/SPEC-canon-generator.md` §1.1:

1. Session N: skill runs → `canonical-draft/*.md` generated from AST (**P3 delivers**).
2. Human / LLM fills intent fields; selected entries promoted to `canonical/*.md` (**manual step — P3's §5.4 existing-canon header points here**).
3. Session N+1: pre-write gate reads `canonical/` before Claude writes → Claude does not re-invent helpers, duplicate types, or misname shared things (**P1 delivers**).
4. **Skill runs observe current AST → compare to promoted canon → report drift (P5 delivers).** ← this session.

P5 closes the loop. Without it, promoted canon becomes stale silently: code evolves, canon stays fixed, and pre-write advisories read a drifted canon that no longer matches reality. P5 surfaces that drift on every audit run so the reviewer knows when re-promotion is needed.

**Scope qualifier.** P5 is a **reporter**, not a **promoter**. Every drift finding is an OBSERVATION that a reviewer (human or LLM) acts on — auto-promotion from fresh draft to canon is OUT OF SCOPE by the same canon-promotion-is-manual principle P3 honors.

## 3. Scope

### 3.1 In (P5 entire phase)

- **`check-canon.mjs`** CLI entry at repo root. Flags: `--root`, `--output`, `--source <type-ownership|helper-registry|topology|naming|all>`, `--canon-dir` (default `<root>/canonical/`), scan-range flags forwarded. **No `--draft-dir`** — v1 NEVER reads existing `canonical-draft/*.md` files (P1-7); fresh records come from the same collector P3 uses, called in-memory via current scan-range flags.
- **Canon Markdown parser** — reverses `docs/spec/SPEC-canon-generator.md` §6 + per-source §4.1 output formats. One parser per source (4 total) since shapes differ. **Parser strictness policy (P0-2)**: recognized generated schema → strict parse (unknown column / missing required column / unknown label → `canon-parse-error` diagnostic); unrecognized-schema / hand-edited free-form → source skipped with `[확인 불가, reason: unrecognized-canon-schema]`. Lenient partial parse is explicitly REJECTED — a drift detector that reads half the rows and reports "no drift" is the worst possible failure mode.
- **Drift engine per source** — structured diff between promoted canon records and fresh records.
- **Drift categories — source-specific canonical enum (P0-3)**. Each drift fact carries `kind` + `category` (source-specific enum) + `family` (generic tag for grouping). Enumerated in `canonical/canon-drift.md` §3 (landed in P5-0):
  - `type-drift`: `identity-added`, `identity-removed`, `label-changed`, `owner-changed`.
  - `helper-drift`: `helper-added`, `helper-removed`, `label-changed`, `contamination-changed`, `fan-in-tier-changed`.
  - `topology-drift`: `submodule-added`, `submodule-removed`, `scc-status-changed`, `oversize-changed`, `cross-edge-added`, `cross-edge-removed`.
  - `naming-drift`: `cohort-added`, `cohort-removed`, `cohort-convention-shifted`, `new-outlier-introduced`, `outlier-resolved`.
  - Generic `family` values: `added` / `removed` / `label-changed` / `structural-status-changed` / `content-shifted`.
- **Rename inference** — **v1 = manual (confirmed, P1-9)**. Report adds+removes separately; reviewer correlates. No Levenshtein. P4 shape-hash (if landed later) unlocks v2+ auto-match.
- **Drift report output** — `<output>/canon-drift.<source>.md` Markdown per source + `<output>/canon-drift.json` structured artifact (single JSON with `perSource` map, not one JSON per source). Minimal JSON shape pinned in P5-0 (see §4.1).
- **Exit codes — standalone `check-canon.mjs` (strict, Unix-style)**:
  - `0` — completed, no drift detected.
  - `1` — completed, drift detected.
  - `2` — required input missing OR parse contract failure. Includes: promoted canon file absent for single-source invocation, `canonical/` directory absent entirely, unrecognized canon schema, fresh draft generator failure.
  - Semantics for `--source all` (P0-4): if NO requested source has promoted canon → exit 2 (nothing to check); if SOME sources have canon and SOME don't, exit reflects ONLY the sources actually checked (missing sources surface as `perSource[source].status = "skipped-missing-canon"` but do not degrade exit code).
- **`audit-repo.mjs --check-canon`** opt-in integration. **Advisory by default (P0-1)**: drift → orchestrator exit 0, `manifest.checkCanon.driftCounts` populated. `--strict-check-canon` escalates drift to exit 1 and parse-contract-failure to exit 2. Lands in P5-4 (not P5-1).
- **`audit-repo.mjs --canon-draft --check-canon` coexistence (P1-8)**: allowed. `--canon-draft` writes draft markdown; `--check-canon` regenerates fresh records in-memory independently. `--check-canon` NEVER consumes files just written by `--canon-draft` — reproducibility requires direct-collector path, not parser-of-renderer-output.
- **Conformance tests** — parser shape + drift-engine-per-category + empty-canon + unknown-schema + `--source all` mixed-presence cases.
- **Canonical `canon-drift.md`** edit — promotes fact-model §7 skeleton into a dedicated canonical file. **Single file containing drift categories + parser contract (P0-5)**; no split into `canon-parser.md` in v1. Structure: §1 Purpose / §2 Drift fact kinds / §3 Drift categories (source-specific) / §4 Identity contract / §5 Parser contract / §6 JSON artifact shape / §7 Non-goals.
- **SKILL.md** update — `## Canon drift mode (Stage 4)` subsection.

### 3.2 Out (deferred or out of scope)

- Auto-promotion from draft to canon (canon promotion stays manual by design).
- LLM-based rename inference (out of skill scope; rename is reviewer's call).
- Cross-source drift (e.g., "type X was renamed to Y and the renaming appears in both type-ownership.md and helper-registry.md") — each source is independent in v1.
- Python / Go canon drift — same Phase 6+ deferral as upstream P3.
- Editing promoted `canonical/*.md`. P5 NEVER writes to `canonical/`.
- Historical drift (drift across git commits / prior runs) — each run compares against CURRENT promoted canon only.
- Drift severity ranking (beyond binary "drift detected yes/no"). Severity lives in reviewer judgment.

## 4. Sub-phase breakdown

Analogous to P3's P3-1/P3-2/P3-3/P3-4 pattern. **P5 has five sub-phases**:

```
P5-0  Canonical canon-drift.md + parser shape contract — prerequisite edit
P5-1  type-ownership drift    — first source, proves the pattern
P5-2  helper-registry drift   — second source, reuses parser infra from P5-1
P5-3  topology drift          — third source, adds SCC / oversize drift semantics
P5-4  naming drift            — fourth source + audit-repo --check-canon orchestrator
```

**Why P5-0 exists.** P3 had a `canonical/classification-gates.md §11/§12` prerequisite per sub-phase. P5 consolidates the canonical-edit dependency into one up-front step (`canonical/canon-drift.md` v1 — formal drift categories, diff identity contract, parser contract) so P5-1/2/3/4 implementation sessions are pure consumers of a finalized canonical surface.

**Why flat per-source (P5-1..P5-4) instead of bundled.** Each source has different drift semantics:
- types: identity add/remove + owner-file changes + label changes.
- helpers: same as types + contamination drift + fan-in tier changes.
- topology: submodule add/remove + SCC status flips + oversize list changes + cross-edge count deltas.
- naming: cohort-dominant convention flips + outlier list changes.

Bundled spec would be dense and the 4 drift semantics would fight for primacy. Per-source specs let each drift story be told cleanly.

### 4.1 P5-0 — `canonical/canon-drift.md` v1 (single file)

Canonical edit — not an implementation commit. Single file with 7 sections:

**§1 Purpose** — stickiness loop step 4. Skill emits drift evidence; human/LLM decides re-promotion. Never writes to `canonical/`.

**§2 Drift fact kinds** (4):
- `type-drift` — consumes `canonical/type-ownership.md` + fresh `collectTypeIdentities` records.
- `helper-drift` — consumes `canonical/helper-registry.md` + fresh `collectHelperIdentities`.
- `topology-drift` — consumes `canonical/topology.md` + fresh `collectTopologyStructure`.
- `naming-drift` — consumes `canonical/naming.md` + fresh `collectNamingCohorts`.

**§3 Drift categories** (source-specific, canonical enum per P0-3):

```
type-drift:
  identity-added    (family: added)
  identity-removed  (family: removed)
  label-changed     (family: label-changed)
  owner-changed     (family: structural-status-changed)

helper-drift:
  helper-added           (family: added)
  helper-removed         (family: removed)
  label-changed          (family: label-changed)
  contamination-changed  (family: content-shifted)
  fan-in-tier-changed    (family: label-changed)

topology-drift:
  submodule-added     (family: added)
  submodule-removed   (family: removed)
  scc-status-changed  (family: structural-status-changed)
  oversize-changed    (family: content-shifted)
  cross-edge-added    (family: added)
  cross-edge-removed  (family: removed)

naming-drift:
  cohort-added               (family: added)
  cohort-removed             (family: removed)
  cohort-convention-shifted  (family: label-changed)
  new-outlier-introduced     (family: content-shifted)
  outlier-resolved           (family: content-shifted)
```

Total 20 canonical categories + 5 `family` values. `family` is for grouping/rollup; `category` is the primary drift identifier.

**§4 Identity contract** — drift records key on the same identity format each source's canon uses:
- `type-drift` / `helper-drift` → `ownerFile::exportedName`.
- `topology-drift` submodule categories → `<submodule>` path.
- `topology-drift` cross-edge categories → `<from-submodule> → <to-submodule>` edge label.
- `topology-drift` oversize → `<ownerFile>`.
- `naming-drift` cohort categories → `<submodule>` (file cohort) or `<submodule>::<kind>` (symbol cohort).
- `naming-drift` outlier categories → per-item identity (`<ownerFile>` or `<ownerFile>::<exportedName>`).

**§5 Parser contract** — each P3 renderer commits to a stable table shape:
- `type-ownership.md` — columns `Name | Identity | Owner | Fan-in | Status | Tags`. Parser extracts `identity`, `status` (label), `owner` (line ref). P3-1 renderer MUST NOT drop/rename columns without updating this contract.
- `helper-registry.md` — columns `Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal`.
- `topology.md` — §1 Submodule inventory (`Submodule | Files | LOC | In-edges | Out-edges | SCC | Status`); §3 cycle listing; §4 oversize table.
- `naming.md` — §1 File cohort table; §2 Symbol cohort table; §3 Outliers (optional).

Parser strictness: recognized marker row + recognized columns → strict parse. Unknown column / extra column / missing required column → `canon-parse-error` diagnostic. Unrecognized first row shape → source-level `unrecognized-canon-schema` skip.

**§6 JSON artifact shape** — `<output>/canon-drift.json` minimal schema (P1-6):

```json
{
  "meta": {
    "tool": "check-canon.mjs",
    "generated": "ISO-8601",
    "root": "/abs/path",
    "canonDir": "/abs/path/canonical",
    "scope": "TS/JS production files | including tests",
    "strict": false
  },
  "summary": {
    "sourcesRequested": 4,
    "sourcesChecked": 2,
    "sourcesSkipped": 2,
    "driftCount": 3
  },
  "perSource": {
    "type-ownership": {
      "status": "drift" | "clean" | "skipped-missing-canon" | "skipped-unrecognized-schema" | "parse-error",
      "driftCount": 2,
      "reportPath": "<output>/canon-drift.type-ownership.md",
      "diagnostics": []
    }
  },
  "drifts": [
    {
      "kind": "type-drift",
      "category": "owner-changed",
      "family": "structural-status-changed",
      "identity": "src/foo.ts::User",
      "canon": { "file": "canonical/type-ownership.md", "line": 42, "label": "single-owner-strong", "owner": "src/foo.ts:14" },
      "fresh": { "label": "single-owner-strong", "owner": "src/types/user.ts:8" },
      "confidence": "high"
    }
  ]
}
```

P5-1 onward MUST use this shape. Fields MAY be added additively; existing fields MUST NOT be removed or renamed without a canonical edit.

**§7 Non-goals** — auto-promotion; LLM rename inference; cross-source drift correlation; historical git-history drift; drift severity tiering beyond binary detection; editing `canonical/*.md`.

**Exit criteria:** `canon-drift.md` passes canonical-spine review (no label-set conflicts with §9/§10/§11/§12; identity-and-alias §2 consistency preserved). `test-classification-gates.mjs` extension pins drift category enum + family tag set.

### 4.2 P5-1 — `type-ownership` drift

Deliverable: `check-canon.mjs --source type-ownership` emits `canon-drift.type-ownership.md` + `canon-drift.json` entry.

- Parser: extract identity + label + owner-line from `canonical/type-ownership.md` table (if it exists).
- Collect fresh draft records via `collectTypeIdentities` + `renderTypeOwnership` pipeline (reuse P3-1 code, don't re-implement).
- Diff engine: set difference + pairwise label compare per identity.
- Drift categories: `identity-added`, `identity-removed`, `label-changed` (e.g., canon says `single-owner-strong`, fresh says `zero-internal-fan-in`), `owner-changed` (same exportedName, different ownerFile).
- Output: table per category + summary counts.

Exit criteria: P5-1 shipped + tested on fixture where canon diverges from fresh by known deltas.

### 4.3 P5-2 — `helper-registry` drift

Same pattern as P5-1 but for helpers. Additional categories:
- `contamination-changed` (`severely-any-contaminated-helper` ↔ `central-helper` flips).
- `fan-in-tier-changed` (`shared-helper` → `central-helper` once fan-in crosses 3).

Parser for helper-registry tables. Reuses P5-0 parser contract.

### 4.4 P5-3 — `topology` drift

Submodule-level drift. Different identity shape (submodule path), so the parser + diff are distinct from types/helpers:
- `submodule-added` / `submodule-removed`.
- `scc-status-changed` — acyclic → cyclic or vice versa (load-bearing: cyclic introduction is a canon invariant violation).
- `oversize-changed` — file crossed LOC threshold.
- `cross-edge-count-delta` — optional v1: surface only when a brand-new cross-submodule edge type appears, not per-count delta.

### 4.5 P5-4 — `naming` drift + `audit-repo.mjs --check-canon` integration

Two deliverables:

1. **Naming drift** — cohort-level diff. Categories:
   - `cohort-convention-shifted` — `camelCase-dominant` → `mixed-convention` or vice versa.
   - `new-outlier-introduced` / `outlier-resolved`.
   - `cohort-added` / `cohort-removed` (new submodule appeared).

2. **`audit-repo.mjs --check-canon` orchestrator** — parallel to `--canon-draft`:
   - New flag `--check-canon` on `audit-repo.mjs`. Not in default profiles.
   - `manifest.checkCanon = {requested, ran, reason?, perSource: {<source>: {ran, exitCode, driftCount, reportPath}}}`.
   - Runs `check-canon.mjs` per source or scoped via `--sources`.

## 5. Integration

### 5.1 `audit-repo.mjs --check-canon` hook (lands in P5-4)

```
audit-repo.mjs --check-canon [--sources type-ownership,helper-registry,topology,naming] [--strict-check-canon]
```

Parallels `--canon-draft` exactly. Opt-in, not in default profiles. `manifest.checkCanon` block mirrors `manifest.canonDraft` shape (`requested` / `ran` / `requestedSources` / `perSource` / `driftCounts`).

**Exit code contract (P0-1 confirmed):**
- **Standalone `check-canon.mjs`**: Unix-style strict (0 clean / 1 drift / 2 missing-input-or-parse-error). CI running `check-canon.mjs` directly gets non-zero on drift.
- **`audit-repo.mjs --check-canon` default**: ADVISORY. Drift → orchestrator exit 0; `manifest.checkCanon.driftCounts` populated so CI/caller reads the structured signal. Parse-contract failure in one source → other sources still run; final exit 0 if at least one ran cleanly.
- **`audit-repo.mjs --check-canon --strict-check-canon`**: escalates. Drift → exit 1. Parse-contract-failure OR missing-input across all requested sources → exit 2. Matches `--strict-post-write` pattern semantically.

**`--canon-draft + --check-canon` coexistence (P1-8 confirmed):** allowed. `--canon-draft` writes draft markdown to `<root>/canonical-draft/`; `--check-canon` regenerates fresh records in-memory (never consumes just-written draft files — reproducibility requires direct-collector path). Both blocks populate independently in the manifest.

**Lifecycle flag matrix update** (extend SKILL.md truth table from P3-4-b):

| --pre-write | --post-write | --canon-draft | --check-canon | Allowed | Exit |
|:-:|:-:|:-:|:-:|:-:|:-:|
| –  | –  | –  | Y  | Y | 0 / 1 / 2 per contract above |
| –  | –  | Y  | Y  | Y | 0 / 1 / 2 (canon-draft + check-canon independent) |
| Y  | –  | –  | Y  | Y | pre-write + check-canon coexist |
| –  | Y  | –  | Y  | Y | post-write + check-canon coexist |
| Y  | Y  | any | any | **N** | 2 (pre/post mutex still applies) |

### 5.2 SKILL.md

New `## Canon drift mode (Stage 4)` subsection after `## Canon draft mode`. Links to `docs/history/phases/p5/session.md` + `canonical/canon-drift.md` + per-source sub-phase docs.

### 5.3 `generate-canon-draft.mjs` vs `check-canon.mjs` boundary

These are **two separate CLIs** with orthogonal jobs:
- `generate-canon-draft.mjs` — reads AST, writes `canonical-draft/*.md`. Never touches `canonical/`.
- `check-canon.mjs` — reads `canonical/*.md` + re-runs draft collectors internally, writes `canon-drift.*.md` to `<output>/`.

P5 must NOT merge these. Separate identity, separate exit-code semantics, separate testing surface.

## 6. Invariants P5 must preserve from P1/P2/P3

- P1/P2/P3 modules unchanged. P5 is a CONSUMER of canon-draft collectors + renderers; never modifies them.
- `canonical/*.md` unchanged by P5. P5 produces `canon-drift.*` reports, never edits canon.
- `execFileSync` argv-array rule for any producer spawning (P1-3 shell-safety).
- FP_BUDGET=0 corpus gate stays at 0. P5 doesn't enter classify-dead-exports path — coupling is minimal.
- `canonical/classification-gates.md` is the single source of truth for label sets. P5 parser MUST accept ONLY labels from §9/§10/§11/§12. Any unknown label in canon → `canon-parse-error` diagnostic, NOT silent skip.
- Identity discipline: every drift record keys on `ownerFile::exportedName` (types/helpers) or submodule path (topology) or cohort ID (naming). No `typeName`/`helperName`/raw file path keying.
- No auto-promotion from draft to canon — violates the manual-promotion invariant from P3.

## 7. Known risks + open questions

### Closed in v2 (kept for trace)

- ~~Exit code semantics~~ — **closed P0-1**: standalone strict (0/1/2); orchestrator advisory default + `--strict-check-canon`.
- ~~Parser strictness~~ — **closed P0-2**: recognized schema strict; unknown → `unrecognized-canon-schema` skip.
- ~~Drift category enum~~ — **closed P0-3**: source-specific primary (20 categories) + `family` tag (5 values).
- ~~Missing input semantics~~ — **closed P0-4**: single-source → exit 2 on missing canon; `--source all` → per-source `skipped-missing-canon`, exit reflects checked sources.
- ~~`canon-drift.md` vs `canon-parser.md` split~~ — **closed P0-5**: single file in v1.
- ~~`canon-drift.json` artifact shape~~ — **closed P1-6**: minimal shape pinned in P5-0 §6.
- ~~`check-canon` reads prior draft files~~ — **closed P1-7**: NEVER reads `canonical-draft/*.md`; always fresh collector in-memory.
- ~~`--canon-draft + --check-canon` coexistence~~ — **closed P1-8**: independent, never cross-consume.
- ~~Rename inference v1 scope~~ — **closed P1-9**: manual (no Levenshtein, no P4 shape-hash dep).

### Still open / risks to track

- **Drift severity ranking.** v1 = binary "drift detected yes/no". v2+ could rank "critical" (label change on canonical hub) vs "minor" (one new outlier). Out of v1 scope.
- **Monorepo workspace canon.** If `canonical/type-ownership.md` lives per-workspace in a monorepo, does `check-canon` aggregate or run per-workspace? v1 = single `<root>/canonical/` only; per-workspace is Phase 6+.
- **Canon edit as drift resolution.** Reviewer may choose to edit `canonical/*.md` directly rather than re-promoting from draft. P5 supports this flow (re-run → re-detect) but does NOT automate it.
- **`canonical/canon-drift.md` formal-spec dependency.** P5-0 MUST land before P5-1 implementation. If reviewer revises drift categories in P5-0 review, all P5-1..P5-4 parsers shift. Mitigation: `test-classification-gates.mjs` extension pins drift category enum at P5-0 landing.
- **Canon table header drift.** If a future P3 renderer patch changes column headers, the parser contract (§5 of `canon-drift.md`) breaks. Mitigation: parser contract + renderer are DESIGN-COUPLED — any renderer commit touching column headers MUST update parser contract + version P5-0 in the same commit. Source-grep pin in `test-classification-gates.mjs` could enforce this (open design decision for P5-0).
- **Fresh collector cost.** `check-canon --source all` runs all 4 collectors fresh (AST pass + topology.json read + triage.json read). On large repos this is ~5-15 seconds cumulative. Acceptable for dev flow / CI gate; not hot-path.

## 8. Non-goals (P5 entire phase)

- No auto-promotion from `canonical-draft/` to `canonical/`.
- No Python / Go canon drift (Phase 6+).
- No LLM rename inference.
- No cross-source drift correlation.
- No historical / git-history drift (compares against CURRENT promoted canon only).
- No drift severity tiering beyond binary detection.
- No editing of `canonical/*.md` files.
- No P1 / P2 / P3 module modifications.
- No per-workspace canon in monorepos (Phase 6+).

## 9. Next session

Proceed to `docs/history/phases/p5/p5-0.md` — the canonical edit + parser contract session plan. Sequence:

```
Step 0 — Reviewer approval of drift categories in §3 of canon-drift.md
         (this session's spec flags categories; canonical edit commits them)

Step 1 — canonical/canon-drift.md v1 written
         Drift fact kinds (4) + categories (per kind) + identity contract
         + parser shape contract + non-goals.

Step 2 — test-classification-gates.mjs extension
         Parse canon-drift.md §3 → enumerate drift categories
         Assert _lib/check-canon-*.mjs (when implemented) matches.

Step 3 — Reviewer closes P5-0; unlocks P5-1 implementation session.
```

After P5-0 lands, the sub-phase order is:
```
P5-1 type-ownership drift
P5-2 helper-registry drift
P5-3 topology drift
P5-4 naming drift + audit-repo --check-canon integration
```

Each sub-phase follows the P3 six-step discipline (test-first RED → implementation GREEN → integration → dogfood → SKILL.md + update-test-doc).

## 10. Handoff to P4 / later

- **P4 shape-hash** — independent phase, not-yet-started. If P4 lands later, P5's rename inference can upgrade from manual-reviewer to auto-match via shape-hash. v1 P5 doesn't require P4.
- **Phase 6+** — Python / Go canon + cross-workspace drift.
- **Canon evolution tracking** — when `canonical/` changes across git commits, a separate tool could track the drift timeline. Out of P5.

## 11. Reviewer review hooks

### v1 → v2 closed (2026-04-21)

Reviewer P0 × 5 + P1 × 4 absorbed. See §7 "Closed in v2" for the 9-item list.

### v2 remaining open (small; P5-0 can proceed without blocking on these)

1. **`test-classification-gates.mjs` extension scope** — add drift-category mirror assertions to the shared drift test OR create `test-canon-drift-categories.mjs` as a sibling. P3 pattern was shared drift-test (single `test-classification-gates.mjs` covering all 4 P3 sub-phases); natural default for P5 = follow same pattern. Flag if reviewer prefers sibling.
2. **Parser-contract + renderer coupling enforcement** — §7 "canon table header drift" risk. Options: (a) source-grep pin in drift-test that parses renderer code + canon-drift.md §5 and diffs column lists; (b) integration test that renders + re-parses + asserts identity round-trip; (c) trust design-coupled commit discipline. Reviewer pick during P5-0.
3. **Output Markdown shape per source** — `canon-drift.<source>.md` (one MD per source + one aggregate JSON) is v1 default. Reviewer can collapse to single `canon-drift.md` if preferred.

Reviewer returns P2/additional list. If no more changes, P5-0 session begins.
