# Generated Artifact Support

> **Role:** maintainer-facing design spec for handling imports that point at
> generated artifacts in source checkouts.
> **Status:** design draft, implemented through diagnostic taxonomy and
> explicit generated consumer blind-zone inventory.
> **Last updated:** 2026-05-06

---

## 1. Problem

Large TS/JS workspaces sometimes import generated package subpaths:

```ts
import { BookingStatus } from "@calcom/prisma/enums";
```

In a source checkout, that target may not exist until a generator runs. The
resolver must not pretend the file exists. At the same time, reporting every
case as a generic workspace subpath miss hides useful evidence. Maintainers
need to know whether a miss is:

```text
ordinary source subpath missing
vs.
generated artifact probably absent from checkout
```

This distinction matters because the remediation is different.

- Ordinary source miss: improve workspace/package/subpath resolution or fix the
  import.
- Generated artifact miss: obtain generated artifacts, run an explicit generator
  path, or add a generator-specific virtual surface.

## 2. Goals

- Keep default static analysis honest: absent generated artifacts remain a blind
  zone, not a successful resolution.
- Classify likely generated-artifact misses separately from ordinary workspace
  subpath misses.
- Provide opt-in paths for projects that want generated artifacts included in
  the graph.
- Keep generator execution out of the default scan path.
- Preserve `Tier != claim`: unresolved generated artifacts must block promotion
  when they could hide consumers.

## 3. Non-goals

- Do not run package scripts, `prisma generate`, framework compilers, or build
  tools by default.
- Do not synthesize arbitrary modules from naming conventions alone.
- Do not make one repository's generated layout a global resolver rule.
- Do not treat a generated-artifact miss as evidence that an export is unused.
- Do not add semantic inference or embeddings.

## 4. Definitions

**Generated artifact:** A file or module expected to be produced by a generator,
compiler, or package script, and not necessarily committed to source control.

**Generated surface:** The exported names and module paths exposed by generated
artifacts when those artifacts are present or can be derived from a supported
generator input.

**Virtual generated module:** A conservative in-memory module surface derived
from a supported generator input, such as a schema file. A virtual module is not
a source file and must be labeled as generated evidence.

**Generated miss:** An unresolved internal specifier whose matched package or
target path has strong generated-artifact evidence.

## 5. Default Contract

Default scans must classify but not resolve missing generated artifacts.

```text
@calcom/prisma/enums
  -> UNRESOLVED_INTERNAL
  -> reason: workspace-generated-artifact-missing
  -> hint: generated-artifact-missing
```

This is intentionally not:

```text
@calcom/prisma/enums
  -> packages/prisma/enums.ts
```

unless that file exists or an explicit generated-artifact mode provides a
verified surface.

## 6. Reason Taxonomy

The resolver uses reason codes to keep blind zones explainable:

| Reason | Meaning |
|---|---|
| `workspace-package-subpath-target-missing` | Workspace package subpath matched, but no source target was found. No generated-artifact evidence was strong enough. |
| `workspace-generated-artifact-missing` | Workspace package subpath matched, no target was found, and package metadata or target shape strongly suggests a missing generated artifact. |
| `tsconfig-path-target-missing` + `hint=generated-artifact-missing` | A tsconfig path matched but the target path itself looks generated. |

`workspace-generated-artifact-missing` is still a blind zone. It is more
actionable than the generic reason, but it does not make downstream findings
safe.

## 7. Evidence Rules

Generated classification must be conservative and versioned. A package name,
dependency, or subpath token alone is not sufficient. The resolver may use
`workspace-generated-artifact-missing` only when a matched workspace package and
target subpath are accompanied by strong generated-artifact evidence.

The current policy version is:

```text
generated-artifact-policy-v1
```

Generated classification requires an evidence quorum. Primary evidence can
justify a generated reason when it is tied to the matched package and target
surface. Supporting evidence can strengthen that claim, but does not promote by
itself.

Primary evidence:

- package metadata that explicitly references generator output
- package scripts or `bin` entries that call known generators for the matched
  surface
