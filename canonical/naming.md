# Naming conventions draft

Generated: 2026-04-22T16:21:26.922Z
Scope: TS/JS including tests
Source: fresh-ast-pass
CohortIdentityShape: submodule | submodule::kind

## 1. File-naming cohorts

| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |
|--------------------|------:|--------------------|----------------:|--------------:|--------|
| `_lib` | 57 | `kebab-case` | 86% | 8 | kebab-case-dominant ✅ |
| `root` | 23 | `kebab-case` | 96% | 1 | kebab-case-dominant ✅ |
| `scripts` | 5 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |
| `tests` | 109 | `kebab-case` | 77% | 25 | kebab-case-dominant ✅ |

## 2. Symbol-naming cohorts

| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |
|--------------------------|------:|--------------------|----------------:|--------------:|--------|
| `_lib::constant-export` | 36 | `UPPER_SNAKE` | 100% | 0 | UPPER_SNAKE-dominant ✅ |
| `_lib::helper-export` | 140 | `camelCase` | 100% | 0 | camelCase-dominant ✅ |
| `tests::helper-export` | 7 | `camelCase` | 100% | 0 | camelCase-dominant ✅ |

## 3. Outliers

| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |
|----------|--------|------|--------------------|--------------------|--------|
| `_lib/artifacts.mjs` | `_lib` | `artifacts` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/cli.mjs` | `_lib` | `cli` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/incremental.mjs` | `_lib` | `incremental` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/lang.mjs` | `_lib` | `lang` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/paths.mjs` | `_lib` | `paths` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/python.mjs` | `_lib` | `python` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/ranking.mjs` | `_lib` | `ranking` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `_lib/vocab.mjs` | `_lib` | `vocab` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `eslint.config.mjs` | `root` | `eslint.config` | `mixed` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-added/src/bar.ts` | `tests` | `bar` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-added/src/c1.ts` | `tests` | `c1` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-added/src/c2.ts` | `tests` | `c2` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-added/src/c3.ts` | `tests` | `c3` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-added/src/foo.ts` | `tests` | `foo` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-clean/src/c1.ts` | `tests` | `c1` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-clean/src/c2.ts` | `tests` | `c2` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-clean/src/c3.ts` | `tests` | `c3` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-clean/src/foo.ts` | `tests` | `foo` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-contamination-changed/src/c1.ts` | `tests` | `c1` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-contamination-changed/src/c2.ts` | `tests` | `c2` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-contamination-changed/src/c3.ts` | `tests` | `c3` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-contamination-changed/src/foo.ts` | `tests` | `foo` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-fan-in-tier-changed/src/c1.ts` | `tests` | `c1` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-fan-in-tier-changed/src/c2.ts` | `tests` | `c2` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-fan-in-tier-changed/src/c3.ts` | `tests` | `c3` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-fan-in-tier-changed/src/foo.ts` | `tests` | `foo` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-label-changed/src/c1.ts` | `tests` | `c1` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-label-changed/src/c2.ts` | `tests` | `c2` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-label-changed/src/c3.ts` | `tests` | `c3` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-label-changed/src/foo.ts` | `tests` | `foo` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-removed/src/c1.ts` | `tests` | `c1` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-removed/src/c2.ts` | `tests` | `c2` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-removed/src/c3.ts` | `tests` | `c3` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
| `tests/fixtures/canon-drift-helpers-removed/src/foo.ts` | `tests` | `foo` | `camelCase` | `kebab-case` | convention-outlier ⚠ |
