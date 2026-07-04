#!/usr/bin/env node
// Build the deployable skill surface from the maintainer repo.
//
// The source repo intentionally keeps tests, research notes, and lab
// artifacts. The generated skill package keeps only the user-facing
// contract, public wrappers, internal engine code, runtime canon,
// templates, and selected references.

import {
  cpSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const DEFAULT_OUT = path.join(ROOT, 'skills', 'lumin-repo-lens-lab');

const PUBLIC_COMMANDS = [
  'audit-repo.mjs',
  'pre-write.mjs',
  'post-write.mjs',
  'generate-canon-draft.mjs',
  'check-canon.mjs',
];
const PUBLIC_UTILITY_SCRIPTS = [
  'scripts/smoke-test.mjs',
];

const PRODUCER_SCRIPTS = [
  'any-inventory.mjs',
  'audit-repo.mjs',
  'build-block-clone-index.mjs',
  'build-call-graph.mjs',
  'build-entry-surface.mjs',
  'build-framework-resource-surfaces.mjs',
  'build-function-clone-index.mjs',
  'build-inline-pattern-index.mjs',
  'build-module-reachability.mjs',
  'build-resolver-diagnostics.mjs',
  'build-shape-index.mjs',
  'build-symbol-graph.mjs',
  'build-unused-deps.mjs',
  'check-barrel-discipline.mjs',
  'check-canon.mjs',
  'checklist-facts.mjs',
  'classify-dead-exports.mjs',
  'compare-repos.mjs',
  'emit-sarif.mjs',
  'export-action-safety.mjs',
  'generate-canon-draft.mjs',
  'measure-discipline.mjs',
  'measure-staleness.mjs',
  'measure-topology.mjs',
  'merge-runtime-evidence.mjs',
  'p6-measurement.mjs',
  'post-write.mjs',
  'pre-write.mjs',
  'rank-fixes.mjs',
  'resolve-method-calls.mjs',
  'triage-repo.mjs',
];

const ROOT_FILES = [
  'SKILL.md',
  'README.md',
];
const MAIN_OPENAI_METADATA = {
  displayName: 'Lumin Repo Lens',
  shortDescription: 'TS/JS repo evidence review',
  defaultPrompt: 'Use $lumin-repo-lens-lab to review this TS/JS repository and tell me what is stable, what to smooth next, and what to leave alone.',
};
const SIBLING_SKILL_SURFACES = [
  {
    dir: 'lumin-repo-lens-lab-codex',
    source: 'SKILL.codex.md',
    openai: {
      displayName: 'Lumin Repo Lens Codex',
      shortDescription: 'Codex-native TS/JS repo review wrapper',
      defaultPrompt: 'Use $lumin-repo-lens-lab-codex to run lumin-repo-lens-lab in Codex and explain what is stable, what to smooth next, and what to leave alone.',
    },
  },
  {
    dir: 'lumin-repo-lens-lab-write-gate',
    source: 'SKILL.write-gate.md',
    openai: {
      displayName: 'Lumin Repo Lens Write Gate',
      shortDescription: 'Pre-write reuse and post-write delta checks',
      defaultPrompt: 'Use $lumin-repo-lens-lab-write-gate before and after this code change to check reuse opportunities and unplanned type escapes.',
    },
  },
  {
    dir: 'lumin-repo-lens-lab-canon',
    source: 'SKILL.canon.md',
    openai: {
      displayName: 'Lumin Repo Lens Canon',
      shortDescription: 'Canonical fact draft and drift checks',
      defaultPrompt: 'Use $lumin-repo-lens-lab-canon to draft or check canonical repository facts from lumin-repo-lens-lab evidence.',
    },
  },
];
const RUNTIME_CANON_FILES = [
  'any-contamination.md',
  'audit-core.md',
  'canon-drift.md',
  'classification-gates.md',
  'evidence-ladder.md',
  'fact-model.md',
  'identity-and-alias.md',
  'index.md',
  'invariants.md',
  'mode-contract.md',
  'oracle-registry.json',
  'pre-write-gate.md',
];
const AUDIT_CORE_SOURCE_WORKSPACE = String.raw`[workspace]
resolver = "2"
members = [
    "rust-common",
    "rust-main/lumin-audit-core",
]

[workspace.package]
version = "0.0.0-lab.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
anyhow = "1"
lumin-rust-common = { path = "rust-common", default-features = false }
serde = "1"
serde_json = "1"
sha2 = "0.10"
tempfile = "3"

[workspace.lints]
rust = {}

[workspace.lints.clippy]
await_holding_invalid_type = "deny"
await_holding_lock = "deny"
identity_op = "deny"
manual_clamp = "deny"
manual_filter = "deny"
manual_find = "deny"
manual_flatten = "deny"
manual_map = "deny"
manual_memcpy = "deny"
manual_non_exhaustive = "deny"
manual_ok_or = "deny"
manual_range_contains = "deny"
manual_retain = "deny"
manual_strip = "deny"
manual_try_fold = "deny"
manual_unwrap_or = "deny"
needless_borrow = "deny"
needless_borrowed_reference = "deny"
needless_collect = "deny"
needless_late_init = "deny"
needless_option_as_deref = "deny"
needless_question_mark = "deny"
needless_update = "deny"
redundant_clone = "deny"
redundant_closure = "deny"
redundant_closure_for_method_calls = "deny"
redundant_static_lifetimes = "deny"
expect_used = "deny"
trivially_copy_pass_by_ref = "deny"
uninlined_format_args = "deny"
unnecessary_filter_map = "deny"
unnecessary_lazy_evaluations = "deny"
unnecessary_sort_by = "deny"
unnecessary_to_owned = "deny"
unwrap_used = "deny"

[profile.dev]
debug = "none"
incremental = false
strip = "symbols"

[profile.release]
lto = "thin"
debug = "none"
split-debuginfo = "off"
strip = "symbols"
codegen-units = 4
`;

function auditCoreExecutableNameFor(platform) {
  return platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
}

function auditCorePlatformKey(platform = process.platform, arch = process.arch) {
  return `${platform}-${arch}`;
}

function auditCoreBinaryEnvName(platform = process.platform, arch = process.arch) {
  return `LUMIN_AUDIT_CORE_BIN_${platform}_${arch}`.replace(/[^A-Z0-9_]/gi, '_').toUpperCase();
}

function cargoBuildAuditCore() {
  const exe = auditCoreExecutableNameFor(process.platform);
  const targetDir = process.env.CARGO_TARGET_DIR
    ? path.resolve(process.env.CARGO_TARGET_DIR)
    : mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-build-skill-'));
  const result = spawnSync('cargo', [
    'build',
    '--manifest-path',
    path.join(ROOT, 'experiments', 'Cargo.toml'),
    '-p',
    'lumin-audit-core',
    '--locked',
    '--target-dir',
    targetDir,
  ], {
    cwd: ROOT,
    stdio: 'inherit',
  });
  if (result.error) {
    throw new Error(`failed to start cargo while building lumin-audit-core: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`cargo build failed while building lumin-audit-core (exit ${result.status ?? 'unknown'})`);
  }
  return path.join(targetDir, 'debug', exe);
}

function validateRunnableAuditCoreBinary(binaryPath) {
  for (const [args, expected] of [
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
      ['framework-resource-surfaces-artifact'],
      'framework-resource-surfaces-artifact: missing --input <path|->',
    ],
    [
      ['function-clones-artifact'],
      'function-clones-artifact: missing --input <path|->',
    ],
    [
      ['unused-deps-artifact'],
      'unused-deps-artifact: missing --input <path|->',
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
  ]) {
    const result = spawnSync(binaryPath, args, {
      cwd: ROOT,
      encoding: 'utf8',
    });
    const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
    if (result.error) {
      throw new Error(`failed to start built lumin-audit-core at ${binaryPath}: ${result.error.message}`);
    }
    if (!output.includes(expected)) {
      throw new Error(
        `built lumin-audit-core at ${binaryPath} does not expose the current CLI contract for ${args[0]}`
      );
    }
  }
  if (!auditCoreBinaryWritesResultFiles(binaryPath)) {
    throw new Error(
      `built lumin-audit-core at ${binaryPath} does not write valid result-output files for migrated manifest commands`
    );
  }
}

function auditCoreBinaryWritesResultFiles(binaryPath) {
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
      const result = spawnSync(binaryPath, [...probe.args, '--result-output', resultPath], {
        cwd: ROOT,
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

function currentAuditCoreBinarySource() {
  const built = cargoBuildAuditCore();
  if (existsSync(built)) {
    validateRunnableAuditCoreBinary(built);
    return built;
  }
  throw new Error(`cargo build finished but lumin-audit-core was not found at ${built}`);
}

function configuredAuditCoreBinarySources() {
  const currentKey = auditCorePlatformKey();
  const sources = new Map();
  sources.set(currentKey, {
    platform: process.platform,
    arch: process.arch,
    path: currentAuditCoreBinarySource(),
  });

  for (const [name, value] of Object.entries(process.env)) {
    const prefix = 'LUMIN_AUDIT_CORE_BIN_';
    if (!name.startsWith(prefix) || name === 'LUMIN_AUDIT_CORE_BIN') continue;
    const suffix = name.slice(prefix.length).toLowerCase();
    const parts = suffix.split('_');
    if (parts.length < 2 || !value) continue;
    const arch = parts.pop();
    const platform = parts.join('_');
    const key = auditCorePlatformKey(platform, arch);
    if (key === currentKey) continue;
    sources.set(key, {
      platform,
      arch,
      path: path.resolve(value),
    });
  }

  return [...sources.values()].sort((left, right) =>
    auditCorePlatformKey(left.platform, left.arch).localeCompare(
      auditCorePlatformKey(right.platform, right.arch)
    )
  );
}

function parseArgs(argv) {
  const out = { output: DEFAULT_OUT };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--out' || arg === '--output') {
      out.output = argv[++i];
    } else if (arg === '--help' || arg === '-h') {
      out.help = true;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return out;
}

function usage() {
  return [
    'usage: node scripts/build-skill.mjs [--out <dir>]',
    '',
    'Default output:',
    `  ${path.relative(ROOT, DEFAULT_OUT)}`,
  ].join('\n');
}

function guardOutputPath(outDir) {
  const resolved = path.resolve(outDir);
  const root = path.parse(resolved).root;
  if (resolved === root || resolved === ROOT || resolved.length < root.length + 8) {
    throw new Error(`refusing unsafe output directory: ${resolved}`);
  }
  return resolved;
}

function ensureDir(filePath) {
  mkdirSync(path.dirname(filePath), { recursive: true });
}

function copyFileRel(srcRel, destRel, outDir) {
  const src = path.join(ROOT, srcRel);
  const dest = path.join(outDir, destRel);
  if (!existsSync(src)) throw new Error(`missing source file: ${srcRel}`);
  ensureDir(dest);
  cpSync(src, dest);
}

function copyDirRel(srcRel, destRel, outDir) {
  const src = path.join(ROOT, srcRel);
  const dest = path.join(outDir, destRel);
  if (!existsSync(src)) throw new Error(`missing source dir: ${srcRel}`);
  mkdirSync(path.dirname(dest), { recursive: true });
  cpSync(src, dest, { recursive: true });
}

function copyAuditCoreSourceFallback(outDir) {
  const rustRoot = path.join(outDir, '_engine', 'rust');
  mkdirSync(rustRoot, { recursive: true });
  writeFileSync(path.join(rustRoot, 'Cargo.toml'), `${AUDIT_CORE_SOURCE_WORKSPACE}\n`);
  writeFileSync(
    path.join(rustRoot, 'Cargo.lock'),
    auditCoreSourceFallbackLock(readFileSync(path.join(ROOT, 'experiments', 'Cargo.lock'), 'utf8'))
  );
  copyFileRel('experiments/rust-common/Cargo.toml', '_engine/rust/rust-common/Cargo.toml', outDir);
  copyDirRel('experiments/rust-common/src', '_engine/rust/rust-common/src', outDir);
  rmSync(path.join(outDir, '_engine', 'rust', 'rust-common', 'src', 'tests'), {
    recursive: true,
    force: true,
  });
  copyFileRel(
    'experiments/rust-main/lumin-audit-core/Cargo.toml',
    '_engine/rust/rust-main/lumin-audit-core/Cargo.toml',
    outDir
  );
  copyDirRel(
    'experiments/rust-main/lumin-audit-core/src',
    '_engine/rust/rust-main/lumin-audit-core/src',
    outDir
  );
}

function auditCoreSourceFallbackLock(lockText) {
  const packages = parseCargoLockPackages(lockText);
  if (packages.length === 0) {
    throw new Error('failed to parse experiments/Cargo.lock while preparing audit-core source fallback');
  }
  const byName = new Map();
  const byNameVersion = new Map();
  for (const pkg of packages) {
    if (!byName.has(pkg.name)) byName.set(pkg.name, []);
    byName.get(pkg.name).push(pkg);
    byNameVersion.set(`${pkg.name}@${pkg.version}`, pkg);
  }

  const queue = ['lumin-audit-core'];
  const reachable = new Set();
  for (let i = 0; i < queue.length; i++) {
    const spec = dependencySpec(queue[i]);
    const pkg = resolveCargoLockDependency(spec, byName, byNameVersion);
    if (!pkg || reachable.has(pkg.id)) continue;
    reachable.add(pkg.id);
    queue.push(...pkg.dependencies);
  }

  const blocks = packages
    .filter((pkg) => reachable.has(pkg.id))
    .map((pkg) => pkg.block.trimEnd());
  return [
    '# This file is automatically @generated by Cargo.',
    '# It is not intended for manual editing.',
    'version = 4',
    '',
    blocks.join('\n\n'),
    '',
  ].join('\n');
}

function parseCargoLockPackages(lockText) {
  const normalized = lockText.replace(/\r\n/g, '\n');
  const starts = [...normalized.matchAll(/^\[\[package\]\]$/gm)].map((match) => match.index);
  return starts
    .map((start, index) => {
      const end = starts[index + 1] ?? normalized.length;
      return normalized.slice(start, end).trimEnd();
    })
    .map((block, index) => {
      const name = lockField(block, 'name');
      const version = lockField(block, 'version') ?? '';
      return {
        id: `${name}@${version}#${index}`,
        name,
        version,
        block,
        dependencies: lockDependencies(block),
      };
    })
    .filter((pkg) => pkg.name);
}