- package `files` or `exports` entries that explicitly expose generated output
- generator configuration that names an output path

Supporting evidence:

- target path segments such as `generated` or `__generated__`
- dependencies that indicate a generator family
- target subpaths such as `client`, `enums`, or `zod`

Weak evidence must not promote a generic miss to
`workspace-generated-artifact-missing` by itself:

- package name only
- dependency only
- broad package text containing `prisma`, `gen`, `client`, or `enums`
- target subpath token only
- target segment `gen` only

Example for a Prisma package:

```json
{
  "name": "@calcom/prisma",
  "main": "index.ts",
  "bin": { "prisma-enum-generator": "./run-enum-generator.js" },
  "scripts": { "generate-schemas": "prisma generate" },
  "dependencies": { "@prisma/client": "6.16.1" },
  "files": ["client", "generated/prisma", "zod"]
}
```

This can justify generated subpath hints such as:

```text
client
generated
enums
zod
```

The engine should not infer broad generated support from a package name alone.
For example, `@scope/prisma/enums` should not become generated unless package
metadata contains Prisma/generator evidence.

Target path segments are supporting evidence, not sufficient evidence by
themselves, unless the segment is explicitly declared by package metadata such
as `files`, `exports`, or generator configuration. The segment `gen` is always
weak supporting evidence and must not promote a miss without package-level
generator evidence.

Generated classification should preserve structured evidence so maintainers can
see why the resolver chose a generated reason. Example evidence packet:

```json
{
  "policyVersion": "generated-artifact-policy-v1",
  "generatorFamily": "prisma",
  "confidence": "strong",
  "matchedPackage": "@calcom/prisma",
  "targetSubpath": "enums",
  "evidence": [
    {
      "kind": "package-bin",
      "field": "bin.prisma-enum-generator",
      "matched": "./run-enum-generator.js"
    },
    {
      "kind": "package-script",
      "field": "scripts.generate-schemas",
      "matched": "prisma generate"
    },
    {
      "kind": "package-files",
      "field": "files",
      "matched": "generated/prisma"
    }
  ]
}
```

## 8. Optional Support Modes

Generated artifact support should be explicit. Recommended modes:

When multiple generated support modes can describe the same specifier, concrete
files included by the current scan policy take precedence over virtual surfaces.
Virtual surfaces are fallback graph surfaces and must not duplicate or override
real files parsed in the same scan.

Recommended precedence:

1. present file included by scan policy
2. prepared generated artifact explicitly included by scan policy
3. run-generated artifact with command evidence and expected-path mutation
4. virtual generated surface
5. unresolved generated miss

### Mode A: Present Artifacts

```text
--generated-artifacts=present
```

The engine labels generated files only if they already exist and are already
included by the current scan policy. No generator is executed. This mode is
mostly a provenance label because included files can already be parsed.

If generated candidates exist but remain excluded by scan policy, the manifest
reports them as `present-but-out-of-scope`. This is diagnostic provenance only:
the resolver must not treat those candidates as resolved source evidence.

### Mode B: External Preparation

```text
--generated-artifacts=prepared
```

The caller promises that project setup has already produced generated artifacts
before the scan. Generated artifact modes must not silently widen scan scope.
The engine may include generated paths that are normally excluded only if scan
policy explicitly allows them, such as through a future
`--include-generated-artifacts` flag or a path-specific include rule.

If generated files exist but remain excluded by scan policy, the manifest should
report them as `present-but-out-of-scope`, not as resolved source evidence.
Prepared out-of-scope entries also carry stale provenance because the engine
cannot prove the prepared output matches current generator inputs.

This mode must report:

```json
{
  "generatedArtifacts": {
    "mode": "prepared",
    "executedGenerators": false
  }
}
```

Prepared generated facts should also record stale provenance. The default is
`staleStatus: "unknown"` unless the engine can compare supported generator input
hashes or read trusted external provenance:

```json
{
  "source": "generated-artifact",
  "mode": "prepared",
  "generator": "prisma",
  "surfaceConfidence": "declared",
  "staleStatus": "unknown",
  "staleReason": "generator-input-hash-not-recorded"
}
```

