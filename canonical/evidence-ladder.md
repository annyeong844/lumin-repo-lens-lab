# canonical/evidence-ladder.md

> **Role:** cross-language evidence confidence and coverage contract. Defines what the skill may claim from syntax, semantic, rule-backed, unavailable, and stale evidence.
> **Owner:** this file plus `canonical/oracle-registry.json`.
> **Status:** spine v1.
> **Last updated:** 2026-06-17

---

## 1. Rule

```text
SYNTAX EVIDENCE IS A CANDIDATE, NOT A FACT.
VERIFIED CLAIMS REQUIRE AN ORACLE WITH COVERAGE.
NOT RUN IS NOT CLEAN.
STALE VERIFIED EVIDENCE IS NOT VERIFIED.
SILENCE REFUTES NOTHING UNLESS THE REGISTRY SAYS IT DOES.
```

This file generalizes rules already present in the TS/JS path:

- fast syntax passes collect candidates;
- compiler-owned or runtime-owned oracles confirm only what they are authorized to confirm;
- unsupported or unobserved cases produce diagnostics, not fake facts;
- missing capability never means clean.

The product is not a lint runner. It is an evidence engine. Tool names are provenance. Claims are controlled by confidence, coverage, oracle authority, and the analysis input set.

## 2. Three Axes

Every claim-bearing finding or absence claim must keep these axes separate.

| Axis | Scope | Examples |
|---|---|---|
| `source` | Which tool emitted the evidence. | `oxc-parser`, `typescript`, `ra_ap_syntax`, `cargo-check`, `vulture`, `ty` |
| `confidence` | What a finding may claim. | `verified`, `rule-backed`, `candidate` |
| `coverage` | Whether the relevant scope was actually analyzed by the relevant oracle. | `ran`, `not-run`, `unavailable`, `unsupported` |

`unanalyzed` is never a confidence. If nothing analyzed a region, there is no finding confidence. There is only coverage.

`sourceKind` is provenance, not epistemic permission. Use values such as `syntax`, `syntax-heuristic`, `semantic-oracle`, and `rule-engine`. Do not use `sourceKind` as a shortcut for confidence.

Coverage is a region-by-oracle matrix, not a file-level boolean. There is no standalone `file.clean`. There is only "clean for oracle X, authority Y, scope Z, and analysis input set H."

## 3. Confidence

### 3.1 `verified`

Use `verified` only when an oracle confirms a fact inside its declared authority.

Examples:

- TypeScript compiler config resolution confirming `compilerOptions.paths`.
- `cargo check --message-format=json` confirming Rust type, borrow, name-resolution, or cfg diagnostics for user code spans.
- `ty` confirming a Python static type diagnostic inside a configured analysis scope.

`verified` is always `verified-about-what`, not a universal truth. An oracle registry entry must declare `authorityIds` and `claimKinds`.

### 3.2 `rule-backed`

Use `rule-backed` for deterministic rule output that can be useful but is not compiler truth.

Examples:

- A style or correctness rule.
- A framework convention detector.
- A generated-file classifier.

Rule-backed evidence may be high priority, but it is still not `verified`. Display severity and epistemic confidence are separate.

### 3.3 `candidate`

Use `candidate` for syntax-only or heuristic prompts.

Examples:

- `ra_ap_syntax` sees `.clone()`.
- Vulture reports a possibly unused Python symbol.
- OXC sees a type shape that needs semantic contamination checks before reuse is safe.

Candidate output may ask a review question. It must not assert semantic truth.

## 4. Coverage

Coverage is attached to an oracle and a scope. Clean can appear only inside `ran`.

```json
[
  {
    "status": "ran",
    "clean": true,
    "cleanKind": "verified-rustc-error-absence",
    "absenceOfClaimKinds": [
      "verified.rust.rustc-error-diagnostic",
      "verified.rust.rustc-codeless-error-diagnostic"
    ],
    "cleanScope": "rust.cargo-check verified rustc error diagnostics for package+target+featureSet+targetTriple+cfgSet+profile"
  },
  {
    "status": "ran",
    "clean": false,
    "cleanKind": "verified-rustc-error-absence",
    "cleanScope": "rust.cargo-check verified rustc error diagnostics for package+target+featureSet+targetTriple+cfgSet+profile"
  },
  { "status": "not-run", "reason": "semantic oracle not requested" },
  { "status": "unavailable", "reason": "toolchain missing" },
  { "status": "unsupported", "reason": "oracle does not support this language surface" }
]
```

