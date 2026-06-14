# docs/history/phases/p3/session.md — Phase 3 canon draft generator

> **Phase:** P3 — canon draft generator. Follows P2 (post-write delta). Independent in scope: does not depend on P2 artifacts and does not alter P1/P2 contracts.
> **Role:** phase-level roadmap for the stickiness loop — read code AST, derive observed ownership / helper / topology / naming, emit Markdown drafts that a human or LLM finalizes into `canonical/*.md` invariants.
> **Status:** phase plan, v2 (2026-04-21 reviewer P0s addressed: canonical label set, audit-repo hook timing, P3-5 renaming to Post-P3, fact-model cross-ref, mandatory Step 0 drift test).
> **Last updated:** 2026-04-21

> **Implementation authority.** This file sequences P3 into reviewable sub-phases; it does NOT re-derive design. When this file conflicts with parent design on P3 internals (algorithm, identity resolution, output format, classification labels), **`docs/spec/SPEC-canon-generator.md` v0.2.2 wins** — that spec has been through three reviewer passes and carries normative decisions.
>
> **Implementer pin**: the authoritative SPEC is the file whose header reads `# Canon Draft Generator — Spec v0.2.2` + `Status: v0.2.2 draft — awaiting user approval`. Older drafts (v0, v0.1, v0.2, v0.2.1) appear in §14 changelog entries of the same file but are HISTORY. Ignore older indexed snippets. If an indexed search result shows a v0.2.1 or earlier statement that contradicts v0.2.2 (e.g. name-keyed fan-in, pre-DUPLICATE_REVIEW label set), follow v0.2.2.

---

## 1. Boot (read before starting)

| File | Why this phase needs it |
|---|---|
| `docs/spec/SPEC-canon-generator.md` v0.2.2 | Parent design spec. Normative for algorithm, shape, labels, and output format. Sections §3 (inputs), §5 (Phase 1 algorithm), §6 (output format), §12 (unit tests), §14 (changelog). |
| `canonical/invariants.md` | Iron Law. §6 "Before writing new code, make Claude aware of what already exists" — canon draft generator is the long-running-memory enabler; it turns AST facts into the reading surface P1 pre-write gate consumes next session. |
| `canonical/identity-and-alias.md` | §4 alias fidelity (`importedName` vs `localName`), §6 `resolveIdentity`, §9 confidence rules. The generator delegates identity resolution here — it does NOT reimplement. |
| `canonical/classification-gates.md` | §2 / §4 classification rules, §3 `LOW_INFO_NAMES`, §9 label set. The canonical 9-label set is: `zero-internal-fan-in`, `low-signal-type-name`, `DUPLICATE_STRONG`, `DUPLICATE_REVIEW`, `LOCAL_COMMON_NAME`, `single-owner-strong`, `single-owner-weak`, `severely-any-contaminated`, `ANY_COLLISION`. `_lib/canon-draft.mjs` mirrors `LOW_INFO_NAMES`; `tests/test-classification-gates.mjs` is the drift guard. |
| `canonical/fact-model.md` | §3.1 `type-owner` + §3.2 `helper-owner` (both carry optional `anyContamination` annotation — what P3-1 reads for severe-any routing). §3.8 use-resolution confidence. **Not** §3.9 — §3.9 is `type-escape` occurrence facts (P2 territory); P3 reads the per-identity contamination ANNOTATION on owner facts, not the occurrence-level delta. |
| `canonical/any-contamination.md` | §3 (annotation shape), §4 (contamination label enumeration). The annotation carries `label` (highest-severity tier) + `labels[]` (full applicable set) + `measurements`. P3-1 routes `severely-any-contaminated` owners through the `severely-any-contaminated` classification label. |
| `_lib/extract-ts.mjs` | Shared TS/JS extractor (`extractDefinitionsAndUses`). Phase 1 reuses this — no duplicate extractor. |
| `docs/history/phases/p2/session.md` | Parallel structure reference. P3 mirrors P2's "phase split into sub-phases with per-session specs + reviewer gates" discipline; otherwise independent. |

