# Canon Draft Generator — Spec v0.2.2

> **Role:** design document for `generate-canon-draft.mjs` and the shared `_lib/canon-draft.mjs`. **Canon observation generator**, not a canon author — emits drafts that a human or LLM finalizes into real canon.
> **Owner:** this file. **This is a P-phase (P3) implementation spec.** Invariants live in `canonical/*.md` (the spine); this spec is subordinate and must not re-state or contradict them.
> **Status:** v0.2.2 draft — awaiting user approval.
> **Last updated:** 2026-04-20
> **Supersedes:** v0.2.1 (third reviewer pass; 3 P0 + 3 P1 items absorbed — see §14 v0.2.2 entries). v0.2.1 supersedes v0.2 (see §14 v0.2.1 entries). v0.2 supersedes v0.1 (see §14 v0.2 entries). v0.1 supersedes v0 (see §14 v0.1 entries).

---

## 1. Purpose

Vibe-coders (non-developers who delegate coding entirely to LLMs) accumulate code-rot across sessions because each LLM session lacks the implicit invariants of the codebase: which file owns which type, which helper is canonical, which module may depend on which. No single LLM session sees enough history to enforce consistency.

**Canonical docs** (as in `ouroboros-ts/canonical/*` and `suyeon-daemon-followup/docs/current/reference/*`) solve this by declaring invariants explicitly. Asking a vibe-coder to write canon by hand is not viable. This generator inverts the direction:

> Read the code's AST, derive **observed** ownership / shape / topology, emit a **draft of observations** that the LLM-or-human finalizes into canon by filling in intent and promoting selectively.

This is an **evidence generator for canon drafting**, not a canon generator. The distinction matters:

- The tool NEVER claims "this IS the canon". It only says "here is what I observed".
- The tool NEVER writes to `canonical/`. It writes to `canonical-draft/`.
- Status labels surface OBSERVATIONS (`zero-internal-fan-in`, `DUPLICATE_STRONG`), not verdicts (`unused`, `bad`).
- Promotion from draft → canon is a manual decision that preserves the human's authority over intent.

### 1.1 The stickiness loop

1. Session N: LLM runs the skill → `canonical-draft/type-ownership.md` generated from AST.
2. LLM fills `purpose / intent` fields on the draft (structure is pre-filled). Selected entries promoted to `canonical/`.
3. Session N+1: LLM reads `canonical/` before writing code → does not re-invent helpers, does not duplicate types.
4. Each skill run observes current AST → compares to promoted canon → reports drift (Phase 5 — separate spec).

The vibe-coder sees: Claude's sessions stay consistent over time. They don't need to read the canon themselves.

---

## 2. Scope

### 2.1 In scope (Phase 1)

- TS / TSX / JS / JSX / MJS / CJS source files.
- **Exported top-level type declarations only** (`export type X = ...`, `export interface X { ... }`, `export enum X { ... }`, `export namespace X { ... }` where the namespace declares types). Local / private types within a module are **explicitly out of Phase 1** — they are not canon-worthy by default. (TypeScript syntax is `export namespace X`, not `export type namespace X`; the v0.1 wording was wrong.)
- Re-export chain resolution (barrel files, `export { X } from './y'`) via existing `symbols.json.reExportsByFile` infrastructure, so fan-in is measured at the defining owner, not the barrel.
- Generating Markdown drafts under `canonical-draft/` (location configurable, never writes to `canonical/`).
- Read-only analysis of source + artifact inputs. No runtime probing. No LLM calls.

### 2.2 Out of scope

- Python / Go canon (TS/JS infra is mature here; other languages are Phase 6+ decisions).
- Business-logic documentation (what a function MEANS domain-wise). The generator emits structural observations; semantic intent is the LLM/human finalize step.
- Auto-promotion from `canonical-draft/` to `canonical/`. Promotion is manual by design.
- Drift detection against existing `canonical/*.md` — that is `check-canon.mjs`, a separate spec (§9 / §10).
- Local / private type tracking (even though the Rust JS/TS fact index surfaces
  some of these, they do not go in the Phase 1 draft).

---

## 3. Inputs

### 3.1 Primary CLI

- `--root <repo>` — repository root.
- `--output <dir>` — artifact output directory (for pipeline artifacts).
- `--canon-output <dir>` — draft markdown directory.
  - Default in standalone mode: `<root>/canonical-draft/`.
  - Default when invoked through `audit-repo.mjs`: `<output>/canonical-draft/` (avoids surprising working-tree writes in CI).
  - Overridable.
- `--source <target>` — which draft to generate. Phase 1: `type-ownership`. Later: `helper-registry`, `topology`, `naming`, `all`.

### 3.2 Consumed artifacts (all optional)

| Artifact | Used for | Fallback if absent |
|---|---|---|
| `symbols.json.reExportsByFile` | re-export chain resolution | degrade: treat barrels as opaque (flag in metadata) |
| `symbols.json.topSymbolFanIn` | quick cross-check on high-fan-in types | degrade: compute per-identity fan-in from fresh pass |
| `call-graph.json` | concern clustering in Phase 2+ | not needed for Phase 1 |
| `checklist-facts.json` | severity anchor for `zero-internal-fan-in` items | compute inline |
| `triage.json` | workspace package names (for identity labels) | degrade: use directory name |

### 3.3 Shared extractor

Fresh helper and naming inventories use the Rust-owned
`js-ts-extract-artifact` command. `_lib/extract-ts.mjs` is only a compatibility
adapter over that command; it does not parse source text or own a fallback
classifier. Canon CLIs batch the complete scoped file set before entering the
existing synchronous aggregation helpers, so `--source all` can share one
current-run extraction index instead of parsing the repository twice.

---

## 4. Outputs

### 4.1 Phase 1 deliverable

One file: `<canon-output>/type-ownership.md`.

### 4.2 Phased roadmap

