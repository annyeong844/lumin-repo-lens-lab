# Topology draft

Generated: 2026-04-23T11:44:48.952Z
Scope: TS/JS including tests
Source: topology.json + triage.json
Lens: runtime
Mode: single-package
TopologyComplete: true
CrossEdgeSource: full-list
ClassificationConfidence: high

## 1. Submodule inventory

| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |
|-----------|------:|----:|---------:|----------:|-----|--------|------|
| `_lib` | 58 | 13396 | 226 | 0 | — | shared-submodule ✅ |  |
| `root` | 23 | 7199 | 0 | 149 | — | leaf-submodule ⚠ |  |
| `scripts` | 5 | 584 | 0 | 0 | — | isolated-submodule ℹ |  |
| `tests` | 110 | 27077 | 0 | 77 | — | leaf-submodule ⚠ |  |

## 2. Cross-submodule edges (top 30)

| From | To | Count |
|------|----|------:|
| `root` | `_lib` | 149 |
| `tests` | `_lib` | 77 |

## 3. Cycles (SCCs)

✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.

## 4. Oversize files (≥ 400 LOC)

| File | LOC | Status |
|------|----:|--------|
| `tests/test-classification-gates.mjs` | 1482 | extreme-oversize ❌ |
| `_lib/check-canon-utils.mjs` | 887 | oversize ⚠ |
| `tests/test-check-canon-utils.mjs` | 800 | oversize ⚠ |
| `audit-repo.mjs` | 760 | oversize ⚠ |
| `tests/test-pre-write-render.mjs` | 741 | oversize ⚠ |
| `tests/test-post-write-delta.mjs` | 681 | oversize ⚠ |
| `tests/test-corpus.mjs` | 666 | oversize ⚠ |
| `tests/test-canon-draft-helper-registry.mjs` | 613 | oversize ⚠ |
| `tests/test-generate-check-canon-cli.mjs` | 604 | oversize ⚠ |
| `tests/test-pre-write-cli.mjs` | 598 | oversize ⚠ |
| `build-symbol-graph.mjs` | 577 | oversize ⚠ |
| `tests/test-canon-draft-type-ownership.mjs` | 546 | oversize ⚠ |
| `tests/test-check-canon-topology.mjs` | 527 | oversize ⚠ |
| `classify-dead-exports.mjs` | 514 | oversize ⚠ |
| `_lib/canon-draft-topology.mjs` | 486 | oversize ⚠ |
| `tests/test-canon-draft-topology-structure.mjs` | 482 | oversize ⚠ |
| `tests/test-p6-measurement.mjs` | 476 | oversize ⚠ |
| `_lib/canon-draft-naming.mjs` | 475 | oversize ⚠ |
| `_lib/p6-measurement.mjs` | 471 | oversize ⚠ |
| `_lib/post-write-delta.mjs` | 470 | oversize ⚠ |