Not required for P3 v1:
- `canonical/any-contamination.md` §6 (post-write delta Stage 2) — P3 is not a delta.
- `canonical/pre-write-gate.md` — P3 emits canon drafts, not advisories; pre-write gate reads promoted canon (not drafts) so P3 does not couple to P1.
- `canonical/mode-contract.md` §2.3 (post-write triggers) — P3 has its own invocation path, not a post-write trigger.
- `canonical/fact-model.md` §3.9 (`type-escape` occurrence facts) — P2 consumes this; P3 does not.

## 2. Optimization target

The stickiness loop from `docs/spec/SPEC-canon-generator.md` §1.1:

1. Session N: skill runs → `canonical-draft/type-ownership.md` + siblings generated from AST.
2. Human / LLM fills intent fields; selected entries promoted to `canonical/`.
3. Session N+1: pre-write gate reads `canonical/` before Claude writes → Claude does not re-invent helpers, duplicate types, or misname shared things.
4. Skill runs observe current AST → compare to promoted canon → report drift (Phase 5 / separate spec).

P3 delivers steps 1 + 2 (draft generation + promotion-ready format). Step 3 already exists via P1 pre-write gate + canonical parser. Step 4 lives in `check-canon.mjs` — its own phase.

**Scope qualifier.** The generator is an **evidence producer for drafting**, not a canon author. Every output carries status labels that are OBSERVATIONS (`zero-internal-fan-in`, `DUPLICATE_STRONG`), not verdicts (`unused`, `bad`). The tool NEVER writes to `canonical/` — only to `canonical-draft/`. Promotion is manual by design.

## 3. Scope

### 3.1 In (P3 entire phase)

- **`generate-canon-draft.mjs`** CLI entry at repo root. Flags per `docs/spec/SPEC-canon-generator.md` §3.1 (`--root`, `--output`, `--canon-output`, `--source`).
- **`_lib/canon-draft.mjs`** shared library — pure functions per `docs/spec/SPEC-canon-generator.md` §5 + §6:
  - `classifyTypeNameGroup`, `classifySingleIdentity` (direct impl of `canonical/classification-gates.md` §2/§4).
  - `escapeMdCell`, `codeCell` (Markdown hygiene helpers).
  - `LOW_INFO_NAMES` mirror of canonical §3; drift-checked by test.
  - Identity aggregation + fan-in counting on `identity = ownerFile::exportedName`.
- **Phase-specific emitters** per the §4.2 roadmap: `type-ownership.md`, `helper-registry.md`, `topology.md`, `naming.md`. Each is a separate sub-phase with its own review gate.
- **Conformance tests** per `docs/spec/SPEC-canon-generator.md` §12.
- **`audit-repo.mjs --canon-draft`** opt-in integration. **Added in P3-4 (not P3-1).** Flag name `--canon-draft` is deliberate: `--canon` would read as "write to canonical/", which the tool explicitly does NOT do. Not in default profiles. See §5.1 for the exact flag contract.
- **SKILL.md** update — `## Canon draft mode` subsection.

### 3.2 Out (deferred or out of scope)

- Python / Go canon (Phase 6+ decision per `docs/spec/SPEC-canon-generator.md` §2.2).
- Drift detection against existing `canonical/*.md` — `check-canon.mjs` (separate spec + phase).
- Auto-promotion from `canonical-draft/` to `canonical/` (manual by design).
- Local / private type tracking (Phase 6+ per spec §2.2).
- LLM calls / runtime probing (pure read-only analysis).
- Business-logic / domain-intent documentation (human finalize step).

## 4. Four-sub-phase breakdown (Phase 1 first)

Analogous to P2's P2-0/1/2 pattern. Each sub-phase is a reviewable gate. **P3 has exactly four sub-phases**; `check-canon.mjs` is NOT a P3 sub-phase (see §4.5).