| Phase | File | Complexity | Dependencies |
|---|---|---|---|
| **1** | `canonical-draft/type-ownership.md` | Low | Rust JS/TS facts or `symbols.json.reExportsByFile` |
| 2 | `canonical-draft/helper-registry.md` | Medium | Phase 1 + `call-graph.json` |
| 3 | `canonical-draft/topology.md` | Medium | `topology.json` + `triage.json` |
| 4 | `canonical-draft/naming.md` | Low-Medium | fresh AST pass |
| 5 | `check-canon.mjs` — drift detection (separate spec) | Medium-High | SPEC all canon shapes first |

Each phase is a **reviewable release gate**. Phase 2 starts only after Phase 1 ships and is calibrated. Phase 5 gets its own spec because parsing markdown canon + owner-mismatch detection + rename inference is a different problem class.

---

## 5. Phase 1 algorithm — `type-ownership.md`

### 5.1 Input collection

1. Run `collectFiles(root, { includeTests: cli.includeTests, exclude: cli.exclude })`.
2. Build one scoped Rust JS/TS fact index and read each file's definitions and uses from it.
3. Load `symbols.json.reExportsByFile` if present.
4. Keep only **exported** top-level defs where `kind ∈ {TSInterfaceDeclaration, TSTypeAliasDeclaration, TSEnumDeclaration, TSModuleDeclaration}`. The Rust fact owner already filters to exports; re-confirm this in a behavior test.

### 5.2 Identity-keyed aggregation

**P0-2 fix: fan-in is measured by identity, not by name.** Identity = `ownerFile::exportedName`.

Build three maps:

```
typeDefsByIdentity:
  Map<"owner/file.ts::TypeName", { file, kind, line, exportedName }>

typeNameIndex:
  Map<"TypeName", Array<identity>>       // used for duplicate detection

typeUsesByIdentity:
  Map<identity, { directConsumers: Set<file>, reExportedThrough: Set<file> }>
```

Resolving a `use` (import) to an identity:

1. Extract an `importRecord` from each Rust-owned `uses` row for the consumer file:

   ```
   { fromSpec, importedName, localName, kind, typeOnly }
   ```

   Alias fidelity is required — `importedName` is source-side, `localName` is consumer-side. Identity resolution MUST use `importedName`, never `localName`. See `canonical/identity-and-alias.md` §4.

2. Identity resolution is delegated to `canonical/identity-and-alias.md` §6 (`resolveIdentity(consumerFile, importRecord)`). The canonical algorithm handles: mixed-file re-export chains (files that own some exports and re-export others), named re-export alias hops (`export { User as PublicUser }`), star re-exports (`export *` — ambiguity-preserving, emits `[확인 불가]` on multi-source name collisions), and the 8-hop depth cap.

3. The canonical algorithm returns `{ identity: (resolvedFile, nameAtBarrier), reExportedThrough }`. The terminal identity is `resolvedFile::nameAtBarrier` — never `resolvedFile::localName` (which would re-introduce the alias-collapse bug).

4. `typeUsesByIdentity[identity].directConsumers.add(consumerFile)`.
5. `typeUsesByIdentity[identity].reExportedThrough` is the union of all `reExportedThrough` sets returned by §6 across all uses that land on this identity.

6. If §6 returns `[확인 불가]` (ambiguous star, depth exceeded, unresolved spec), DO NOT pin the use to any identity. The use contributes to `resolverConfidence` downgrade signals, not to fan-in. See `canonical/fact-model.md` §3.8 + `canonical/identity-and-alias.md` §9.

**Fan-in on identity, not name** means `apps/admin/types.ts::User` and `apps/blog/types.ts::User` are two distinct identities with separate consumer sets, never mixed.

### 5.3 Status assignment

**Classification table and label set are defined by `canonical/classification-gates.md`. This spec MUST NOT duplicate them.** Any table here would be a second source of truth and would drift; the canonical file owns the rules (including precedence, Rule 0 contamination handling, Rule 1 fan-in thresholds, low-info name behavior, and single-identity ordering).

Implementation consumes two pure functions exported by `_lib/canon-draft.mjs`:

```
classifyTypeNameGroup(name, identities, fanInByIdentity, contaminationByIdentity) → { label, marker, anyMembers, severeAnyMembers, semanticConfidence, tags }
classifySingleIdentity(identity, fanIn, kind, contamination) → { label, marker }
```

Both functions are direct implementations of `canonical/classification-gates.md` §2 and §4. They must pass a conformance test that enumerates the ordering cases from the canonical file (`tests/test-classification-gates.mjs`).

`LOW_INFO_NAMES` is **normatively defined** in `canonical/classification-gates.md` §3. Because the canonical file is Markdown and JS cannot import Markdown, `_lib/canon-draft.mjs` MIRRORS the list as a code constant. The mirror is drift-checked by `tests/test-classification-gates.mjs`, which parses the canonical Markdown at test time and asserts the code constant matches the canonical list exactly (same names, same order). If the mirror drifts, the test fails.

Treat the code constant as a derivative, not a source: any edit to the list must happen in the canonical file first, then be propagated to the mirror. Phase 2+ per-repo override config is on the roadmap but lives in the canonical file's evolution, not this spec.

If an implementation outputs a label not in `canonical/classification-gates.md` §9, it is wrong. If its ordering disagrees with the canonical file, the canonical file wins.

### 5.4 Existing canon behavior (non-drift)

**P1-8 fix: Phase 1 does not attempt drift detection.**

If `<root>/canonical/type-ownership.md` exists, Phase 1 emits the draft with this header block:

```md
> ⚠ Existing canon detected: `canonical/type-ownership.md`.
> This draft is OBSERVATIONAL ONLY — it reports what AST shows, not what canon
> declares. Full drift detection is the job of `check-canon.mjs` (Phase 5).
> Do not promote this file over the existing canon without manual review.
```

