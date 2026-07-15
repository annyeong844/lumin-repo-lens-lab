// Declarative JS contract for selecting a compatible lumin-audit-core helper.
// Runtime probing and process execution remain owned by audit-core.mjs.

export const AUDIT_CORE_CONTRACT_PROBES = [
  [
    ['producer-performance-runtime-artifact'],
    'producer-performance-runtime-artifact: missing --input',
  ],
  [
    ['producer-performance-audit-run-artifact'],
    'producer-performance-audit-run-artifact: missing --input',
  ],
  [
    ['manifest-companion-update'],
    'manifest-companion-update: missing --input',
  ],
  [
    ['manifest-root-with-evidence'],
    'manifest-root-with-evidence: missing --input <path|->',
  ],
  [
    ['manifest-evidence-refresh'],
    'manifest-evidence-refresh: missing --root <repo>',
  ],
  [
    ['manifest-evidence-refresh-with-reads'],
    'manifest-evidence-refresh-with-reads: missing --root <repo>',
  ],
  [
    ['manifest-lifecycle-evidence-refresh'],
    'manifest-lifecycle-evidence-refresh: missing --input <path|->',
  ],
  [
    ['manifest-evidence-summary-with-reads'],
    'manifest-evidence-summary-with-reads: missing --root <repo>',
  ],
  [
    ['manifest-closeout-update'],
    'manifest-closeout-update: missing --input',
  ],
  [
    ['manifest-artifacts-produced-update'],
    'manifest-artifacts-produced-update: missing --output <dir>',
  ],
  [
    ['audit-review-pack-render'],
    'audit-review-pack-render: missing --input <path|->',
  ],
  [
    ['audit-summary-render'],
    'audit-summary-render: missing --input <path|->',
  ],
  [
    ['manifest-write'],
    'manifest-write: missing --output <dir>',
  ],
  [
    ['manifest-closeout-write'],
    'manifest-closeout-write: missing --input <path|->',
  ],
  [
    ['finalize-audit-run'],
    'finalize-audit-run: missing --input <path|->',
  ],
  [
    ['finalize-audit-run-with-companions'],
    'finalize-audit-run-with-companions: missing --input <path|->',
  ],
  [
    ['execute-js-pre-write'],
    'execute-js-pre-write: missing --input <path|->',
  ],
  [
    ['execute-post-write'],
    'execute-post-write: missing --input <path|->',
  ],
  [
    ['execute-audit-lifecycle'],
    'execute-audit-lifecycle: missing --input <path|->',
  ],
  [
    ['barrel-discipline-artifact'],
    'barrel-discipline-artifact: missing --input <path|->',
  ],
  [
    ['block-clones-artifact'],
    'block-clones-artifact: missing --input <path|->',
  ],
  [
    ['call-graph-artifact'],
    'call-graph-artifact: missing --input <path|->',
  ],
  [
    ['checklist-facts-artifact'],
    'checklist-facts-artifact: missing --input <path|->',
  ],
  [
    ['compare-repos-artifact'],
    'compare-repos-artifact: missing --input <path|->',
  ],
  [
    ['dead-classify-artifact'],
    'dead-classify-artifact: missing --input <path|->',
  ],
  [
    ['discipline-artifact'],
    'discipline-artifact: missing --input <path|->',
  ],
  [
    ['entry-surface-artifact'],
    'entry-surface-artifact: missing --input <path|->',
  ],
  [
    ['export-action-safety-artifact'],
    'export-action-safety-artifact: missing --input <path|->',
  ],
  [
    ['unused-deps-artifact'],
    'unused-deps-artifact: missing --input <path|->',
  ],
  [
    ['framework-resource-surfaces-artifact'],
    'framework-resource-surfaces-artifact: missing --input <path|->',
  ],
  [
    ['function-clones-artifact'],
    'function-clones-artifact: missing --input <path|->',
  ],
  [
    ['js-ts-extract-artifact'],
    'js-ts-extract-artifact: missing --input <path|->',
  ],
  [
    ['sfc-file-facts-artifact'],
    'sfc-file-facts-artifact: missing --input <path|->',
  ],
  [
    ['js-ts-pre-write-evidence'],
    'js-ts-pre-write-evidence: missing --input <path|->',
  ],
  [
    ['module-reachability-artifact'],
    'module-reachability-artifact: missing --input <path|->',
  ],
  [
    ['rank-fixes-artifact'],
    'rank-fixes-artifact: missing --input <path|->',
  ],
  [
    ['resolver-diagnostics-artifacts'],
    'resolver-diagnostics-artifacts: missing --input <path|->',
  ],
  [
    ['runtime-evidence-artifact'],
    'runtime-evidence-artifact: missing --input <path|->',
  ],
  [
    ['sarif-artifact'],
    'sarif-artifact: missing --input <path|->',
  ],
  [
    ['shape-index-artifact'],
    'shape-index-artifact: missing --input <path|->',
  ],
  [
    ['source-use-assembly-artifact'],
    'source-use-assembly-artifact: missing --input <path|->',
  ],
  [
    ['staleness-artifact'],
    'staleness-artifact: missing --input <path|->',
  ],
  [
    ['symbol-graph-artifact'],
    'symbol-graph-artifact: missing --input <path|->',
  ],
  [
    ['topology-artifact'],
    'topology-artifact: missing --input <path|->',
  ],
  [
    ['topology-mermaid-render'],
    'topology-mermaid-render: missing --input <path|->',
  ],
];