```
P3-1  type-ownership draft    — Phase 1 of docs/spec/SPEC-canon-generator.md §4.2
P3-2  helper-registry draft   — Phase 2
P3-3  topology draft          — Phase 3
P3-4  naming draft + audit-repo --canon-draft integration — Phase 4 + orchestrator hook
```

`docs/spec/SPEC-canon-generator.md` describes Phases 1–4 in §4.2.

**Step 0 is mandatory, not optional, and lives inside P3-1.** The `LOW_INFO_NAMES` mirror drift-test + canonical label-set drift-test are structural safety guards for the P3-1 classifier — a rename in `canonical/classification-gates.md` that the mirror misses would silently corrupt every subsequent P3 draft. Step 0 runs BEFORE Step 1 (pure helpers) per §4.1 below. Earlier draft described P3-0 as "optional, may collapse into P3-1" — that wording is withdrawn in v2; the drift test is required.

### 4.1 P3-1 — `type-ownership.md` (Phase 1 of parent)

P3-1 is a STANDALONE CLI in this phase. No `audit-repo.mjs` hook yet — that lands in P3-4. Running `audit-repo.mjs --canon-draft` BEFORE P3-4 should error.

Step sequence (test-first):

**Step 0 — drift test (mandatory).** `tests/test-classification-gates.mjs`. Parses `canonical/classification-gates.md`; asserts the `LOW_INFO_NAMES` code-constant mirror in `_lib/canon-draft.mjs` is byte-equal to the canonical list (same names, same order), AND the classifier's emitted label set is the canonical 9-label set exactly: `zero-internal-fan-in`, `low-signal-type-name`, `DUPLICATE_STRONG`, `DUPLICATE_REVIEW`, `LOCAL_COMMON_NAME`, `single-owner-strong`, `single-owner-weak`, `severely-any-contaminated`, `ANY_COLLISION`. Drift in either direction fails.

**Step 1 — `_lib/canon-draft.mjs` pure helpers.**
- `LOW_INFO_NAMES` constant (mirror; locked by Step 0 test).
- `classifyTypeNameGroup(name, identities, fanInByIdentity, contaminationByIdentity)` — implements `canonical/classification-gates.md` §2 group-classification precedence (Rule 0 `ANY_COLLISION` → Rule 1 `DUPLICATE_STRONG` → Rule 2 `LOCAL_COMMON_NAME` → Rule 3 `DUPLICATE_REVIEW`).
- `classifySingleIdentity(identity, fanIn, kind, contamination)` — single-owner classification per §4 (`single-owner-strong` / `single-owner-weak` / `zero-internal-fan-in` / `severely-any-contaminated`).
- `escapeMdCell`, `codeCell` — Markdown hygiene helpers.

**Step 2 — type-ownership emitter.** Identity-keyed aggregation:
- Filter to exported top-level type declarations (§2.1 scope: `TSInterfaceDeclaration`, `TSTypeAliasDeclaration`, `TSEnumDeclaration`, `TSModuleDeclaration`).
- `typeDefsByIdentity: Map<ownerFile::exportedName, ...>`.
- `typeNameIndex: Map<name, Array<identity>>` for duplicate detection.
- `typeUsesByIdentity: Map<identity, {directConsumers, reExportedThrough}>`.
- Identity resolution via `canonical/identity-and-alias.md §6` (canonical algorithm; NOT reimplemented).
- `anyContamination` annotation read from `canonical/fact-model.md §3.1` `type-owner` facts (per-identity; `label` field is the highest-severity tier — routes `severely-any-contaminated` + feeds `ANY_COLLISION` Rule 0).
- `[확인 불가]` emissions on ambiguous star re-exports, 8-hop depth cap exceed, unresolved specs.

**Step 3 — `generate-canon-draft.mjs` CLI.** Flags per `docs/spec/SPEC-canon-generator.md` §3.1: `--root`, `--output`, `--canon-output`, `--source type-ownership`. Non-overwrite file versioning (§5.5). Existing-canon observational-only header block (§5.4).