No comparison logic runs in Phase 1. The draft is a snapshot of the current AST state; the user or LLM compares manually if needed.

### 5.5 Output file naming (non-overwrite)

If `<canon-output>/type-ownership.md` already exists from a prior run: emit `<canon-output>/type-ownership.v{N}.md` where `N = max(existing) + 1`. Never overwrite a prior draft. The user diffs manually before merging or discarding the older version.

---

## 6. Output format — `type-ownership.md`

Format mirrors `ouroboros-ts/canonical/type-ownership.md` so promotion is a rename + manual annotation, not a reformat.

All Markdown cells use `escapeMdCell(s)` (escapes `|`, `\`, newlines) and `codeCell(s)` (wraps in backticks, escapes inner backticks) — see `_lib/canon-draft.mjs`.

The example below uses a 4-backtick outer fence so the nested ` ```json ` block in §4 renders correctly.

````markdown
# canonical-draft/type-ownership.md — DRAFT

> **Role:** observed type ownership derived from AST. Promote to `canonical/type-ownership.md` after review.
> **Owner:** AUTO-GENERATED — edit freely once promoted.
> **Status:** draft, v{N}
> **Generated:** {ISO timestamp}
> **Root:** {repo absolute path}
> **Source:** Rust `js-ts-extract-artifact` pass ({N} files scanned) + `symbols.json.reExportsByFile` ({present|absent})
> **Scope:** exported top-level TS type declarations only. Local / private types are not in Phase 1.

[optional existing-canon warning header — see §5.4]

---

## 1. Summary

Counts distinguish **groups** (distinct type names) from **identities** (`ownerFile::name` rows). A group of three `User` definitions contributes 1 to "group count" and 3 to "identity count". See `canonical/classification-gates.md` §7.

| Bucket | Groups | Identities |
|---|---:|---:|
| Single owner, strong fan-in (≥ 3 consumers) | — | {N} |
| Single owner, weak fan-in (1–2 consumers) | — | {N} |
| Single owner, zero internal fan-in | — | {N} |
| severely-any-contaminated (single-owner, Rule 0) | — | {N} |
| low-signal-type-name (1-char exported alias) | — | {N} |
| DUPLICATE_STRONG | {N} | {N} |
| DUPLICATE_REVIEW | {N} | {N} |
| LOCAL_COMMON_NAME | {N} | {N} |
| ANY_COLLISION | {N} | {N} |
| Total type-name groups | {N} | — |
| Total identities | — | {N} |

Action items to review: {N} DUPLICATE_STRONG + {N} ANY_COLLISION rows should be resolved before promotion.

---

## 2. Type registry

Every registry table carries a `Tags` column and an `Any / unknown signal` column. Column composition:

- **`Tags` column** is the union of two tag streams, deduped and joined by space — **semantic tags first, path tags second**:
  - Semantic tags from `classifyTypeNameGroup` (`has-any-member`, `has-severe-any-member`) — required by `canonical/classification-gates.md` §2.1 so mixed duplicate groups' contaminated members do NOT disappear. Empty on singleton rows.
  - Path-based tags (`test-only` via `_lib/test-paths.mjs::isTestLikePath`; future: `generated`, `public-api`) — required by `canonical/classification-gates.md` §8.
- **`Any / unknown signal` column** surfaces the `anyContamination` label + key raw measurement when present (e.g. `severely-any-contaminated (anyFieldRatio 0.85, asAnyCount 3)`). Required by `canonical/any-contamination.md` §11 — raw measurements, not just labels.

Classification produces the core label and semantic tags (per `canonical/classification-gates.md` §2/§4/§2.1). The renderer (`renderTypeOwnershipRow`, see §13 step 1) merges classification.tags ∪ pathMeta.tags → `Tags`, and merges contamination label+measurements → `Any / unknown signal`. Classification functions do NOT compose the final `Tags` cell; that is a rendering concern.

### 2.1 Single owner (strong)

| Type | Owner | Kind | Line | Fan-in | Re-exported through | Status | Tags | Any / unknown signal |
|---|---|---|---|---:|---|---|---|---|
| `SessionId` | `src/protocol/ids.ts` | TSTypeAliasDeclaration | 14 | 8 | `src/index.ts` | ✅ | — | — |

### 2.2 Single owner (weak / zero-internal-fan-in)

| Type | Owner | Kind | Line | Fan-in | Re-exported through | Status | Tags | Any / unknown signal | Note |
|---|---|---|---|---:|---|---|---|---|---|
| `InternalFlag` | `src/engine/flag.ts` | TSTypeAliasDeclaration | 3 | 1 | — | ⚠ weak | — | — | only `src/engine/flag-consumer.ts` |
| `OrphanShape` | `src/old/orphan.ts` | TSInterfaceDeclaration | 10 | 0 | — | ⚠ zero-internal-fan-in | — | — | May be public API. Cross-check `dead-classify.json` before removal. |
| `TestOnlyHelperShape` | `tests/fixtures/foo.ts` | TSInterfaceDeclaration | 4 | 2 | — | ⚠ weak | `test-only` | — | Test fixture. Typically dropped on promotion. |

### 2.3 severely-any-contaminated (single-owner, Rule 0)

Per `canonical/classification-gates.md` §4 Rule 0: severely-contaminated identities are never promoted to `single-owner-strong` even at high fan-in. Omitted from canon; consumer reuse is possible but carries the contamination warning (see `canonical/any-contamination.md` §5).

| Type | Owner | Kind | Line | Fan-in | Tags | Any / unknown signal |
|---|---|---|---|---:|---|---|
| `LegacyPayload` | `src/legacy/payload.ts` | TSInterfaceDeclaration | 3 | 6 | — | severely-any-contaminated (anyFieldRatio 0.85, anyFields 6/7) |

### 2.4 DUPLICATE_STRONG — likely shared concept, needs resolution

| Type | Files defining | Kinds | Max fan-in | Total fan-in | Tags | Any / unknown signal | Suggested action |
|---|---|---|---:|---:|---|---|---|
| `Result` | `src/protocol/errors.ts:5`, `src/engine/runner.ts:22` | 2× TSTypeAliasDeclaration | 18 | 21 | — | — | Same meaning? Pick `src/protocol/errors.ts` (higher fan-in). Different meanings? Rename one (`RunnerResult`?). |
| `Payload` | `src/api/types.ts:4`, `src/worker/types.ts:11` | 2× TSInterfaceDeclaration | 7 | 10 | — | has-any-member (1 of 2 members any-contaminated); semanticConfidence=low | Mixed group: one member is contaminated. Compare shapes manually — semantic match cannot be grounded until the contaminated member is tightened. |

### 2.5 DUPLICATE_REVIEW — same name, may be local

| Type | Files | Kinds | Fan-in per identity | Tags | Any / unknown signal | Suggested action |
|---|---|---|---|---|---|---|
| `Config` | `apps/admin/config.ts:3`, `apps/blog/config.ts:3` | 2× TSInterfaceDeclaration | 2, 1 | — | — | Local `Config` per app is legitimate. Canon can omit, or rename to `AdminConfig` / `BlogConfig` if collision concerns arise. |

### 2.6 LOCAL_COMMON_NAME — omit from canon

| Name | Locations | Count | Tags | Note |
|---|---|---:|---|---|
| `Props` | 14 files | 14 | — | Local React-component shape. Not canon material. |
| `Options` | 7 files | 7 | — | Context-local option bags. |

### 2.7 ANY_COLLISION — every member contaminated

Per `canonical/classification-gates.md` §2 Rule 0: when every identity in a duplicate group is `any-contaminated` or `severely-any-contaminated`, the group is `ANY_COLLISION` — a name collision of opaque blobs, not a meaningful shared concept.

| Name | Files | Fan-in per identity | Tags | Any / unknown signal per identity | Suggested action |
|---|---|---|---|---|---|
| `Data` | `src/ingest/data.ts:3`, `src/export/data.ts:3` | 4, 2 | — | any-contaminated (0.5), severely-any-contaminated (0.7, asAnyCount 2) | Neither side is safe to share. Tighten at least one before claiming duplication. |

### 2.8 low-signal-type-name

| Name | Owner | Fan-in | Tags | Note |
|---|---|---:|---|---|
| `T` | `src/generic-util.ts:2` | 1 | — | One-char exported alias; omit from canon unless fan-in grows. |

---

## 3. Finalize checklist

Before promoting to `canonical/type-ownership.md`:

1. **Resolve every DUPLICATE_STRONG row** (§2.4). Canon should not carry unresolved duplicates for shared/public concepts. Mixed-contamination rows: tighten the contaminated member first — shape comparison across contaminated/clean pairs is NOT grounded.
2. **Review ANY_COLLISION rows** (§2.7). These are not merge candidates; they are opaque-blob name collisions. Tighten at least one identity before considering shared ownership.
3. **Review DUPLICATE_REVIEW rows** (§2.5). Legitimate local duplicates are often fine to omit; cross-file collisions of the same concept may need renaming.
4. LOCAL_COMMON_NAME (§2.6), low-signal-type-name (§2.8), severely-any-contaminated (§2.3) are informational — typically dropped from canon, not "fixed" by promotion.
5. Drop rows with `test-only` tag unless explicitly intended for canonical promotion.
6. Add a **purpose** column for types that deserve documented intent (optional).
7. Remove types that are implementation details not meant to be canon.
8. Rename: `<canon-output>/type-ownership.md` → `<root>/canonical/type-ownership.md`.
9. Commit. Phase 5 (`check-canon.mjs`) will then diff-check future runs against this canon.

---

## 4. Generation metadata (do not edit)

```json
{
  "tool": "generate-canon-draft.mjs",
  "schemaVersion": 1,
  "target": "type-ownership",
  "filesScanned": {N},
  "identitiesFound": {N},
  "inputs": {
    "symbols.json.reExportsByFile": {present|absent},
    "checklist-facts.json": {present|absent}
  },
  "thresholds": {
    "strongFanIn": 3,
    "weakFanIn": 1,
    "lowInfoNames": [...]
  }
}
```
````

---

## 7. Calibration plan (reporting, not gating in Phase 1)

**External review corollary: Phase 1 reports calibration metrics but does NOT gate on them.** Gating begins in Phase 1.1 or Phase 2 after thresholds are empirically tuned.

### 7.1 Corpus

- `suyeon-daemon-followup-p-work/` — `docs/current/reference/geulbat-function-helper-canonical-brief-v1-codex-direct.md`.
- `ouroboros-ts/` — `canonical/type-ownership.md`.

### 7.2 Metrics

Same four metrics as v0 (recall, precision, owner agreement, DUPLICATE_STRONG detection), measured and reported on each corpus run. Targets (recall ≥ 90%, precision ≥ 70%, owner agreement ≥ 95%, duplicate recall 100%) become Phase 1.1+ gates after first real calibration run.

### 7.3 Unit tests vs calibration

- **Unit tests** (`tests/test-generate-canon-draft.mjs`) — local fixtures in `tmpdir()`. Cover: normal single owner, DUPLICATE_STRONG, DUPLICATE_REVIEW, LOCAL_COMMON_NAME, zero-internal-fan-in, re-export chain, existing-canon header, v{N} non-overwrite, missing symbols.json, empty repo, markdown escaping.
- **Calibration** — manual, against external corpora in a maintainer-local scratch directory. Not part of `npm test`. Ship a `scripts/calibrate-canon.mjs` helper for repeatability.

---

## 8. Open questions (resolved from v0)

- **Q1 (symbols.json extension vs fresh pass)** — ✅ CLOSED. The generator builds a fresh, scoped Rust `js-ts-extract-artifact` index. `symbols.json.reExportsByFile` is consumed when present.
- **Q2 (output location)** — ✅ CLOSED. `--canon-output` separate from `--output`. Defaults per mode (§3.1).
- **Q3 (canonical/ exists)** — ✅ CLOSED. Phase 1 emits observational draft with warning header. Full drift = Phase 5 separate spec.
- **Q4 (tunable thresholds)** — ✅ CLOSED. Baked in Phase 1. CLI flags open up in Phase 2.
- **Q5 (test-only types)** — ✅ CLOSED. Include but tag with `test-only` in the **Tags column** (per `canonical/classification-gates.md` §8 + v0.2.1 output format). The optional `Note` column carries prose; the `Tags` column carries the machine-readable path tag that drives the finalize-step filter ("drop rows with `test-only` tag unless explicitly intended for canonical promotion"). Let human decide on promotion.
- **Q6 (Phase 5 scope)** — ✅ CLOSED. Phase 5 gets its own spec, not bundled here.

All v0 questions resolved. New questions raised during review also absorbed into the spec (see §14).

---

## 9. Non-goals

- NOT a replacement for human review.
- NOT a style-guide generator. Naming patterns surface in Phase 4; code restyling is out.
- NOT a refactor tool. Proposes structure; never rewrites source.
- NOT opinionated about repo shape.
- NOT a drift detector. Phase 1 emits observations; drift vs `canonical/` is Phase 5.

---

## 10. Success criteria for Phase 1

Phase 1 ships when:

1. `node generate-canon-draft.mjs --root <r> --canon-output <d> --source type-ownership` emits valid `type-ownership.md` on:
   - This skill's own repo.
   - ouroboros-ts.
   - suyeon-daemon-followup-p-work.
2. All unit tests in `tests/test-generate-canon-draft.mjs` pass. Calibration is reported but not a gate.
3. `audit-repo.mjs` accepts an opt-in `--canon` flag (not in default profiles). Adding the flag triggers canon draft emission.
4. `SKILL.md` gets a brief `Canon bootstrap mode` subsection pointing at the generator.
5. Full suite runs with zero regression.

**Explicitly NOT in Phase 1 release criteria** (per external review): corpus calibration metrics as hard gates. Those promote to criteria in Phase 1.1 or Phase 2 once Phase 1 runs on real corpora reveal realistic target values.

---

## 11. Rejected alternatives

(carried from v0, still valid)

- **LLM-only canon drafting** — rejected: no reproducibility, no calibration, expensive, AST-trivial tasks done badly.
- **Full schema upfront** — rejected: locks format before empirical feedback. Phased rollout is required.
- **Write directly to `canonical/`** — rejected: erases the draft-vs-canon decision gate. Humans must own promotion.
- **Mixed extractor** (copy into generator + leave in build-symbol-graph) — rejected. Rust `js-ts-extract-artifact` is the semantic owner; duplicate JS parsing or fallback classification is exactly the anti-pattern this skill exists to detect.

---

## 12. Decision log

- **2026-04-20 (v0)**: Spec drafted. 6 open questions flagged.
- **2026-04-20 (v0.1)**: External review absorbed. 10 changes applied (see §14). All v0 open questions resolved. Spec renamed framing from "canon generator" to "canon draft generator" to reflect observation-vs-authorship distinction.
- **2026-04-20 (v0.2)**: Canonical-spine promotion. This spec demoted from standalone source-of-truth to P3 subordinate. §5.3 classification table removed and delegated to `canonical/classification-gates.md` (the canonical file reversed the v0.1 Rule 1 / Rule 2 precedence conflict; this spec must NOT re-assert the v0.1 ordering). `classifyIdentity` API renamed to `classifyTypeNameGroup` + `classifySingleIdentity` to match the canonical file's distinct group-vs-identity split. "Export type namespace" wording corrected to "Export namespace". Alias fidelity handoff now explicitly references `canonical/identity-and-alias.md`.
- **2026-04-20 (v0.2.1)**: Second reviewer pass. Five P0 items and three P1 items absorbed, closing remaining drift risk against the canonical spine. Primary repairs: §5.2 now takes a full `importRecord` and delegates chain resolution to `canonical/identity-and-alias.md` §6 (including mixed-file and ambiguous-star handling); `LOW_INFO_NAMES` treated as a Markdown-normative canonical + code-mirror pair with drift test; §6 output format widened to separate group-vs-identity counts, surface `Tags` and `Any / unknown signal` columns, and add `severely-any-contaminated` / `ANY_COLLISION` bucket rows; classification functions' responsibility clarified (core label only; renderer owns Tags + contamination merging); `tests/test-classification-gates.mjs` promoted to a distinct handoff step with enumerated case coverage. See §14 v0.2.1 change log.
- **2026-04-20 (v0.2.2)**: Third reviewer pass. Three P0 items + three P1 items absorbed. `fact-model.md` §3.1 `type-owner` now carries `exportedName` as the canonical identity field (§14 P0-1). SPEC §13 classification handoff restored the `tags` field on `classifyTypeNameGroup` return (dropped in v0.2.1 handoff while retained in §5.3) and split Tags-column composition between semantic tags (classification) and path tags (pathMeta) in `renderTypeOwnershipRow` (§14 P0-2). `any-contamination.md` §10 producer responsibilities now enumerate all 10 `escapeKind` values 1:1 with `fact-model.md` §3.9 (§14 P0-3). Classification-gates `any-contamination.md §5` cross-refs retargeted to §6 Stage 1 / §9 via title-based anchor. Pre-write output example's flat `anyContamination` schema upgraded to `{label, labels, measurements}`. SPEC §6 output-format fence widened to 4 backticks so the nested `json` block in Generation Metadata renders cleanly. Q5 test-only storage moved to `Tags` column wording.
- **2026-07-14 (Rust fact-owner migration)**: Canon helper and naming scans now build one scoped Rust `js-ts-extract-artifact` index per invocation. `_lib/extract-ts.mjs` remains only as a fail-closed compatibility adapter; it owns no parser or JS fallback. `check-canon --source all` shares the same index across both sources.

---

## 13. Implementation handoff (activated on approval)

Revised sequence after v0.1:

1. **`_lib/canon-draft.mjs`** — shared helpers:
   - `aggregateTypes(files, reExportsByFile)` → returns `{typeDefsByIdentity, typeNameIndex, typeUsesByIdentity}`. Identity keying per `canonical/identity-and-alias.md` §2; import/re-export alias preservation per §4–§5; chain resolution per §6 (mixed-file aware, ambiguity-preserving for star re-exports).
   - `classifyTypeNameGroup(name, identities, fanInByIdentity, contaminationByIdentity)` → returns `{label, marker, anyMembers, severeAnyMembers, semanticConfidence, tags}` per `canonical/classification-gates.md` §2 and §2.1. Handles Rule 0 (ANY_COLLISION — only when every member is `any-contaminated`/`severely-any-contaminated`) → Rule 1 (DUPLICATE_STRONG) → Rule 2 (LOCAL_COMMON_NAME) → Rule 3 (DUPLICATE_REVIEW) in that order. The `tags` field carries SEMANTIC annotations required by §2.1 (`["has-any-member"]`, `["has-severe-any-member"]`, empty array when the group is all-clean). It is NOT where path-based tags (`test-only`, future `generated` / `public-api`) live — those flow through `pathMeta`, see step 1d below.
   - `classifySingleIdentity(identity, fanIn, kind, contamination)` → returns `{label, marker}` per `canonical/classification-gates.md` §4. Handles Rule 0 (severely-any-contaminated) → Rule 1 (low-signal-type-name) → Rule 2/3/4 (fan-in tiers) in that order. Classification returns the CORE label only — it does NOT compose the final `Tags` column (which is a display concern merging semantic + path tags), does NOT merge contamination annotations into prose, and does NOT own path-based `test-only` tagging.
   - `renderTypeOwnershipRow({identity, classification, contamination, pathMeta})` → row model for Markdown. Two columns are composed here:
     - **`Tags` column** = `classification.tags` (semantic any-tags from `classifyTypeNameGroup`; empty on singleton rows) ∪ `pathMeta.tags` (path-based: `test-only` per `_lib/test-paths.mjs::isTestLikePath`, future `generated` / `public-api`). Deduped, stable order — semantic tags first, then path tags. Joined by space in the rendered cell.
     - **`Any / unknown signal` column** = `contamination.label` + key raw measurements from `contamination.measurements` (e.g. `severely-any-contaminated (anyFieldRatio 0.85, asAnyCount 3)`). Empty when `contamination` is absent.
     Single point of truth for "how a row is displayed"; classification + contamination + path-meta facts flow in, a rendered row flows out.
   - `escapeMdCell(s)` / `codeCell(s)` — Markdown escape helpers.
   - `nextDraftVersionPath(dir, baseName)` — produces `.v{N}.md` on collision.
   - `LOW_INFO_NAMES` code constant mirroring `canonical/classification-gates.md` §3. Since the canonical file is Markdown (not JS-importable), the mirror is drift-tested by `tests/test-classification-gates.mjs`. The canonical markdown is the source; the code constant is derivative.
2. **`generate-canon-draft.mjs`** — CLI entry:
   - Arg parsing via `_lib/cli.mjs`.
   - `--source type-ownership` only in Phase 1.
   - `--canon-output` default per §3.1.
   - Emits Markdown per §6 format.
3. **`tests/test-classification-gates.mjs`** — canonical-aware conformance harness (required before §13 step 4 implementation work). Test cases MUST cover:
   - `LOW_INFO_NAMES` code mirror matches `canonical/classification-gates.md` §3 exactly (same names, same order; parses the Markdown at test time).
   - Rule 0 ANY_COLLISION fires ONLY when every member is `any-contaminated` or `severely-any-contaminated`. `has-any`-only groups and `unknown-surface`-only groups DO NOT trigger Rule 0 — they fall through to Rule 1/2/3.
   - Rule 1 DUPLICATE_STRONG precedes Rule 2 LOCAL_COMMON_NAME when fan-in ≥ 3 even for a `LOW_INFO_NAMES` name (fixes the v0.1 precedence conflict).
   - Mixed duplicate groups preserve `anyMembers`, `severeAnyMembers`, `semanticConfidence: "low"` on classification output (§2.1 invariant: contaminated members must never disappear).
   - Single-identity Rule 0 (severely-any-contaminated) fires BEFORE Rule 2 (single-owner-strong) even at high fan-in.
   - Single-identity Rule 1 (low-signal-type-name) fires BEFORE Rule 3/4 (fan-in-based) when fanIn < 3 AND name length == 1.
   - The label set returned is a subset of `canonical/classification-gates.md` §9 — any other label is a defect.
4. **`tests/test-generate-canon-draft.mjs`** — local fixtures covering §7.3 cases. Classification conformance is in #3 above; this file covers end-to-end Markdown emission (row rendering, table shape, escape helpers, v{N} non-overwrite, existing-canon header, missing symbols.json degrade path, empty repo).
5. **`audit-repo.mjs`** — add opt-in `--canon` flag; not in default `quick` / `full` profiles.
6. **`SKILL.md`** — add `## Canon bootstrap mode` subsection after `## Structural review mode` with a one-paragraph pointer.
7. **`update-test-doc.mjs`** — register the new tests (both #3 and #4).
8. **Memory** — record v0.2 spec approval + Phase 1 ship in `project_grounded_audit.md`.

Expected session length: 1 focused session for the test harness (#3) + 1 for the generator (#2, #4 onwards). Do not bundle them; the canonical conformance test must be green before the generator builds against it.

---

## 14. Change log

### v0.2.1 → v0.2.2 (2026-04-20)

Third reviewer pass. All three P0 items and three P1 items absorbed; canonical spine + SPEC now internally consistent on identity fields, Tags-column responsibility, and type-escape producer coverage.

**P0-1 — `exportedName` on `type-owner` facts.** `canonical/identity-and-alias.md` §2 defines identity as `ownerFile::exportedName`. `canonical/fact-model.md` §3.1 example was keyed on `typeName` alone, leaving implementers free to use the wrong field. v0.2.2 adds `exportedName` as the canonical identity field on `type-owner` facts; `typeName` is kept as a display alias (always equal on owner facts), but identity matching MUST read `exportedName`. `fact-model.md` §8 gains a matching invariant. The SPEC's internal `typeDefsByIdentity` shape was already correct; the canonical example is now aligned.

**P0-2 — Tags column responsibility split.** v0.2.1's §5.3 listed `tags` on the `classifyTypeNameGroup` return shape; §13 handoff silently dropped it, and the `renderTypeOwnershipRow` description left semantic `any` tags unlocated. v0.2.2 restores `tags` to the §13 handoff, and §6 output-format intro + `renderTypeOwnershipRow` description now state the split explicitly:
- `classifyTypeNameGroup.tags` carries SEMANTIC group-level annotations (`has-any-member`, `has-severe-any-member`) required by `canonical/classification-gates.md` §2.1.
- `pathMeta.tags` carries PATH-based annotations (`test-only`; future `generated` / `public-api`).
- `renderTypeOwnershipRow` composes `Tags` = semantic ∪ path, deduped, semantic first.

**P0-3 — `type-escape` producer 1:1 coverage.** `canonical/any-contamination.md` §10 producer responsibilities previously enumerated ~4 escape shapes. `canonical/fact-model.md` §3.9 defines 10 `escapeKind` values. A producer that covers only 4 silently breaks the "no silent new any" invariant (invariants.md §9). §10 now contains a full 10-row table mapping every `escapeKind` to its AST / source shape: `explicit-any`, `as-any`, `angle-any`, `as-unknown-as-T`, `rest-any-args`, `index-sig-any`, `generic-default-any`, `ts-ignore`, `ts-expect-error`, `no-explicit-any-disable`. Missing any row is declared a producer defect.

**P1 cross-reference drift cleanup.** `canonical/classification-gates.md` §4 Rule 0 and the rationale paragraph below referenced `any-contamination.md §5` (which is `type-escape` occurrence fact). Correct target is "Pre-write gate interaction" (§6 Stage 1 + §9). Switched to title-based cross-ref so future section-numbering drift does not break the link again.

**P1 stale `anyContamination` flat schema.** `pre-write-gate.md` §5 output example still showed `{label, anyFieldRatio, totalFields, anyFields}` on one line. Upgraded to the canonical `{label, labels, measurements: {...}}` shape so the example matches `canonical/any-contamination.md` §4 + `canonical/fact-model.md` §3.1.

**P1 nested markdown fence.** SPEC §6 output-format block was a triple-backtick `markdown` fence wrapping a triple-backtick `json` sub-fence — a syntactic collision that breaks many Markdown renderers. Outer fence widened to four backticks (CommonMark 4-backtick rule) + short note explaining why. Rendering is now correct in any CommonMark-conformant viewer.

**P1 `test-only` storage column.** SPEC §8 Q5 resolution said "tag with `test-only` note in the Note column" while the v0.2.1 output format put it in the `Tags` column. v0.2.2 Q5 updated to match: `test-only` lives in the Tags column (machine-readable, drives the finalize-step filter); the optional Note column is prose only.

### v0.2 → v0.2.1 (2026-04-20)

Second reviewer pass on the canonical-spine-promoted spec. Five P0 items and three P1 items absorbed.

**P0-1 — Mixed-file re-export chain.** v0.2's §5.2 inherited the loose "resolvedFile is a re-export-only barrel" condition; reviewer noted mixed files (some owned exports + some re-exports) fail this condition and would be wrongly claimed as owners for re-exported names. Fixed by delegating to `canonical/identity-and-alias.md` §6, where the loop terminates per-name ("does this file own `nameAtBarrier`?"), not per-file. `identity-and-alias.md` §6 was rewritten in the same pass to make this explicit and to add ambiguous-star handling. See §5.2.

**P0-2 — `importRecord` in §5.2.** v0.2 still described extraction as `{ fromSpec, name }`. Fixed to `{ fromSpec, importedName, localName, kind, typeOnly }` with identity resolution explicitly delegated to `canonical/identity-and-alias.md` §6 (`resolveIdentity(consumerFile, importRecord)`). The terminal identity now uses `nameAtBarrier`, not consumer-side `localName`. See §5.2.

**P0-3 — `LOW_INFO_NAMES` mirror rather than "import".** v0.2 said "imported from the canonical module". JS cannot import Markdown. Reworded: the canonical Markdown is the normative source; `_lib/canon-draft.mjs` mirrors the list as a code constant; `tests/test-classification-gates.mjs` fails if the mirror drifts. See §5.3, §13 step 1.

**P0-4 — Output format widened.** v0.2 Summary did not distinguish groups from identities, did not surface `ANY_COLLISION`, and registry tables had neither `Tags` nor `Any / unknown signal` columns. Fixed per `canonical/classification-gates.md` §7 (group vs identity counting) and §8 (Tags column required). Summary now has separate Groups / Identities columns. Registry tables (§2.1–2.8) now carry `Tags` and `Any / unknown signal`. New bucket §2.3 `severely-any-contaminated (single-owner)` and §2.7 `ANY_COLLISION` make the Rule 0 cases visible. See §6.

**P0-5 — Pre-write cross-ref retargeted.** `pre-write-gate.md` referenced `any-contamination.md §5`, which is the `type-escape` occurrence fact. Actual pre-write interaction lives in `any-contamination.md` §6 Stage 1 ("Pre-write") and §9 ("Pre-write gate interaction"). Fixed both references in `pre-write-gate.md` (Step 3 lookup table row + "Any-contamination demotion rule" block). Switched to section-title-based cross-refs to reduce numbering-churn risk.

**P1-6 — `tests/test-classification-gates.mjs` promoted.** v0.2 implementation handoff §13 had only `tests/test-generate-canon-draft.mjs`. The canonical conformance test is architecturally distinct (tests the classification gate, not the generator) and must be green before the generator builds against it. Now a separate step (§13 step 3) with enumerated test cases: LOW_INFO_NAMES mirror, Rule 0 contamination filter, Rule 1 precedence, mixed-group preservation, single Rule 0 severe-first, single Rule 1 low-signal-before-fanin, label set ⊆ §9.

**P1-7 — `@ts-expect-error` in invariants.** `canonical/invariants.md` §9 no-silent-new-any list omitted `@ts-expect-error`. Added. `invariants.md` now cross-refs `fact-model.md` §3.9 `type-escape.escapeKind` as the authoritative enumeration — so the invariant list follows the fact model, not the other way around.

**P1-8 — Classification vs rendering responsibility.** v0.2 had `classifySingleIdentity` returning `{label, marker}` but said nothing about where `Tags` / contamination merge happens; risk is those concerns leaking into classification. Made explicit: classification returns core label + (for groups) `anyMembers`/`severeAnyMembers`/`semanticConfidence` per §2.1; rendering is `renderTypeOwnershipRow(...)`, new helper in §13 step 1, which owns `Tags` column composition and `Any / unknown signal` merging from contamination facts.

### v0.1 → v0.2 (2026-04-20)

Canonical-spine promotion absorbed. Four reviewer P0 items resolved.

1. **Spec demoted to P3 subordinate.** Header revised to state that invariants live in `canonical/*.md`; this spec is implementation sequencing only. Prior framing as standalone source-of-truth was contributing to drift risk.
2. **§5.3 classification table deleted.** Delegated to `canonical/classification-gates.md`. Any local table would be a second source-of-truth and would drift. The canonical file also reverses the v0.1 Rule 1 / Rule 2 precedence conflict (DUPLICATE_STRONG before LOCAL_COMMON_NAME when fan-in is high), so re-stating the v0.1 table would have re-introduced the bug.
3. **API renamed.** `classifyIdentity(entry)` → `classifyTypeNameGroup(...)` + `classifySingleIdentity(...)`. The canonical file distinguishes per-group vs per-identity evaluation; the v0.1 monolithic function conflated them. Implementation now has two pure functions with distinct signatures, each matching a canonical ruleset (§2 and §4).
4. **"Export type namespace" syntax corrected.** TypeScript has `export namespace X { ... }`, not `export type namespace X { ... }`. v0.1 wording was invalid TS.
5. **Alias fidelity handoff strengthened.** §13 step 1 now cross-references `canonical/identity-and-alias.md` §2 (identity rule), §4 (import alias preservation), §5 (re-export alias preservation), §6 (chain resolution) — so the implementer doesn't re-derive the fidelity rules.
6. **LOW_INFO_NAMES provenance.** v0.1 defined the list here. v0.2 imports it from the canonical module; the spec does not own the list.
7. **Conformance test required.** Added `tests/test-classification-gates.mjs` as a canonical-aware conformance harness. Any label produced by `_lib/canon-draft.mjs` that is not in `canonical/classification-gates.md` §9 is a defect.

### v0 → v0.1 (2026-04-20)

Absorbed from external review 2026-04-20. Each item below maps to a specific review concern.

1. **Framing rename** — "canon generator" → "canon draft generator". The tool emits evidence for canon drafting; it does not author canon.
2. **P0-1 resolved**: confirmed `_lib/extract-ts.mjs` exists since v1.10.1. Spec §3.3 notes the shared extractor. Review item was based on stale v1.9.11 state.
3. **P0-2 applied**: identity-keyed aggregation (`typeUsesByIdentity`) replaces name-keyed. Re-export chains resolved through `symbols.json.reExportsByFile`. See §5.2.
4. **P0-3 applied**: three-tier duplicate classification (`DUPLICATE_STRONG` / `DUPLICATE_REVIEW` / `LOCAL_COMMON_NAME`). `LOW_INFO_NAMES` list added. Softened "Canon cannot carry duplicates" to "should not carry unresolved duplicates for shared/public concepts". See §5.3, §6.
5. **P0-4 applied**: `unused-export` → `zero-internal-fan-in`. Note text calls out external API possibility. See §5.3, §6.2.2.
6. **P0-5 applied**: Phase 1 includes minimal re-export chain resolution. Output table has `Re-exported through` column. See §5.2, §6.2.1.
7. **P1-6 applied**: removed incorrect `generic-parameter` rule. Added `low-signal-type-name` for one-char exported aliases. See §5.3.
8. **P1-7 applied**: Phase 1 scope explicitly limited to "exported top-level TS type declarations only". Local/private types deferred. See §2.1, §6 header.
9. **P1-8 applied**: drift mode completely removed from Phase 1. Existing-canon case emits warning header only. Drift detection = Phase 5 (separate spec). See §5.4.
10. **P1-9 applied**: `--canon-output` CLI flag separates from `--output`. Defaults per mode (standalone vs audit-repo) to avoid surprising working-tree writes in CI. See §3.1.
11. **P1-10 applied**: Markdown escape helpers in `_lib/canon-draft.mjs`. See §6 preamble, §13 step 1.
12. **Calibration rebalanced**: Phase 1 reports metrics; gating moves to Phase 1.1 / Phase 2. External corpora are calibration-only, not unit tests. See §7, §10.
