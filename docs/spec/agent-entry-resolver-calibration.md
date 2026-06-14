# Agent Entry, Resolver Completeness, And Calibration

> **Role:** maintainer-facing cross-cutting contract and backlog anchor for
> reducing adoption friction while preserving Lumin Repo Lens' evidence
> contract.
> **Status:** design draft, implementation deferred.
> **Last updated:** 2026-05-08

---

## 1. Problem

Lumin Repo Lens is a plugin/skill for coding agents, not only a standalone
static analyzer. If the useful path requires users to remember manual slash
commands such as:

```text
/lumin-repo-lens-lab:pre-write
```

then many agent sessions will skip the tool entirely. The engine can be
accurate and still fail as a product if the entry point is too manual.

At the same time, JS/TS static analysis depends on module resolution. Import
resolution has many legitimate shapes:

- relative and absolute paths,
- packages from `node_modules`,
- `tsconfig` / `jsconfig` paths,
- workspace packages,
- `package.json#exports` subpaths and conditions,
- `main` / `module` / `types` / `browser`,
- Node.js `#imports`,
- extensionless and directory imports,
- JSON imports,
- generated or virtual imports,
- conditional browser/node surfaces,
- re-export aliases.

Missing any one family can create blind zones. Treating those blind zones as
"no consumer found" breaks the engine's absence-claim contract.

Finally, several existing ranking or cue policies contain numeric thresholds.
Thresholds such as score cutoffs, body-location similarity, or occurrence
limits may be reasonable, but they are hard to trust if the artifact does not
say which policy version chose them, what evidence calibrated them, and which
corpus was used.

This spec captures that product/accuracy debt so it is not lost while the
generated-artifact and PCEF work continues.

This document is an umbrella contract. It does not by itself define a complete
implementation shape for resolver logic, agent hooks, or calibration harnesses.
Each phase that changes analyzer behavior must either add a narrower
implementation spec or extend this document with concrete artifact shapes,
fixtures, and acceptance tests first.

## 2. Goals

- Reduce manual-trigger friction for agent workflows without requiring a
  daemon or editor-specific integration in the first slice.
- Define resolver completeness as an explicit capability matrix, not an
  ad hoc set of one-repository fixes.
- Ensure unresolved or unsupported resolver families degrade to structured
  blind-zone evidence rather than false absence claims.
- Make numeric thresholds versioned, documented, and calibration-backed.
- Keep the engine usable as a plugin/skill: expensive full-profile evidence
  should remain opt-in or scheduled, while lightweight gate evidence should
  stay suitable for agent loops.
- Preserve existing proof contracts: this spec does not lower `SAFE_FIX`,
  `SAFE_CUE`, or generated-artifact precision bars.

## 3. Non-goals

- Do not implement a daemon, file watcher, editor extension, or background
  service in this spec.
- Do not make `quick` equivalent to `full`.
- Do not run build tools or generators by default.
- Do not add embeddings, broad synonym dictionaries, or semantic duplicate
  claims.
- Do not overfit resolver behavior to `@calcom/*`, a single monorepo, or one
  package manager layout.
- Do not promote `SAFE_FIX` from resolver confidence alone. Action proof and
  clean deadness proof remain required.

## 4. Definitions

**Agent entry point:** A way an AI coding agent knows when and how to invoke the
tool. Manual slash commands are one entry point. Skill metadata, default
prompts, host hooks, or structured advisory protocols are other possible entry
points.

**Resolver family:** A class of import/export resolution behavior, such as
`tsconfig paths`, `workspace package subpath`, `Node #imports`, or
`conditional exports`.

**Resolver capability matrix:** A versioned table describing each resolver
family's status: supported, partially supported, unsupported, or intentionally
out of scope.

**Threshold policy:** A versioned bundle of numeric cutoffs and the evidence
used to justify them.

**Calibration corpus:** A named set of repositories or fixtures used to measure
precision, recall proxies, runtime, and user-facing noise.

## 5. Agent Entry Contract

The public user experience should not depend on the user remembering every
manual command.

Manual commands remain valid:

```text
/lumin-repo-lens-lab
/lumin-repo-lens-lab:pre-write
/lumin-repo-lens-lab:post-write
```

But the skill surface should also tell agents when to invoke the engine:

- before creating a new helper, type, file, or public export;
- after a patch that changes TS/JS source;
- before accepting dead-export cleanup suggestions;
- before large refactors that move code across module boundaries;
- before trusting generated code absence claims.