**Step 4 — integration tests.**
- Unit: pure helper coverage for each classification gate rule.
- Integration: CLI fixture with same-file + cross-file duplicate types → emits `DUPLICATE_STRONG` / `DUPLICATE_REVIEW` / `LOCAL_COMMON_NAME` per gates. (Draft-era wording used a non-canonical `DUPLICATE_WEAK` label; v2 uses `DUPLICATE_REVIEW` — the actual canonical name per §9.)
- Shell safety: fixture under `my $root/` path survives end-to-end.

**Step 5 — dogfood + SKILL.md.** Dogfood on this skill's own repo; register new tests in `update-test-doc.mjs`; brief `## Canon draft mode` subsection in `SKILL.md` (mentions the standalone CLI; orchestrator path noted as P3-4).

Exit criteria: Step 0 test green; `type-ownership.md` emitted on a conforming fixture; every emitted label is in canonical §9; `LOW_INFO_NAMES` drift test green; CLI integration test green; `update-test-doc.mjs` clean.

### 4.2 P3-2 — `helper-registry.md` (Phase 2 of parent)

Deliverables per `docs/spec/SPEC-canon-generator.md` §4.2 Phase 2:

- Extends `_lib/canon-draft.mjs` with helper-specific classifier.
- Reads `call-graph.json` (from `build-call-graph.mjs`) to identify central helpers.
- Emits `canonical-draft/helper-registry.md`.

Dependency: `call-graph.json` must exist. Pipeline step is `build-call-graph.mjs` (already present).

Tests: helper classification rules; barrel / re-export handling for functions (parallels type re-export chain work in P3-1).

Exit criteria: `helper-registry.md` emitted with centrality-ranked helpers; tests green.

### 4.3 P3-3 — `topology.md` (Phase 3 of parent)

Deliverables per `docs/spec/SPEC-canon-generator.md` §4.2 Phase 3:

- Consumes `topology.json` + `triage.json`.
- Emits `canonical-draft/topology.md` — submodule / package-level structure.

Exit criteria: topology draft emitted, workspace boundaries correctly surfaced.

### 4.4 P3-4 — `naming.md` draft + `audit-repo.mjs --canon-draft` integration

Two deliverables. The naming draft follows the parent SPEC §4.2 Phase 4; the orchestrator integration closes the standalone-CLI era by wiring all four generators into `audit-repo.mjs`.

**Naming draft** (per `docs/spec/SPEC-canon-generator.md` §4.2 Phase 4):
- Fresh AST pass for naming conventions (camelCase / PascalCase / kebab-case patterns per file kind).
- Emits `canonical-draft/naming.md` — observed conventions.

**`audit-repo.mjs --canon-draft` integration** (see §5.1 for full contract):
- New flag `--canon-draft` on `audit-repo.mjs`. Not in default profiles.
- `manifest.canonDraft = {requested, ran, reason?, draftPaths?}` block.
- Runs all 4 draft generators on the same output dir. `--sources` optional sub-flag to scope.

Exit criteria: naming draft emitted; `audit-repo.mjs --canon-draft` writes 4 drafts + populates `manifest.canonDraft`; both paths covered by tests.

### 4.5 Out of P3 — `check-canon.mjs` (Post-P3, separate P-phase)

Drift detection against existing `canonical/*.md` is a DIFFERENT problem class: parsing markdown canon + owner-mismatch detection + rename inference. Per `docs/spec/SPEC-canon-generator.md` §4.2 footnote, it requires its own spec. Treat as a later P-phase (call it P5 or Post-P3; the precise number is assigned when the spec starts). **It is NOT a sub-phase of P3** — earlier draft labeled it "P3-5" which wrongly implied P3 ownership. The v2 wording is "Post-P3".

## 5. Integration

