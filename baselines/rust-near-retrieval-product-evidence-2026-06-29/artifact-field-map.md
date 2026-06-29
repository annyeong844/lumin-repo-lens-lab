# Artifact Field Map

This packet checks that Rust near retrieval transparency survives the path:

```text
rust-source-health engine
  -> rust-source-health compact artifact
  -> lumin-rust-analyzer product summary
  -> lumin-rust-analyzer phases.syntax.summary
```

## rust-source-health compact artifact

Location:

```text
functionCloneGroups
```

Fields preserved:

```text
candidateGenerationPolicy
candidateGenerationSummary
skippedLowDiscriminationBucketCount
skippedLowDiscriminationRawPairEstimate
skippedLowDiscriminationPairEstimateKind
skippedLowDiscriminationBucketExamples
```

The compact artifact still does not embed all raw function clone groups. It
preserves the product-relevant transparency metadata and capped examples.

## lumin-rust-analyzer top-level summary

Location:

```text
summary
```

Fields added:

```text
syntaxFunctionCloneNearCandidateCountScope
syntaxFunctionCloneCandidateGenerationMode
syntaxFunctionCloneRetrievalContractVersion
syntaxFunctionCloneSkippedLowDiscriminationBuckets
syntaxFunctionCloneSkippedLowDiscriminationRawPairEstimate
syntaxFunctionCloneSkippedLowDiscriminationPairEstimateKind
```

These fields make the top-level summary honest about the scope of
`syntaxFunctionCloneNearCandidates`.

## lumin-rust-analyzer syntax phase brief

Location:

```text
phases.syntax.summary
```

Fields added:

```text
functionCloneCandidateGenerationPolicy
functionCloneCandidateGenerationSummary
functionCloneSkippedLowDiscriminationBucketCount
functionCloneSkippedLowDiscriminationRawPairEstimate
functionCloneSkippedLowDiscriminationPairEstimateKind
functionCloneSkippedLowDiscriminationBuckets
```

The raw syntax lane remains omitted:

```text
phases.syntax.rawEmbedded = false
```

This keeps product artifacts compact while still exposing why bounded retrieval
did or did not surface near clone review candidates.

## Contract Checks

The Rust analyzer artifact contract now asserts these values through the real
unified analyzer path:

```text
functionCloneCandidateGenerationPolicy.mode = bounded-retrieval
functionCloneCandidateGenerationPolicy.retrievalContractVersion = function-clone-near-retrieval.v1
functionCloneCandidateGenerationSummary.nearFunctionCandidateCountScope = scored-candidates-from-retained-retrieval-evidence
functionCloneSkippedLowDiscriminationPairEstimateKind = raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens
```

The source-health compact artifact contract asserts the same policy/scope
surface directly at `functionCloneGroups`.