export const RESULT_FILE_REQUIRED_SUBCOMMANDS = new Set([
  'manifest-root-with-evidence',
  'manifest-lifecycle-evidence-refresh',
  'execute-js-pre-write',
  'execute-rust-pre-write',
  'execute-post-write',
  'manifest-evidence-summary-with-reads',
  'manifest-evidence-refresh-with-reads',
  'audit-review-pack-render',
  'audit-summary-render',
  'finalize-audit-run-with-companions',
  'execute-audit-lifecycle',
  'barrel-discipline-artifact',
  'block-clones-artifact',
  'call-graph-artifact',
  'checklist-facts-artifact',
  'compare-repos-artifact',
  'dead-classify-artifact',
  'discipline-artifact',
  'entry-surface-artifact',
  'export-action-safety-artifact',
  'framework-resource-surfaces-artifact',
  'function-clones-artifact',
  'js-ts-extract-artifact',
  'js-ts-pre-write-evidence',
  'module-reachability-artifact',
  'rank-fixes-artifact',
  'resolver-diagnostics-artifacts',
  'runtime-evidence-artifact',
  'sarif-artifact',
  'sfc-file-facts-artifact',
  'shape-index-artifact',
  'source-use-assembly-artifact',
  'staleness-artifact',
  'symbol-graph-artifact',
  'topology-artifact',
  'topology-mermaid-render',
  'unused-deps-artifact',
]);

export const AUDIT_CORE_RUNTIME_CONTRACT_SCHEMA_VERSION =
  'lumin-audit-core-runtime-contract.v1';
export const AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION =
  'audit-core-js-runtime-bridge.v62';
export const AUDIT_CORE_REQUIRED_FEATURES = [
  'resultOutput',
  'resultOutputSilencesStdout',
  'jsTsExtractNamedImportEvidence',
  'jsTsExtractImportMetaGlobEvidence',
  'jsTsExtractCjsRequireEvidence',
  'jsTsExtractCjsExportSurfaceEvidence',
  'jsTsExtractLiteralDynamicImportEvidence',
  'jsTsExtractDynamicImportOpacity',
  'jsTsExtractPathBackedInput',
  'jsTsExtractLocalOperations',
  'jsTsExtractVueGlobalComponentEvidence',
  'sfcFileFacts',
  'sfcFileConventionFacts',
  'jsTsPreWriteEvidence',
  'jsTsPreWriteDiscovery',
  'jsTsPreWriteIncrementalCache',
  'jsTsPreWriteExactWorktreeByteCache',
  'jsTsPreWriteCanonicalSourceContainment',
  'jsTsPreWriteSingleFlight',
  'checklistFactsIncrementalCache',
  'jsTsPreWritePhaseTiming',
  'jsTsPreWriteShapeEvidence',
  'nativeJsTsPreWriteLifecycle',
  'boundedPreWriteResultHandoff',
  'nativeLifecycleHostEvidenceTransport',
  'jsTsPreWriteFunctionSignatures',
  'jsTsPreWriteInlinePatterns',
  'jsTsPreWriteCurrentEvidenceOnly',
  'sourceUseAssembly',
  'sourceUseAssemblyResolvedRecordTargets',
  'sourceUseAssemblyExternalRecordIds',
  'nonSourceAssetSourceUseAssembly',
  'sourceUseAssemblyConsumerSourceCounters',
  'sourceUseAssemblyProjectionOnlyNonSourceAssets',
  'sourceUseAssemblyRootRelativeSourceFiles',
  'sourceUseAssemblySourceFileIds',
  'sourceUseAssemblyRootRelativeRecordPaths',
  'sourceUseAssemblySyntheticRecordIds',
  'sourceUseAssemblyPathTable',
  'sourceUseAssemblyEnumTable',
  'sourceUseAssemblySpecifierTable',
  'sourceUseAssemblyRecordRows',
  'sourceUseAssemblyNameTable',
  'sourceUseAssemblyTypeOnlyState',
  'sourceUseAssemblyDerivedReExportMaps',
  'sourceUseAssemblyTerminalRecordOutcomes',
  'sourceUseAssemblyResolvedDottedAliases',
  'lintEnforcementFailClosed',
  'workspaceDependencyOwnerManifests',
  'symbolGraphStrictRequestV2',
  'symbolGraphDeadTestCandidates',
  'stalenessBatchPickaxe',
  'generatedVirtualSourceUseAssembly',
  'importMetaGlobSourceUseAssembly',
  'sfcScriptSrcSourceUseAssembly',
  'sharedSourceInventory',
  'sourceInventoryRunBinding',
  'failClosedLifecycleArtifacts',
  'postWriteOnlyBasePipelineSkip',
  'postWriteScopedBaseEvidence',
  'nativePostWriteLifecycle',
  'lifecycleScopedArtifacts',
  'functionCloneBoundedRetrieval',
];

export const AUDIT_CORE_REQUIRED_SUBCOMMANDS = new Set(
  AUDIT_CORE_CONTRACT_PROBES.map(([args]) => args[0]),
);
