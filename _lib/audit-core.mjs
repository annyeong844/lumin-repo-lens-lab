// _lib/audit-core.mjs
//
// Runtime bridge for migrated audit-core contracts.
// Owns locating, validating, building, and invoking the lumin-audit-core helper.

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

let auditCoreAutoBuildFailure = null;
let auditCoreBinaryCache = null;
const auditCoreContractCache = new Map();

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
    ['execute-js-pre-write'],
    'execute-js-pre-write: missing --input <path|->',
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
];

const RESULT_FILE_REQUIRED_SUBCOMMANDS = new Set([
  'manifest-root-with-evidence',
  'manifest-lifecycle-evidence-refresh',
  'execute-js-pre-write',
  'manifest-evidence-summary-with-reads',
  'manifest-evidence-refresh-with-reads',
  'barrel-discipline-artifact',
  'block-clones-artifact',
  'call-graph-artifact',
  'checklist-facts-artifact',
  'dead-classify-artifact',
  'discipline-artifact',
  'entry-surface-artifact',
  'export-action-safety-artifact',
  'function-clones-artifact',
  'module-reachability-artifact',
  'rank-fixes-artifact',
  'resolver-diagnostics-artifacts',
  'runtime-evidence-artifact',
  'sarif-artifact',
  'shape-index-artifact',
  'staleness-artifact',
  'symbol-graph-artifact',
  'topology-artifact',
]);

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
  const packagedPlatform = path.resolve(here, '../bin', `${process.platform}-${process.arch}`, exe);
  if (auditCoreCandidateSupportsCurrentContract(packagedPlatform)) return remember(packagedPlatform);
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

function auditCoreBinarySupportsCurrentContract(command) {
  for (const [args, expected] of AUDIT_CORE_CONTRACT_PROBES) {
    const result = spawnSync(command, args, {
      encoding: 'utf8',
    });
    if (result.error) return false;
    const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
    if (!output.includes(expected)) return false;
  }
  return auditCoreBinaryWritesResultFiles(command);
}

