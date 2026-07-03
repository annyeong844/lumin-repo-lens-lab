pub(super) const USAGE: &str = r#"usage: lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran|--rust-analysis-block <path|->]
       lumin-audit-core artifact-size-summary --output <dir> --input <path|->
       lumin-audit-core artifact-read-metrics-summary --input <path|->
       lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>
       lumin-audit-core rust-analysis-run-merge --input <path|->
       lumin-audit-core generated-artifacts-summary --root <repo> [--symbols <path>] [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--exclude <path> ...]
       lumin-audit-core artifact-summary --artifact-kind <framework-resource-surfaces|unused-deps|block-clones> --artifact <path>
       lumin-audit-core unused-deps-artifact --input <path|-> [--result-output <path>]
       lumin-audit-core resolver-diagnostics-summary [--symbols <path>] [--resolver-capabilities <path>] [--resolver-diagnostics <path>]
       lumin-audit-core blind-zones-summary [--input <fixture.json>|--cases <cases.json>|--root <repo> --output <dir> [--rust-analysis-ran]]
       lumin-audit-core lifecycle-summary --input <path|->
       lumin-audit-core manifest-lifecycle-update --input <path|->
       lumin-audit-core lifecycle-exit-policy --input <path|->
       lumin-audit-core lifecycle-request-guard --input <path|->
       lumin-audit-core manifest-meta --generated <iso> --profile <quick|full|ci> --root <repo> --output <dir>
       lumin-audit-core manifest-root --input <path|->
       lumin-audit-core manifest-root-with-evidence --input <path|-> [--result-output <path>]
       lumin-audit-core manifest-write --output <dir> --input <path|->
       lumin-audit-core manifest-lifecycle-evidence-refresh --input <path|-> [--result-output <path>]
       lumin-audit-core manifest-evidence-update --input <path|->
       lumin-audit-core manifest-evidence-refresh --root <repo> --output <dir> [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--production|--no-production] [--rust-analysis-ran|--rust-analysis-run-block <path|->] [--exclude <path> ...] [--auto-exclude <path> ...]
       lumin-audit-core manifest-evidence-refresh-with-reads --root <repo> --output <dir> [--result-output <path>] [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--production|--no-production] [--rust-analysis-ran|--rust-analysis-run-block <path|->] [--exclude <path> ...] [--auto-exclude <path> ...]
       lumin-audit-core manifest-companion-update --input <path|->
       lumin-audit-core manifest-artifacts-produced-update --output <dir> [--rust-analysis-block <path|->]
       lumin-audit-core manifest-final-summary-update --output <dir> --producer-performance <path> [--rust-analysis-ran|--rust-analysis-block <path|->]
       lumin-audit-core manifest-closeout-update --input <path|->
       lumin-audit-core manifest-closeout-write --input <path|->
       lumin-audit-core finalize-audit-run --input <path|->
       lumin-audit-core manifest-core-summary --root <repo> [--triage <path>] [--symbols <path>] [--include-tests|--no-include-tests] [--production|--no-production] [--exclude <path> ...] [--auto-exclude <path> ...]
       lumin-audit-core manifest-evidence-summary --root <repo> --output <dir> [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--production|--no-production] [--rust-analysis-ran|--rust-analysis-run-block <path|->] [--exclude <path> ...] [--auto-exclude <path> ...]
       lumin-audit-core manifest-evidence-summary-with-reads --root <repo> --output <dir> [--result-output <path>] [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--production|--no-production] [--rust-analysis-ran|--rust-analysis-run-block <path|->] [--exclude <path> ...] [--auto-exclude <path> ...]
       lumin-audit-core orchestration-plan [--profile <quick|full|ci>] [--sarif] [--pre-write] [--post-write] [--canon-draft] [--check-canon] [--rust-analyzer]
       lumin-audit-core execute-base-plan --input <path|->
       lumin-audit-core execute-base-runtime --input <path|->
       lumin-audit-core execute-canon-draft --input <path|->
       lumin-audit-core execute-check-canon --input <path|->
       lumin-audit-core pre-write-route --input <path|->
       lumin-audit-core execute-rust-pre-write --input <path|-> [--result-output <path>]
       lumin-audit-core execute-post-write --input <path|-> [--result-output <path>]
       lumin-audit-core orchestration-result-summary --artifact <path>
       lumin-audit-core producer-performance-summary --artifact <path>
       lumin-audit-core producer-performance-artifact --input <path|->
       lumin-audit-core producer-performance-runtime-artifact --input <path|->
       lumin-audit-core producer-performance-audit-run-artifact --input <path|-> --generated <iso> --root <repo> --output <dir> --profile <quick|full|ci> [--include-tests|--no-include-tests] [--production|--no-production] [--exclude <path> ...] [--auto-exclude <path> ...] [--no-incremental] --cache-root <dir> [--clear-incremental-cache] --generated-artifacts <default|present|prepared>
       lumin-audit-core living-audit-summary --root <repo>"#;