This is an agent guidance contract, not an automatic blocking policy.

### 5.1 Agent invocation decision table

The skill surface should eventually express the intended entry points as a
decision table that agents can follow:

| Agent intent | Suggested command | Profile | Reason |
|---|---|---|---|
| Create a new helper, function, type, file, or public export | `/lumin-repo-lens-lab:pre-write` | quick | Check existing names, files, dependencies, shapes, and review cues before writing. |
| Apply or review a TS/JS source patch | `/lumin-repo-lens-lab:post-write` | quick | Compare before/after artifacts and detect unplanned type escapes or cue drift. |
| Remove an export or accept dead-export cleanup | `/lumin-repo-lens-lab` or full audit flow | full when practical | Needs resolver, public surface, action-safety, and generated/blind-zone evidence. |
| Move code across packages or module boundaries | `/lumin-repo-lens-lab` | full or targeted full | Needs import graph, resolver diagnostics, and public contract evidence. |
| Trust a generated-code absence claim | `/lumin-repo-lens-lab` | full | Needs generated artifact and resolver blind-zone diagnostics. |

### 5.2 Minimum viable improvement

The first improvement should be metadata and prompt guidance, not a daemon:

- tighten skill/default prompt wording so agents know when to run pre-write and
  post-write checks;
- add examples that pair common coding intents with the relevant command;
- expose advisory output paths clearly enough that an agent can call the tool
  once and keep using the artifacts in the same session.

### 5.3 Prompt-lint wording

Agent guidance must not imply claims the engine cannot prove.

Disallowed wording:

- automatically safe
- no consumers exist
- guaranteed unused
- blocking failure
- resolver complete
- semantically equivalent

Allowed wording:

- run a check
- review cue
- constructed graph
- unresolved resolver family
- evidence artifact
- no consumer was found in the constructed graph

Prompt-lint tests should fail if skill guidance, command docs, or generated
advisories imply automatic blocking, semantic certainty, or full resolver
completeness.

### 5.4 Future entry points

Future host-specific integrations may add automatic hooks, but those hooks must
consume the same public commands and artifacts. They must not introduce a
separate hidden analysis mode.

Any future automatic trigger must report:

```json
{
  "triggerSource": "manual-command | skill-guidance | host-hook | script",
  "triggerIntent": "pre-write | post-write | audit",
  "blocking": false,
  "analysisProfile": "quick | full | custom",
  "workingTreeState": "clean | dirty | unknown",
  "artifactFreshness": "fresh | stale | unknown"
}
```

In this spec, cue tiers do not change exit behavior.

## 6. Resolver Completeness Contract

The resolver should be designed as a capability matrix. Each resolver family
must have one of these statuses:

```text
supported
partial
unsupported
deferred
not_applicable
```

For `partial`, `unsupported`, and `deferred`, the engine must preserve an
explicit reason code when the family can affect an absence claim.

### 6.1 Static capability matrix vs per-run diagnostics

Analyzer capability and per-run repository evidence are different artifacts.
They may be summarized in `manifest.json`, but their full records should not be
mixed.

Recommended split:

```text
manifest.json
  summary only:
    resolverVersion
    resolverCapabilityArtifact
    resolverDiagnosticsArtifact
    top unresolved / unsupported family counts

resolver-capabilities.json
  static engine capability matrix for this analyzer version

resolver-diagnostics.json
  per-run unresolved imports, unsupported families, blind zones,
  candidate targets, and blocked absence claims

symbols.json
  symbol and graph evidence; may reference resolver artifacts but should not
  own the resolver capability matrix
```

The static capability matrix describes what the engine can evaluate. Per-run
diagnostics describe what actually happened in one repository.

### 6.2 Static capability artifact shape

`resolver-capabilities.json` should be deterministic and versioned:

```json
{
  "schemaVersion": "resolver-capabilities.v1",
  "resolverVersion": "resolver-2026-05-v1",
  "conditionProfiles": [
    {
      "profileId": "node-esm-default",
      "conditions": ["node", "import", "default"],
      "configuredBy": "default"
    }
  ],
  "families": [
    {
      "family": "tsconfig-paths",
      "status": "partial",
      "supportedCases": ["baseUrl", "single-star paths"],
      "unsupportedCases": [
        "ambiguous multi-target fallback",
        "project-reference redirected output"
      ],
      "reasonCodes": [
        "tsconfig-path-target-missing",
        "tsconfig-path-target-ambiguous"
      ],
      "absenceClaimPolicy": "fail-closed-when-relevant",
      "fixtureRefs": ["resolver-tsconfig-paths-basic"]
    },
    {
      "family": "node-imports",
      "status": "unsupported",
      "supportedCases": [],
      "unsupportedCases": ["package-local #imports maps"],
      "reasonCodes": ["node-imports-unsupported"],
      "absenceClaimPolicy": "fail-closed-when-encountered",
      "fixtureRefs": ["resolver-node-imports-unsupported"]
    }
  ]
}
```

