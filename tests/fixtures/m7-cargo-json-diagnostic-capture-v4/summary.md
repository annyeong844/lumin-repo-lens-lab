# M7 Cargo JSON Diagnostic Capture v4

Purpose: empirical cargo `--message-format=json` evidence for M7 cargo diagnostic classification.

## Toolchain

- cargo: `cargo 1.96.0 (30a34c682 2026-05-25)`
- host: `x86_64-pc-windows-msvc`
- profile: `dev`
```text
rustc 1.96.0 (ac68faa20 2026-05-25)
binary: rustc
commit-hash: ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96
commit-date: 2026-05-25
host: x86_64-pc-windows-msvc
release: 1.96.0
LLVM version: 22.1.2
```

## Classifier Rules

- level not in {error, warning} -> non-finding; never candidate
- diagnostics without selected user-code primary spans are not user-facing findings
- level error with only non-user primary span -> coverage unavailable; not candidate
- rustc-non-ecode -> rule-backed lint diagnostic before level-based verified classification
- rustc-error + level error + user primary -> verified rustc error diagnostic
- rustc-codeless + level error + user primary -> verified codeless rustc error diagnostic
- remaining real warning/error diagnostics with user primary -> candidate fallback; never verified

## Runs

### borrow-error-e0502

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`1`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-error/rustc-error-code disposition=finding confidence=verified claim=verified.rust.rustc-error-diagnostic rule=ecode-error-user-code-primary primary=[src\lib.rs [user-code]] message="cannot borrow `value` as mutable because it is also borrowed as immutable"
- failure-note rustc-codeless/null-error-code disposition=non-finding confidence=<none> claim=<none> rule=note-help-failure-note-are-not-findings primary=[<none>] message="For more information about this error, try `rustc --explain E0502`."

### build-script-failure

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":2,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`0`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- diagnostics: none

### compile-error-codeless

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":2,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-codeless/null-error-code disposition=finding confidence=verified claim=verified.rust.rustc-codeless-error-diagnostic rule=codeless-error-user-code-primary primary=[src\lib.rs [user-code]] message="intentional compile_error fixture"

### deny-lint-error

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":2,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-non-ecode/non-ecode-name disposition=finding confidence=rule-backed claim=rule-backed.rust.rustc-lint-diagnostic rule=non-ecode-code-name-treated-as-rule-backed-before-level primary=[src\lib.rs [user-code]] message="unused variable: `unused_value`"

### dependency-error-outside-workspace

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`0`, nonFindings=`1`, coverageUnavailableDiagnostics=`1`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false; non-user-code primary error diagnostic encountered"
- error rustc-error/rustc-error-code disposition=coverage-unavailable confidence=<none> claim=<none> rule=non-user-code-primary-span-not-user-facing-finding primary=[bad_dep\src\lib.rs [dependency]] message="mismatched types"
- failure-note rustc-codeless/null-error-code disposition=non-finding confidence=<none> claim=<none> rule=note-help-failure-note-are-not-findings primary=[<none>] message="For more information about this error, try `rustc --explain E0308`."

### feature-gated-error-default

- args: `cargo check --message-format=json`
- exitCode: `0`
- stream: `{"lineCount":2,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":true}}`
- counts: findings=`0`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`ran` cleanKind=`verified-rustc-error-absence` clean=`true`
- diagnostics: none

### feature-gated-error-features-bad

- args: `cargo check --message-format=json --features bad`
- exitCode: `101`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`1`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-error/rustc-error-code disposition=finding confidence=verified claim=verified.rust.rustc-error-diagnostic rule=ecode-error-user-code-primary primary=[src\lib.rs [user-code]] message="mismatched types"
- failure-note rustc-codeless/null-error-code disposition=non-finding confidence=<none> claim=<none> rule=note-help-failure-note-are-not-findings primary=[<none>] message="For more information about this error, try `rustc --explain E0308`."