Definitions:

- `ran`: the oracle executed over the declared scope. `clean` means absence of the declared claim kinds in that scope, authority, and analysis input set.
- `not-run`: the oracle was not requested. This is not clean.
- `unavailable`: the oracle is supported, but the environment or inputs prevented completion. Examples: missing toolchain, offline dependency fetch, unbuildable crate graph.
- `unsupported`: the oracle does not support the requested language, syntax surface, or analysis target.

Build or configuration failures are usually coverage failures, not findings. A Rust dependency build failure is not a verified defect in user code unless the diagnostic has a user-code primary span and the oracle authority covers it.

Requested cfg, feature, package, or target surfaces that were not selected are `not-run`, not clean.

For Rust, `cargo check` coverage is not just `crate-target`. It is a crate target configuration: package, target, feature set, target triple, cfg set, profile, cargo version, and rustc version. A clean default-feature host-target run says nothing about non-default features or disabled cfg code. If `cfgSetComplete` is false, render the cfg scope as best-effort, never as an exhaustive cfg claim.

## 5. Partial Semantic Runs

Semantic tools can emit useful diagnostics before they fail to complete full absence coverage. Rust is the common case:

- `cargo check` can emit a user-code primary diagnostic and later fail because a dependency, build script, proc macro, toolchain, or target setup failed.
- The emitted user-code diagnostic may be a `verified` finding if it is inside `rust.cargo-check` authority.
- The crate-target absence claim remains `unavailable`; it must not become `ran.clean`.
- `cargo check --message-format=json` mixes diagnostic classes. `level: "error"` user-code primary diagnostics may be `verified`; rustc lint diagnostics are `rule-backed`.
- Cargo diagnostic code namespace is derived before confidence classification: `message.code === null` is `rustc-codeless`, `message.code.code` matching `^E[0-9]+$` is `rustc-error`, any other non-empty `message.code.code` is `rustc-non-ecode`, and malformed or missing code is `unknown`.
- Lint classification is namespace-first, not level-first. If a rustc lint is denied and arrives as `level: "error"`, it remains `rule-backed`.
- Codeless rustc errors may be `verified` only when they carry a user-code primary span. Summary, note, help, and failure-note messages are non-findings.
- User-code primary span means the primary span resolves to a workspace member source file, excluding `target/` and `OUT_DIR` generated files. Macro expansion chains resolve to the nearest user-written span; if none exists, the diagnostic is not user-code-primary for verified classification.
- Non-user-code primary diagnostics are not user-facing findings. Non-user-code primary error diagnostics make absence-clean coverage unavailable for the selected user scope.
- Unmatched real user-code error or warning diagnostics remain visible as diagnostics or candidates. They must not be promoted to `verified`.
- An empty diagnostic stream is not clean. Rust clean coverage requires a completely parsed cargo JSON stream with zero invalid JSON lines and a `build-finished` message with `success: true` for the same scope and analysis input set.

Keep these ledgers separate:

- `findings[]`: emitted diagnostics and review signals with their own source, confidence, authority, and span.
- `coverage[]`: oracle execution state for absence and clean claims over scopes.

One verified finding does not prove complete verified coverage.

## 6. Staleness

Verified evidence must carry provenance sufficient to reject stale reuse.

Required provenance:

- oracle id and version;
- command or API mode;
- policy version;
- registry content hash;
- scan scope;
- `analysisInputSetHash`;
- generated timestamp;
- coverage result.

Use `analysisInputSetHash`, not a loose file hash, for semantic evidence.

Examples of semantic input set members:

- Rust: local package Rust source bytes, `Cargo.toml`, `Cargo.lock`, `rust-toolchain*`, selected package, selected target, feature flags, target triple, cfg set, profile, cargo/rustc versions, relevant cargo config, environment values that affect compilation, and build script or proc-macro influence.
- Python: source bytes, `pyproject.toml`, ty configuration, Python version, import roots, dependency and stub search paths, and ty version.
- TS/JS: `tsconfig`, extends chain, `baseUrl`, `paths`, package boundary files, compiler version, and compiler API mode.