A family with `status: "partial"` must include:

- `supportedCases`,
- `unsupportedCases`,
- `reasonCodes`,
- `absenceClaimPolicy`,
- at least one fixture or diagnostic reference.

### 6.3 Per-run diagnostics artifact shape

`resolver-diagnostics.json` should describe repository-specific resolver
events:

```json
{
  "schemaVersion": "resolver-diagnostics.v1",
  "resolverVersion": "resolver-2026-05-v1",
  "capabilityArtifact": "resolver-capabilities.json",
  "blindZones": [
    {
      "family": "conditional-exports",
      "reason": "condition-profile-ambiguous",
      "importer": "packages/app/src/client.ts",
      "specifier": "example-package/subpath",
      "conditionsSeen": ["browser", "node", "import", "require", "default"],
      "affectedPackageScope": "packages/example-package",
      "blocksAbsenceClaims": true,
      "relevance": "same-package-or-provider-edge",
      "blockedCandidates": [
        {
          "file": "packages/example-package/src/subpath.ts",
          "exportName": "foo"
        }
      ]
    }
  ],
  "candidateTargets": [
    {
      "specifier": "example-package/subpath",
      "importer": "packages/app/src/client.ts",
      "family": "package-json-exports",
      "candidatePaths": ["packages/example-package/src/subpath.ts"],
      "notResolvedBecause": "condition-profile-ambiguous"
    }
  ]
}
```

### 6.4 Required resolver families

The matrix should include at least:

| Family | Required behavior |
|---|---|
| Relative paths | Resolve extensionless and directory imports across JS/TS extensions. |
| Absolute project paths | Resolve only when a base URL or package root contract exists. |
| Node packages | Distinguish external packages from internal workspace packages. |
| `tsconfig` / `jsconfig` paths | Follow extends chains and path target ordering. |
| Workspace packages | Resolve package root and source-direct package entries. |
| `package.json#exports` | Respect subpaths, wildcards, and condition maps conservatively. |
| `main` / `module` / `types` / `browser` | Use as package entry candidates, with mode/provenance labels. |
| Node `#imports` | Resolve package-local import maps when present. |
| JSON imports | Record file-level edges without pretending named JS exports exist. |
| Generated / virtual imports | Follow `docs/spec/generated-artifact-support.md`. |
| Conditional exports | Carry condition context; do not choose browser/node silently when ambiguous. |
| Re-export aliases | Preserve exported-name to definition identity mapping. |

This matrix must be generic. A fixture may reproduce cal.com-like behavior, but
the rule must be expressed as workspace package + source-direct subpath
resolution, not as a literal `@calcom/*` rule.

JSON imports deserve one special invariant: a JSON import may establish
file-level reachability, but it must not create symbol-level JS export identity
unless a supported transform or generated artifact explicitly provides that
mapping.

Conditional exports must record the condition profile used for a decision. If
the engine cannot choose a condition profile without guessing between browser,
node, import, require, default, or custom conditions, it must emit a reason such
as `condition-profile-ambiguous`.

### 6.5 Resolver output levels

A resolver attempt can produce:

```text
resolved
candidate
unresolved_with_reason
unsupported_family
external
```

Only `resolved` may create concrete graph edges.

`candidate` may support diagnostics, but it is not proof that the file exists
or was scanned.

`unresolved_with_reason` and `unsupported_family` can create blind-zone records
and block unsafe promotion when relevant.

`candidate` means the resolver identified one or more plausible target paths
without satisfying all requirements for a concrete graph edge.

A candidate target:

- must not create a consumer edge;
- must not satisfy reachability evidence;
- must not clear deadness;
- may only appear in diagnostics or review wording;
- may cause fail-closed behavior if it overlaps the candidate under review.

If the file is confirmed to exist and the selected resolution mode is fully
supported, the output should be `resolved`, not `candidate`. If multiple
possible targets remain after applying supported rules, the output should be
`candidate` or `unresolved_with_reason`, depending on whether the resolver can
enumerate plausible targets.

