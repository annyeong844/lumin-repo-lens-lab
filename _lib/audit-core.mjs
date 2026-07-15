// _lib/audit-core.mjs
//
// Runtime bridge for migrated audit-core contracts.
// Owns locating, validating, building, and invoking the lumin-audit-core helper.

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { symbolGraphContractProbeRequest } from './audit-core-contract-fixtures.mjs';

let auditCoreAutoBuildFailure = null;
let auditCoreBinaryCache = null;
const auditCoreContractCache = new Map();
let windowsHostTempRootCache;

const AUDIT_CORE_CONTRACT_PROBES = [
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

const RESULT_FILE_REQUIRED_SUBCOMMANDS = new Set([
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

const AUDIT_CORE_RUNTIME_CONTRACT_SCHEMA_VERSION = 'lumin-audit-core-runtime-contract.v1';
export const AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION = 'audit-core-js-runtime-bridge.v61';
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
const AUDIT_CORE_REQUIRED_SUBCOMMANDS = new Set(
  AUDIT_CORE_CONTRACT_PROBES.map(([args]) => args[0])
);

function executableOnPath(exe) {
  for (const dir of (process.env.PATH ?? '').split(path.delimiter)) {
    if (!dir) continue;
    const candidate = path.join(dir, exe);
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

function auditCoreBinary() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  const platformKey = `${process.platform}-${process.arch}`;
  const packagedPlatform = path.resolve(here, '../bin', platformKey, exe);
  const sourceCheckoutPlatform = path.resolve(
    here,
    '../skills/lumin-repo-lens-lab/_engine/bin',
    platformKey,
    exe,
  );
  const platformEnv = `LUMIN_AUDIT_CORE_BIN_${process.platform}_${process.arch}`
    .replace(/[^A-Z0-9_]/gi, '_')
    .toUpperCase();
  const cacheKey = JSON.stringify({
    here,
    platform: process.platform,
    arch: process.arch,
    platformOverride: process.env[platformEnv] ?? null,
    genericOverride: process.env.LUMIN_AUDIT_CORE_BIN ?? null,
    path: process.env.PATH ?? '',
    cargoTargetDir: process.env.CARGO_TARGET_DIR ?? null,
    noAutoBuild: process.env.LUMIN_AUDIT_CORE_NO_AUTO_BUILD ?? null,
    preferredCandidates: candidateSignatureKey([
      packagedPlatform,
      sourceCheckoutPlatform,
    ]),
  });
  const configuredOverrides = [process.env[platformEnv], process.env.LUMIN_AUDIT_CORE_BIN]
    .map((configured) => configured ? path.resolve(configured) : null)
    .filter(Boolean);
  const overrideSignatureKey = candidateSignatureKey(configuredOverrides);
  if (auditCoreBinaryCache?.key === cacheKey) {
    const signature = fileSignature(auditCoreBinaryCache.command);
    if (
      signature &&
      signature === auditCoreBinaryCache.signature &&
      overrideSignatureKey === auditCoreBinaryCache.overrideSignatureKey
    ) {
      return auditCoreBinaryCache.command;
    }
  }
  const remember = (command) => {
    auditCoreBinaryCache = {
      key: cacheKey,
      command,
      signature: fileSignature(command),
      overrideSignatureKey,
    };
    return command;
  };
  for (const resolved of configuredOverrides) {
    if (resolved && auditCoreCandidateSupportsCurrentContract(resolved)) return remember(resolved);
  }
  if (auditCoreCandidateSupportsCurrentContract(packagedPlatform)) return remember(packagedPlatform);
  if (auditCoreCandidateSupportsCurrentContract(sourceCheckoutPlatform)) {
    return remember(sourceCheckoutPlatform);
  }
  const packagedSourceManifest = path.resolve(here, '../rust', 'Cargo.toml');
  if (isLuminAuditCoreWorkspace(path.dirname(packagedSourceManifest))) {
    const built = auditCoreBinaryFromManifest(packagedSourceManifest, autoBuildCandidatePath(packagedSourceManifest, exe));
    if (built) return remember(built);
  }
  let cursor = here;
  for (;;) {
    const workspaceRoot = path.join(cursor, 'experiments');
    const manifest = path.join(workspaceRoot, 'Cargo.toml');
    if (isLuminAuditCoreWorkspace(workspaceRoot)) {
      const localCandidate = path.join(workspaceRoot, 'target', 'debug', exe);
      if (auditCoreCandidateSupportsCurrentContract(localCandidate)) return remember(localCandidate);
      const built = auditCoreBinaryFromManifest(manifest, autoBuildCandidatePath(manifest, exe));
      if (built) return remember(built);
    }
    const parent = path.dirname(cursor);
    if (parent === cursor) break;
    cursor = parent;
  }
  const pathBinary = executableOnPath(exe);
  if (pathBinary && auditCoreCandidateSupportsCurrentContract(pathBinary)) return remember(pathBinary);
  const stalePackagedPlatform = [packagedPlatform, sourceCheckoutPlatform]
    .find((candidate) => existsSync(candidate));
  if (stalePackagedPlatform) {
    throw new Error(
      `lumin-audit-core binary at ${stalePackagedPlatform} does not satisfy the required runtime contract.${auditCorePlatformHint()}`
    );
  }
  return remember(packagedPlatform);
}

function isLuminAuditCoreWorkspace(workspaceRoot) {
  return existsSync(path.join(workspaceRoot, 'Cargo.toml')) &&
    existsSync(path.join(workspaceRoot, 'rust-common', 'Cargo.toml')) &&
    existsSync(path.join(workspaceRoot, 'rust-main', 'lumin-audit-core', 'Cargo.toml'));
}

function auditCoreBinaryFromManifest(manifestPath, candidate) {
  if (auditCoreCandidateSupportsCurrentContract(candidate)) {
    return candidate;
  }
  return ensureAuditCoreBuiltFromManifest(manifestPath, candidate)
    ? candidate
    : null;
}

function auditCoreCandidateSupportsCurrentContract(command) {
  const signature = fileSignature(command);
  if (!signature) return false;
  const cached = auditCoreContractCache.get(command);
  if (cached?.signature === signature) return cached.supports;
  const supports = auditCoreBinarySupportsCurrentContract(command);
  auditCoreContractCache.set(command, { signature, supports });
  return supports;
}

function fileSignature(filePath) {
  try {
    const stat = statSync(filePath);
    if (!stat.isFile()) return null;
    return `${stat.size}:${stat.mtimeMs}:${stat.ctimeMs}`;
  } catch {
    return null;
  }
}

function candidateSignatureKey(commands) {
  return JSON.stringify(commands.map((command) => [command, fileSignature(command)]));
}

export function auditCoreRuntimeCandidateSignature() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  const platformKey = `${process.platform}-${process.arch}`;
  const platformEnv = `LUMIN_AUDIT_CORE_BIN_${process.platform}_${process.arch}`
    .replace(/[^A-Z0-9_]/gi, '_')
    .toUpperCase();
  const candidates = [
    process.env[platformEnv] ? path.resolve(process.env[platformEnv]) : null,
    process.env.LUMIN_AUDIT_CORE_BIN ? path.resolve(process.env.LUMIN_AUDIT_CORE_BIN) : null,
    path.resolve(here, '../bin', platformKey, exe),
    path.resolve(
      here,
      '../skills/lumin-repo-lens-lab/_engine/bin',
      platformKey,
      exe,
    ),
  ];

  const packagedSourceManifest = path.resolve(here, '../rust', 'Cargo.toml');
  if (isLuminAuditCoreWorkspace(path.dirname(packagedSourceManifest))) {
    candidates.push(autoBuildCandidatePath(packagedSourceManifest, exe));
  }
  let cursor = here;
  for (;;) {
    const workspaceRoot = path.join(cursor, 'experiments');
    const manifest = path.join(workspaceRoot, 'Cargo.toml');
    if (isLuminAuditCoreWorkspace(workspaceRoot)) {
      candidates.push(path.join(workspaceRoot, 'target', 'debug', exe));
      candidates.push(autoBuildCandidatePath(manifest, exe));
    }
    const parent = path.dirname(cursor);
    if (parent === cursor) break;
    cursor = parent;
  }
  candidates.push(executableOnPath(exe));
  return candidateSignatureKey([...new Set(candidates.filter(Boolean))]);
}

function auditCoreBinarySupportsCurrentContract(command) {
  if (!auditCoreBinaryReportsCurrentContract(command)) return false;
  if (process.env.LUMIN_AUDIT_CORE_FULL_CONTRACT_PROBE === '1') {
    return auditCoreBinarySupportsFixtureContract(command);
  }
  return true;
}

function readAuditCoreRuntimeContract(command, { cwd } = {}) {
  const result = spawnSync(command, ['runtime-contract'], {
    cwd,
    encoding: 'utf8',
  });
  if (result.error || result.status !== 0) return null;
  if ((result.stderr ?? '').trim().length > 0) return null;

  try {
    return JSON.parse(result.stdout ?? '');
  } catch {
    return null;
  }
}

export function auditCoreBinaryReportsCurrentContract(command, options = {}) {
  const contract = readAuditCoreRuntimeContract(command, options);
  if (!contract) return false;

  if (contract?.schemaVersion !== AUDIT_CORE_RUNTIME_CONTRACT_SCHEMA_VERSION) return false;
  if (contract?.contractVersion !== AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION) return false;
  for (const feature of AUDIT_CORE_REQUIRED_FEATURES) {
    if (contract?.features?.[feature] !== true) return false;
  }

  const supported = new Set(Array.isArray(contract.supportedSubcommands)
    ? contract.supportedSubcommands
    : []);
  for (const subcommand of AUDIT_CORE_REQUIRED_SUBCOMMANDS) {
    if (!supported.has(subcommand)) return false;
  }

  const resultOutput = new Set(Array.isArray(contract.resultOutputSubcommands)
    ? contract.resultOutputSubcommands
    : []);
  for (const subcommand of RESULT_FILE_REQUIRED_SUBCOMMANDS) {
    if (!resultOutput.has(subcommand)) return false;
  }

  return true;
}

export function auditCoreRuntimeFeatureEnabled(feature) {
  const command = auditCoreBinary();
  if (!existsSync(command)) return false;
  const contract = readAuditCoreRuntimeContract(command);
  return contract?.features?.[feature] === true;
}

export function auditCoreBinarySupportsFixtureContract(command, { cwd } = {}) {
  for (const [args, expected] of AUDIT_CORE_CONTRACT_PROBES) {
    const result = spawnSync(command, args, {
      cwd,
      encoding: 'utf8',
    });
    if (result.error) return auditCoreContractProbeFailure(
      `${args[0]} failed to start: ${result.error.message}`
    );
    const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
    if (!output.includes(expected)) return auditCoreContractProbeFailure(
      `${args[0]} did not emit the expected missing-input contract`
    );
  }
  return auditCoreBinaryWritesResultFiles(command, { cwd });
}

function auditCoreContractProbeFailure(message) {
  if (process.env.LUMIN_AUDIT_CORE_CONTRACT_DEBUG === '1') {
    console.error(`[audit-core contract] ${message}`);
  }
  return false;
}

function auditCoreBinaryWritesResultFiles(command, { cwd } = {}) {
  const tempDir = mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-contract-'));
  const rootDir = path.join(tempDir, 'root');
  const outputDir = path.join(tempDir, 'out');
  const lifecycleOutputDir = path.join(tempDir, 'lifecycle-out');
  const rootInputPath = path.join(tempDir, 'manifest-root-with-evidence.json');
  const lifecycleInputPath = path.join(tempDir, 'manifest-lifecycle-evidence-refresh.json');
  const auditLifecycleInputPath = path.join(tempDir, 'execute-audit-lifecycle.json');
  const auditLifecycleIntentPath = path.join(tempDir, 'execute-audit-lifecycle-intent.json');
  const jsPreWriteInputPath = path.join(tempDir, 'execute-js-pre-write.json');
  const postWriteInputPath = path.join(tempDir, 'execute-post-write.json');
  const postWriteOutputDir = path.join(tempDir, 'post-write-out');
  const postWriteAdvisoryPath = path.join(
    postWriteOutputDir,
    'pre-write-advisory.PROBE-POST-WRITE.json',
  );
  const barrelDisciplineInputPath = path.join(tempDir, 'barrel-discipline-artifact.json');
  const blockClonesInputPath = path.join(tempDir, 'block-clones-artifact.json');
  const callGraphInputPath = path.join(tempDir, 'call-graph-artifact.json');
  const checklistFactsInputPath = path.join(tempDir, 'checklist-facts-artifact.json');
  const compareReposInputPath = path.join(tempDir, 'compare-repos-artifact.json');
  const compareLeftDir = path.join(tempDir, 'compare-left');
  const compareRightDir = path.join(tempDir, 'compare-right');
  const deadClassifyInputPath = path.join(tempDir, 'dead-classify-artifact.json');
  const disciplineInputPath = path.join(tempDir, 'discipline-artifact.json');
  const entrySurfaceInputPath = path.join(tempDir, 'entry-surface-artifact.json');
  const exportActionSafetyInputPath = path.join(tempDir, 'export-action-safety-artifact.json');
  const functionClonesInputPath = path.join(tempDir, 'function-clones-artifact.json');
  const jsTsExtractInputPath = path.join(tempDir, 'js-ts-extract-artifact.json');
  const sfcFileFactsInputPath = path.join(tempDir, 'sfc-file-facts-artifact.json');
  const jsTsPreWriteInputPath = path.join(tempDir, 'js-ts-pre-write-evidence.json');
  const moduleReachabilityInputPath = path.join(tempDir, 'module-reachability-artifact.json');
  const rankFixesInputPath = path.join(tempDir, 'rank-fixes-artifact.json');
  const resolverDiagnosticsInputPath = path.join(tempDir, 'resolver-diagnostics-artifacts.json');
  const runtimeEvidenceInputPath = path.join(tempDir, 'runtime-evidence-artifact.json');
  const sarifInputPath = path.join(tempDir, 'sarif-artifact.json');
  const shapeIndexInputPath = path.join(tempDir, 'shape-index-artifact.json');
  const sourceUseAssemblyInputPath = path.join(tempDir, 'source-use-assembly-artifact.json');
  const stalenessInputPath = path.join(tempDir, 'staleness-artifact.json');
  const symbolGraphInputPath = path.join(tempDir, 'symbol-graph-artifact.json');
  const topologyInputPath = path.join(tempDir, 'topology-artifact.json');
  const topologyMermaidInputPath = path.join(tempDir, 'topology-mermaid-render.json');
  const topologyMermaidOutputPath = path.join(tempDir, 'topology.mermaid.md');
  const auditReviewPackInputPath = path.join(tempDir, 'audit-review-pack-render.json');
  const auditReviewPackOutputPath = path.join(tempDir, 'audit-review-pack.latest.md');
  const auditSummaryInputPath = path.join(tempDir, 'audit-summary-render.json');
  const auditSummaryOutputPath = path.join(tempDir, 'audit-summary.latest.md');
  const finalizeWithCompanionsInputPath = path.join(tempDir, 'finalize-audit-run-with-companions.json');
  try {
    mkdirSync(rootDir, { recursive: true });
    mkdirSync(path.join(rootDir, 'src'), { recursive: true });
    mkdirSync(outputDir, { recursive: true });
    mkdirSync(lifecycleOutputDir, { recursive: true });
    mkdirSync(postWriteOutputDir, { recursive: true });
    mkdirSync(compareLeftDir, { recursive: true });
    mkdirSync(compareRightDir, { recursive: true });
    spawnSync('git', ['init'], { cwd: rootDir, encoding: 'utf8' });
    writeFileSync(path.join(outputDir, 'triage.json'), JSON.stringify({
      shape: { totalFiles: 1, tsFiles: 0, rsFiles: 1 },
      byLanguage: { rs: 1 },
    }));
    writeFileSync(path.join(outputDir, 'symbols.json'), JSON.stringify({
      uses: {
        external: 0,
        resolvedInternal: 0,
        unresolvedInternal: 0,
        unresolvedInternalRatio: 0,
      },
      meta: {
        supports: {
          anyContamination: true,
        },
      },
      helperOwnersByIdentity: {},
      typeOwnersByIdentity: {},
    }));
    writeFileSync(path.join(outputDir, 'topology.json'), JSON.stringify({
      meta: { generated: '2026-07-02T00:00:00.000Z' },
      summary: { lens: 'runtime', sccCount: 0 },
      crossSubmoduleEdges: [
        { from: 'apps/web', to: 'packages/ui', count: 4 },
      ],
      sccs: [],
      edges: [],
    }));
    writeFileSync(path.join(outputDir, 'fix-plan.json'), JSON.stringify({
      summary: {
        SAFE_FIX: 1,
        REVIEW_FIX: 2,
        DEGRADED: 0,
        MUTED: 0,
      },
    }));
    writeFileSync(path.join(outputDir, 'checklist-facts.json'), JSON.stringify({
      E2_silent_catch: {
        count: 1,
        nonEmptyAnonymousCount: 0,
        unusedParamCount: 0,
      },
    }));
    writeFileSync(rootInputPath, JSON.stringify({
      generated: '2026-07-02T00:00:00.000Z',
      profile: 'quick',
      root: rootDir,
      output: outputDir,
      commandsRun: [],
      skipped: [],
      includeTests: true,
      production: false,
      generatedArtifactsMode: 'default',
    }));
    writeFileSync(lifecycleInputPath, JSON.stringify({
      manifest: {
        meta: { generated: '2026-07-02T00:00:00.000Z' },
        artifactsProduced: [],
      },
      lifecycle: {},
      evidence: {
        root: rootDir,
        output: outputDir,
        includeTests: true,
        production: false,
        generatedArtifactsMode: 'default',
      },
    }));
    writeFileSync(path.join(rootDir, 'package.json'), JSON.stringify({
      name: 'native-pre-write-contract-probe',
    }));
    writeFileSync(path.join(rootDir, 'src', 'thing.ts'), [
      'export function Thing(value: string): number {',
      '  try { return value.length; } catch { cleanup(); }',
      '}',
      '',
    ].join('\n'));
    for (const file of ['second.ts', 'third.ts']) {
      writeFileSync(
        path.join(rootDir, 'src', file),
        'export function work(): void { try { perform(); } catch { cleanup(); } }\n',
      );
    }
    const nativeJsIntent = JSON.stringify({
      language: 'js-ts',
      names: ['Thing'],
      files: ['src/thing.ts'],
      shapes: [{ typeLiteral: '(value: string) => number' }],
      dependencies: [],
      refactorSources: [{ file: 'src/thing.ts' }],
      plannedTypeEscapes: [],
    });
    writeFileSync(jsPreWriteInputPath, JSON.stringify({
      schemaVersion: 'lumin-js-pre-write-lifecycle-request.v3',
      root: rootDir,
      output: outputDir,
      invocationId: 'PROBE',
      intentInput: nativeJsIntent,
      engineSelection: {
        requested: 'auto',
        selected: 'js',
        reason: 'contract-probe',
      },
      generated: '2026-07-13T00:00:00.000Z',
      includeTests: false,
      production: true,
      excludes: [],
      incremental: { enabled: false },
    }));
    writeFileSync(postWriteAdvisoryPath, JSON.stringify({
      invocationId: 'PROBE-POST-WRITE',
      intentHash: 'probe-post-write-intent',
      intent: {
        names: [],
        shapes: [],
        files: ['src/probe.ts'],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      scanRange: { output: postWriteOutputDir },
      preWrite: {
        anyInventoryPath: 'any-inventory.pre.PROBE-POST-WRITE.json',
        fileInventory: { status: 'available', files: [] },
      },
      capabilities: {
        language: 'js-ts',
        postWriteTypeEscapes: 'available',
      },
    }));
    writeFileSync(
      path.join(postWriteOutputDir, 'any-inventory.pre.PROBE-POST-WRITE.json'),
      JSON.stringify({
        meta: {
          complete: true,
          scope: 'TS/JS production files',
          includeTests: false,
          exclude: [],
          filesWithParseErrors: [],
          supports: {
            typeEscapes: true,
            escapeKinds: [
              'explicit-any',
              'as-any',
              'angle-any',
              'as-unknown-as-T',
              'rest-any-args',
              'index-sig-any',
              'generic-default-any',
              'ts-ignore',
              'ts-expect-error',
              'no-explicit-any-disable',
              'jsdoc-any',
            ],
          },
        },
        typeEscapes: [],
      }),
    );
    writeFileSync(postWriteInputPath, JSON.stringify({
      schemaVersion: 'lumin-post-write-lifecycle-request.v3',
      root: rootDir,
      output: postWriteOutputDir,
      advisoryPath: postWriteAdvisoryPath,
      deltaOut: null,
      deltaInvocationId: 'PROBE-DELTA',
      generated: '2026-07-13T00:00:00.000Z',
      includeTests: false,
      excludes: [],
      incremental: { enabled: false, clear: false },
    }));
    writeFileSync(auditLifecycleInputPath, JSON.stringify({
      schemaVersion: 'lumin-audit-lifecycle-execution-request.v1',
      baseExitCode: 0,
      lifecycleRequestGuard: {
        schemaVersion: 'lumin-lifecycle-request-guard.v1',
        preWriteRequested: true,
        postWriteRequested: false,
        preWriteIntentPresent: true,
        requestedPreWriteEngine: 'auto',
      },
      preWrite: {
        requested: true,
        routingInput: {
          schemaVersion: 'lumin-pre-write-routing-input.v1',
          requestedEngine: 'auto',
          intentFlag: auditLifecycleIntentPath,
        },
        rust: {
          root: rootDir,
          output: lifecycleOutputDir,
          invocationId: 'PROBE-LIFECYCLE',
          rustNativeArtifactPath: path.join(lifecycleOutputDir, 'rust-pre-write-artifact.PROBE-LIFECYCLE.json'),
          rustNativeLatestPath: path.join(lifecycleOutputDir, 'rust-pre-write-artifact.latest.json'),
          analyzer: null,
          includeTests: true,
          production: false,
          excludes: [],
          fileInventory: { status: 'available', files: [] },
          failures: [],
        },
        js: {
          root: rootDir,
          output: lifecycleOutputDir,
          invocationId: 'PROBE-LIFECYCLE',
          generated: '2026-07-13T00:00:00.000Z',
          includeTests: false,
          production: true,
          excludes: [],
          incremental: { enabled: false },
        },
      },
      exitPolicy: {
        strictPostWrite: false,
        strictPostWriteConfidence: false,
      },
    }));
    writeFileSync(auditLifecycleIntentPath, nativeJsIntent);
    writeFileSync(path.join(rootDir, 'probe.ts'), 'const value: any = input as any; // TODO\n');
    writeFileSync(path.join(rootDir, 'src', 'App.css'), '.probe { color: red; }\n');
    writeFileSync(barrelDisciplineInputPath, JSON.stringify({
      schemaVersion: 'lumin-barrel-discipline-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      mode: 'single-package',
      skipped: true,
      reason: 'contract-probe',
    }));
    const blockCloneToken = (value, file, index) => ({
      value,
      file,
      start: index,
      end: index + 1,
      line: index + 1,
      endLine: index + 1,
      container: null,
    });
    const blockCloneValues = ['A', 'B', 'C', 'D'];
    writeFileSync(blockClonesInputPath, JSON.stringify({
      schemaVersion: 'lumin-block-clones-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      includeTests: true,
      exclude: [],
      files: [
        {
          relFile: 'src/a.ts',
          tokens: blockCloneValues.map((value, index) => blockCloneToken(value, 'src/a.ts', index)),
          skipped: null,
          diagnostics: [],
          tokenLimitExceeded: false,
        },
        {
          relFile: 'src/b.ts',
          tokens: blockCloneValues.map((value, index) => blockCloneToken(value, 'src/b.ts', index)),
          skipped: null,
          diagnostics: [],
          tokenLimitExceeded: false,
        },
      ],
      thresholds: {
        minTokens: 3,
        minLines: 1,
        minOccurrences: 2,
        maxInstancesPerGroup: 20,
        maxCandidateGroups: 100,
        maxReviewGroups: 100,
        maxMutedGroups: 100,
        maxTokensPerFile: 200000,
      },
      incremental: {
        enabled: false,
        reason: 'contract-probe',
      },
    }));
    writeFileSync(callGraphInputPath, JSON.stringify({
      schemaVersion: 'lumin-call-graph-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      fileCount: 2,
      parseErrors: 0,
      parseErrorDetails: [],
      totalCallExpressions: 2,
      totalDirectCalls: 1,
      resolvedDirectCalls: 2,
      typeOnlyResolved: 0,
      callEdges: [
        {
          from: path.join(rootDir, 'src', 'b.ts'),
          to: path.join(rootDir, 'src', 'a.ts'),
          callee: 'alpha',
          count: 2,
        },
      ],
      exportAliasMap: {
        'src/a.ts::alpha': 'src/a.ts#FunctionDeclaration:7-37',
      },
      boundedOutMemberCallsByFile: {
        'src/a.ts': 0,
        'src/b.ts': 0,
      },
      memberCallsByFile: {
        'src/a.ts': 0,
        'src/b.ts': 1,
      },
      semiDeadList: [],
      semiDeadReactFiltered: 0,
      prototypeCalls: [],
    }));
    const functionCloneProbeFact = (
      file,
      name,
      line,
      { exactHash, structureHash, signatureHash, callTokens },
    ) => ({
      kind: 'function-body-fingerprint',
      identity: `${file}::${name}`,
      exportedName: name,
      localName: name,
      visibility: 'exported',
      exported: true,
      ownerFile: file,
      line,
      endLine: line + 4,
      bodyLineStart: line + 1,
      bodyLineEnd: line + 3,
      bodyLoc: 3,
      declarationKind: 'FunctionDeclaration',
      functionKind: 'FunctionDeclaration',
      async: false,
      generator: false,
      paramCount: 1,
      statementCount: 2,
      exactBodyHash: `raw-${exactHash}`,
      normalizedExactHash: exactHash,
      normalizedStructureHash: structureHash,
      normalizedSignatureHash: signatureHash,
      signature: 'fn(value)',
      callTokens,
      source: 'fresh-ast-pass',
      scope: 'scope',
      confidence: 'high',
    });
    const functionCloneFacts = [
      functionCloneProbeFact('src/a.ts', 'alpha', 1, {
        exactHash: 'exact-grouped',
        structureHash: 'structure-grouped',
        signatureHash: 'signature-grouped',
        callTokens: ['groupedProbeCall'],
      }),
      functionCloneProbeFact('src/b.ts', 'beta', 4, {
        exactHash: 'exact-grouped',
        structureHash: 'structure-grouped',
        signatureHash: 'signature-grouped',
        callTokens: ['groupedProbeCall'],
      }),
      functionCloneProbeFact('src/c.ts', 'loadProbeAlpha', 7, {
        exactHash: 'exact-near-a',
        structureHash: 'structure-near-a',
        signatureHash: 'signature-near-a',
        callTokens: ['rareProbeCall'],
      }),
      functionCloneProbeFact('src/d.ts', 'loadProbeBeta', 10, {
        exactHash: 'exact-near-b',
        structureHash: 'structure-near-b',
        signatureHash: 'signature-near-b',
        callTokens: ['rareProbeCall'],
      }),
      ...Array.from({ length: 60 }, (_, index) => functionCloneProbeFact(
        `src/noise-${index}.ts`,
        `noiseFunction${index}`,
        index + 20,
        {
          exactHash: `exact-noise-${index}`,
          structureHash: `structure-noise-${index}`,
          signatureHash: `signature-noise-${index}`,
          callTokens: [`noiseProbeCall${index}`],
        },
      )),
    ];
    writeFileSync(functionClonesInputPath, JSON.stringify({
      schemaVersion: 'lumin-function-clones-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      includeTests: true,
      exclude: [],
      scope: 'TS/JS including tests, top-level exported and file-local functions',
      fileCount: functionCloneFacts.length,
      facts: functionCloneFacts,
      diagnostics: [],
      filesWithParseErrors: [],
      filesWithReadErrors: [],
    }));
    const jsTsExtractProbeSource = 'import { api, bare } from "./dep";\napi.foo();\nconst cjsApi = require("./cjs-api");\ncjsApi.run();\nconst { cjsExact } = require("./cjs-exact");\nrequire("./cjs-side-effect");\nrequire(target);\nexport const view = bare;\nexport const routes = import.meta.glob("./pages/*.ts");\nexport async function load(target) {\n  const mod = await import("web-tree-sitter");\n  Parser = mod.Parser;\n  const lazy = await import("./lazy");\n  lazy.boot();\n  await import(`./pages/${target}.ts`);\n  return import(target);\n}\nexport function buildProbeRepository() {\n  function getProbe() { return null; }\n}\nexports.probe = 1;\nmodule.exports.namedProbe = 2;\nexports[dynamicName] = 3;\nmodule.exports = { objectProbe: 4 };\nmodule.exports = makeExports();\nimport ProbeCard from "./ProbeCard.vue";\nconst app = createApp({});\napp.component("ProbeCard", ProbeCard);\n';
    writeFileSync(path.join(rootDir, 'src', 'consumer.mjs'), jsTsExtractProbeSource);
    writeFileSync(path.join(rootDir, 'src', 'dep.ts'), 'export const api = { foo() {} };\nexport const bare = 1;\nexport const escaped = bare as any;\nexport interface ProbeShape { id: string }\n');
    writeFileSync(jsTsExtractInputPath, JSON.stringify({
      schemaVersion: 'lumin-js-ts-extract-request.v1',
      files: [
        {
          filePath: path.join(rootDir, 'src', 'consumer.mjs'),
          artifactFilePath: 'src/consumer.mjs',
        },
      ],
    }));
    writeFileSync(sfcFileFactsInputPath, JSON.stringify({
      schemaVersion: 'lumin-sfc-file-facts-request.v1',
      files: [
        {
          filePath: 'src/App.vue',
          source: [
            '<template><ProbeCard /><UI.Panel /></template>',
            '<script setup lang="ts">',
            'import ProbeCard from "./ProbeCard.vue";',
            'import * as UI from "./ui";',
            'defineOptions({ components: { ProbeCard } });',
            '</script>',
            '<script src="./setup.ts"></script>',
            '<style>@import "./theme.css";</style>',
          ].join('\n'),
        },
        {
          filePath: 'src/Options.vue',
          source: [
            '<script>',
            'import OptionsCard from "./OptionsCard.vue";',
            'export default { components: { OptionsCard } };',
            '</script>',
            '<template><OptionsCard /></template>',
          ].join('\n'),
        },
        {
          filePath: 'src/Page.astro',
          source: [
            '---',
            'import AstroCard from "./AstroCard.astro";',
            '---',
            '<AstroCard client:load />',
          ].join('\n'),
        },
        {
          filePath: 'src/Panel.svelte',
          source: [
            '<script>',
            'import { enhance } from "./actions";',
            'import { count } from "./stores";',
            '</script>',
            '<form use:enhance>{$count}</form>',
          ].join('\n'),
        },
      ],
    }));
    writeFileSync(jsTsPreWriteInputPath, JSON.stringify({
      schemaVersion: 'lumin-js-ts-pre-write-evidence-request.v1',
      root: rootDir,
      evidenceArtifact: 'pre-write-evidence.PROBE.json',
      anyInventoryArtifact: 'any-inventory.pre.PROBE.json',
      generated: '2026-07-11T00:00:00.000Z',
      includeTests: true,
      excludes: [],
      dependencyRoots: ['web-tree-sitter'],
      shapeTypeLiterals: ['{ id: string }'],
      discoverFiles: true,
      files: [],
    }));
    writeFileSync(
      symbolGraphInputPath,
      JSON.stringify(symbolGraphContractProbeRequest(rootDir)),
    );
    writeFileSync(checklistFactsInputPath, JSON.stringify({
      schemaVersion: 'lumin-checklist-facts-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      filesScanned: 1,
      inputs: {
        triage: {
          boundaries: [],
          lintEnforcement: {
            status: 'degraded',
            unsupportedCommands: [
              { scriptName: 'lint', command: 'newlint .' },
            ],
          },
        },
      },
      incremental: {
        enabled: true,
        changedFiles: 0,
        reusedFiles: 1,
      },
      astFacts: {
        functionSize: {
          parseErrors: 0,
          entries: [
            { file: 'probe.ts', line: 1, name: 'probe', loc: 1, fileRole: 'production' },
          ],
        },
        silentCatch: {
          analysis: 'oxc-ast-catch-clause',
          parseErrors: 0,
          sites: [],
          documentedSites: [],
          anonymousSites: [],
          nonEmptyAnonymousSites: [],
          unusedParamSites: [],
        },
      },
    }));
    writeFileSync(path.join(compareLeftDir, 'triage.json'), JSON.stringify({
      summary: { files: 1, loc: 10 },
    }));
    writeFileSync(path.join(compareRightDir, 'triage.json'), JSON.stringify({
      summary: { files: 3, loc: 16 },
    }));
    writeFileSync(path.join(compareRightDir, 'runtime-evidence.json'), '{}');
    writeFileSync(compareReposInputPath, JSON.stringify({
      schemaVersion: 'lumin-compare-repos-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      left: compareLeftDir,
      right: compareRightDir,
      leftLabel: 'left',
      rightLabel: 'right',
    }));
    writeFileSync(deadClassifyInputPath, JSON.stringify({
      schemaVersion: 'lumin-dead-classify-producer-request.v1',
      classifiedCandidates: [
        {
          file: 'src/dead.ts',
          line: 1,
          symbol: 'Dead',
          kind: 'FunctionDeclaration',
          fileInternalUses: 0,
          fileInternalUsesEvidence: 'ast-ident-ref-count',
        },
        {
          file: 'src/hub.ts',
          line: 2,
          symbol: 'Hub',
          kind: 'TSInterfaceDeclaration',
          fileInternalUses: 3,
        },
      ],
      excludedCandidates: [],
      unprocessedCandidates: [],
      excludedSummary: {
        config_FP22: 0,
        publicApi_FP23: 0,
        scriptEntrypoint_FP45: 0,
        htmlEntrypoint_FP47: 0,
        frameworkSentinel_FP27: 0,
        nuxtNitro_FP30: 0,
        vitePress_FP46: 0,
        declarationSidecar_FP48: 0,
        dynamicImportOpacity_FP18: 0,
        testConsumer_FP44: 0,
        transitiveBarrelAdded_FP25: 0,
        isNuxtNitroDetected: false,
        testConsumerDiagnostics_FP44: 0,
      },
      frameworkPolicy: {},
      performance: { deadCandidatesProcessed: 2 },
      incomplete: false,
    }));
    writeFileSync(disciplineInputPath, JSON.stringify({
      schemaVersion: 'lumin-discipline-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      files: ['probe.ts'],
    }));
    writeFileSync(entrySurfaceInputPath, JSON.stringify({
      schemaVersion: 'lumin-entry-surface-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      includeTests: true,
      knownFiles: ['probe.ts'],
      parseErrorCount: 0,
      submoduleByFile: { 'probe.ts': '.' },
      publicApi: {
        files: ['probe.ts'],
        transitiveAdded: 0,
        evidenceByFile: {
          'probe.ts': [{ source: 'contract-probe' }],
        },
      },
      script: {},
      html: {},
      framework: {},
      config: {},
    }));
    writeFileSync(exportActionSafetyInputPath, JSON.stringify({
      schemaVersion: 'lumin-export-action-safety-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      findings: [
        {
          id: 'dead-export:probe.ts:value:1',
          file: 'probe.ts',
          symbol: 'value',
          line: 1,
          bucket: 'C',
          safeAction: null,
          actionBlockers: ['contract-probe'],
        },
      ],
      warnings: [],
    }));
    writeFileSync(moduleReachabilityInputPath, JSON.stringify({
      schemaVersion: 'lumin-module-reachability-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      symbols: {
        defIndex: {},
        reExportsByFile: {},
        resolvedInternalEdges: [],
      },
      entrySurface: {
        entryFiles: [],
      },
      maxFilesVisited: 200000,
      maxEdgesVisited: 400000,
    }));
    writeFileSync(rankFixesInputPath, JSON.stringify({
      schemaVersion: 'lumin-rank-fixes-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      artifacts: {
        deadClassify: {
          proposal_C_remove_symbol: [],
          proposal_A_demote_to_internal: [],
          proposal_B_review: [],
          proposal_remove_export_specifier: [],
          proposal_DEGRADED_unprocessed: [],
          excludedCandidates: [],
        },
      },
      publicDeepImportRiskByFile: {
        '__lumin_contract_probe__': { risk: false, reason: 'contract-probe' },
      },
    }));
    writeFileSync(resolverDiagnosticsInputPath, JSON.stringify({
      schemaVersion: 'lumin-resolver-diagnostics-producer-request.v1',
      symbols: {
        uses: {
          unresolvedInternal: 0,
          unresolvedInternalRatio: 0,
          external: 0,
        },
        unresolvedInternalSpecifierRecords: [],
        generatedConsumerBlindZones: [],
      },
    }));
    writeFileSync(runtimeEvidenceInputPath, JSON.stringify({
      schemaVersion: 'lumin-runtime-evidence-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      symbolsSource: 'symbols.json',
      coverageSource: 'coverage-final.json',
      coverageMtime: '2026-07-02T00:00:00.000Z',
      symbols: {
        deadProdList: [],
      },
      coverage: {},
    }));
    writeFileSync(sarifInputPath, JSON.stringify({
      schemaVersion: 'lumin-sarif-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      fixPlan: {
        safeFixes: [],
        reviewFixes: [],
        degraded: [],
        muted: [],
      },
    }));
    const shapeHash = `sha256:${'0'.repeat(64)}`;
    writeFileSync(shapeIndexInputPath, JSON.stringify({
      schemaVersion: 'lumin-shape-index-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      includeTests: true,
      exclude: [],
      scope: 'TS/JS including tests, exported types only',
      observedAt: '2026-07-02T00:00:00.000Z',
      fileCount: 1,
      facts: [
        {
          kind: 'shape-hash',
          hash: shapeHash,
          identities: ['probe.ts::Probe'],
          identity: 'probe.ts::Probe',
          exportedName: 'Probe',
          ownerFile: 'probe.ts',
          typeKind: 'TSInterfaceDeclaration',
          shapeKind: 'object',
          line: 1,
          fields: [],
          source: 'fresh-ast-pass',
          scope: 'TS/JS including tests, exported types only',
          confidence: 'high',
        },
      ],
      diagnostics: [],
      filesWithParseErrors: [],
      filesWithReadErrors: [],
      incremental: {
        enabled: false,
        identityMode: null,
        cacheVersion: 1,
        cacheRoot: null,
        changedFiles: 1,
        reusedFiles: 0,
        droppedFiles: 0,
        invalidatedFiles: 0,
        reason: 'contract-probe',
      },
    }));
    writeFileSync(sourceUseAssemblyInputPath, JSON.stringify({
      schemaVersion: 'lumin-source-use-assembly-request.v1',
      root: rootDir,
      pathTable: [
        path.join(rootDir, 'src', 'consumer.ts'),
        path.join(rootDir, 'src', 'dep.ts'),
        path.join(rootDir, 'src', 'setup.ts'),
      ],
      sourceFileIds: [0, 1, 2],
      nameTable: ['value'],
      recordRowFields: [
        'recordId',
        'consumerFileId',
        'fromSpec',
        'nameId',
        'kind',
        'typeOnlyState',
        'line',
        'resolverStage',
        'consumerSource',
      ],
      recordRows: [[
        'src/consumer.ts#0',
        0,
        './dep',
        0,
        'import',
        1,
        1,
        'relative',
        'mdx-import',
      ]],
      records: [
        {
          recordId: 'src/consumer.ts#1',
          consumerFile: path.join(rootDir, 'src', 'consumer.ts'),
          fromSpec: './dep',
          kind: 'namespace',
          resolverStage: 'relative',
          consumerSource: 'sfc-script-import',
        },
        {
          recordId: 'src/consumer.ts#2',
          consumerFile: path.join(rootDir, 'src', 'consumer.ts'),
          fromSpec: '@/missing',
          kind: 'import',
          typeOnly: false,
          typeOnlyPresent: true,
          resolverStage: 'unresolved-internal',
          unresolvedEvidence: {
            reason: 'tsconfig-path-target-missing',
            resolverStage: 'tsconfig-paths',
            matchedPattern: '@/*',
            targetCandidates: ['src/missing.ts'],
            hint: 'check-tsconfig-paths',
          },
        },
        {
          recordId: 'src/consumer.ts#3',
          consumerFile: path.join(rootDir, 'src', 'consumer.ts'),
          fromSpec: '@pkg/db/enums',
          name: 'Role',
          kind: 'import',
          typeOnly: false,
          typeOnlyPresent: true,
          resolverStage: 'generated-virtual',
          generatedVirtualSurface: {
            id: 'generated-virtual:prisma-enums:@pkg/db:enums',
            source: 'generated-virtual',
            mode: 'virtual',
            virtual: true,
            exports: [{
              name: 'Role',
              kind: 'prisma-enum',
              spaces: ['value', 'type'],
            }],
          },
        },
        {
          recordId: 'src/consumer.ts#4:glob:src/dep.ts',
          consumerFile: path.join(rootDir, 'src', 'consumer.ts'),
          resolvedFile: path.join(rootDir, 'src', 'dep.ts'),
          fromSpec: './*.ts',
          kind: 'dynamic-import-meta-glob',
          resolverStage: 'resolved-internal',
        },
        {
          recordId: 'sfc-script-src:0:src/App.vue:./setup',
          consumerFile: path.join(rootDir, 'src', 'App.vue'),
          resolvedFile: path.join(rootDir, 'src', 'setup.ts'),
          fromSpec: './setup',
          kind: 'sfc-script-src',
          sfcLanguage: 'vue',
          resolverStage: 'relative',
          consumerSource: 'sfc-script-src',
        },
        {
          recordId: 'src/consumer.ts#6',
          consumerFile: path.join(rootDir, 'src', 'consumer.ts'),
          resolvedFile: path.join(rootDir, 'src', 'style.css'),
          fromSpec: './style.css',
          kind: 'import-side-effect',
          resolverStage: 'non-source-asset',
        },
        {
          recordId: 'src/consumer.ts#7',
          consumerFile: path.join(rootDir, 'src', 'consumer.ts'),
          resolvedFile: path.join(rootDir, 'src', 'layout.config.ts'),
          fromSpec: '@/layout.config',
          name: 'layoutConfig',
          kind: 'import',
          resolverStage: 'resolved-internal',
        },
      ],
    }));
    writeFileSync(stalenessInputPath, JSON.stringify({
      schemaVersion: 'lumin-staleness-producer-request.v1',
      root: rootDir,
      generated: '2026-07-02T00:00:00.000Z',
      symbolsSource: 'symbols.json',
      symbols: {
        deadProdList: [],
      },
      skipPickaxe: true,
      incrementalEnabled: false,
    }));
    const topologyProbe = path.join(rootDir, 'probe.ts');
    const topologyDep = path.join(rootDir, 'dep.ts');
    writeFileSync(topologyInputPath, JSON.stringify({
      schemaVersion: 'lumin-topology-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      mode: 'single-package',
      rootPkgName: 'contract-probe',
      includeTypeEdges: false,
      files: [topologyProbe, topologyDep],
      sourceEntries: {
        [topologyProbe]: {
          loc: 1,
          edges: [{ to: topologyDep }],
          externalCount: 0,
          unresolvedCount: 0,
          parseError: false,
        },
        [topologyDep]: {
          loc: 1,
          edges: [],
          externalCount: 0,
          unresolvedCount: 0,
          parseError: false,
        },
      },
      submoduleByFile: {
        [topologyProbe]: 'root',
        [topologyDep]: 'root',
      },
      performance: {
        filesCollected: 2,
        scannerPolicyVersion: 'contract-probe',
      },
      rustMetadata: {},
    }));
    writeFileSync(topologyMermaidInputPath, JSON.stringify({
      schemaVersion: 'lumin-topology-mermaid-render-request.v1',
      outputPath: topologyMermaidOutputPath,
      topology: {
        meta: { generated: '2026-07-02T00:00:00.000Z' },
        summary: { lens: 'runtime' },
        crossSubmoduleEdges: [
          { from: 'apps/web', to: 'packages/ui', count: 4 },
        ],
        sccs: [],
        edges: [],
      },
    }));
    writeFileSync(auditReviewPackInputPath, JSON.stringify({
      schemaVersion: 'lumin-audit-review-pack-render-request.v1',
      outputPath: auditReviewPackOutputPath,
      manifest: {
        profile: 'full',
        scanRange: {
          files: 2,
          languages: ['ts'],
          includeTests: true,
        },
        rustAnalysis: {
          requested: false,
        },
      },
      checklistFacts: {
        E2_silent_catch: {
          count: 1,
          nonEmptyAnonymousCount: 0,
          unusedParamCount: 0,
        },
      },
      fixPlan: {
        summary: {
          SAFE_FIX: 1,
          REVIEW_FIX: 2,
          DEGRADED: 0,
          MUTED: 0,
        },
      },
      topology: {
        summary: {
          sccCount: 1,
        },
        sccs: [],
      },
      discipline: {
        totals: {
          ':any': 1,
        },
      },
      callGraph: {
        summary: {
          semiDead: 1,
        },
      },
      barrels: {
        root: {},
      },
      symbols: {
        meta: {
          supports: {
            anyContamination: true,
          },
        },
        helperOwnersByIdentity: {},
        typeOwnersByIdentity: {},
      },
    }));
    writeFileSync(auditSummaryInputPath, JSON.stringify({
      schemaVersion: 'lumin-audit-summary-render-request.v1',
      outputPath: auditSummaryOutputPath,
      manifest: {
        meta: { generated: '2026-07-02T00:00:00.000Z' },
        profile: 'full',
        scanRange: {
          files: 2,
          languages: ['ts'],
          includeTests: true,
        },
        confidence: {
          parseErrors: 0,
          unresolvedInternalRatio: 0,
        },
        rustAnalysis: {
          requested: false,
        },
      },
      checklistFacts: {
        E2_silent_catch: {
          count: 1,
          nonEmptyAnonymousCount: 0,
          unusedParamCount: 0,
        },
      },
      fixPlan: {
        summary: {
          SAFE_FIX: 1,
          REVIEW_FIX: 2,
          DEGRADED: 0,
          MUTED: 0,
        },
      },
      topology: {
        summary: {
          sccCount: 1,
        },
        sccs: [],
      },
      discipline: {
        totals: {
          ':any': 1,
        },
      },
      callGraph: {
        summary: {
          semiDead: 1,
        },
      },
      symbols: {
        meta: {
          supports: {
            anyContamination: true,
          },
        },
        helperOwnersByIdentity: {},
        typeOwnersByIdentity: {},
      },
    }));
    writeFileSync(finalizeWithCompanionsInputPath, JSON.stringify({
      manifest: {
        meta: { generated: '2026-07-02T00:00:00.000Z' },
        profile: 'full',
        scanRange: {
          files: 2,
          languages: ['ts'],
          includeTests: true,
        },
        confidence: {
          parseErrors: 0,
          unresolvedInternalRatio: 0,
        },
        artifactsProduced: [],
        blindZones: [],
        rustAnalysis: {
          requested: false,
        },
      },
      context: {
        generated: '2026-07-02T00:00:00.000Z',
        root: rootDir,
        output: outputDir,
        profile: 'full',
        includeTests: true,
        production: false,
        excludes: [],
        autoExcludes: [],
        noIncremental: false,
        cacheRoot: path.join(outputDir, '.cache'),
        clearIncrementalCache: false,
        generatedArtifactsMode: 'default',
      },
      artifactReadEvents: {
        schemaVersion: 'lumin-audit-artifact-read-events.v1',
        rootDir: outputDir,
        largestLimit: 10,
        reads: [],
      },
      commandsRun: [],
      skipped: [],
      rustAnalysis: null,
      companionPolicy: {
        basePipelinePlanned: true,
      },
    }));

    const probes = [
      {
        subcommand: 'manifest-root-with-evidence',
        args: ['manifest-root-with-evidence', '--input', rootInputPath],
        requiredField: 'manifest',
      },
      {
        subcommand: 'manifest-lifecycle-evidence-refresh',
        args: ['manifest-lifecycle-evidence-refresh', '--input', lifecycleInputPath],
        requiredField: 'manifest',
      },
      {
        subcommand: 'execute-js-pre-write',
        args: ['execute-js-pre-write', '--input', jsPreWriteInputPath],
        requiresArtifactReads: false,
        outputDir,
        expectedStdoutIncludes: [
          '## pre-write advisory',
          'Summary: cueCards=',
          'stdout does not duplicate its per-candidate rows',
        ],
        expectedStdoutExcludes: ['### Agent review cues', '### Lookup results'],
      },
      {
        subcommand: 'execute-post-write',
        args: ['execute-post-write', '--input', postWriteInputPath],
        requiresArtifactReads: false,
        outputDir: postWriteOutputDir,
        expectedStdoutIncludes: '## post-write delta',
      },
      {
        subcommand: 'execute-audit-lifecycle',
        args: ['execute-audit-lifecycle', '--input', auditLifecycleInputPath],
        requiresArtifactReads: false,
        outputDir: lifecycleOutputDir,
        expectedStdoutIncludes: [
          '## pre-write advisory',
          'Summary: cueCards=',
          'stdout does not duplicate its per-candidate rows',
        ],
        expectedStdoutExcludes: ['### Agent review cues', '### Lookup results'],
      },
      {
        subcommand: 'manifest-evidence-summary-with-reads',
        args: [
          'manifest-evidence-summary-with-reads',
          '--root', rootDir,
          '--output', outputDir,
          '--include-tests',
          '--no-production',
        ],
        requiredField: 'evidence',
      },
      {
        subcommand: 'manifest-evidence-refresh-with-reads',
        args: [
          'manifest-evidence-refresh-with-reads',
          '--root', rootDir,
          '--output', outputDir,
          '--include-tests',
          '--no-production',
        ],
        requiredField: 'evidence',
      },
      {
        subcommand: 'barrel-discipline-artifact',
        args: ['barrel-discipline-artifact', '--input', barrelDisciplineInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'block-clones-artifact',
        args: ['block-clones-artifact', '--input', blockClonesInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'call-graph-artifact',
        args: ['call-graph-artifact', '--input', callGraphInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'function-clones-artifact',
        args: ['function-clones-artifact', '--input', functionClonesInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'js-ts-extract-artifact',
        args: ['js-ts-extract-artifact', '--input', jsTsExtractInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'sfc-file-facts-artifact',
        args: ['sfc-file-facts-artifact', '--input', sfcFileFactsInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'js-ts-pre-write-evidence',
        args: ['js-ts-pre-write-evidence', '--input', jsTsPreWriteInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'symbol-graph-artifact',
        args: ['symbol-graph-artifact', '--input', symbolGraphInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'checklist-facts-artifact',
        args: ['checklist-facts-artifact', '--input', checklistFactsInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'compare-repos-artifact',
        args: ['compare-repos-artifact', '--input', compareReposInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'dead-classify-artifact',
        args: ['dead-classify-artifact', '--input', deadClassifyInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'discipline-artifact',
        args: ['discipline-artifact', '--input', disciplineInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'entry-surface-artifact',
        args: ['entry-surface-artifact', '--input', entrySurfaceInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'export-action-safety-artifact',
        args: ['export-action-safety-artifact', '--input', exportActionSafetyInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'module-reachability-artifact',
        args: ['module-reachability-artifact', '--input', moduleReachabilityInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'rank-fixes-artifact',
        args: ['rank-fixes-artifact', '--input', rankFixesInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'resolver-diagnostics-artifacts',
        args: ['resolver-diagnostics-artifacts', '--input', resolverDiagnosticsInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'runtime-evidence-artifact',
        args: ['runtime-evidence-artifact', '--input', runtimeEvidenceInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'sarif-artifact',
        args: ['sarif-artifact', '--input', sarifInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'shape-index-artifact',
        args: ['shape-index-artifact', '--input', shapeIndexInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'source-use-assembly-artifact',
        args: ['source-use-assembly-artifact', '--input', sourceUseAssemblyInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'staleness-artifact',
        args: ['staleness-artifact', '--input', stalenessInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'topology-artifact',
        args: ['topology-artifact', '--input', topologyInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'topology-mermaid-render',
        args: ['topology-mermaid-render', '--input', topologyMermaidInputPath],
        requiresArtifactReads: false,
        outputPath: topologyMermaidOutputPath,
      },
      {
        subcommand: 'audit-review-pack-render',
        args: ['audit-review-pack-render', '--input', auditReviewPackInputPath],
        requiresArtifactReads: false,
        outputPath: auditReviewPackOutputPath,
      },
      {
        subcommand: 'audit-summary-render',
        args: ['audit-summary-render', '--input', auditSummaryInputPath],
        requiresArtifactReads: false,
        outputPath: auditSummaryOutputPath,
      },
      {
        subcommand: 'finalize-audit-run-with-companions',
        args: ['finalize-audit-run-with-companions', '--input', finalizeWithCompanionsInputPath],
        requiresArtifactReads: false,
        outputDir,
      },
    ];

    for (const probe of probes) {
      const resultPath = path.join(tempDir, `${probe.subcommand}.json`);
      const result = spawnSync(command, [...probe.args, '--result-output', resultPath], {
        cwd,
        encoding: 'utf8',
      });
      if (result.error || result.status !== 0) return auditCoreContractProbeFailure(
        `${probe.subcommand} failed with status ${result.status ?? 'spawn-error'}: ${result.error?.message ?? result.stderr ?? ''}`
      );
      const stdout = result.stdout ?? '';
      if (probe.expectedStdoutIncludes) {
        const expected = Array.isArray(probe.expectedStdoutIncludes)
          ? probe.expectedStdoutIncludes
          : [probe.expectedStdoutIncludes];
        if (expected.some((snippet) => !stdout.includes(snippet))) {
          return auditCoreContractProbeFailure(
            `${probe.subcommand} omitted its expected stdout rendering`
          );
        }
      } else if (stdout.trim().length > 0) {
        return auditCoreContractProbeFailure(
          `${probe.subcommand} wrote JSON to stdout while using --result-output`
        );
      }
      if (probe.expectedStdoutExcludes?.some((snippet) => stdout.includes(snippet))) {
        return auditCoreContractProbeFailure(
          `${probe.subcommand} emitted forbidden repository-sized stdout rows`
        );
      }
      if (!existsSync(resultPath)) return auditCoreContractProbeFailure(
        `${probe.subcommand} did not create ${resultPath}`
      );
      const json = JSON.parse(readFileSync(resultPath, 'utf8'));
      if (!resultPayloadMatchesProbe(json, probe)) {
        if (process.env.LUMIN_AUDIT_CORE_CONTRACT_DEBUG === '1') {
          console.error(`[audit-core contract] ${probe.subcommand} payload: ${JSON.stringify(json)}`);
        }
        return auditCoreContractProbeFailure(
          `${probe.subcommand} returned an incompatible result payload`
        );
      }
      if (probe.requiresArtifactReads !== false && !Array.isArray(json.artifactReads?.reads)) {
        return auditCoreContractProbeFailure(
          `${probe.subcommand} omitted artifactReads.reads`
        );
      }
    }
    return true;
  } catch (error) {
    return auditCoreContractProbeFailure(`fixture setup failed: ${error.message}`);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function nativeJsPreWriteArtifactsMatch(outputDir, invocationId) {
  const evidencePath = path.join(outputDir, `pre-write-evidence.${invocationId}.json`);
  const inventoryPath = path.join(outputDir, `any-inventory.pre.${invocationId}.json`);
  const advisoryPath = path.join(outputDir, `pre-write-advisory.${invocationId}.json`);
  const latestPath = path.join(outputDir, 'pre-write-advisory.latest.json');
  if (
    !existsSync(evidencePath) ||
    !existsSync(inventoryPath) ||
    !existsSync(advisoryPath) ||
    !existsSync(latestPath)
  ) {
    return false;
  }
  const evidence = JSON.parse(readFileSync(evidencePath, 'utf8'));
  const inventory = JSON.parse(readFileSync(inventoryPath, 'utf8'));
  const advisoryText = readFileSync(advisoryPath, 'utf8');
  const advisory = JSON.parse(advisoryText);
  return evidence.schemaVersion === 'lumin-js-ts-pre-write-evidence-response.v1' &&
    evidence.functionSignatures?.meta?.complete === true &&
    evidence.functionSignatures?.meta?.normalizerVersion === 'function-signature.normalized.v1' &&
    evidence.functionSignatures?.facts?.some((fact) =>
      fact?.exportedName === 'Thing' && fact?.ownerFile === 'src/thing.ts'
    ) &&
    evidence.inlinePatterns?.meta?.groupCount === 1 &&
    evidence.shapeIntentNormalizations?.some((entry) => entry?.shapeKind === 'function-signature') &&
    inventory.meta?.complete === true &&
    inventory.meta?.supports?.typeEscapes === true &&
    advisory.invocationId === invocationId &&
    advisory.evidenceAvailability?.status === 'available' &&
    advisory.evidenceAvailability?.freshAudit === true &&
    advisory.lookups?.some((lookup) => lookup?.result === 'SIGNATURE_MATCH') &&
    advisory.lookups?.some((lookup) => lookup?.result === 'INLINE_PATTERN_MATCH') &&
    readFileSync(latestPath, 'utf8') === advisoryText;
}

function resultPayloadMatchesProbe(json, probe) {
  if (probe.subcommand === 'barrel-discipline-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'check-barrel-discipline.mjs' &&
      json.mode === 'single-package' &&
      json.skipped === true &&
      json.reason === 'contract-probe';
  }
  if (probe.subcommand === 'block-clones-artifact') {
    return json.schemaVersion === 'block-clones.v1' &&
      json.policyVersion === 'block-clone-review-policy-v1' &&
      json.status === 'complete' &&
      isObject(json.summary) &&
      json.summary.fileCount === 2 &&
      json.summary.reviewGroupCount === 1 &&
      Array.isArray(json.groups) &&
      json.groups[0]?.visibility === 'review' &&
      isObject(json.noisePolicy) &&
      json.noisePolicy.policyId === 'block-clone-noise-policy-v1';
  }
  if (probe.subcommand === 'call-graph-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'build-call-graph.mjs' &&
      json.meta.complete === true &&
      json.meta.supports?.callFanInByDefinitionId === true &&
      isObject(json.summary) &&
      json.summary.files === 2 &&
      json.summary.callEdges === 1 &&
      Array.isArray(json.topCallees) &&
      json.topCallees[0]?.file === 'src/a.ts' &&
      json.topCallees[0]?.count === 2 &&
      json.callFanInByIdentity?.['src/a.ts::alpha'] === 2 &&
      json.callFanInByDefinitionId?.['src/a.ts#FunctionDeclaration:7-37'] === 2;
  }
  if (probe.subcommand === 'function-clones-artifact') {
    return json.schemaVersion === 'function-clones.v3' &&
      isObject(json.meta) &&
      json.meta.tool === 'build-function-clone-index.mjs' &&
      json.meta.complete === true &&
      json.meta.supports?.nearFunctionCandidates === true &&
      json.meta.supports?.nearFunctionBoundedRetrieval === true &&
      json.meta.nearFunctionCandidateCount === 1 &&
      json.meta.nearFunctionCandidateProjectionLimit === 50 &&
      json.meta.thresholdPolicies?.[0]?.policyId === 'function-clone-near-policy' &&
      Array.isArray(json.facts) &&
      json.facts.length === 64 &&
      Array.isArray(json.exactBodyGroups) &&
      json.exactBodyGroups[0]?.identities?.[0] === 'src/a.ts::alpha' &&
      json.meta.exactBodyGroupCount === 1 &&
      json.candidateGenerationPolicy?.mode === 'bounded-retrieval' &&
      json.candidateGenerationPolicy?.retrievalContractVersion ===
        'function-clone-near-retrieval.v1' &&
      json.candidateGenerationSummary?.generatedUniquePairCount === 1 &&
      json.candidateGenerationSummary?.scoredPairCount === 1 &&
      json.nearFunctionCandidates?.some((candidate) =>
        candidate?.generationToken === 'rareProbeCall' &&
        candidate?.sharedCallTokenIdfSum >= 3 &&
        candidate?.sharedSignificantCallTokens?.some((token) =>
          token?.token === 'rareProbeCall' && token?.retained === true
        )
      );
  }
  if (probe.subcommand === 'symbol-graph-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'build-symbol-graph.mjs' &&
      json.meta.schemaVersion === 3 &&
      json.meta.supports?.identityFanIn === true &&
      json.files === 4 &&
      json.totalDefs === 4 &&
      json.totalUsesResolved === 3 &&
      json.unresolvedUses === 3 &&
      json.uses?.resolvedInternal === 3 &&
      json.uses?.external === 1 &&
      json.uses?.unresolvedInternal === 2 &&
      json.uses?.unresolvedInternalRatio === 0.4 &&
      json.dependencyImportConsumers?.some((consumer) =>
        consumer?.depRoot === 'react' &&
        consumer?.fromSpec === 'react/jsx-runtime'
      ) &&
      json.generatedConsumerBlindZones?.some((zone) =>
        zone?.reason === 'generated-consumer-blind-zone' &&
        zone?.sourceReason === 'workspace-generated-artifact-missing' &&
        zone?.specifier === '@scope/generated-client' &&
        zone?.consumerFile === 'src/c.ts' &&
        zone?.candidatePath === 'packages/api/generated/client.ts' &&
        zone?.scopePackageRoot === 'packages/api'
      ) &&
      json.artifactSummary?.generatedConsumerBlindZoneCount === 1 &&
      json.defIndex?.['src/a.ts']?.alpha?.name === 'alpha' &&
      json.defIndex?.['src/a.ts']?.alpha?.anyContamination?.label === 'any-contaminated' &&
      json.helperOwnersByIdentity?.['src/a.ts::alpha']?.anyContamination?.measurements?.explicitAnyCount === 1 &&
      json.fanInByIdentity?.['src/a.ts::alpha'] === 1 &&
      json.fanInByIdentity?.['src/a.ts::beta'] === 0 &&
      json.fanInByIdentity?.['src/a.ts::gamma'] === 1 &&
      json.deadProdList?.[0]?.symbol === 'beta' &&
      json.deadTestList?.[0]?.file === 'tests/setup/server.js' &&
      json.deadTestList?.[0]?.symbol === 'unusedTestServer' &&
      json.unresolvedInternalSummaryByReason?.['alias-miss']?.count === 1 &&
      json.unresolvedInternalSummaryByReason?.['workspace-generated-artifact-missing']?.count === 1;
  }
  if (probe.subcommand === 'js-ts-pre-write-evidence') {
    return json.schemaVersion === 'lumin-js-ts-pre-write-evidence-response.v1' &&
      json.summary?.runtime?.singleFlight?.status === 'acquired' &&
      json.summary?.runtime?.singleFlight?.scope === 'canonical-root' &&
      typeof json.summary?.runtime?.timing?.lockWaitMs === 'number' &&
      typeof json.summary?.runtime?.timing?.discoveryMs === 'number' &&
      typeof json.summary?.runtime?.timing?.sourceReadHashMs === 'number' &&
      typeof json.summary?.runtime?.timing?.parseMs === 'number' &&
      typeof json.summary?.runtime?.timing?.cacheWriteMs === 'number' &&
      typeof json.summary?.runtime?.timing?.projectionMs === 'number' &&
      typeof json.summary?.runtime?.timing?.scanHeldMs === 'number' &&
      json.symbols?.meta?.supports?.identityFanIn === true &&
      json.symbols?.meta?.supports?.dependencyImportConsumers === true &&
      json.symbols?.defIndex?.['src/dep.ts']?.api?.name === 'api' &&
      json.symbols?.fanInByIdentity?.['src/dep.ts::api'] === 1 &&
      json.symbols?.dependencyImportConsumers?.some((consumer) =>
        consumer?.depRoot === 'web-tree-sitter' &&
        consumer?.file === 'src/consumer.mjs'
      ) &&
      json.anyInventory?.meta?.artifact === 'any-inventory.pre.PROBE.json' &&
      json.anyInventory?.meta?.supports?.typeEscapes === true &&
      json.anyInventory?.typeEscapes?.some((escape) =>
        escape?.file === 'src/dep.ts' && escape?.escapeKind === 'as-any'
      ) &&
      json.shapeIndex?.schemaVersion === 'shape-index.v1' &&
      json.shapeIndex?.meta?.supports?.normalizedVersion === 'shape-hash.normalized.v1' &&
      json.shapeIndex?.facts?.some((fact) =>
        fact?.identity === 'src/dep.ts::ProbeShape' &&
        fact?.hash === 'sha256:a97f556e6454ed1e3862416c986810198b0bc796dd54ba4b8aca1ee75697df34'
      ) &&
      json.shapeIntentNormalizations?.some((entry) =>
        entry?.typeLiteral === '{ id: string }' &&
        entry?.ok === true &&
        entry?.hash === 'sha256:a97f556e6454ed1e3862416c986810198b0bc796dd54ba4b8aca1ee75697df34'
      ) &&
      json.files?.includes('src/consumer.mjs') &&
      json.files?.includes('src/dep.ts') &&
      json.topology?.meta?.complete === true &&
      json.topology?.edges?.some((edge) =>
        edge?.from === 'src/consumer.mjs' && edge?.to === 'src/dep.ts'
      );
  }
  if (probe.subcommand === 'js-ts-extract-artifact') {
    const file = json.files?.[0];
    const uses = Array.isArray(file?.uses) ? file.uses : [];
    const localOperations = Array.isArray(file?.localOperations) ? file.localOperations : [];
    const globalRegistrations = Array.isArray(file?.globalComponentRegistrations)
      ? file.globalComponentRegistrations
      : [];
    return json.schemaVersion === 'lumin-js-ts-extract-response.v1' &&
      isObject(file) &&
      file.filePath.endsWith(path.join('src', 'consumer.mjs')) &&
      file.error === undefined &&
      uses.some((use) =>
        use?.fromSpec === './dep' &&
        use?.name === 'api' &&
        use?.kind === 'imported-namespace-member' &&
        use?.memberName === 'foo' &&
        use?.localName === 'api'
      ) &&
      uses.some((use) =>
        use?.fromSpec === './dep' &&
        use?.name === 'bare' &&
        use?.kind === 'imported-namespace-escape' &&
        use?.localName === 'bare' &&
        use?.degraded === true
      ) &&
      uses.some((use) =>
        use?.fromSpec === './pages/*.ts' &&
        use?.name === '*' &&
        use?.kind === 'import-meta-glob' &&
        use?.degraded === true &&
        use?.resolverStage === 'import-meta-glob'
      ) &&
      uses.some((use) =>
        use?.fromSpec === './cjs-api' &&
        use?.name === 'run' &&
        use?.kind === 'cjs-namespace-member' &&
        use?.localName === 'cjsApi'
      ) &&
      uses.some((use) =>
        use?.fromSpec === './cjs-exact' &&
        use?.name === 'cjsExact' &&
        use?.kind === 'cjs-require-exact'
      ) &&
      uses.some((use) =>
        use?.fromSpec === './cjs-side-effect' &&
        use?.name === '*' &&
        use?.kind === 'cjs-side-effect-only'
      ) &&
      file.cjsRequireOpacity?.some((entry) =>
        entry?.kind === 'dynamic-require'
      ) &&
      file.cjsExportSurface?.exact?.some((entry) =>
        entry?.name === 'probe' &&
        entry?.kind === 'exports-member'
      ) &&
      file.cjsExportSurface?.exact?.some((entry) =>
        entry?.name === 'namedProbe' &&
        entry?.kind === 'module-exports-member'
      ) &&
      file.cjsExportSurface?.exact?.some((entry) =>
        entry?.name === 'objectProbe' &&
        entry?.kind === 'module-exports-object'
      ) &&
      file.cjsExportSurface?.opaque?.some((entry) =>
        entry?.kind === 'computed-export-name'
      ) &&
      file.cjsExportSurface?.opaque?.some((entry) =>
        entry?.kind === 'module-exports-assignment'
      ) &&
      uses.some((use) =>
        use?.fromSpec === 'web-tree-sitter' &&
        use?.name === '*' &&
        use?.kind === 'dynamic' &&
        use?.localName === 'mod' &&
        use?.degraded === true
      ) &&
      uses.some((use) =>
        use?.fromSpec === './lazy' &&
        use?.name === 'boot' &&
        use?.kind === 'dynamic-member' &&
        use?.localName === 'lazy' &&
        use?.degraded !== true
      ) &&
      file.dynamicImportOpacity?.some((entry) =>
        entry?.kind === 'nonliteral' &&
        entry?.line === 16
      ) &&
      file.dynamicImportOpacity?.some((entry) =>
        entry?.kind === 'template-prefix' &&
        entry?.line === 15 &&
        entry?.prefix === './pages/'
      ) &&
      file.defs?.[0]?.name === 'view' &&
      file.defs?.[0]?.kind === 'const-var' &&
      localOperations.some((operation) =>
        operation?.identity === 'src/consumer.mjs::buildProbeRepository#getProbe' &&
        operation?.containerName === 'buildProbeRepository' &&
        operation?.name === 'getProbe' &&
        operation?.operationFamily === 'read-query' &&
        Array.isArray(operation?.domainTokens) &&
        operation.domainTokens.includes('probe')
      ) &&
      globalRegistrations.some((registration) =>
        registration?.registrationFile.endsWith(path.join('src', 'consumer.mjs')) &&
        registration?.framework === 'vue' &&
        registration?.api === 'app.component' &&
        registration?.componentName === 'ProbeCard' &&
        registration?.bindingName === 'ProbeCard' &&
        registration?.bindingSource === './ProbeCard.vue' &&
        registration?.fromSpec === './ProbeCard.vue' &&
        registration?.status === 'registration-syntax' &&
        registration?.source === 'sfc-global-component-registration' &&
        registration?.eligibleForFanIn === false &&
        registration?.eligibleForSafeFix === false
      );
  }
  if (probe.subcommand === 'sfc-file-facts-artifact') {
    const files = Array.isArray(json.files) ? json.files : [];
    const file = files.find((row) => row?.filePath === 'src/App.vue');
    const optionsFile = files.find((row) => row?.filePath === 'src/Options.vue');
    const astroFile = files.find((row) => row?.filePath === 'src/Page.astro');
    const svelteFile = files.find((row) => row?.filePath === 'src/Panel.svelte');
    return json.schemaVersion === 'lumin-sfc-file-facts-response.v1' &&
      file?.filePath === 'src/App.vue' &&
      file.scriptImportConsumers?.some((row) =>
        row?.fromSpec === './ProbeCard.vue' &&
        row?.name === 'default' &&
        row?.localName === 'ProbeCard'
      ) &&
      file.scriptSources?.some((row) =>
        row?.fromSpec === './setup.ts' && row?.kind === 'sfc-script-src'
      ) &&
      file.styleAssetReferences?.some((row) =>
        row?.fromSpec === './theme.css' && row?.kind === 'sfc-style-import'
      ) &&
      file.templateComponentRefs?.some((row) =>
        row?.tagName === 'ProbeCard' && row?.bindingSource === './ProbeCard.vue'
      ) &&
      file.templateComponentRefs?.some((row) =>
        row?.tagName === 'UI.Panel' &&
        row?.memberName === 'Panel' &&
        row?.reason === 'sfc-template-namespace-component'
      ) &&
      file.frameworkConventionComponents?.some((row) =>
        row?.conventionKind === 'macro-registration' &&
        row?.componentName === 'ProbeCard' &&
        row?.bindingSource === './ProbeCard.vue' &&
        row?.reason === 'sfc-framework-vue-macro-registration'
      ) &&
      optionsFile?.frameworkConventionComponents?.some((row) =>
        row?.conventionKind === 'options-registration' &&
        row?.componentName === 'OptionsCard' &&
        row?.bindingSource === './OptionsCard.vue'
      ) &&
      astroFile?.frameworkConventionComponents?.some((row) =>
        row?.conventionKind === 'client-directive' &&
        row?.tagName === 'AstroCard' &&
        row?.directiveName === 'client:load'
      ) &&
      svelteFile?.frameworkConventionComponents?.some((row) =>
        row?.conventionKind === 'action-directive' &&
        row?.actionName === 'enhance' &&
        row?.bindingSource === './actions'
      ) &&
      svelteFile?.frameworkConventionComponents?.some((row) =>
        row?.conventionKind === 'store-auto-subscription' &&
        row?.storeName === 'count' &&
        row?.bindingSource === './stores'
      );
  }
  if (probe.subcommand === 'checklist-facts-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'checklist-facts.mjs' &&
      json.meta.schemaVersion === 9 &&
      json.meta.incremental?.reusedFiles === 1 &&
      json.A2_function_size?.gate === 'ok' &&
      json.C5_lint_enforcement?.gate === 'unknown' &&
      json.C5_lint_enforcement?.available === false &&
      json.C5_lint_enforcement?.lintEvidenceStatus === 'degraded' &&
      json.C5_lint_enforcement?.unsupportedCommands?.[0]?.scriptName === 'lint' &&
      json.E2_silent_catch?.analysis === 'oxc-ast-catch-clause' &&
      Array.isArray(json._not_computed) &&
      json._not_computed.length >= 20;
  }
  if (probe.subcommand === 'compare-repos-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'compare-repos.mjs' &&
      json.left?.label === 'left' &&
      json.right?.label === 'right' &&
      json.deltas?.files === 2 &&
      json.deltas?.loc === 6 &&
      Array.isArray(json.right?.artifactsFound) &&
      json.right.artifactsFound.includes('runtime-evidence.json') &&
      Array.isArray(json.missingArtifacts?.left) &&
      json.missingArtifacts.left.includes('runtime-evidence.json');
  }
  if (probe.subcommand === 'dead-classify-artifact') {
    return json.summary?.category_C === 1 &&
      json.summary?.category_B === 1 &&
      json.proposal_C_remove_symbol?.[0]?.symbol === 'Dead' &&
      json.proposal_C_remove_symbol?.[0]?.fileInternalUses === 0 &&
      json.proposal_B_review?.[0]?.symbol === 'Hub' &&
      Array.isArray(json.excludedCandidates);
  }
  if (probe.subcommand === 'discipline-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'measure-discipline.mjs' &&
      json.scannedFiles === 1 &&
      json.totals?.[':any'] === 1 &&
      json.totals?.['as any'] === 1 &&
      json.totals?.TODO === 1 &&
      Array.isArray(json.overallTopOffenders);
  }
  if (probe.subcommand === 'entry-surface-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'build-entry-surface.mjs' &&
      json.meta.schemaVersion === 'entry-surface.v1' &&
      Array.isArray(json.entryFiles) &&
      json.entryFiles.includes('probe.ts') &&
      isObject(json.evidenceByFile) &&
      isObject(json.completenessBySubmodule);
  }
  if (probe.subcommand === 'export-action-safety-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'export-action-safety.mjs' &&
      json.meta.schemaVersion === 1 &&
      json.meta.total === 1 &&
      Array.isArray(json.findings) &&
      json.findings.length === 1 &&
      isObject(json.byId) &&
      json.byId['dead-export:probe.ts:value:1']?.actionBlockers?.[0] === 'contract-probe';
  }
  if (probe.subcommand === 'module-reachability-artifact') {
    return isObject(json.meta) &&
      json.meta.schemaVersion === 'module-reachability.v1' &&
      Array.isArray(json.reachableFiles) &&
      isObject(json.summary) &&
      typeof json.summary.reachable === 'number';
  }
  if (probe.subcommand === 'rank-fixes-artifact') {
    return isObject(json.meta) &&
      json.meta.executionOwner === 'lumin-audit-core' &&
      isObject(json.summary) &&
      typeof json.summary.total === 'number' &&
      Array.isArray(json.safeFixes) &&
      Array.isArray(json.reviewFixes) &&
      Array.isArray(json.degraded) &&
      Array.isArray(json.muted);
  }
  if (probe.subcommand === 'resolver-diagnostics-artifacts') {
    return isObject(json.capabilities) &&
      json.capabilities.schemaVersion === 'resolver-capabilities.v1' &&
      Array.isArray(json.capabilities.families) &&
      isObject(json.diagnostics) &&
      json.diagnostics.schemaVersion === 'resolver-diagnostics.v1' &&
      isObject(json.diagnostics.summary) &&
      typeof json.diagnostics.summary.unresolvedImportCount === 'number';
  }
  if (probe.subcommand === 'runtime-evidence-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'merge-runtime-evidence.mjs' &&
      isObject(json.summary) &&
      typeof json.summary.total === 'number' &&
      typeof json.summary.grounded_dead === 'number' &&
      Array.isArray(json.merged) &&
      Array.isArray(json.orphanFilesSample);
  }
  if (probe.subcommand === 'sarif-artifact') {
    const run = json.runs?.[0];
    return json.version === '2.1.0' &&
      isObject(run) &&
      isObject(run.tool?.driver) &&
      run.tool.driver.name === 'lumin-repo-lens-lab' &&
      Array.isArray(run.tool.driver.rules) &&
      Array.isArray(run.results) &&
      isObject(run.properties) &&
      typeof run.properties.totalFindings === 'number';
  }
  if (probe.subcommand === 'shape-index-artifact') {
    return json.schemaVersion === 'shape-index.v1' &&
      isObject(json.meta) &&
      json.meta.tool === 'build-shape-index.mjs' &&
      json.meta.factCount === 1 &&
      json.meta.hashGroupCount === 1 &&
      json.meta.supports?.normalizedVersion === 'shape-hash.normalized.v1' &&
      Array.isArray(json.facts) &&
      json.facts.length === 1 &&
      isObject(json.groupsByHash) &&
      Array.isArray(json.groupsByHash[`sha256:${'0'.repeat(64)}`]);
  }
  if (probe.subcommand === 'source-use-assembly-artifact') {
    return json.schemaVersion === 'lumin-source-use-assembly-response.v1' &&
      json.summary?.recordCount === 8 &&
      json.summary?.handledCount === 8 &&
      json.counters?.totalUses === 6 &&
      json.counters?.resolvedInternalUses === 6 &&
      json.counters?.rustResolvedRelativeUses === 3 &&
      json.counters?.nonSourceAssetUses === 1 &&
      json.counters?.mdxConsumerUses === 1 &&
      json.counters?.sfcScriptConsumerUses === 1 &&
      json.counters?.sfcScriptSrcReachabilityUses === 1 &&
      json.counters?.resolvedGeneratedVirtualUses === 1 &&
      json.counters?.unresolvedUses === 1 &&
      json.counters?.unresolvedInternalUses === 1 &&
      json.branchCounts?.resolvedInternal === 5 &&
      json.branchCounts?.unresolved === 1 &&
      json.branchCounts?.generatedVirtual === 1 &&
      json.branchCounts?.asset === 1 &&
      json.branchCounts?.sfcScriptSrcReachability === 1 &&
      json.branchCounts?.directConsumer === 2 &&
      json.branchCounts?.broadNamespace === 2 &&
      json.nonSourceAssetRecordIds?.includes('src/consumer.ts#6') &&
      json.nonSourceAssetRecordTargets?.some((entry) =>
        entry?.recordId === 'src/consumer.ts#6' &&
        entry?.resolvedFile?.replaceAll('\\', '/').endsWith('/src/style.css')
      ) &&
      json.generatedVirtualRecordIds?.includes('src/consumer.ts#3') &&
      json.resolvedRecordTargets?.filter((entry) =>
        entry?.recordId === 'src/consumer.ts#0' &&
        entry?.resolvedFile?.replaceAll('\\', '/').endsWith('/src/dep.ts')
      ).length === 1 &&
      json.resolvedRecordTargets?.filter((entry) =>
        entry?.recordId === 'sfc-script-src:0:src/App.vue:./setup' &&
        entry?.resolvedFile?.replaceAll('\\', '/').endsWith('/src/setup.ts')
      ).length === 1 &&
      json.resolvedInternalEdges?.some((edge) =>
        edge?.from === 'src/consumer.ts' &&
        edge?.to === 'src/dep.ts' &&
        edge?.kind === 'import-named'
      ) &&
      json.resolvedInternalEdges?.some((edge) =>
        edge?.from === 'src/consumer.ts' &&
        edge?.to === 'src/dep.ts' &&
        edge?.kind === 'dynamic-import-meta-glob'
      ) &&
      json.resolvedInternalEdges?.some((edge) =>
        edge?.from === 'src/App.vue' &&
        edge?.to === 'src/setup.ts' &&
        edge?.kind === 'sfc-script-src' &&
        edge?.sfcLanguage === 'vue'
      ) &&
      json.resolvedInternalEdges?.some((edge) =>
        edge?.from === 'src/consumer.ts' &&
        edge?.to === 'src/layout.config.ts' &&
        edge?.source === '@/layout.config' &&
        edge?.kind === 'import-named'
      ) &&
      json.unresolvedInternalSpecifierRecords?.[0]?.reason === 'tsconfig-path-target-missing' &&
      json.generatedVirtualSurfaces?.[0]?.id === 'generated-virtual:prisma-enums:@pkg/db:enums' &&
      json.generatedVirtualImportConsumers?.[0]?.surfaceId === 'generated-virtual:prisma-enums:@pkg/db:enums' &&
      json.directConsumers?.some((entry) => entry?.symbol === 'value') &&
      json.namespaceUsers?.filter((entry) =>
        entry?.defFile === 'src/dep.ts' &&
        entry?.consumerFile === 'src/consumer.ts'
      ).length === 1;
  }
  if (probe.subcommand === 'staleness-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'measure-staleness.mjs' &&
      isObject(json.summary) &&
      typeof json.summary.total === 'number' &&
      isObject(json.summary.byTier) &&
      isObject(json.summary.performance) &&
      Array.isArray(json.enriched);
  }
  if (probe.subcommand === 'topology-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'm2s1-topology.mjs' &&
      json.meta.complete === true &&
      isObject(json.summary) &&
      json.summary.files === 2 &&
      json.summary.internalEdges === 1 &&
      isObject(json.nodes) &&
      Array.isArray(json.edges) &&
      json.edges.length === 1;
  }
  if (probe.subcommand === 'topology-mermaid-render') {
    if (
      json.schemaVersion !== 'lumin-topology-mermaid-render-result.v1' ||
      json.path !== probe.outputPath ||
      typeof json.bytes !== 'number' ||
      json.bytes <= 0
    ) {
      return false;
    }
    const markdown = readFileSync(probe.outputPath, 'utf8');
    return markdown.startsWith('# Topology Mermaid') &&
      markdown.includes('sub0["apps/web"]') &&
      markdown.includes('sub0 -->|4| sub1') &&
      markdown.includes('## Citation Contract') &&
      json.bytes === Buffer.byteLength(markdown, 'utf8');
  }
  if (probe.subcommand === 'audit-review-pack-render') {
    if (
      json.schemaVersion !== 'lumin-audit-review-pack-render-result.v1' ||
      json.path !== probe.outputPath ||
      typeof json.bytes !== 'number' ||
      json.bytes <= 0
    ) {
      return false;
    }
    const markdown = readFileSync(probe.outputPath, 'utf8');
    return markdown.startsWith('# Audit Review Pack') &&
      markdown.includes('Lane 1') &&
      markdown.includes('Lane 4') &&
      markdown.includes('Merge Instructions') &&
      json.bytes === Buffer.byteLength(markdown, 'utf8');
  }
  if (probe.subcommand === 'audit-summary-render') {
    if (
      json.schemaVersion !== 'lumin-audit-summary-render-result.v1' ||
      json.path !== probe.outputPath ||
      typeof json.bytes !== 'number' ||
      json.bytes <= 0 ||
      typeof json.preview !== 'string'
    ) {
      return false;
    }
    const markdown = readFileSync(probe.outputPath, 'utf8');
    return markdown.startsWith('# Audit Artifact Brief') &&
      markdown.includes('## Read First') &&
      markdown.includes('## Measured Cues (Unranked)') &&
      markdown.includes('## Guardrails') &&
      json.preview.includes('[audit-repo] artifact brief preview:') &&
      json.preview.includes('[audit-repo]   Read First:') &&
      json.bytes === Buffer.byteLength(markdown, 'utf8');
  }
  if (probe.subcommand === 'finalize-audit-run-with-companions') {
    const manifestPath = path.join(probe.outputDir, 'manifest.json');
    const producerPerformancePath = path.join(probe.outputDir, 'producer-performance.json');
    const topologyMermaidPath = path.join(probe.outputDir, 'topology.mermaid.md');
    const auditSummaryPath = path.join(probe.outputDir, 'audit-summary.latest.md');
    const reviewPackPath = path.join(probe.outputDir, 'audit-review-pack.latest.md');
    if (
      json.manifestPath !== manifestPath ||
      json.producerPerformancePath !== producerPerformancePath ||
      json.topologyMermaidPath !== topologyMermaidPath ||
      json.auditSummaryPath !== auditSummaryPath ||
      json.reviewPackPath !== reviewPackPath ||
      typeof json.auditSummaryPreview !== 'string' ||
      typeof json.artifactsProducedCount !== 'number' ||
      !Array.isArray(json.blindZones) ||
      typeof json.blindZonesSummary !== 'string' ||
      !isObject(json.closeoutUpdate)
    ) {
      return false;
    }
    if (!existsSync(manifestPath) || !existsSync(producerPerformancePath)) return false;
    const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
    const performance = JSON.parse(readFileSync(producerPerformancePath, 'utf8'));
    const topologyMermaid = readFileSync(topologyMermaidPath, 'utf8');
    const auditSummary = readFileSync(auditSummaryPath, 'utf8');
    const reviewPack = readFileSync(reviewPackPath, 'utf8');
    return Array.isArray(manifest.artifactsProduced) &&
      manifest.auditSummary?.path === auditSummaryPath &&
      manifest.reviewPack?.path === reviewPackPath &&
      manifest.topologyMermaid?.path === topologyMermaidPath &&
      isObject(performance.artifactReads) &&
      topologyMermaid.startsWith('# Topology Mermaid') &&
      auditSummary.startsWith('# Audit Artifact Brief') &&
      reviewPack.startsWith('# Audit Review Pack') &&
      json.auditSummaryPreview.includes('[audit-repo] artifact brief preview:');
  }
  if (probe.subcommand === 'execute-js-pre-write') {
    return json.schemaVersion === 'lumin-pre-write-lifecycle-result.v1' &&
      isObject(json.block) &&
      json.block.executionOwner === 'lumin-audit-core' &&
      json.block.engine === 'js' &&
      json.block.language === 'js-ts' &&
      json.block.producer === 'lumin-audit-core js-ts-pre-write' &&
      json.block.ran === true &&
      json.block.advisoryInvocationId === 'PROBE' &&
      json.block.rustEvidencePath === 'pre-write-evidence.PROBE.json' &&
      json.block.anyInventoryPath === 'any-inventory.pre.PROBE.json' &&
      json.exitCode === 0 &&
      json.stdout === undefined &&
      json.stderr === undefined &&
      nativeJsPreWriteArtifactsMatch(probe.outputDir, 'PROBE');
  }
  if (probe.subcommand === 'execute-post-write') {
    const latestPath = path.join(probe.outputDir, 'post-write-delta.latest.json');
    const specificPath = path.join(
      probe.outputDir,
      'post-write-delta.PROBE-POST-WRITE.PROBE-DELTA.json',
    );
    if (!existsSync(latestPath) || !existsSync(specificPath)) return false;
    const latest = JSON.parse(readFileSync(latestPath, 'utf8'));
    const specific = JSON.parse(readFileSync(specificPath, 'utf8'));
    return json.schemaVersion === 'lumin-post-write-lifecycle-result.v2' &&
      isObject(json.block) &&
      json.block.executionOwner === 'lumin-audit-core' &&
      json.block.ran === true &&
      json.block.preWriteInvocationId === 'PROBE-POST-WRITE' &&
      json.block.deltaInvocationId === 'PROBE-DELTA' &&
      json.block.deltaSchemaVersion === 'lumin-post-write-delta.v1' &&
      json.block.baselineStatus === 'available' &&
      json.block.scanRangeParity === 'ok' &&
      json.block.typeEscapeDeltaStatus === 'computed' &&
      json.exitCode === 0 &&
      json.stdout === undefined &&
      json.stderr === undefined &&
      latest.schemaVersion === 'lumin-post-write-delta.v1' &&
      latest.preWriteInvocationId === 'PROBE-POST-WRITE' &&
      latest.deltaInvocationId === 'PROBE-DELTA' &&
      JSON.stringify(latest) === JSON.stringify(specific);
  }
  if (probe.subcommand === 'execute-audit-lifecycle') {
    return json.schemaVersion === 'lumin-audit-lifecycle-execution-result.v1' &&
      isObject(json.preWrite) &&
      json.preWrite.executionOwner === 'lumin-audit-core' &&
      json.preWrite.engine === 'js' &&
      json.preWrite.language === 'js-ts' &&
      json.preWrite.producer === 'lumin-audit-core js-ts-pre-write' &&
      json.preWrite.ran === true &&
      json.preWrite.advisoryInvocationId === 'PROBE-LIFECYCLE' &&
      json.preWrite.rustEvidencePath === 'pre-write-evidence.PROBE-LIFECYCLE.json' &&
      json.preWrite.anyInventoryPath === 'any-inventory.pre.PROBE-LIFECYCLE.json' &&
      typeof json.preWrite.advisoryPath === 'string' &&
      json.preWrite.advisoryPath.includes('pre-write-advisory.PROBE-LIFECYCLE.json') &&
      json.postWrite === null &&
      json.canonDraft === null &&
      json.checkCanon === null &&
      json.finalExitCode === 0 &&
      nativeJsPreWriteArtifactsMatch(probe.outputDir, 'PROBE-LIFECYCLE');
  }
  const payload = json[probe.requiredField];
  return isObject(payload) &&
    isObject(payload.scanRange) &&
    typeof payload.scanRange.files === 'number';
}

function ensureAuditCoreBuiltFromManifest(manifestPath, candidate) {
  if (process.env.LUMIN_AUDIT_CORE_NO_AUTO_BUILD === '1') return false;
  try {
    execFileSync('cargo', [
      'build',
      '--manifest-path',
      manifestPath,
      '-p',
      'lumin-audit-core',
      '--locked',
      '--target-dir',
      path.dirname(path.dirname(candidate)),
    ], {
      cwd: path.dirname(manifestPath),
      stdio: 'inherit',
    });
  } catch (error) {
    auditCoreAutoBuildFailure = error?.message ?? String(error);
  }
  return auditCoreCandidateSupportsCurrentContract(candidate);
}

function autoBuildCandidatePath(manifestPath, exe) {
  const targetDir = process.env.CARGO_TARGET_DIR
    ? path.resolve(process.env.CARGO_TARGET_DIR)
    : path.join(
      tmpdir(),
      'lumin-audit-core-target',
      `${process.platform}-${process.arch}`,
      sourceKeyForPath(path.dirname(manifestPath))
    );
  return path.join(targetDir, 'debug', exe);
}

function sourceKeyForPath(sourcePath) {
  return path.resolve(sourcePath).replace(/[^A-Za-z0-9_.-]/g, '_').slice(-96);
}

function auditCorePlatformHint() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  const manifestPath = path.resolve(here, '../bin/audit-core-platforms.json');
  let supported = [];
  let packageScope = null;
  let sourceFallback = null;
  if (existsSync(manifestPath)) {
    try {
      const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
      packageScope = typeof manifest.packageScope === 'string' ? manifest.packageScope : null;
      sourceFallback = manifest.sourceFallback?.manifest ?? null;
      supported = (manifest.platforms ?? [])
        .map((platform) => platform.key)
        .filter((key) => typeof key === 'string' && key.length > 0)
        .sort();
    } catch {
      supported = [];
    }
  }
  const platformEnv = `LUMIN_AUDIT_CORE_BIN_${process.platform}_${process.arch}`
    .replace(/[^A-Z0-9_]/gi, '_')
    .toUpperCase();
  const supportedText = supported.length > 0
    ? ` packaged audit-core platforms: ${supported.join(', ')}.`
    : '';
  const sourceText = sourceFallback
    ? ` packaged source fallback: ${sourceFallback}.`
    : '';
  const scopeText = packageScope && !packageScope.startsWith('multi-platform')
    ? ` This skill package is scoped to ${packageScope}.`
    : '';
  const buildText = ' The wrapper can build a packaged or source-checkout lumin-audit-core helper for the current platform with cargo; set LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1 to disable that fallback.';
  const buildFailureText = auditCoreAutoBuildFailure
    ? ` Last auto-build failure: ${auditCoreAutoBuildFailure}.`
    : '';
  return `${supportedText}${sourceText}${scopeText} Provide ${platformEnv} or LUMIN_AUDIT_CORE_BIN for this platform, put ${exe} on PATH, or install a package built for ${process.platform}-${process.arch}.${buildText}${buildFailureText}`;
}

function missingAuditCoreBinaryError(label, command) {
  return new Error(`${label}: lumin-audit-core binary missing at ${command}.${auditCorePlatformHint()}`);
}

export function runAuditCoreJson(args, label, options = {}) {
  const subcommand = args?.[0];
  if (RESULT_FILE_REQUIRED_SUBCOMMANDS.has(subcommand)) {
    throw new Error(
      `${label}: ${subcommand} can emit repository-sized JSON and must use runAuditCoreJsonResultFile`
    );
  }
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw missingAuditCoreBinaryError(label, command);
  }
  const childOptions = {
    encoding: 'utf8',
    stdio: [options.input === undefined ? 'ignore' : 'pipe', 'pipe', 'pipe'],
  };
  if (options.input !== undefined) childOptions.input = options.input;
  const stdout = execFileSync(command, args, childOptions);
  return JSON.parse(stdout);
}

export function runAuditCoreJsonResultFile(args, label, options = {}) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw missingAuditCoreBinaryError(label, command);
  }
  const tempDir = mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-'));
  const resultPath = path.join(tempDir, 'result.json');
  try {
    const stdinMode = options.input === undefined
      ? (options.inheritStdin ? 'inherit' : 'ignore')
      : 'pipe';
    const childOptions = {
      encoding: 'utf8',
      stdio: [stdinMode, 'inherit', 'inherit'],
    };
    if (options.input !== undefined) childOptions.input = options.input;
    execFileSync(command, [...args, '--result-output', resultPath], childOptions);
    return JSON.parse(readFileSync(resultPath, 'utf8'));
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

export function wslPathToWindowsHost(value) {
  if (process.platform !== 'linux' || typeof value !== 'string' || value.length === 0) {
    return null;
  }
  const normalized = path.resolve(value).replaceAll('\\', '/');
  const match = /^\/mnt\/([A-Za-z])(?:\/(.*))?$/.exec(normalized);
  if (!match) return null;
  return `${match[1].toUpperCase()}:/${match[2] ?? ''}`;
}

export function windowsHostPathToWsl(value) {
  if (process.platform !== 'linux' || typeof value !== 'string' || value.length === 0) {
    return null;
  }
  const normalized = value.replaceAll('\\', '/');
  const match = /^([A-Za-z]):\/(.*)$/.exec(normalized);
  if (!match) return null;
  return `/mnt/${match[1].toLowerCase()}/${match[2]}`;
}

function windowsHostAuditCoreBinary() {
  if (process.platform !== 'linux' || process.arch !== 'x64') return null;
  if (!process.env.WSL_INTEROP && !process.env.WSL_DISTRO_NAME) return null;

  const currentPlatformEnv = `LUMIN_AUDIT_CORE_BIN_${process.platform}_${process.arch}`
    .replace(/[^A-Z0-9_]/gi, '_')
    .toUpperCase();
  if (process.env[currentPlatformEnv] || process.env.LUMIN_AUDIT_CORE_BIN) return null;

  const here = path.dirname(fileURLToPath(import.meta.url));
  const candidates = [
    process.env.LUMIN_AUDIT_CORE_BIN_WIN32_X64,
    path.resolve(here, '../bin/win32-x64/lumin-audit-core.exe'),
    path.resolve(
      here,
      '../skills/lumin-repo-lens-lab/_engine/bin/win32-x64/lumin-audit-core.exe',
    ),
  ].filter(Boolean);
  const resolvedCandidates = candidates.map((value) =>
    windowsHostPathToWsl(value) ?? path.resolve(value));
  for (const candidate of [...new Set(resolvedCandidates)]) {
    if (existsSync(candidate) && auditCoreBinaryReportsCurrentContract(candidate)) {
      return candidate;
    }
  }
  return null;
}

export function windowsHostAuditCoreEvidenceTransport({
  root,
  output,
  incremental = {},
} = {}) {
  const command = windowsHostAuditCoreBinary();
  if (!command) return null;
  const windowsRoot = wslPathToWindowsHost(root);
  const windowsOutput = wslPathToWindowsHost(output);
  if (!windowsRoot || !windowsOutput) return null;

  const transport = {
    schemaVersion: 'lumin-js-ts-pre-write-host-transport.v1',
    command,
    root: windowsRoot,
    output: windowsOutput,
  };
  if (incremental?.enabled === true) {
    const windowsCacheRoot = wslPathToWindowsHost(incremental.cacheRoot);
    if (!windowsCacheRoot) return null;
    transport.cacheRoot = windowsCacheRoot;
  }
  return transport;
}

function windowsHostTempRoot() {
  if (windowsHostTempRootCache !== undefined) return windowsHostTempRootCache;
  const commands = [
    process.env.COMSPEC
      ? windowsHostPathToWsl(process.env.COMSPEC) ?? process.env.COMSPEC
      : null,
    '/mnt/c/Windows/System32/cmd.exe',
    'cmd.exe',
  ].filter(Boolean);
  for (const command of [...new Set(commands)]) {
    const result = spawnSync(command, ['/d', '/s', '/c', 'echo %TEMP%'], {
      encoding: 'utf8',
    });
    if (result.error || result.status !== 0) continue;
    const windowsRoot = (result.stdout ?? '').trim();
    const wslRoot = windowsHostPathToWsl(windowsRoot);
    if (!wslRoot) continue;
    windowsHostTempRootCache = { windowsRoot, wslRoot };
    return windowsHostTempRootCache;
  }
  return null;
}

export function runWindowsHostAuditCoreJsonResultFile(
  args,
  label,
  { input, resultTempRoot } = {},
) {
  const command = windowsHostAuditCoreBinary();
  if (!command) return undefined;
  let sharedResultRoot = resultTempRoot;
  let windowsResultRoot = wslPathToWindowsHost(sharedResultRoot);
  if (!windowsResultRoot && resultTempRoot === undefined) {
    const hostTemp = windowsHostTempRoot();
    if (!hostTemp) return undefined;
    sharedResultRoot = hostTemp.wslRoot;
    windowsResultRoot = hostTemp.windowsRoot;
  }
  if (!windowsResultRoot) return undefined;

  let tempDir;
  try {
    mkdirSync(sharedResultRoot, { recursive: true });
    tempDir = mkdtempSync(path.join(sharedResultRoot, 'lumin-audit-core-host-'));
  } catch (error) {
    if (resultTempRoot === undefined) return undefined;
    throw error;
  }
  const resultPath = path.join(tempDir, 'result.json');
  const relativeResultPath = path.relative(sharedResultRoot, resultPath).replaceAll('\\', '/');
  const windowsResultPath = `${windowsResultRoot.replace(/\/$/, '')}/${relativeResultPath}`;
  try {
    const childOptions = {
      encoding: 'utf8',
      stdio: [input === undefined ? 'ignore' : 'pipe', 'inherit', 'inherit'],
    };
    if (input !== undefined) childOptions.input = input;
    execFileSync(command, [...args, '--result-output', windowsResultPath], childOptions);
    if (!existsSync(resultPath)) {
      throw new Error(`${label}: Windows host audit-core did not write ${resultPath}`);
    }
    const result = JSON.parse(readFileSync(resultPath, 'utf8'));
    if (result === null) {
      throw new Error(`${label}: Windows host audit-core wrote JSON null`);
    }
    return result;
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

export function runAuditCoreJsonToResultFile(args, label, resultPath, options = {}) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw missingAuditCoreBinaryError(label, command);
  }
  const childOptions = {
    encoding: 'utf8',
    stdio: [options.input === undefined ? 'ignore' : 'pipe', 'inherit', 'inherit'],
  };
  if (options.input !== undefined) childOptions.input = options.input;
  execFileSync(command, [...args, '--result-output', resultPath], childOptions);
  if (!existsSync(resultPath)) {
    throw new Error(`${label}: audit-core did not write result file at ${resultPath}`);
  }
}
