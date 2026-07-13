# Internal Engine Map

This document groups the root engine scripts that remain useful for
development, narrow repro work, and step-by-step debugging, but are
not the primary public entrypoint.

If you are entering the repo as a user, start with:

- `README.md`
- `SKILL.md`
- `audit-repo.mjs`
- `canonical/`

If you are maintaining or debugging the engine, the root sibling
scripts are grouped like this.

## Collection and measurement

These scripts build the evidence surfaces the rest of the engine reads:

- `triage-repo.mjs`
- `measure-topology.mjs`
- `measure-discipline.mjs`
- `measure-staleness.mjs`
- `any-inventory.mjs`
- `build-symbol-graph.mjs`
- `build-call-graph.mjs`
- `build-shape-index.mjs`
- `resolve-method-calls.mjs`

## Classification and reporting

These scripts turn collected evidence into findings, fix proposals,
comparisons, or output formats:

- `classify-dead-exports.mjs`
- `rank-fixes.mjs`
- `checklist-facts.mjs`
- `compare-repos.mjs`
- `merge-runtime-evidence.mjs`
- `emit-sarif.mjs`
- `p6-measurement.mjs`

## Lifecycle and canon workflows

Write-gate execution is available only through `audit-repo.mjs`; audit-core
owns native pre-write and post-write computation behind that public lifecycle.
The remaining focused script entrypoints are:

- `generate-canon-draft.mjs`
- `check-canon.mjs`

## Policy

- `audit-repo.mjs` is the recommended public CLI.
- Root sibling scripts remain available and documented because they are
  part of the engine's reproducibility story.
- Public docs should describe these as internal engine entrypoints, not
  as the first decision a user has to make.