### macro-expansion-error

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`1`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-error/rustc-error-code disposition=finding confidence=verified claim=verified.rust.rustc-error-diagnostic rule=ecode-error-user-code-primary primary=[src\lib.rs [user-code, expansion]] message="mismatched types"
- failure-note rustc-codeless/null-error-code disposition=non-finding confidence=<none> claim=<none> rule=note-help-failure-note-are-not-findings primary=[<none>] message="For more information about this error, try `rustc --explain E0308`."

### name-resolution-e0425

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`1`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-error/rustc-error-code disposition=finding confidence=verified claim=verified.rust.rustc-error-diagnostic rule=ecode-error-user-code-primary primary=[src\lib.rs [user-code]] message="cannot find function `missing_symbol` in this scope"
- failure-note rustc-codeless/null-error-code disposition=non-finding confidence=<none> claim=<none> rule=note-help-failure-note-are-not-findings primary=[<none>] message="For more information about this error, try `rustc --explain E0425`."

### parse-error-codeless

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":2,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-codeless/null-error-code disposition=finding confidence=verified claim=verified.rust.rustc-codeless-error-diagnostic rule=codeless-error-user-code-primary primary=[src\lib.rs [user-code]] message="this file contains an unclosed delimiter"

### success-clean

- args: `cargo check --message-format=json`
- exitCode: `0`
- stream: `{"lineCount":2,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":true}}`
- counts: findings=`0`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`ran` cleanKind=`verified-rustc-error-absence` clean=`true`
- diagnostics: none

### type-error-e0308

- args: `cargo check --message-format=json`
- exitCode: `101`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":false}}`
- counts: findings=`1`, nonFindings=`1`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`unavailable` cleanKind=`verified-rustc-error-absence` clean=`null` reason="build-finished success was false"
- error rustc-error/rustc-error-code disposition=finding confidence=verified claim=verified.rust.rustc-error-diagnostic rule=ecode-error-user-code-primary primary=[src\lib.rs [user-code]] message="mismatched types"
- failure-note rustc-codeless/null-error-code disposition=non-finding confidence=<none> claim=<none> rule=note-help-failure-note-are-not-findings primary=[<none>] message="For more information about this error, try `rustc --explain E0308`."

### warning-only

- args: `cargo check --message-format=json`
- exitCode: `0`
- stream: `{"lineCount":3,"invalidJsonLineCount":0,"streamParseStatus":"complete","buildFinished":{"success":true}}`
- counts: findings=`1`, nonFindings=`0`, coverageUnavailableDiagnostics=`0`
- coverage cargo-event-stream: status=`ran`
- coverage absence-clean: status=`ran` cleanKind=`verified-rustc-error-absence` clean=`true`
- warning rustc-non-ecode/non-ecode-name disposition=finding confidence=rule-backed claim=rule-backed.rust.rustc-lint-diagnostic rule=non-ecode-code-name-treated-as-rule-backed-before-level primary=[src\lib.rs [user-code]] message="unused variable: `unused_value`"

## Observations

- `failure-note` diagnostics are not findings and are not candidate fallback material.
- Dependency/non-user primary error diagnostics drive absence-clean coverage unavailable; they are not user review candidates.
- User-facing findings require selected user-code primary ownership, including rule-backed lint diagnostics.
- Denied rustc lint can be `level:error`; non-E code names must be classified before level-based verified classification.
- `rule-backed.rust.rustc-lint-diagnostic` intentionally avoids calling denied lints warnings.
- Codeless rustc errors are represented as `code: null`, not omitted code fields.
- E-code rustc errors are represented as `code.code` matching `^E[0-9]+$`.
- `build-finished { success: true }` plus complete JSONL parsing is required for absence/clean coverage.
- `absence-clean.clean` only means absence of verified rustc error claim kinds for the scoped package/target/features/profile; rule-backed findings can coexist.
- Feature-gated code confirms coverage must include feature set.