Unknown stale status may support graph construction, but it must not become
positive `SAFE_FIX` evidence by itself.

### Mode C: Run Generators

```text
--generated-artifacts=run
```

The engine may execute configured generator commands. This must remain opt-in
because it can be slow, mutate the working tree, need secrets, or execute
project code.

This mode requires guardrails:

- list planned commands before execution
- never run in pre-write by default
- capture command, cwd, exit code, duration, and generated paths
- require an explicit allowlist of generator families or commands
- set a timeout and record timeout policy
- record whether the working tree changed and whether changes were limited to
  expected generated paths
- fail closed if generation fails
- include command evidence in the manifest

### Mode D: Virtual Surface

Generator-specific modules can derive export surfaces without running project
code. Example: parse Prisma schema and expose enum names for
`@pkg/prisma/enums`.

This mode should be added one generator at a time with fixture tests. It must
not claim runtime equivalence. It only supplies a conservative export/import
surface for graph construction.

Virtual facts must carry provenance and completeness:

```json
{
  "source": "generated-virtual",
  "mode": "virtual",
  "generator": "prisma-enums",
  "surfaceConfidence": "declared",
  "surfaceCompleteness": "partial",
  "runtimeEquivalence": false,
  "derivedFrom": ["packages/prisma/schema.prisma"]
}
```

Virtual generated modules must not emit function facts, body hashes, call edges,
or semantic behavior claims unless those facts are directly derived from
supported generator input and labeled as virtual/generated.

The initial supported virtual surface is Prisma enum generation. The engine may
derive enum export names from `schema.prisma` only when the matched workspace
package already has strong Prisma enum-generator evidence and the schema itself
declares a `prisma-enum-generator` generator. The virtual surface exposes enum
names as value/type export surface only. Imports of names absent from the schema
remain unresolved; virtual surface support must not invent missing exports.

## 9. Ranking Contract

Generated-artifact evidence is not positive evidence for `SAFE_FIX`.

```text
generated artifact missing
  -> relevant blind zone
  -> blocks SAFE_FIX if the missing module could hide a consumer
```

If generated artifacts are later supplied by an opt-in mode, their evidence
must carry provenance:

```json
{
  "source": "generated-artifact",
  "mode": "prepared",
  "generator": "prisma",
  "surfaceConfidence": "declared"
}
```

Ranking may consume that graph only if the generated surface is complete enough
for the relevant symbol. Partial generated surfaces must remain review/degraded
evidence.

Generated misses should block `SAFE_FIX` only when the blind zone is relevant to
the candidate export, package scope, or unresolved import surface under review.
A generated miss in an unrelated package must not globally block all safe fixes.

Two blind-zone shapes matter:

| Shape | Meaning | Ranking impact |
|---|---|---|
| provider blind zone | Current source imports a generated module, but that generated module is absent. The generated provider surface is unknown. | Blocks candidates that depend on that provider surface. |
| consumer blind zone | Generated files that would be in scan scope are absent or excluded. Those files could contain imports/consumers of source exports. | Blocks candidates whose exports could be consumed from that generated scope. |

Consumer blind zones must be represented explicitly. A generated miss in
`apps/web` does not by itself block every candidate in `apps/web`; instead, the
engine records the missing or out-of-scope generated target and blocks only
candidates in the generated target's package scope or target submodule.

Ranking artifacts should record the generated blind zone that blocked promotion:

```json
{
  "blockedPromotion": true,
  "blockedBy": [
    {
      "reason": "workspace-generated-artifact-missing",
      "specifier": "@calcom/prisma/enums",
      "scope": "packages/prisma",
      "impact": "provider-surface-unresolved",
      "relevance": "same-package-or-provider-edge",
      "candidateIdentity": "packages/prisma/index.ts::SomeExport"
    }
  ]
}
```

A generated blind zone is relevant only when the candidate export, package
scope, or known dependency/import surface intersects the missing generated
surface. If relevance cannot be established, the blind zone may be reported as
a general confidence limitation, but should not automatically block unrelated
`SAFE_FIX` promotions.