function lockField(block, field) {
  return block.match(new RegExp(`^${field} = "([^"]+)"`, 'm'))?.[1] ?? null;
}

function lockDependencies(block) {
  const match = block.match(/^dependencies = \[\n([\s\S]*?)^\]/m);
  if (!match) return [];
  return [...match[1].matchAll(/^\s*"([^"]+)"/gm)].map((dep) => dep[1]);
}

function dependencySpec(value) {
  const versioned = value.match(/^(.+) (\d+\.\d+\.\d+(?:[-+][^ ]+)?)$/);
  if (!versioned) return { name: value, version: null };
  return { name: versioned[1], version: versioned[2] };
}

function resolveCargoLockDependency(spec, byName, byNameVersion) {
  if (spec.version) return byNameVersion.get(`${spec.name}@${spec.version}`) ?? null;
  const matches = byName.get(spec.name) ?? [];
  if (matches.length === 1) return matches[0];
  if (matches.length === 0) return null;
  throw new Error(`ambiguous Cargo.lock dependency without version: ${spec.name}`);
}

function rewriteProducerSource(text) {
  return rewritePackagedSource(text).replaceAll('./_lib/', '../lib/');
}

function rewritePackagedSource(text) {
  return text
    .replace(/docs\/history\/phases\/[^\s`)]+/g, 'maintainer history notes')
    .replace(/docs\/history\/[^\s`)]+/g, 'maintainer history notes')
    .replace(/docs\/spec\/[^\s`)]+/g, 'maintainer spec notes');
}

function writeProducerScript(name, outDir) {
  const src = readFileSync(path.join(ROOT, name), 'utf8');
  const dest = path.join(outDir, '_engine', 'producers', name);
  ensureDir(dest);
  writeFileSync(dest, rewriteProducerSource(src));
}

function wrapperSource(command) {
  return `#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const target = path.resolve(__dirname, '../_engine/producers/${command}');
const result = spawnSync(process.execPath, [target, ...process.argv.slice(2)], {
  stdio: 'inherit',
});

if (result.error) {
  process.stderr.write(\`[${command}] failed to start: \${result.error.message}\\n\`);
  process.exit(1);
}

process.exit(result.status ?? 1);
`;
}

function writePublicWrapper(command, outDir) {
  const dest = path.join(outDir, 'scripts', command);
  ensureDir(dest);
  writeFileSync(dest, wrapperSource(command));
}

function writeRuntimeCanonFile(file, outDir) {
  const src = path.join(ROOT, 'canonical', file);
  const dest = path.join(outDir, 'canonical', file);
  if (!existsSync(src)) throw new Error(`missing canonical file: ${file}`);
  ensureDir(dest);
  const text = readFileSync(src, 'utf8');
  writeFileSync(dest, rewritePackagedCanonicalMarkdown(text));
}

function writeEngineReadme(outDir) {
  const dest = path.join(outDir, '_engine', '_README.md');
  ensureDir(dest);
  writeFileSync(dest, [
    '# Internal Engine',
    '',
    'This directory is packaged with the skill because the public',
    '`scripts/*.mjs` wrappers need it at runtime.',
    '',
    'Files under `_engine/` are internal implementation details. They',
    'are not a stable user-facing API; use `scripts/audit-repo.mjs` or',
    'the other public wrappers instead.',
    '',
    '`_engine/bin/<platform>-<arch>/` contains the packaged audit-core',
    'binary for each platform supplied at package build time. The current',
    'build platform is rebuilt before packaging so stale CLI commands are',
    'not copied. Additional platform binaries can be supplied with',
    '`LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>`.',
    '',
    'The package also carries a minimal `_engine/rust` Cargo workspace for',
    '`lumin-audit-core`. If no matching packaged/env binary exists and',
    'Cargo is available, the runtime wrapper builds that helper for the',
    'current platform before invoking it.',
    '',
    'If Cargo is not available, set a runtime override variable:',
    '',
    '- `LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>` for one platform',
    '- `LUMIN_AUDIT_CORE_BIN` as a generic external binary override',
    '- `lumin-audit-core` / `lumin-audit-core.exe` on `PATH`',
    '',
    'Override binaries must match the current runtime platform. They',
    'are supported when this package does not include',
    '`_engine/bin/<platform>-<arch>/` for the current platform.',
    '',
    'When the wrapper is running from a source checkout that still has',
    '`experiments/Cargo.toml`, it can also build the current-platform helper',
    'from that checkout if no matching packaged/env/package-source',
    'binary exists. Set',
    '`LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1` to disable that source-checkout',
    'fallback and fail fast instead.',
    '',
  ].join('\n'));
}

function rewritePackagedMarkdown(text) {
  return text
    .replaceAll('_lib/', '_engine/lib/')
    .replace(/docs\/history\/phases\/[^\s`)]+/g, 'maintainer history notes')
    .replace(/docs\/history\/[^\s`)]+/g, 'maintainer history notes')
    .replace(/docs\/spec\/[^\s`)]+/g, 'maintainer spec notes');
}

function rewritePackagedCanonicalMarkdown(text) {
  return rewritePackagedMarkdown(text)
    .replace(/^> \*\*(?:Status|Last updated|Consumed by|v[\d.]+ change):\*\*.*(?:\r?\n|$)/gm, '')
    .replace(/^> \*\*v[\d.]+ change\b.*(?:\r?\n|$)/gm, '')
    .replace(/^Methodology borrowed from .*$(?:\r?\n)?/gm, '')
    .replace(/`rustlike3-clone\/canonical\/\*` \+ `p\{N\}\/session\.md` — methodology reference for this spine\.\r?\n?/g, '')
    .replace(/\n## 4\. What's deferred[\s\S]*?(?=\n## 5\. External reference material)/g, '')
    .replace(/\n## 5\. External reference material[\s\S]*?(?=\n## 6\. How to change the spine)/g, '\n')
    .replace(/\n## 6\. How to change the spine/g, '\n## 4. How to change the spine')
    .replace(/^> ?$(?:\r?\n)?/gm, '')
    .replace(/\s+See `maintainer history notes`[^.]*\./g, '')
    .replace(/\s+per `maintainer history notes`[^.)]*(?=[.)])/g, '')
    .replace(/\s+\(landed \d{4}-\d{2}-\d{2}[^)]*\)/g, '')
    .replace(/^.*promoted \d{4}-\d{2}-\d{2}.*$(?:\r?\n)?/gm, '')
    .replace(/\n{3,}/g, '\n\n');
}

function rewritePackagedMarkdownFiles(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      rewritePackagedMarkdownFiles(full);
    } else if (entry.isFile() && entry.name.endsWith('.md')) {
      const before = readFileSync(full, 'utf8');
      const after = rewritePackagedMarkdown(before);
      if (after !== before) writeFileSync(full, after);
    }
  }
}

function rewritePackagedSourceFiles(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      rewritePackagedSourceFiles(full);
    } else if (entry.isFile() && entry.name.endsWith('.mjs')) {
      const before = readFileSync(full, 'utf8');
      const after = rewritePackagedSource(before);
      if (after !== before) writeFileSync(full, after);
    }
  }
}

function buildSkillPackageJson(outDir, auditCoreBinaries = []) {
  const source = JSON.parse(readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
  const packagedPlatforms = auditCoreBinaries.map((source) =>
    auditCorePlatformKey(source.platform, source.arch)
  );
  const singlePlatform = auditCoreBinaries.length === 1 ? auditCoreBinaries[0] : null;
  const pkg = {
    name: 'lumin-repo-lens-lab-skill',
    version: source.version,
    description: 'Deployable lumin-repo-lens-lab repository evidence skill package.',
    type: 'module',
    private: true,
    license: source.license,
    luminRepoLens: {
      distribution: 'skill',
      auditCore: {
        packagedPlatforms,
        platformScope: 'current-platform-binary-with-source-fallback',
        binaryPlatformScope: singlePlatform
          ? auditCorePlatformKey(singlePlatform.platform, singlePlatform.arch)
          : 'multi-platform',
        sourceFallback: true,
        sourceFallbackManifest: '_engine/rust/Cargo.toml',
        platformOverrideEnv: 'LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>',
        genericOverrideEnv: 'LUMIN_AUDIT_CORE_BIN',
        pathFallback: true,
      },
    },
    bin: {
      'lumin-repo-lens-lab': './scripts/audit-repo.mjs',
    },
    scripts: {
      audit: 'node scripts/audit-repo.mjs',
      'pre-write': 'node scripts/audit-repo.mjs --pre-write --pre-write-engine auto',
      'post-write': 'node scripts/audit-repo.mjs --post-write',
      'canon-draft': 'node scripts/audit-repo.mjs --canon-draft',
      'check-canon': 'node scripts/audit-repo.mjs --check-canon',
      smoke: 'node scripts/smoke-test.mjs',
    },
    dependencies: source.dependencies ?? {},
    engines: source.engines ?? {},
  };
  writeFileSync(path.join(outDir, 'package.json'), `${JSON.stringify(pkg, null, 2)}\n`);
}

function normalizeLockBin(bin) {
  return Object.fromEntries(
    Object.entries(bin ?? {}).map(([name, target]) => [
      name,
      String(target).replace(/^\.\//, ''),
    ])
  );
}

function buildSkillPackageLock(outDir) {
  const srcPath = path.join(ROOT, 'package-lock.json');
  if (!existsSync(srcPath)) return;
  const lock = JSON.parse(readFileSync(srcPath, 'utf8'));
  const pkg = JSON.parse(readFileSync(path.join(outDir, 'package.json'), 'utf8'));
  const packages = lock.packages ?? {};
  const reachable = new Set(['']);
  const queue = Object.keys(pkg.dependencies ?? {});

  function packageKey(name) {
    return `node_modules/${name}`;
  }

  while (queue.length > 0) {
    const name = queue.shift();
    const key = packageKey(name);
    if (reachable.has(key)) continue;
    const entry = packages[key];
    if (!entry) continue;
    reachable.add(key);
    for (const dep of Object.keys(entry.dependencies ?? {})) queue.push(dep);
    for (const dep of Object.keys(entry.optionalDependencies ?? {})) queue.push(dep);
  }

  lock.name = pkg.name;
  lock.version = pkg.version;
  lock.packages = {};
  for (const key of reachable) {
    if (key === '') continue;
    lock.packages[key] = packages[key];
  }
  lock.packages[''] = {
    name: pkg.name,
    version: pkg.version,
    license: pkg.license,
    dependencies: pkg.dependencies,
    bin: normalizeLockBin(pkg.bin),
    engines: pkg.engines,
  };
  if (pkg.os) lock.packages[''].os = pkg.os;
  if (pkg.cpu) lock.packages[''].cpu = pkg.cpu;
  writeFileSync(path.join(outDir, 'package-lock.json'), `${JSON.stringify(lock, null, 2)}\n`);
}

function yamlString(value) {
  return JSON.stringify(value);
}

function writeOpenAiYaml(outDir, metadata) {
  const dest = path.join(outDir, 'agents', 'openai.yaml');
  ensureDir(dest);
  writeFileSync(dest, [
    'interface:',
    `  display_name: ${yamlString(metadata.displayName)}`,
    `  short_description: ${yamlString(metadata.shortDescription)}`,
    `  default_prompt: ${yamlString(metadata.defaultPrompt)}`,
    'policy:',
    '  allow_implicit_invocation: true',
    '',
  ].join('\n'));
}

function copyAuditCoreBinaries(outDir) {
  const sources = configuredAuditCoreBinarySources();
  const currentKey = auditCorePlatformKey();
  for (const source of sources) {
    if (!existsSync(source.path)) {
      throw new Error(`configured lumin-audit-core binary does not exist: ${source.path}`);
    }
    if (auditCorePlatformKey(source.platform, source.arch) === currentKey) {
      validateRunnableAuditCoreBinary(source.path);
    }
    const dest = path.join(
      outDir,
      '_engine',
      'bin',
      auditCorePlatformKey(source.platform, source.arch),
      auditCoreExecutableNameFor(source.platform)
    );
    ensureDir(dest);
    cpSync(source.path, dest);
  }
  writeAuditCorePlatformManifest(outDir, sources);
  return sources;
}

function writeAuditCorePlatformManifest(outDir, sources) {
  const dest = path.join(outDir, '_engine', 'bin', 'audit-core-platforms.json');
  ensureDir(dest);
  writeFileSync(dest, `${JSON.stringify({
    schemaVersion: 'lumin-audit-core-packaged-platforms.v1',
    packageScope: 'current-platform-binary-with-source-fallback',
    binaryPackageScope: sources.length === 1
      ? auditCorePlatformKey(sources[0].platform, sources[0].arch)
      : 'multi-platform',
    platforms: sources.map((source) => ({
      key: auditCorePlatformKey(source.platform, source.arch),
      platform: source.platform,
      arch: source.arch,
      executable: auditCoreExecutableNameFor(source.platform),
    })),
    fallback: {
      kind: 'packaged-source-build-env-or-path',
      requiredWhenRuntimePlatformMissing: true,
      message: 'Use the packaged Cargo source fallback, set LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH> / LUMIN_AUDIT_CORE_BIN to a matching external binary, or put lumin-audit-core on PATH.',
    },
    runtimeResolution: {
      packageBinaryLayout: '_engine/bin/<platform>-<arch>/<executable>',
      currentPlatformOrder: [
        'LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>',
        'LUMIN_AUDIT_CORE_BIN',
        '_engine/bin/<platform>-<arch>/<executable>',
        '_engine/rust/Cargo.toml cargo build',
        'source-checkout experiments/Cargo.toml cargo build',
        'PATH',
      ],
      missingPlatformBinaryBehavior: 'build-packaged-source-with-cargo-or-use-env-or-path-override',
      requiresCargoWhenPackagedBinaryIsMissing: true,
    },
    sourceFallback: {
      kind: 'packaged-cargo-workspace',
      manifest: '_engine/rust/Cargo.toml',
      package: 'lumin-audit-core',
    },
    buildPolicy: {
      currentPlatformBinary: 'rebuilt-before-copy',
      contractValidation: 'required-cli-commands-before-copy',
    },
    overrideEnv: {
      platformSpecific: 'LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>',
      generic: 'LUMIN_AUDIT_CORE_BIN',
    },
  }, null, 2)}\n`);
}

function build(outDir) {
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });

  for (const file of ROOT_FILES) copyFileRel(file, file, outDir);
  for (const file of RUNTIME_CANON_FILES) writeRuntimeCanonFile(file, outDir);
  copyDirRel('templates', 'templates', outDir);
  copyDirRel('references', 'references', outDir);
  copyDirRel('_lib', '_engine/lib', outDir);
  const auditCoreBinaries = copyAuditCoreBinaries(outDir);
  copyAuditCoreSourceFallback(outDir);

  for (const script of PRODUCER_SCRIPTS) writeProducerScript(script, outDir);
  for (const command of PUBLIC_COMMANDS) writePublicWrapper(command, outDir);
  for (const script of PUBLIC_UTILITY_SCRIPTS) copyFileRel(script, script, outDir);

  writeEngineReadme(outDir);
  rewritePackagedSourceFiles(path.join(outDir, '_engine'));
  rewritePackagedMarkdownFiles(outDir);
  buildSkillPackageJson(outDir, auditCoreBinaries);
  buildSkillPackageLock(outDir);
  writeOpenAiYaml(outDir, MAIN_OPENAI_METADATA);

  const skillsRoot = path.dirname(outDir);
  for (const surface of SIBLING_SKILL_SURFACES) {
    const surfaceDir = guardOutputPath(path.join(skillsRoot, surface.dir));
    rmSync(surfaceDir, { recursive: true, force: true });
    mkdirSync(surfaceDir, { recursive: true });
    copyFileRel(surface.source, 'SKILL.md', surfaceDir);
    writeOpenAiYaml(surfaceDir, surface.openai);
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(usage());
    process.exit(0);
  }
  const outDir = guardOutputPath(args.output);
  build(outDir);
  console.log(`[build-skill] wrote ${path.relative(ROOT, outDir) || outDir}`);
  for (const surface of SIBLING_SKILL_SURFACES) {
    const surfaceDir = path.join(path.dirname(outDir), surface.dir);
    console.log(`[build-skill] wrote ${path.relative(ROOT, surfaceDir) || surfaceDir}`);
  }
} catch (e) {
  console.error(`[build-skill] ${e.message}`);
  process.exit(1);
}