### 6.6 Candidate relevance rule

A resolver blind zone is candidate-relevant only when the unresolved or
unsupported resolver family could plausibly affect the candidate's package,
file, exported name, or re-export identity.

A blind zone is relevant to a candidate when at least one of these is true:

1. The unresolved specifier names the same workspace package as the candidate's
   nearest package.
2. The unresolved specifier is a relative or absolute path whose normalized
   candidate target could fall under the candidate file's package scope.
3. The unresolved specifier is a package subpath that maps to the same package
   root as the candidate under at least one supported package/workspace rule.
4. The unresolved family affects export-name identity for a module that
   re-exports or may re-export the candidate's definition.
5. The resolver cannot decide whether the import is external or internal, and
   the specifier prefix overlaps a known workspace package name.

Blind zones must be recorded globally, but they must not become repo-global
blockers unless the resolver cannot determine package ownership or
internal/external status. If the unresolved specifier cannot plausibly target
the candidate package, it should remain diagnostic evidence and should not
block that candidate's absence claim.

### 6.7 Absence-claim rule

If a resolver family is relevant to a candidate export and the engine cannot
evaluate it, the result must not be phrased as:

```text
no consumer exists
```

It may be phrased as:

```text
no consumer was found in the constructed graph; resolver family X was incomplete
```

This applies to `SAFE_FIX`, pre-write reuse cues, reachability boosters, and
call-graph evidence.

## 7. Threshold Calibration Contract

Numeric thresholds are allowed, but they must not be invisible magic numbers.

Every threshold used for ranking, cue rendering, suppression, similarity, or
candidate pruning should belong to a versioned policy object:

```json
{
  "policyId": "prewrite-cue-policy",
  "policyVersion": "prewrite-cue-policy-v1",
  "policyHash": "sha256:...",
  "thresholds": {
    "nearNameScoreMin": 0.62,
    "bodyLocSimilarityMin": 0.34,
    "minInlineOccurrences": 3
  },
  "calibration": {
    "corpus": "calibration-2026-05",
    "lastMeasured": "2026-05-08",
    "notes": "threshold rationale lives in maintainer calibration notes"
  }
}
```

The artifact does not need to repeat all calibration data, but it should expose
the policy version and enough metadata for maintainers to know which policy was
used.

No bare numeric threshold may affect ranking, rendering, suppression, pruning,
or promotion unless it belongs to a named policy object. Tests should fail when
a threshold value changes without a `policyVersion` change or an updated
calibration-note reference.

### 7.1 Threshold classes

Thresholds should be grouped by purpose:

- `promotion`: affects `SAFE_FIX` or other high-confidence labels;
- `review`: affects `AGENT_REVIEW_CUE` or review surfacing;
- `suppression`: hides noisy cues by default while preserving diagnostics;
- `pruning`: caps output volume or candidate enumeration;
- `performance`: limits work to keep the skill usable in agent loops.

Promotion thresholds require the strongest calibration. Review and suppression
thresholds may iterate faster, but they still need policy versions and snapshot
tests.

Promotion thresholds must not promote to `SAFE_FIX` by score alone. They may
rank or suppress candidates, but `SAFE_FIX` still requires explicit proof
objects for deadness, contract safety, and action safety.

### 7.2 Calibration corpus split

Calibration should avoid one-repo overfit:

- small fixtures for deterministic edge cases;
- accepted calibration corpora with zero-regression expectations;
- stress repos such as large workspaces for runtime and unresolved coverage;
- external sample repos for noise checks.

One repository may motivate a fixture, but a policy should not be accepted
because it works on that repository alone.

Calibration corpora should have a manifest:

```json
{
  "corpusId": "calibration-2026-05-prewrite-v1",
  "entries": [
    {
      "kind": "fixture",
      "name": "resolver-conditional-exports-basic",
      "revision": "fixture:v1"
    },
    {
      "kind": "repo",
      "name": "large-workspace-sample-a",
      "revision": "git:abcdef",
      "purpose": "runtime-and-unresolved-coverage"
    }
  ],
  "metrics": [
    "precisionProxy",
    "noiseRate",
    "runtimeMs",
    "unresolvedImportRate"
  ]
}
```

Stress-repo runtime findings must not be treated as correctness acceptance.

## 8. Relationship To Existing Specs