For provider blind zones, the generated module's importing consumer file is not
by itself enough to block every candidate in that consumer's submodule. The
candidate must intersect the generated provider package, target candidate
surface, or an explicit generated consumer-blind-zone record.

## 10. Artifact Shape

`manifest.json` should eventually expose:

```json
{
  "generatedArtifacts": {
    "mode": "default",
    "generatedArtifactPolicyVersion": "generated-artifact-policy-v1",
    "executedGenerators": false,
    "reasonSummary": {
      "workspace-generated-artifact-missing": 423
    },
    "generatedConsumerBlindZoneCount": 17,
    "topGeneratedConsumerBlindZones": [
      {
        "scopePackageRoot": "packages/prisma",
        "count": 12,
        "statuses": {
          "missing": 10,
          "present-but-out-of-scope": 2
        },
        "topSpecifiers": [
          {
            "specifier": "@calcom/prisma/enums",
            "count": 9
          }
        ],
        "examples": [
          {
            "specifier": "@calcom/prisma/enums",
            "consumerFile": "apps/web/src/foo.ts",
            "candidatePath": "packages/prisma/enums.ts",
            "status": "missing",
            "mode": "default"
          }
        ]
      }
    ],
    "topGeneratedMisses": [
      {
        "specifier": "@calcom/prisma/enums",
        "matchedPackage": "@calcom/prisma",
        "count": 187,
        "generatorFamily": "prisma",
        "confidence": "strong"
      }
    ],
    "supportedGenerators": []
  }
}
```

`symbols.json` unresolved records should keep the existing fields:

```json
{
  "specifier": "@calcom/prisma/enums",
  "consumerFile": "apps/web/src/foo.ts",
  "reason": "workspace-generated-artifact-missing",
  "resolverStage": "wildcard-alias",
  "matchedPattern": "@calcom/prisma/*",
  "source": "legacy-subpath",
  "targetCandidates": ["packages/prisma/enums"],
  "hint": "generated-artifact-missing",
  "generatedArtifact": {
    "policyVersion": "generated-artifact-policy-v1",
    "generatorFamily": "prisma",
    "confidence": "strong",
    "matchedPackage": "@calcom/prisma",
    "targetSubpath": "enums",
    "evidence": [
      {
        "kind": "package-bin",
        "field": "bin.prisma-enum-generator",
        "matched": "./run-enum-generator.js"
      }
    ]
  }
}
```

`tsconfig-path-target-missing` records can also carry generated evidence when
the target path itself looks generated. In that case the reason remains
`tsconfig-path-target-missing`; the generated packet explains the hint without
turning the miss into a successful resolution:

```json
{
  "reason": "tsconfig-path-target-missing",
  "hint": "generated-artifact-missing",
  "generatedArtifact": {
    "policyVersion": "generated-artifact-policy-v1",
    "confidence": "supporting",
    "targetCandidate": "packages/generated/generated/client",
    "evidence": [
      {
        "kind": "target-path-segment",
        "matched": "generated"
      }
    ]
  }
}
```

Virtual resolver outputs must remain distinguishable from real file resolution:

```json
{
  "resolverStage": "generated-virtual",
  "source": "generated-virtual",
  "virtual": true,
  "runtimeEquivalence": false
}
```

`symbols.json` also records generated consumer blind zones derived from
generated unresolved records:

```json
{
  "generatedConsumerBlindZones": [
    {
      "reason": "generated-consumer-blind-zone",
      "sourceReason": "workspace-generated-artifact-missing",
      "specifier": "@calcom/prisma/enums",
      "consumerFile": "apps/web/src/foo.ts",
      "matchedPackage": "@calcom/prisma",
      "targetSubpath": "enums",
      "generatorFamily": "prisma",
      "confidence": "strong",
      "candidatePath": "packages/prisma/enums.ts",
      "status": "missing",
      "scopePackageRoot": "packages/prisma",
      "mode": "default"
    }
  ]
}
```