If the analysis input set hash for a scope no longer matches, prior verified evidence becomes a coverage gap. It must not remain verified.

Build script and proc-macro influence is not always statically knowable. When a crate uses build scripts or proc macros and the complete influence set is not captured, semantic cache reuse is disabled for that scope. The analyzer must rerun the oracle or mark coverage unavailable; it must not reuse verified semantic results from an incomplete hash.

## 7. Oracle Registry

`canonical/oracle-registry.json` is the single source of truth for:

- oracle ids;
- language surfaces;
- source kind;
- confidence tier;
- machine-readable `authorityIds`;
- machine-readable `claimKinds`;
- candidate kinds;
- explicit `refutesCandidateKinds`;
- analysis input set members;
- coverage behavior;
- stale policy;
- initial rollout status.

This markdown explains the contract. The registry defines the concrete oracle slots. Future validators should check code and docs against the registry, not hand-maintained prose.

Artifact metadata must include `oracleRegistryVersion` and `registryContentHash` so a finding can be tied to the exact registry that authorized its claim.

Registry source-of-truth is not a slogan. Validation should check at least:

- markdown oracle ids are present in `oracle-registry.json`;
- prose confidence tier names are present in `confidenceTiers`;
- implementation-emitted oracle ids, claim kinds, and candidate kinds are registry-listed;
- rendered refutations cite a registry-listed `refutesCandidateKinds` entry.

## 8. Refutation

Silence from a stronger oracle does not automatically refute lower-tier candidates.

The only allowed refutation path is:

1. the stronger oracle has `coverage.status === "ran"` for the declared scope;
2. the candidate kind and scope relation appear in that oracle's `refutesCandidateKinds`;
3. the analysis input set hash matches the candidate scope;
4. the renderer cites the refuting oracle and authority id.

M7 starts conservative: semantic silence does not refute syntax candidates unless the registry explicitly lists that relation. This prevents “cargo check was quiet” from becoming “`.clone()` is cheap” or “deadness is disproved.”

When the first refutation relation is added, use this object shape:

```json
{
  "candidateOracleId": "rust.ra-ap-syntax",
  "candidateKind": "rust.parse-error.syntax",
  "scopeRelation": "same-file",
  "requiresCoverageStatus": "ran",
  "authorityId": "rust.rustc.cfg-expanded-diagnostic"
}
```

## 9. Language Mapping

| Language | Candidate layer | Verified semantic layer | Deferred rule-backed layer |
|---|---|---|---|
| TS/JS | OXC parser facts | TypeScript compiler API where configured | ESLint is not a current product oracle |
| Rust | `ra_ap_syntax` | `cargo metadata` + `cargo check --message-format=json`; `rust.ra-hir` reserved as a deferred middle tier | Clippy, deferred until rule-backed policy exists |
| Python | Vulture-style candidate collection | `ty` static type oracle; `pyright` reserved as a deferred higher-conformance alternative | Ruff or equivalent, deferred |

These mappings are examples of the registry. Downstream code should use registry ids and confidence fields, not language-specific if/else trees.

Python `ty` must be version-pinned before clean claims depend on it. Diagnostic code or format drift is a registry and policy change, not an invisible implementation detail.

`ty` silence is narrow: no ty diagnostics means ty-clean only under ty's configured model. It is not Python type-clean in general.

`rust.ra-hir` is intentionally reserved but deferred. It fills the future gap between syntax-only candidates and full `cargo check` for offline or unbuildable repos, but it is not part of the first M7 implementation.

## 10. Rendering Rules

- `verified` may be stated as a grounded finding, bounded by authority.
- `rule-backed` may be rendered as rule evidence with its rule class.
- `candidate` may be rendered only as a review prompt.
- `not-run`, `unavailable`, and `unsupported` must be visible when a clean or absence claim would otherwise be tempting.
- A lower tier can never promote itself to a higher tier.
- Silence from a higher-tier oracle does not erase lower-tier candidates unless the registry explicitly allows that candidate kind to be refuted under matching coverage.
- `clean: true` must always render with the oracle, scope, authority, and analysis input set that made it true.
