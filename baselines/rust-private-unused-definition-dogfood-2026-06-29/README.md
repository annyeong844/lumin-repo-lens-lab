# Rust Private Unused Definition Dogfood - 2026-06-29

Scope: validate the second Rust unused-definition slice after adding private
module-owned candidates.

Command surface: `lumin-rust-source-health --artifact-profile compact` with the
offline Rust 1.95 basepack. No Node commands were run.

Artifacts:

- `C:\Users\endof\Downloads\lumin-perf-lab\review\rust-private-unused-definition-dogfood\ripgrep-rust-health.json`
- `C:\Users\endof\Downloads\lumin-perf-lab\review\rust-private-unused-definition-dogfood\codex-core-rust-health.json`
- `C:\Users\endof\Downloads\lumin-perf-lab\review\rust-private-unused-definition-dogfood\codex-app-server-rust-health-after.json`

Key result:

- `ripgrep-master`: 0 findings after test/entrypoint gates; 0 unsafe findings;
  0 safe actions.
- `codex-core`: 0 findings after macro-token, format-capture, and attribute
  string-path references are counted as local references.
- `codex-app-server`: 0 findings; this covers the
  `migration_item_type_label` regression where a private helper was only used
  inside a `vec!` macro input.
- Public, trait, generated, test-only, and entrypoint surfaces stayed in
  `excludedCandidates[]` with RUST-FP blockers.
- Public inherent impl methods are included in the RUST-FP-A public-surface
  blocker lane instead of silently falling out of the analysis.

This confirms the intended split: Rust source-health may now report raw private
unused-definition evidence only when local reference evidence is complete enough
for the slice, and it still does not emit `SAFE_FIX` or edit instructions.