If the target file exists but remains excluded by scan policy, the zone uses
`status: "present-but-out-of-scope"` and includes `scanScopeReason`. In
prepared mode, the zone also carries `staleStatus: "unknown"` unless generator
input hashes or trusted external provenance prove freshness.

## 11. Implementation Phases

### P0: Diagnostic Taxonomy

- Split likely generated workspace subpath misses into
  `workspace-generated-artifact-missing`.
- Keep generic workspace misses separate.
- Add a versioned generated evidence classifier.
- Attach structured generated evidence to unresolved records.
- Add generated consumer blind-zone inventory to `symbols.json`.
- Do not change resolution success.

### P1: Manifest Summary

- Add generated-artifact summary metadata to `manifest.json`.
- Surface top packages/specifiers by generated-miss count.
- Keep this reporting-only.
- Add reporting summaries for generated consumer blind zones:
  `generatedConsumerBlindZoneCount` and `topGeneratedConsumerBlindZones`.

### P1.5: Ranking Relevance

- Consume explicit generated consumer blind zones for relevance-scoped
  `SAFE_FIX` blocking.
- Keep provider miss relevance scoped to provider package/target surface.
- Add relevance-scoped generated blind-zone blocking diagnostics for ranking.

### P2: Prepared Artifact Mode

- Add opt-in mode for callers that generated files before scanning.
- Include generated paths only through explicit scan policy.
- Add provenance fields to generated-source facts.
- Default prepared generated facts to `staleStatus: "unknown"` unless generator
  input hashes or trusted external provenance prove freshness.

### P3: Generator-Specific Virtual Surface

- Start with Prisma enum surfaces before broader Prisma client support.
- Use strict fixtures and no command execution.
- Mark all virtual facts as generated and partial unless proven complete.

### P4: Generator Execution

- Optional and last.
- Require explicit CLI opt-in and manifest command evidence.
- Never enable for default quick/full/pre-write scans.

## 12. Acceptance Criteria

- Missing generated workspace subpaths are counted separately from ordinary
  workspace subpath misses.
- The default resolver never treats a missing generated artifact as resolved.
- `symbols.json` exposes `generatedConsumerBlindZones` for missing or
  out-of-scope generated target surfaces.
- `SAFE_FIX` does not promote through relevant generated-artifact blind zones.
- Unrelated generated blind zones do not globally block every `SAFE_FIX`.
- Generated classifications include `generatedArtifactPolicyVersion` and
  structured evidence.
- Prepared/virtual/generated modes carry provenance in artifacts.
- Present/prepared modes do not silently widen scan scope.
- Generator execution is never implicit.
- A fixture with ordinary missing workspace source remains
  `workspace-package-subpath-target-missing`.
- A fixture with strong Prisma enum-generation evidence and missing `enums`
  subpath becomes `workspace-generated-artifact-missing`.
- Package name, dependency, or target token alone does not become
  `workspace-generated-artifact-missing`.
- Target segment `gen` alone does not become
  `workspace-generated-artifact-missing`.
- `generated` or `__generated__` target segments only promote when combined with
  package-level generator evidence or explicit package metadata for that path.
- `tsconfig-path-target-missing` can carry generated structured evidence without
  becoming a resolved module.
- Generated file present but excluded by scan policy is reported
  `present-but-out-of-scope`.
- Prepared mode without explicit include policy does not widen scan scope.
- Prisma enum virtual surface requires both package-level enum-generator
  evidence and schema-level `prisma-enum-generator` evidence.
- Prisma enum virtual surface resolves only enum names present in
  `schema.prisma`; missing names remain unresolved.
- Virtual surface never emits function facts, body hashes, or call edges unless
  a generator-specific design explicitly supports and labels them.

## 13. Open Questions

- After Prisma enums, which generated family should be next: Prisma client,
  Kysely types, or framework route manifests?
- What explicit include flag should control prepared generated paths:
  `--include-generated-artifacts`, path-specific include rules, or both?
- How should stale generated artifacts be detected without requiring git? A
  likely first answer is generator-input hashes when available, otherwise
  `staleStatus: "unknown"` that cannot be used as positive safe evidence.
