#!/usr/bin/env node
// Public CLI for the lumin-repo-lens-lab repository evidence skill.
//
// Blessed entrypoint:
//
//   node audit-repo.mjs --root <repo> --output <dir> [--profile NAME] [--sarif]
//
// Stable user-facing capabilities:
//   - audit        (--profile quick|full|ci)
//   - pre-write    (--pre-write --intent <file|->)
//   - post-write   (--post-write --pre-write-advisory <file>)
//   - canon-draft  (--canon-draft [--sources <csv>])
//   - check-canon  (--check-canon [--sources <csv>])
//
// Sibling root scripts (build-symbol-graph.mjs, measure-topology.mjs,
// check-canon.mjs, etc.) remain internal engine entrypoints for
// development, testing, and narrow step-by-step debugging. Public docs
// should point users here first.
//
// Runs the requested capability in the order SKILL.md documents and
// writes `manifest.json` summarizing what happened. Audit profiles run
// the structural pipeline; pre-write-only invocations deliberately skip
// that base pipeline and delegate to the intent-shaped cold-cache gate.
// Partial failure is OK — a step's non-zero exit demotes it to `skipped`
// in the manifest rather than aborting the whole run. This matches
// reviewer's intent: the orchestrator is a convenience, not an
// all-or-nothing gate.
//
// Profiles:
//   quick — triage, topology, discipline, symbol graph, classify, rank
//   full  — quick + call graph + barrel discipline + shape index + function clone cues
//           + runtime fusion (if coverage present) + staleness (if git)
//   ci    — full + emit SARIF always
//
// manifest.json fields:
//   profile         which profile ran
//   commandsRun     scripts actually invoked, in order, with status
//   scanRange       root, includeTests, languages, excludes
//   confidence      parseErrors, unresolvedInternalRatio, externalImports
//   blindZones      standardized blind-zone list (lumin-audit-core)
//   livingAudit     existing living audit docs that the model should read/update
//   skipped         scripts that were intentionally skipped (with reason)
//
// Design: this script does NOT re-implement any analysis. Every real
// step is a child process invocation of the existing .mjs. Failure of
// any step is captured but never hidden.

import { existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { normalizeIncludeTests } from '../lib/cli.mjs';
import { assertRuntimeSetup, formatRuntimeSetupError } from '../lib/dependency-guard.mjs';
import { detectMaintainerSelfAuditExcludes, mergeExcludes } from '../lib/self-audit-excludes.mjs';
import {
  clearIncrementalCache,
  openIncrementalCacheStore,
} from '../lib/incremental-cache-store.mjs';
import {
  createArtifactReadMetrics,
  executeBaseRuntime,
  executeAuditLifecycle,
  buildManifestRootWithEvidence,
  finalizeAuditRunWithCompanions,
  applyLifecycleAndRefreshManifestEvidence,
} from '../lib/audit-manifest.mjs';
import { normalizeGeneratedArtifactsMode } from '../lib/generated-artifact-mode.mjs';
import {
  generateInvocationId,
} from '../lib/pre-write-artifact.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HELP_TEXT = `
lumin-repo-lens-lab public CLI

Recommended entrypoint:
  lumin-repo-lens-lab --root <repo> --output <dir>
  node scripts/audit-repo.mjs --root <repo> --output <dir>    # generated skill package
  node audit-repo.mjs --root <repo> --output <dir>            # maintainer checkout

Stable capabilities:
  audit
    lumin-repo-lens-lab --root <repo> --output <dir> --profile quick|full|ci

  pre-write
    lumin-repo-lens-lab --root <repo> --output <dir> --pre-write --pre-write-engine auto --intent intent.json
    lumin-repo-lens-lab --root <repo> --output <dir> --pre-write --rust-pre-write --intent intent.json

  post-write
    lumin-repo-lens-lab --root <repo> --output <dir> --post-write --pre-write-advisory advisory.json

  canon-draft
    lumin-repo-lens-lab --root <repo> --output <dir> --canon-draft [--sources type-ownership,naming] [--canon-output <dir>]

  check-canon
    lumin-repo-lens-lab --root <repo> --output <dir> --check-canon [--sources all]

Flags:
  --root, -r               repo root to scan
  --output, -o             artifact output dir (default: <root>/.audit)
  --profile                quick | full | ci (default: quick)
  --sarif                  force SARIF emission
  --production             exclude test files
  --include-tests          include test files (default)
  --include-tests=false    exclude test files
  --no-include-tests       exclude test files
  --no-tests               exclude test files
  --exclude-tests          exclude test files
  --exclude <pattern>      repeatable dir-segment or file-path exclusion
  --no-incremental        force cold producer artifacts where incremental is supported
  --cache-root <path>     stable incremental cache root (default: <root>/.audit/.cache)
  --clear-incremental-cache
                           clear this repo's incremental cache before supported producers run
  --generated-artifacts <mode>
                           default | present | prepared (diagnostic provenance only; does not run generators)
  --rust-analyzer          opt in to the Rust-owned unified analyzer artifact when triage counts .rs files
  --no-self-audit-excludes do not auto-exclude maintainer lab/corpus mirrors
  --strict-post-write      exit 2 when post-write orchestration cannot run
  --strict-post-write-confidence
                           exit 2 when post-write delta confidence is limited
  --pre-write-engine <js|rust|auto>
                         pre-write owner selection (default: auto)
                           pre-write execution surface
  --rust-pre-write         alias for --pre-write-engine rust
  --strict-check-canon     escalate drift to exit 1, all-fail to exit 2
  --sources, --source      canon source CSV (alias; --sources wins)
  --canon-output <dir>     canon-draft proposal dir (default: <root>/canonical-draft)

Internal note:
  Root sibling scripts such as build-symbol-graph.mjs, measure-topology.mjs,
  generate-canon-draft.mjs, and check-canon.mjs are internal engine
  entrypoints. They remain available for engine development and debugging,
  but the public surface is audit-repo.mjs plus the validation modes above.

Profiles:
  quick  default fast structural pass
  full   quick + call graph + barrel discipline + shape index + function clone cues + optional runtime/staleness
  ci     full + SARIF
`.trim();

// ─── Lifecycle flag matrix (see SKILL.md § Lifecycle flag matrix) ───
//
// Three lifecycle flags layered on top of the base pipeline:
//   --pre-write     (P1-3)  needs --intent <file|->
//   --post-write    (P2-2)  needs --pre-write-advisory <file>
//   --canon-draft   (P3-4)  no companion; optional --sources <csv>
//
// Interaction rules (authoritative: SKILL.md table):
//   pre-write ↔ post-write  : MUTUALLY EXCLUSIVE (exit 2 if both set)
//   canon-draft             : ORTHOGONAL to both
//   any combination below   : allowed — manifest.{preWrite,postWrite,canonDraft}
//                              populated independently
//
// None of these flags enter the default quick/full/ci profiles.
const CLI_OPTIONS = {
  help: { type: 'boolean', short: 'h' },
  root: { type: 'string', short: 'r' },
  output: { type: 'string', short: 'o' },
  profile: { type: 'string', default: 'quick' },
  sarif: { type: 'boolean', default: false },
  'include-tests': { type: 'boolean', default: true },
  production: { type: 'boolean', default: false },
  'no-tests': { type: 'boolean', default: false },
  'exclude-tests': { type: 'boolean', default: false },
  verbose: { type: 'boolean', default: false },
  // P1-3: opt-in pre-write integration. Not in default profiles.
  'pre-write': { type: 'boolean', default: false },
  intent: { type: 'string' },
  'pre-write-engine': { type: 'string', default: 'auto' },
  'rust-pre-write': { type: 'boolean', default: false },
  // P2-2: opt-in post-write integration. Mutually exclusive with --pre-write.
  'post-write': { type: 'boolean', default: false },
  'pre-write-advisory': { type: 'string' },
  'delta-out': { type: 'string' },
  'no-include-tests': { type: 'boolean', default: false },
  'no-fresh-audit': { type: 'boolean', default: false },
  'no-self-audit-excludes': { type: 'boolean', default: false },
  'no-incremental': { type: 'boolean', default: false },
  'cache-root': { type: 'string' },
  'clear-incremental-cache': { type: 'boolean', default: false },
  'generated-artifacts': { type: 'string', default: 'default' },
  'rust-analyzer': { type: 'boolean', default: false },
  exclude: { type: 'string', multiple: true, default: [] },
  // P2-2 follow-up: strict mode converts manifest.postWrite.ran === false
  // into exit code 2. Closes the "silent CI green on unreadable advisory"
  // gap without changing default advisory semantics.
  'strict-post-write': { type: 'boolean', default: false },
  'strict-post-write-confidence': { type: 'boolean', default: false },
  // P3-4-b: opt-in canon-draft orchestrator. Thin spawn wrapper that
  // invokes `generate-canon-draft.mjs` per source. NOT in default profiles.
  // `--sources <csv>` scopes to a subset; default runs all four sources
  // from `CANON_DRAFT_SOURCES`. Mutually orthogonal with --pre-write /
  // --post-write (canon draft operates outside the lifecycle-stage axis).
  'canon-draft': { type: 'boolean', default: false },
  'sources': { type: 'string' },
  'source': { type: 'string' },
  'canon-output': { type: 'string' },
  // P5-4: opt-in check-canon orchestrator. Thin spawn wrapper that
  // invokes `check-canon.mjs`; when all sources are requested it uses the
  // CLI's single-invocation `--source all` path to avoid duplicate child
  // startup/scans. NOT in default profiles. Mutually orthogonal with
  // --pre-write / --post-write (can coexist; pre/post mutex still applies
  // separately).
  // Per p5-4.md §4.4: advisory default (orchestrator exit 0 if ran);
  // `--strict-check-canon` escalates drift→1, all-fail→2.
  'check-canon': { type: 'boolean', default: false },
  'strict-check-canon': { type: 'boolean', default: false },
};

const { values, tokens } = parseArgs({
  options: CLI_OPTIONS,
  strict: false,
  tokens: true,
});

if (values.help) {
  console.log(HELP_TEXT);
  process.exit(0);
}

const KNOWN_OPTIONS = new Set(Object.keys(CLI_OPTIONS));
const unknownOptions = [...new Set(tokens
  .filter((token) => token.kind === 'option' && !KNOWN_OPTIONS.has(token.name))
  .map((token) => `--${token.name}`))];
if (unknownOptions.length > 0) {
  console.error(`[audit-repo] unknown option(s): ${unknownOptions.join(', ')}`);
  process.exit(2);
}

if (!values.root) {
  console.error('usage: audit-repo.mjs --root <repo> [--output <dir>] [--profile quick|full|ci] [--sarif] [--production]\n       audit-repo.mjs --help');
  process.exit(1);
}

const PROFILE_EXPLICIT = process.argv.slice(2).some((arg) =>
  arg === '--profile' || arg.startsWith('--profile='));
const WRITE_LIFECYCLE_ONLY =
  (values['pre-write'] === true || values['post-write'] === true) &&
  !(values['pre-write'] === true && values['post-write'] === true) &&
  values['canon-draft'] !== true &&
  values['check-canon'] !== true &&
  values.sarif !== true &&
  values['rust-analyzer'] !== true &&
  !PROFILE_EXPLICIT;
try {
  await assertRuntimeSetup({
    startDir: __dirname,
    commandName: 'audit-repo',
    requireAnalysisDependencies: !WRITE_LIFECYCLE_ONLY,
  });
} catch (error) {
  console.error(formatRuntimeSetupError(error));
  process.exit(error?.exitCode ?? 2);
}

const ROOT = path.resolve(values.root);
const OUT = path.resolve(values.output ?? path.join(ROOT, '.audit'));
const OUTPUT_WAS_DEFAULT = !values.output;
const PROFILE = values.profile;
const AUDIT_RUN_ID = generateInvocationId();
const SOURCES_VALUE = values.sources ?? values.source;
const INCLUDE_TESTS = normalizeIncludeTests(values, process.argv.slice(2));
const PRODUCTION = !INCLUDE_TESTS;
let GENERATED_ARTIFACTS_MODE = 'default';
try {
  GENERATED_ARTIFACTS_MODE = normalizeGeneratedArtifactsMode(values['generated-artifacts']);
} catch (error) {
  console.error(`[audit-repo] ${error.message}`);
  process.exit(2);
}
const REQUESTED_PRE_WRITE_ENGINE = values['rust-pre-write'] ? 'rust' : values['pre-write-engine'];
const AUTO_EXCLUDES = values['no-self-audit-excludes']
  ? []
  : detectMaintainerSelfAuditExcludes(ROOT);
const EFFECTIVE_EXCLUDES = mergeExcludes(values.exclude ?? [], AUTO_EXCLUDES);

mkdirSync(OUT, { recursive: true });

function isWithinPath(child, parent) {
  const rel = path.relative(parent, child);
  return rel === '' || (!!rel && !rel.startsWith('..') && !path.isAbsolute(rel));
}

if (OUTPUT_WAS_DEFAULT) {
  process.stderr.write(
    `[audit-repo] privacy note: default artifacts are written to ${path.join(ROOT, '.audit')}.\n` +
    `[audit-repo] Add ".audit/" to .gitignore or use --output outside the repo if these artifacts should not be committed.\n`
  );
} else if (!isWithinPath(OUT, ROOT)) {
  process.stderr.write(
    `[audit-repo] note: --output is outside --root; artifacts will be written to: ${OUT}\n`
  );
}

if (!['quick', 'full', 'ci'].includes(PROFILE)) {
  console.error(`[audit-repo] unknown profile: ${PROFILE}. Use quick|full|ci.`);
  process.exit(1);
}

if (!['js', 'rust', 'auto'].includes(REQUESTED_PRE_WRITE_ENGINE)) {
  console.error(`[audit-repo] unknown --pre-write-engine: ${REQUESTED_PRE_WRITE_ENGINE}. Use js|rust|auto.`);
  process.exit(2);
}

if (values['clear-incremental-cache'] === true) {
  const cacheStore = openIncrementalCacheStore({
    root: ROOT,
    cacheRoot: values['cache-root'],
  });
  clearIncrementalCache(cacheStore);
}

const commandsRun = [];
const skipped = [];
let rustAnalysisRun = { requested: values['rust-analyzer'] === true, ran: false, status: 'not-requested' };

const artifactReadMetrics = createArtifactReadMetrics({ rootDir: OUT });

function forwardedScanArgs() {
  const args = [];
  if (!INCLUDE_TESTS) args.push('--production');
  for (const exc of EFFECTIVE_EXCLUDES) args.push('--exclude', exc);
  return args;
}

function forwardedIncrementalArgs() {
  const args = [];
  if (values['no-incremental'] === true) args.push('--no-incremental');
  if (values['cache-root']) args.push('--cache-root', path.resolve(values['cache-root']));
  return args;
}

function rustAnalyzerInvocation() {
  const configuredBinary = process.env.LUMIN_RUST_ANALYZER_BIN?.trim();
  if (configuredBinary) {
    return {
      command: configuredBinary,
      prefixArgs: [],
      source: 'env:LUMIN_RUST_ANALYZER_BIN',
    };
  }

  const manifestCandidates = [
    path.join(__dirname, 'experiments', 'Cargo.toml'),
    path.join(__dirname, '..', 'experiments', 'Cargo.toml'),
    path.join(__dirname, '..', '..', '..', '..', 'experiments', 'Cargo.toml'),
  ];
  const manifestPath = manifestCandidates.find((candidate) => existsSync(candidate));
  if (!manifestPath) {
    throw new Error(
      'rust analyzer requested but no Rust analyzer was found; set LUMIN_RUST_ANALYZER_BIN or run from a maintainer checkout with experiments/Cargo.toml'
    );
  }

  return {
    command: 'cargo',
    prefixArgs: [
      'run',
      '--quiet',
      '--manifest-path',
      manifestPath,
      '--package',
      'lumin-rust-analyzer',
      '--',
    ],
    source: 'cargo:experiments',
    manifestPath,
  };
}

function forwardedRustAnalyzerArgs() {
  const args = [];
  if (!INCLUDE_TESTS) args.push('--production');
  for (const exc of EFFECTIVE_EXCLUDES) args.push('--exclude', exc);
  if (values['no-incremental'] === true) args.push('--no-incremental');
  if (values['cache-root']) args.push('--cache-root', path.resolve(values['cache-root']));
  if (values['clear-incremental-cache'] === true) args.push('--clear-incremental-cache');
  return args;
}

function buildPreWriteRoutingInput(requestedEngine, intentFlag) {
  return {
    schemaVersion: 'lumin-pre-write-routing-input.v1',
    requestedEngine,
    intentFlag: intentFlag === '-' ? '-' : path.resolve(intentFlag),
  };
}

function buildLifecycleRequestGuardRequest() {
  return {
    schemaVersion: 'lumin-lifecycle-request-guard.v1',
    preWriteRequested: values['pre-write'] === true,
    postWriteRequested: values['post-write'] === true,
    preWriteIntentPresent: Boolean(values.intent),
    requestedPreWriteEngine: REQUESTED_PRE_WRITE_ENGINE,
  };
}

function manifestEvidenceOptions() {
  return {
    root: ROOT,
    outDir: OUT,
    includeTests: INCLUDE_TESTS,
    production: PRODUCTION,
    excludes: EFFECTIVE_EXCLUDES,
    autoExcludes: AUTO_EXCLUDES,
    generatedArtifactsMode: GENERATED_ARTIFACTS_MODE,
    rustAnalysisRun,
    mergeRustAnalysisRun: true,
    onArtifactRead: artifactReadMetrics.observeRead,
    basePipelinePlanned: RUN_BASE_PIPELINE,
    basePipelineSkipReason: BASE_PIPELINE_SKIP_REASON,
  };
}

function performanceCacheRoot() {
  return path.resolve(values['cache-root'] ?? path.join(ROOT, '.audit', '.cache'));
}

function rustAnalyzerInvocationOrNull() {
  try {
    const invocation = rustAnalyzerInvocation();
    return {
      command: invocation.command,
      prefixArgs: invocation.prefixArgs,
      source: invocation.source,
      ...(invocation.manifestPath ? { manifestPath: invocation.manifestPath } : {}),
    };
  } catch {
    return null;
  }
}

function buildRuntimeExecutorRequest() {
  return {
    schemaVersion: 'lumin-audit-runtime-executor-request.v2',
    runId: AUDIT_RUN_ID,
    profile: PROFILE,
    profileExplicit: PROFILE_EXPLICIT,
    sarif: values.sarif === true,
    preWrite: values['pre-write'] === true,
    postWrite: values['post-write'] === true,
    canonDraft: values['canon-draft'] === true,
    checkCanon: values['check-canon'] === true,
    root: ROOT,
    output: OUT,
    scriptsDir: __dirname,
    nodeExecutable: process.execPath,
    verbose: values.verbose === true,
    scanRange: {
      includeTests: INCLUDE_TESTS,
      production: PRODUCTION,
      excludes: EFFECTIVE_EXCLUDES,
      autoExcludes: AUTO_EXCLUDES,
    },
    cache: {
      noIncremental: values['no-incremental'] === true,
      cacheRoot: performanceCacheRoot(),
      clearIncrementalCache: values['clear-incremental-cache'] === true,
    },
    generatedArtifacts: {
      mode: GENERATED_ARTIFACTS_MODE,
    },
    rustAnalyzer: {
      requested: values['rust-analyzer'] === true,
      rustFiles: 0,
      sourceCommit: null,
      invocation: values['rust-analyzer'] === true ? rustAnalyzerInvocationOrNull() : null,
      forwardedArgs: forwardedRustAnalyzerArgs(),
    },
  };
}

function buildCanonDraftLifecycleRequest() {
  return {
    schemaVersion: 'lumin-canon-draft-lifecycle-request.v1',
    sourcesValue: SOURCES_VALUE ?? null,
    root: ROOT,
    output: OUT,
    canonOutput: values['canon-output'] ? path.resolve(values['canon-output']) : null,
    scriptsDir: __dirname,
    nodeExecutable: process.execPath,
    scanArgs: forwardedScanArgs(),
  };
}

function buildCheckCanonLifecycleRequest() {
  return {
    schemaVersion: 'lumin-check-canon-lifecycle-request.v1',
    sourcesValue: SOURCES_VALUE ?? null,
    strict: !!values['strict-check-canon'],
    root: ROOT,
    output: OUT,
    scriptsDir: __dirname,
    nodeExecutable: process.execPath,
    scanArgs: forwardedScanArgs(),
  };
}

function buildPostWriteLifecycleRequest() {
  return {
    schemaVersion: 'lumin-post-write-lifecycle-request.v1',
    root: ROOT,
    output: OUT,
    scriptsDir: __dirname,
    nodeExecutable: process.execPath,
    advisoryPath: values['pre-write-advisory'] ? path.resolve(values['pre-write-advisory']) : null,
    deltaOut: values['delta-out'] ? path.resolve(values['delta-out']) : null,
    noFreshAudit: values['no-fresh-audit'] === true,
    scanArgs: forwardedScanArgs(),
    incrementalArgs: forwardedIncrementalArgs(),
  };
}

function buildRustPreWriteLifecycleRequest({
  invocation,
  advisoryInvocationId,
  rustNativePath,
  rustNativeLatestPath,
}) {
  return {
    schemaVersion: 'lumin-rust-pre-write-lifecycle-request.v1',
    root: ROOT,
    output: OUT,
    invocationId: advisoryInvocationId,
    rustNativeArtifactPath: rustNativePath,
    rustNativeLatestPath,
    analyzer: invocation ? {
      command: invocation.command,
      prefixArgs: invocation.prefixArgs,
      source: invocation.source,
      ...(invocation.manifestPath ? { manifestPath: invocation.manifestPath } : {}),
    } : null,
    includeTests: INCLUDE_TESTS,
    production: INCLUDE_TESTS === false,
    excludes: EFFECTIVE_EXCLUDES,
    failures: [],
  };
}

function buildJsPreWriteLifecycleRequest() {
  return {
    schemaVersion: 'lumin-js-pre-write-lifecycle-request.v1',
    root: ROOT,
    output: OUT,
    scriptsDir: __dirname,
    nodeExecutable: process.execPath,
    noFreshAudit: values['no-fresh-audit'] === true,
    scanArgs: forwardedScanArgs(),
  };
}

console.log(`[audit-repo] profile=${PROFILE}  root=${ROOT}  output=${OUT}`);

const baseExecution = executeBaseRuntime(buildRuntimeExecutorRequest());
const ORCHESTRATION_PLAN = baseExecution.plan;
const RUN_BASE_PIPELINE = ORCHESTRATION_PLAN?.basePipeline?.status === 'planned';
const BASE_PIPELINE_SKIP_REASON = RUN_BASE_PIPELINE
  ? null
  : ORCHESTRATION_PLAN?.basePipeline?.reason ?? 'base-pipeline-skipped';
commandsRun.push(...(baseExecution.commandsRun ?? []));
skipped.push(...(baseExecution.skipped ?? []));
rustAnalysisRun = baseExecution.rustAnalysisRun ?? rustAnalysisRun;
const basePipelineExitCode = Number(baseExecution.exitPolicy?.recommendedExitCode ?? 0);

// ─── Build manifest ───────────────────────────────────────
const manifestGenerated = new Date().toISOString();
const manifest = buildManifestRootWithEvidence({
  generated: manifestGenerated,
  profile: PROFILE,
  commandsRun,
  skipped,
  ...manifestEvidenceOptions(),
});

// ─── P1-3: pre-write opt-in step ──────────────────────────
// ─── P2-2: post-write opt-in step — mutually exclusive with --pre-write ─
//
// Exit-code contract (maintainer history notes §4.4, maintainer history notes v2 §4.2):
//   0 — audit succeeded; pre-write OR post-write (whichever was requested)
//       ran and succeeded; or neither was requested.
//   1 — existing audit-step-failed path; OR dispatched pre-write child failed.
//   2 — --pre-write without --intent; OR --post-write without
//       --pre-write-advisory; OR --pre-write AND --post-write together.

let preWriteBlock = undefined;
let postWriteBlock = undefined;
let canonDraftBlock = undefined;
let checkCanonBlock = undefined;
let finalExitCode = basePipelineExitCode;
let auditSummaryPreview = null;

const advisoryInvocationId = generateInvocationId();
const lifecycleRequestGuard = buildLifecycleRequestGuardRequest();
const lifecycleExecution = executeAuditLifecycle({
  schemaVersion: 'lumin-audit-lifecycle-execution-request.v1',
  baseExitCode: basePipelineExitCode,
  lifecycleRequestGuard,
  preWrite: values['pre-write'] === true ? {
    requested: values['pre-write'] === true,
    routingInput: buildPreWriteRoutingInput(REQUESTED_PRE_WRITE_ENGINE, values.intent ?? 'missing'),
    rust: buildRustPreWriteLifecycleRequest({
      invocation: rustAnalyzerInvocationOrNull(),
      advisoryInvocationId,
      rustNativePath: path.join(OUT, `rust-pre-write-artifact.${advisoryInvocationId}.json`),
      rustNativeLatestPath: path.join(OUT, 'rust-pre-write-artifact.latest.json'),
    }),
    js: buildJsPreWriteLifecycleRequest(),
  } : null,
  postWrite: values['post-write'] === true ? {
    requested: values['post-write'] === true,
    request: buildPostWriteLifecycleRequest(),
  } : null,
  canonDraft: values['canon-draft'] === true ? {
    requested: values['canon-draft'] === true,
    request: buildCanonDraftLifecycleRequest(),
  } : null,
  checkCanon: values['check-canon'] === true ? {
    requested: values['check-canon'] === true,
    request: buildCheckCanonLifecycleRequest(),
  } : null,
  exitPolicy: {
    strictPostWrite: values['strict-post-write'] === true,
    strictPostWriteConfidence: values['strict-post-write-confidence'] === true,
  },
});
preWriteBlock = lifecycleExecution.preWrite ?? undefined;
postWriteBlock = lifecycleExecution.postWrite ?? undefined;
canonDraftBlock = lifecycleExecution.canonDraft ?? undefined;
checkCanonBlock = lifecycleExecution.checkCanon ?? undefined;
finalExitCode = lifecycleExecution.finalExitCode ?? basePipelineExitCode;

Object.assign(manifest, applyLifecycleAndRefreshManifestEvidence({
  manifest,
  lifecycle: {
    preWrite: preWriteBlock ?? null,
    postWrite: postWriteBlock ?? null,
    canonDraft: canonDraftBlock ?? null,
    checkCanon: checkCanonBlock ?? null,
  },
  ...manifestEvidenceOptions(),
}));
const manifestWrite = finalizeAuditRunWithCompanions({
  manifest,
  root: ROOT,
  outDir: OUT,
  profile: PROFILE,
  includeTests: INCLUDE_TESTS,
  production: PRODUCTION,
  excludes: EFFECTIVE_EXCLUDES,
  autoExcludes: AUTO_EXCLUDES,
  noIncremental: values['no-incremental'] === true,
  cacheRoot: performanceCacheRoot(),
  clearIncrementalCache: values['clear-incremental-cache'] === true,
  generatedArtifactsMode: GENERATED_ARTIFACTS_MODE,
  artifactReadEvents: artifactReadMetrics.events(),
  rustAnalysis: manifest.rustAnalysis,
  commandsRun,
  skipped,
  companionPolicy: {
    basePipelinePlanned: RUN_BASE_PIPELINE,
    basePipelineSkipReason: BASE_PIPELINE_SKIP_REASON,
  },
});
Object.assign(manifest, manifestWrite.closeoutUpdate ?? {});
auditSummaryPreview = manifestWrite.auditSummaryPreview ?? null;

// ─── Console report ───────────────────────────────────────
console.log('');
console.log(`[audit-repo] wrote ${manifestWrite.manifestPath ?? path.join(OUT, 'manifest.json')}`);
console.log(`[audit-repo] artifacts: ${manifestWrite.artifactsProducedCount ?? manifest.artifactsProduced.length} produced`);
if (manifestWrite.auditSummaryPath) {
  console.log(`[audit-repo] summary: ${manifestWrite.auditSummaryPath}`);
}
if (manifestWrite.reviewPackPath) {
  console.log(`[audit-repo] review pack: ${manifestWrite.reviewPackPath}`);
}
if (auditSummaryPreview) {
  console.log('');
  console.log(auditSummaryPreview);
}
const finalBlindZones = manifestWrite.blindZones ?? manifest.blindZones ?? [];
if (finalBlindZones.length > 0) {
  console.log(`[audit-repo] ${manifestWrite.blindZonesSummary}`);
  for (const z of finalBlindZones) {
    console.log(`             • ${z.area} (${z.severity}) — ${z.effect.slice(0, 80)}${z.effect.length > 80 ? '…' : ''}`);
  }
} else {
  console.log('[audit-repo] blindZones: none detected');
}
console.log('');
console.log('Next: review manifest.blindZones before making absence/removal claims.');

if (finalExitCode !== 0) process.exit(finalExitCode);