### 5.1 `audit-repo.mjs --canon-draft` hook (lands in P3-4)

**The audit-repo integration is a P3-4 deliverable, not P3-1.** P3-1/2/3 ship standalone CLIs (`generate-canon-draft.mjs --source ...`); P3-4 adds the orchestrator flag that chains them.

```
audit-repo.mjs --canon-draft [--sources type-ownership,helper-registry,topology,naming]
```

Flag name `--canon-draft` is deliberate — `--canon` would suggest writing to `canonical/`, which the tool explicitly does NOT do. Not in default profiles; canon draft generation is a deliberate step, not part of routine audits.

`manifest.canonDraft = {requested, ran, reason?, draftPaths?}` — analogous to `manifest.preWrite` / `manifest.postWrite`. P3-4 session decides whether `--canon-draft` is mutually exclusive with `--pre-write` / `--post-write` (probably not — canon draft is orthogonal to lifecycle-stage flags).

**Note on docs/spec/SPEC-canon-generator.md §4.2 Phase 1 success criteria**: if the SPEC still lists `audit-repo.mjs` integration under Phase 1, that wording predates the P3 sub-phase split. The implementation-authority rule (this file wins on sub-phase sequencing; SPEC wins on algorithm) means P3-1 ships WITHOUT the audit-repo hook. A future SPEC v0.2.3 should move the `audit-repo.mjs` success criterion from Phase 1 to Phase 4; this file's §4.1 is the current authority on that sequencing.

### 5.2 SKILL.md

New `## Canon draft mode` subsection after "Post-write delta". Links to `docs/spec/SPEC-canon-generator.md`, lists the 4 `--source` values, mentions `canonical-draft/` non-overwrite versioning.

## 6. Invariants P3 must preserve from P1/P2

- P1 modules unchanged — canon draft consumes `symbols.json` but does NOT modify `build-symbol-graph.mjs`.
- P2 modules unchanged — no coupling to post-write delta.
- `canonical/*.md` unchanged — P3 generates DRAFTS; promotion is manual. A sub-phase that touches `canonical/*.md` directly is out of scope (would be a different tool).
- `execFileSync` argv-array rule for any producer spawning (P1-3 shell-safety).
- FP_BUDGET=0 corpus gate stays at 0. Canon draft additions must not introduce precision regressions in classify-dead-exports path (canon draft does NOT run classify, so coupling is minimal; but E2E audit-repo profile should stay green).
- `canonical/classification-gates.md` is the single source of truth for `LOW_INFO_NAMES` + label set. The `_lib/canon-draft.mjs` mirror must drift-test against it. Any label in the output that is not in canonical §9 is a bug.

## 7. Known risks + open questions

- **Canon-draft output directory convention.** `--canon-output` default differs between standalone (`<root>/canonical-draft/`) vs audit-repo (`<output>/canonical-draft/`). Per parent spec §3.1 — reviewer-reviewed. Pin via CLI test.
- **Non-overwrite versioning.** `<canon-output>/type-ownership.v{N}.md` per §5.5. Race: two concurrent runs could pick the same N. P3-1 v1 uses a simple `max(existing)+1` scan; concurrent-write race is acceptable for dev flow (manual review step follows).
- **`LOW_INFO_NAMES` mirror drift.** Parent spec §5.3 mandates the mirror drift-test. If canonical file is updated, mirror must follow. P3-1 Step 0 (mandatory) lands the test.
- **Identity resolution via `canonical/identity-and-alias.md §6`.** The canonical algorithm is the source of truth. P3 implementation calls into the shared resolver module; it does NOT re-derive resolution logic. Cross-check with existing `_lib/resolver-core.mjs` usage — P3 either reuses or imports from there. Decision deferred to P3-1 session.
- **Existing canon behavior.** If `<root>/canonical/type-ownership.md` already exists, P3-1 emits a draft with the "observational only" header block (§5.4). No drift detection in P3 — that's Phase 5.
- **Star re-export ambiguity.** `canonical/identity-and-alias.md §9` says multi-source name collisions emit `[확인 불가]`. P3 renders these as ambiguous entries, not silent drops.
- **Phase sequencing flexibility.** Parent spec §4.2 says each phase is a reviewable release gate. P3-2/3/4 may need per-session specs of their own (mirror p2-0/1/2). For now, P3-1 is the concrete starting point.

