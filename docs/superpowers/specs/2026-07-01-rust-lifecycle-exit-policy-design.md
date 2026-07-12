# Rust Lifecycle Exit Policy Design

Date: 2026-07-01
Owner: `lumin-audit-core`

## Checked JS Contract

The current `audit-repo.mjs` lifecycle tail applies strict post-write exit
policies after all raw lifecycle blocks have been produced:

- `--strict-post-write` escalates to exit 2 only when `postWrite.ran === false`
  and the current final exit code is still 0.
- `--strict-post-write-confidence` escalates to exit 2 only when post-write ran,
  the current final exit code is still 0, and the post-write delta confidence is
  limited.
- Existing non-zero exits are not overwritten by strict post-write policy.
- A missing `postWrite` block does not escalate.

The confidence rule is:

- when `typeEscapeDeltaStatus === "not-applicable"`, require
  `fileDeltaStatus === "computed"`;
- otherwise require `baselineStatus === "available"`,
  `scanRangeParity === "ok"`, and `afterComplete === true`.

## Rust Owner Boundary

`lifecycle_exit_policy.rs` owns the typed projection from the current
orchestrator exit code plus lifecycle blocks to the final lifecycle strict
policy exit code and diagnostic stderr text.

It must not own:

- raw lifecycle block construction;
- `post-write.mjs` delta computation;
- check-canon drift semantics;
- final manifest file writing.

## JS Wrapper

`audit-repo.mjs` still decides the overall orchestration order and final
manifest write. After `manifest.lifecycle` is built, it calls
`applyLifecycleExitPolicy` with the current exit code, strict flags, and the
raw `postWrite` block. If Rust returns stderr text, the JS wrapper writes it
unchanged, then replaces `finalExitCode` with the Rust result.

## Exit Contract

The Rust result returns `{ exitCode, stderr? }`.

- `stderr` is present only when Rust applies a strict post-write escalation.
- The result never executes producers and never reads artifacts.
