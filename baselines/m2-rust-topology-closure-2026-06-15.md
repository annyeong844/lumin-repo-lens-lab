# M2 Rust Topology Closure

Date: 2026-06-15

## Decision

M2 is closed as a compare-only success.

The Rust topology sidecar has enough evidence to stay in the lab as a parity-checked comparison scanner. It does not have replacement approval.

## Source State

- Lab repo: `annyeong844/lumin_lab`
- Merged PR: `#1` (`Close M2 Nuxt topology scanner parity gaps`)
- Merge commit: `472f188c8e10b5b0661d8dec430cbe5c43679561`
- Final M2 implementation commit: `87116819c23d1e1adfbfca5def44552856e4f464`
- Review packet: `C:\Users\endof\Downloads\lumin-perf-lab\review\m2-rust-topology-review-8711681.zip`
- Stable plugin touched: no
- Private CI triggered: no; private workflow remains manual-only.

## Accepted Evidence

All four M2 compare corpora matched with zero mismatches:

| Corpus | Status | Compared | Mismatches |
| --- | --- | ---: | ---: |
| `geulbat-phase1` | `matched` | 11 | 0 |
| `lumin-repo-lens-lab` self-scan | `matched` | 701 | 0 |
| `stable-source-clean` | `matched` | 2,071 | 0 |
| `nuxt-main` | `matched` | 625 | 0 |

The former Nuxt 3 mismatch set is closed for M2 compare evidence. The closure came from precise JS-tokenizer-state parity tests and refreshed corpus evidence, not from a broad `scanner-state-ambiguous` heuristic.

## Guardrails That Stay

- `prefer` remains disabled.
- JS remains the topology oracle.
- Rust compare metadata is evidence, not replacement permission.
- Broad scanner-state heuristics stay rejected.
- Private CI remains manual-only.

## What M2 Proved

- The Node bridge can invoke the Rust sidecar in compare mode and record explicit metadata.
- The Rust scanner can match the JS scanner on the selected real corpora.
- Real Nuxt mismatch samples can be converted into precise parity fixtures without poisoning clean corpora.
- The lab plugin can evolve independently of stable `/lumin-repo-lens:*`.

## What M2 Did Not Prove

- It did not prove Rust is ready to replace JS topology output.
- It did not enable `prefer`.
- It did not prove full-repo Rust migration is safe.
- It did not make performance claims outside the recorded compare runs.

## Next Gate

M3 should be a gate-design phase, not a rewrite phase.

Recommended M3 entry criteria:

- Define the exact conditions for a future `prefer` mode.
- Decide how many real corpus runs must stay at `mismatches=0`.
- Define failure handling for sidecar timeout, invalid JSON, non-zero exit, count mismatch, edge mismatch, and risk mismatch.
- Decide whether replacement eligibility is per-corpus, per-file, or global.
- Keep artifact contracts stable until a replacement contract is deliberately designed.

## Final Status

M2 compare parity is complete. Rust stays in the lab, compare-only, with JS authoritative.