This spec does not replace:

- `docs/spec/generated-artifact-support.md`
- `docs/spec/proof-carrying-export-fix.md`
- `docs/spec/proof-carrying-export-fix-implementation-plan.md`
- `docs/spec/pre-write-inline-extraction-cues.md`
- `docs/spec/incremental-engine-architecture.md`

Instead, it records cross-cutting product and evidence constraints:

```text
agent entry friction
+ resolver completeness
+ threshold calibration provenance
```

Generated artifact work should continue on its own spec. PCEF should continue
to own `SAFE_FIX` proof contracts. Pre-write cue specs should continue to own
cue semantics. This document only prevents those tracks from forgetting the
larger adoption and calibration debt.

## 9. Implementation Phases

### P0: Documentation and inventory

- Add this spec and keep it linked from `docs/spec/README.md`.
- Inventory current resolver families and threshold constants.
- Mark each threshold with an owner policy name, even before all calibration
  notes exist.
- Document current manual entry points and intended agent-trigger guidance.
- No analyzer behavior change.

### P1: Resolver capability matrix artifact

- Add `resolver-capabilities.json` schema and deterministic serialization.
- Add `resolver-diagnostics.json` schema and deterministic serialization.
- Add manifest summary pointers to both artifacts.
- Preserve per-family support status and relevant reason codes.
- Add fixtures for workspace source-direct subpaths, conditional exports,
  `#imports`, JSON imports, and generated misses.

### P2: Agent entry guidance

- Update skill and README guidance so agents know when to invoke pre-write,
  post-write, and audit flows.
- Keep CLI exit behavior unchanged.
- Add prompt-lint tests so future wording does not imply semantic certainty or
  automatic blocking.

### P3: Threshold policy metadata

- Move numeric cutoffs into named policy objects.
- Emit policy versions in affected artifacts.
- Add tests that fail when a threshold changes without a policy-version update
  or calibration note update.
- Add policy hashes where artifact size permits.

### P4: Calibration harness

- Define calibration corpora and measurement commands.
- Track precision/noise/runtime metrics separately.
- Keep stress-repo runtime findings separate from correctness acceptance.

## 10. Acceptance Criteria

- The spec exists and is linked from `docs/spec/README.md`.
- No implementation claims automatic triggering, full resolver completeness, or
  calibrated threshold certainty before the corresponding phases exist.
- Resolver diagnostics can distinguish unsupported families from ordinary
  unresolved imports.
- Static resolver capability matrix and per-run resolver diagnostics are
  deterministic and separately serialized or clearly separated by schema.
- An unsupported Node `#imports` specifier creates resolver diagnostics and does
  not create a concrete graph edge.
- An unresolved workspace package subpath that overlaps a candidate package can
  block `SAFE_FIX` wording for that candidate.
- An unresolved import in an unrelated external package must not become a
  repo-global blocker.
- Conditional exports with no configured condition profile produce
  `condition-profile-ambiguous` and block only relevant absence claims.
- JSON imports create file-level reachability only and no named JS export
  identity.
- Threshold-changing PRs must identify the policy and expected behavior change.
- A threshold change without `policyVersion` or calibration-note update fails.
- Artifacts affected by thresholds emit `policyId` and `policyVersion`.
- Prompt-lint fails if skill guidance says or implies automatic blocking,
  semantic certainty, or full resolver completeness.
- Public docs avoid implying that users must manually know every command before
  receiving value from the skill.

## 11. Current Recommendations And Open Questions

Current recommendation for agent entry:

- Use skill guidance, examples, and prompt-lint first.
- Defer host hooks until they can consume the same public commands and
  artifacts.
- Keep daemon/file-watch behavior out of scope.

Current recommendation for resolver artifacts:

- `manifest.json`: summary and artifact pointers.
- `resolver-capabilities.json`: static analyzer capability matrix.
- `resolver-diagnostics.json`: per-run unresolved/unsupported/blind-zone
  evidence.
- `symbols.json`: graph and symbol facts; resolver references only.

Current recommendation for the first calibration corpus:

- `calibration-2026-05-prewrite-v1`.
- Include deterministic fixtures, one medium TS/JS workspace fixture, one large
  workspace stress fixture, and one external sample set for noise checks.

Open question:

- Which thresholds remain necessary after resolver and cue evidence become more
  structured? The expected direction is to reduce promotion thresholds and keep
  review, suppression, pruning, and performance thresholds where they serve
  artifact usability.