function auditCoreBinaryWritesResultFiles(command) {
  const tempDir = mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-contract-'));
  const rootDir = path.join(tempDir, 'root');
  const outputDir = path.join(tempDir, 'out');
  const rootInputPath = path.join(tempDir, 'manifest-root-with-evidence.json');
  const lifecycleInputPath = path.join(tempDir, 'manifest-lifecycle-evidence-refresh.json');
  const jsPreWriteInputPath = path.join(tempDir, 'execute-js-pre-write.json');
  const jsPreWriteScriptsDir = path.join(tempDir, 'js-pre-write-scripts');
  const barrelDisciplineInputPath = path.join(tempDir, 'barrel-discipline-artifact.json');
  const blockClonesInputPath = path.join(tempDir, 'block-clones-artifact.json');
  const callGraphInputPath = path.join(tempDir, 'call-graph-artifact.json');
  const checklistFactsInputPath = path.join(tempDir, 'checklist-facts-artifact.json');
  const deadClassifyInputPath = path.join(tempDir, 'dead-classify-artifact.json');
  const disciplineInputPath = path.join(tempDir, 'discipline-artifact.json');
  const entrySurfaceInputPath = path.join(tempDir, 'entry-surface-artifact.json');
  const exportActionSafetyInputPath = path.join(tempDir, 'export-action-safety-artifact.json');
  const functionClonesInputPath = path.join(tempDir, 'function-clones-artifact.json');
  const moduleReachabilityInputPath = path.join(tempDir, 'module-reachability-artifact.json');
  const rankFixesInputPath = path.join(tempDir, 'rank-fixes-artifact.json');
  const resolverDiagnosticsInputPath = path.join(tempDir, 'resolver-diagnostics-artifacts.json');
  const runtimeEvidenceInputPath = path.join(tempDir, 'runtime-evidence-artifact.json');
  const sarifInputPath = path.join(tempDir, 'sarif-artifact.json');
  const shapeIndexInputPath = path.join(tempDir, 'shape-index-artifact.json');
  const stalenessInputPath = path.join(tempDir, 'staleness-artifact.json');
  const symbolGraphInputPath = path.join(tempDir, 'symbol-graph-artifact.json');
  const topologyInputPath = path.join(tempDir, 'topology-artifact.json');
  try {
    mkdirSync(rootDir, { recursive: true });
    mkdirSync(outputDir, { recursive: true });
    mkdirSync(jsPreWriteScriptsDir, { recursive: true });
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
    writeFileSync(path.join(jsPreWriteScriptsDir, 'pre-write.mjs'), `
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
let output = null;
for (let i = 0; i < process.argv.length; i += 1) {
  if (process.argv[i] === '--output') output = process.argv[i + 1];
}
if (!output) process.exit(2);
mkdirSync(output, { recursive: true });
const specific = path.join(output, 'pre-write-advisory.PROBE.json');
const latest = path.join(output, 'pre-write-advisory.latest.json');
const advisory = {
  invocationId: 'PROBE',
  artifactPaths: { invocationSpecific: specific, latest },
  evidenceAvailability: { status: 'available', producer: 'pre-write.mjs' },
};
writeFileSync(specific, JSON.stringify(advisory));
writeFileSync(latest, JSON.stringify(advisory));
`);
    writeFileSync(jsPreWriteInputPath, JSON.stringify({
      schemaVersion: 'lumin-js-pre-write-lifecycle-request.v1',
      root: rootDir,
      output: outputDir,
      scriptsDir: jsPreWriteScriptsDir,
      nodeExecutable: process.execPath,
      childIntentFlag: '-',
      childIntentInput: '{}\n',
      engineSelection: {
        requested: 'auto',
        selected: 'js',
        reason: 'contract-probe',
      },
      noFreshAudit: false,
      scanArgs: [],
    }));
    writeFileSync(path.join(rootDir, 'probe.ts'), 'const value: any = input as any; // TODO\n');
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
    const functionCloneProbeFact = (file, name, line) => ({
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
      exactBodyHash: 'raw-a',
      normalizedExactHash: 'exact-a',
      normalizedStructureHash: 'structure-a',
      normalizedSignatureHash: 'sig-a',
      signature: 'fn(value)',
      callTokens: ['fetchUser'],
      source: 'fresh-ast-pass',
      scope: 'scope',
      confidence: 'high',
    });
    writeFileSync(functionClonesInputPath, JSON.stringify({
      schemaVersion: 'lumin-function-clones-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      includeTests: true,
      exclude: [],
      scope: 'TS/JS including tests, top-level exported and file-local functions',
      fileCount: 2,
      facts: [
        functionCloneProbeFact('src/a.ts', 'alpha', 1),
        functionCloneProbeFact('src/b.ts', 'beta', 4),
      ],
      diagnostics: [],
      filesWithParseErrors: [],
      filesWithReadErrors: [],
    }));
    writeFileSync(symbolGraphInputPath, JSON.stringify({
      schemaVersion: 'lumin-symbol-graph-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      files: [path.join(rootDir, 'src', 'a.ts'), path.join(rootDir, 'src', 'b.ts')],
      defIndex: [
        {
          filePath: path.join(rootDir, 'src', 'a.ts'),
          definitions: {
            alpha: { name: 'alpha', kind: 'FunctionDeclaration', line: 1 },
          },
        },
      ],
      fileData: [
        {
          filePath: path.join(rootDir, 'src', 'a.ts'),
          reExports: [{ source: './b', line: 2 }],
          classMethods: [],
          localOperations: [],
          dynamicImportOpacity: [],
          cjsExportSurface: null,
          cjsRequireOpacity: [],
        },
      ],
      parseErrors: 0,
      warnings: [],
      nextCacheEntries: {},
      unresolvedInternalByPrefix: [{ key: '@/missing', count: 1 }],
      prefixExamples: { '@/missing': '@/missing/foo' },
      unresolvedInternalSpecifiers: ['@/missing/foo'],
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '@/missing/foo',
          consumerFile: 'src/b.ts',
          kind: 'import',
          typeOnly: false,
          reason: 'alias-miss',
        },
      ],
      languageSupport: { ts: { enabled: true, reason: null } },
      totalUses: 1,
      unresolvedUses: 1,
      resolvedInternalUses: 1,
      resolvedGeneratedVirtualUses: 0,
      nonSourceAssetUses: 0,
      externalUses: 0,
      dependencyImportConsumers: [],
      resolvedInternalEdges: [
        { from: 'src/b.ts', to: 'src/a.ts', kind: 'import', source: './a', typeOnly: false },
      ],
      generatedConsumerBlindZones: [],
      generatedVirtualSurfaces: [],
      generatedVirtualImportConsumers: [],
      unresolvedInternalUses: 1,
      mdxConsumerUses: 0,
      sfcScriptConsumerUses: 0,
      sfcScriptSrcReachabilityUses: 0,
      sfcStyleAssetReferenceUses: 0,
      sfcTemplateComponentRefUses: 0,
      sfcGlobalComponentRegistrationUses: 0,
      sfcGeneratedComponentManifestUses: 0,
      sfcFrameworkConventionComponentUses: 0,
      sfcStyleAssetReferences: [],
      sfcTemplateComponentRefs: [],
      sfcGlobalComponentRegistrations: [],
      sfcGeneratedComponentManifests: [],
      sfcFrameworkConventionComponents: [],
      dead: [{ file: 'src/a.ts', symbol: 'alpha', line: 1 }],
      trulyDead: [{ file: 'src/a.ts', symbol: 'alpha', line: 1 }],
      deadInProd: [{ file: 'src/a.ts', symbol: 'alpha', line: 1 }],
      deadInTest: [],
      symbolFanIn: [
        { defFile: 'src/a.ts', symbol: 'alpha', count: 0, kind: 'FunctionDeclaration' },
      ],
      fanInByIdentity: { 'src/a.ts::alpha': 0 },
      fanInByIdentitySpace: { 'src/a.ts::alpha': { value: 0, type: 0, broad: 0 } },
      namespaceReExportDiagnostics: [],
      anyContaminationFacts: {
        helperOwnersByIdentity: {},
        typeOwnersByIdentity: {},
      },
    }));
    writeFileSync(checklistFactsInputPath, JSON.stringify({
      schemaVersion: 'lumin-checklist-facts-producer-request.v1',
      generated: '2026-07-02T00:00:00.000Z',
      root: rootDir,
      filesScanned: 1,
      inputs: {},
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
        subcommand: 'staleness-artifact',
        args: ['staleness-artifact', '--input', stalenessInputPath],
        requiresArtifactReads: false,
      },
      {
        subcommand: 'topology-artifact',
        args: ['topology-artifact', '--input', topologyInputPath],
        requiresArtifactReads: false,
      },
    ];

    for (const probe of probes) {
      const resultPath = path.join(tempDir, `${probe.subcommand}.json`);
      const result = spawnSync(command, [...probe.args, '--result-output', resultPath], {
        encoding: 'utf8',
      });
      if (result.error || result.status !== 0) return false;
      if ((result.stdout ?? '').trim().length > 0) return false;
      if (!existsSync(resultPath)) return false;
      const json = JSON.parse(readFileSync(resultPath, 'utf8'));
      if (!resultPayloadMatchesProbe(json, probe)) return false;
      if (probe.requiresArtifactReads !== false && !Array.isArray(json.artifactReads?.reads)) return false;
    }
    return true;
  } catch {
    return false;
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
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
      json.meta.thresholdPolicies?.[0]?.policyId === 'function-clone-near-policy' &&
      Array.isArray(json.facts) &&
      json.facts.length === 2 &&
      Array.isArray(json.exactBodyGroups) &&
      json.exactBodyGroups[0]?.identities?.[0] === 'src/a.ts::alpha' &&
      json.meta.exactBodyGroupCount === 1;
  }
  if (probe.subcommand === 'symbol-graph-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'build-symbol-graph.mjs' &&
      json.meta.schemaVersion === 3 &&
      json.meta.supports?.identityFanIn === true &&
      json.files === 2 &&
      json.totalDefs === 1 &&
      json.uses?.unresolvedInternalRatio === 0.5 &&
      json.defIndex?.['src/a.ts']?.alpha?.name === 'alpha' &&
      json.fanInByIdentity?.['src/a.ts::alpha'] === 0 &&
      json.deadProdList?.[0]?.symbol === 'alpha' &&
      json.unresolvedInternalSummaryByReason?.['alias-miss']?.count === 1;
  }
  if (probe.subcommand === 'checklist-facts-artifact') {
    return isObject(json.meta) &&
      json.meta.tool === 'checklist-facts.mjs' &&
      json.meta.schemaVersion === 9 &&
      json.A2_function_size?.gate === 'ok' &&
      json.E2_silent_catch?.analysis === 'oxc-ast-catch-clause' &&
      Array.isArray(json._not_computed) &&
      json._not_computed.length >= 20;
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
  if (probe.subcommand === 'execute-js-pre-write') {
    return json.schemaVersion === 'lumin-pre-write-lifecycle-result.v1' &&
      isObject(json.block) &&
      json.block.executionOwner === 'lumin-audit-core' &&
      json.block.engine === 'js' &&
      json.block.language === 'js-ts' &&
      json.block.producer === 'pre-write.mjs' &&
      json.block.ran === true &&
      json.block.advisoryInvocationId === 'PROBE' &&
      json.exitCode === 0 &&
      json.stdout === undefined &&
      json.stderr === undefined;
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
    const childOptions = {
      encoding: 'utf8',
      stdio: [options.input === undefined ? 'ignore' : 'pipe', 'inherit', 'inherit'],
    };
    if (options.input !== undefined) childOptions.input = options.input;
    execFileSync(command, [...args, '--result-output', resultPath], childOptions);
    return JSON.parse(readFileSync(resultPath, 'utf8'));
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