## 8. Non-goals (P3 entire phase)

- No drift detection against existing canon (Phase 5 / separate P-phase).
- No Python / Go canon generators (Phase 6+).
- No auto-promotion from draft → canon.
- No LLM calls / runtime probing.
- No business-logic / domain-intent documentation.
- No `canonical/*.md` edits.
- No P1 / P2 module modifications.
- No local / private type coverage (exported-only per `docs/spec/SPEC-canon-generator.md` §2.1).

## 9. Next session

Proceed to `docs/history/phases/p3/p3-1.md` — the Phase 1 `type-ownership.md` session plan. Sequence for P3-1:

```
Step 0 — drift test (MANDATORY, not optional)
         tests/test-classification-gates.mjs
         - LOW_INFO_NAMES mirror matches canonical 1:1
         - emitted label set matches canonical §9 (9 labels exactly):
           zero-internal-fan-in, low-signal-type-name,
           DUPLICATE_STRONG, DUPLICATE_REVIEW, LOCAL_COMMON_NAME,
           single-owner-strong, single-owner-weak,
           severely-any-contaminated, ANY_COLLISION

Step 1 — _lib/canon-draft.mjs pure helpers
         classifyTypeNameGroup (§2 precedence: ANY_COLLISION → DUPLICATE_STRONG
                                 → LOCAL_COMMON_NAME → DUPLICATE_REVIEW)
         classifySingleIdentity (§4 single-owner + contamination routing)
         escapeMdCell, codeCell
         LOW_INFO_NAMES constant

Step 2 — type-ownership emitter
         identity-keyed aggregation
         consume symbols.json.reExportsByFile (optional)
         read anyContamination from type-owner fact (fact-model §3.1)
         call canonical/identity-and-alias.md §6 resolver
         render per docs/spec/SPEC-canon-generator.md §6

Step 3 — generate-canon-draft.mjs CLI (STANDALONE, no audit-repo hook yet)
         --root / --output / --canon-output / --source type-ownership
         non-overwrite versioning (v{N})
         existing-canon header block

Step 4 — tests
         (Step 0 drift test already green from Step 0)
         unit: identity aggregation + duplicate detection
         integration: DUPLICATE_STRONG / DUPLICATE_REVIEW / LOCAL_COMMON_NAME
                      fixtures (NOT the draft-era DUPLICATE_WEAK;
                      v2 uses canonical §9 names)
         shell safety: my $root path

Step 5 — dogfood on self + SKILL.md subsection
         SKILL.md mentions standalone CLI path; notes audit-repo --canon-draft
         as P3-4 deliverable (not available yet in P3-1)
```

Each step ends in a reviewable green state. Scope budget: the parent SPEC is already dense; P3-1 should be a writing/wiring session, not a design session. If reviewer feedback requires re-opening design questions, revise `docs/spec/SPEC-canon-generator.md` → v0.2.3 rather than editing docs/history/phases/p3/p3-1.md.

## 10. Handoff to P4 / later

- **P4 shape-hash** — independent of P3. Shape-hash produces a separate fact kind (structural-identity-of-a-type) that would feed into canon-draft `DUPLICATE_*` classification, but P3-1 can ship without it. Integration point: when P4 lands, extend `classifyTypeNameGroup` to read shape-hash as an additional signal.
- **`check-canon.mjs` (Phase 5)** — its own spec and phase. Depends on all four draft shapes being stable.
- **Per-repo `LOW_INFO_NAMES` override config** — on the roadmap per parent spec §5.3. Lives in canonical file evolution, not P3.
