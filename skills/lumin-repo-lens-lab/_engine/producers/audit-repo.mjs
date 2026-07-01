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
//   blindZones      standardized blind-zone list (_lib/blind-zones.mjs)
//   livingAudit     existing living audit docs that the model should read/update
//   skipped         scripts that were intentionally skipped (with reason)
//
// Design: this script does NOT re-implement any analysis. Every real
// step is a child process invocation of the existing .mjs. Failure of
// any step is captured but never hidden.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, existsSync, mkdirSync, statSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { formatBlindZonesSummary } from '../lib/blind-zones.mjs';
import {
  createArtifactReadMetrics,
  loadIfExists as loadArtifact,
} from '../lib/artifacts.mjs';
import { atomicWrite } from '../lib/atomic-write.mjs';
import { normalizeIncludeTests } from '../lib/cli.mjs';
import { renderAuditSummary } from '../lib/audit-summary.mjs';
import { renderAuditReviewPack } from '../lib/audit-review-pack.mjs';
import { renderTopologyMermaid } from '../lib/topology-mermaid.mjs';
import { assertRuntimeSetup, formatRuntimeSetupError } from '../lib/dependency-guard.mjs';
import { detectMaintainerSelfAuditExcludes, mergeExcludes } from '../lib/self-audit-excludes.mjs';
import { runCanonDraftLifecycle } from '../lib/audit-canon-draft.mjs';
import { runCheckCanonLifecycle } from '../lib/audit-check-canon.mjs';
import {
  clearIncrementalCache,
  openIncrementalCacheStore,
} from '../lib/incremental-cache-store.mjs';
import {
  buildProducerPerformanceArtifactFromLedger,
  buildOrchestrationPlan,
  buildManifestFinalSummaryUpdate,
  buildLifecycleSummary,
  buildManifestRoot,
  buildManifestEvidence,
  collectProducedArtifacts,
  refreshManifestEvidence,
  mergeRustAnalysisRun,
} from '../lib/audit-manifest.mjs';
import { normalizeGeneratedArtifactsMode } from '../lib/generated-artifact-mode.mjs';
import {
  clearProducerPhaseTiming,
  readProducerPhaseTiming,
} from '../lib/producer-phase-timing.mjs';
import { collectFiles } from '../lib/collect-files.mjs';
import { repoRelativeFileList } from '../lib/post-write-file-delta.mjs';
import {
  generateInvocationId,
  hashIntent,
  writeAdvisory,
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

try {
  await assertRuntimeSetup({ startDir: __dirname, commandName: 'audit-repo' });
} catch (error) {
  console.error(formatRuntimeSetupError(error));
  process.exit(error?.exitCode ?? 2);
}

const ROOT = path.resolve(values.root);
const OUT = path.resolve(values.output ?? path.join(ROOT, '.audit'));
const OUTPUT_WAS_DEFAULT = !values.output;
const PROFILE = values.profile;
const SOURCES_VALUE = values.sources ?? values.source;
const INCLUDE_TESTS = normalizeIncludeTests(values, process.argv.slice(2));
const PRODUCTION = !INCLUDE_TESTS;
const PRE_POST_MUTEX = values['pre-write'] && values['post-write'];
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

const ORCHESTRATION_PLAN = buildOrchestrationPlan({
  profile: PROFILE,
  sarif: values.sarif,
  preWrite: values['pre-write'],
  postWrite: values['post-write'],
  canonDraft: values['canon-draft'],
  checkCanon: values['check-canon'],
  rustAnalyzer: values['rust-analyzer'],
});
const EMIT_SARIF = ORCHESTRATION_PLAN.emitSarif === true;
const RUN_BASE_PIPELINE = ORCHESTRATION_PLAN.basePipeline?.status === 'planned';

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
const INCREMENTAL_PRODUCER_STEPS = new Set([
  'measure-topology.mjs',
  'measure-staleness.mjs',
  'build-block-clone-index.mjs',
  'build-symbol-graph.mjs',
  'build-shape-index.mjs',
  'build-function-clone-index.mjs',
]);

const artifactReadMetrics = createArtifactReadMetrics({ rootDir: OUT });
const loadIfExists = (name) => loadArtifact(OUT, name, {
  onRead: artifactReadMetrics.observeRead,
});

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

function forwardedGeneratedArtifactArgs(stepName) {
  return stepName === 'build-symbol-graph.mjs'
    ? ['--generated-artifacts', GENERATED_ARTIFACTS_MODE]
    : [];
}

function gitHeadCommit(root) {
  try {
    return execFileSync('git', ['rev-parse', 'HEAD'], {
      cwd: root,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return 'unknown';
  }
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

function rustFileCountFromTriage(triage) {
  const byLanguage = triage?.byLanguage ?? triage?.languages ?? triage?.summary?.byLanguage;
  if (byLanguage && typeof byLanguage === 'object') {
    const count = byLanguage.rs;
    const n = typeof count === 'number' ? count : (count?.files ?? 0);
    if (Number.isFinite(n) && n > 0) return n;
  }
  const shapeCount = triage?.shape?.rustFiles ?? triage?.shape?.rsFiles ?? 0;
  return typeof shapeCount === 'number' && Number.isFinite(shapeCount) ? shapeCount : 0;
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

function isPlainObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function readPreWriteIntentText(intentFlag) {
  if (intentFlag === '-') {
    try {
      return readFileSync(0, 'utf8');
    } catch (error) {
      throw new Error(`failed to read --intent -: ${error.message}`);
    }
  }

  const intentPath = path.resolve(intentFlag);
  if (!existsSync(intentPath)) {
    throw new Error(`intent file not found: ${intentPath}`);
  }
  try {
    return readFileSync(intentPath, 'utf8');
  } catch (error) {
    throw new Error(`failed to read intent: ${error.message}`);
  }
}

function normalizePreWriteIntentLanguage(value) {
  if (value === undefined) return null;
  if (value === 'rust' || value === 'js-ts') return value;
  throw new Error('intent.language must be "rust" or "js-ts" when present');
}

function stripPreWriteRouteOnlyFields(intentText) {
  let parsed;
  try {
    parsed = JSON.parse(intentText);
  } catch {
    return intentText;
  }
  if (!isPlainObject(parsed) || parsed.language === undefined) return intentText;
  delete parsed.language;
  return `${JSON.stringify(parsed, null, 2)}\n`;
}

function readPreWriteIntentForRouting(intentFlag) {
  const intentText = readPreWriteIntentText(intentFlag);
  let parsed;
  try {
    parsed = JSON.parse(intentText);
  } catch (error) {
    throw new Error(`intent JSON parse failed before engine selection: ${error.message}`);
  }
  if (!isPlainObject(parsed)) {
    throw new Error('intent must be a plain object before engine selection');
  }
  return {
    intentText,
    intentLanguage: normalizePreWriteIntentLanguage(parsed.language),
  };
}

function resolvePreWriteEngineForIntent(requestedEngine, intentFlag) {
  const { intentText, intentLanguage } = readPreWriteIntentForRouting(intentFlag);

  if (requestedEngine === 'js') {
    if (intentLanguage === 'rust') {
      throw new Error('intent.language "rust" is owned by lumin-rust-analyzer; use --pre-write-engine auto or --pre-write-engine rust');
    }
    return {
      engine: 'js',
      childIntentFlag: intentFlag === '-' ? '-' : path.resolve(intentFlag),
      childIntentInput: intentFlag === '-' ? intentText : null,
      engineSelection: {
        requested: requestedEngine,
        selected: 'js',
        reason: 'explicit-cli',
        ...(intentLanguage !== null ? { intentLanguage } : {}),
      },
    };
  }

  if (requestedEngine === 'rust') {
    if (intentLanguage === 'js-ts') {
      throw new Error('intent.language "js-ts" is owned by pre-write.mjs; use --pre-write-engine js or --pre-write-engine auto');
    }
    return {
      engine: 'rust',
      childIntentFlag: '-',
      childIntentInput: stripPreWriteRouteOnlyFields(intentText),
      engineSelection: {
        requested: requestedEngine,
        selected: 'rust',
        reason: 'explicit-cli',
        ...(intentLanguage !== null ? { intentLanguage } : {}),
      },
    };
  }

  const selected = intentLanguage === 'rust' ? 'rust' : 'js';
  return {
    engine: selected,
    childIntentFlag: '-',
    childIntentInput: selected === 'rust'
      ? stripPreWriteRouteOnlyFields(intentText)
      : intentText,
    engineSelection: {
      requested: requestedEngine,
      selected,
      reason: intentLanguage === null
        ? 'intent-language-absent-default-js'
        : 'intent-language',
      ...(intentLanguage !== null ? { intentLanguage } : {}),
    },
  };
}

function childProcessOptionsForIntent(route, originalIntentFlag) {
  if (route.childIntentInput !== null && route.childIntentInput !== undefined) {
    return {
      input: route.childIntentInput,
      stdio: ['pipe', 'inherit', 'inherit'],
    };
  }
  return {
    stdio: [originalIntentFlag === '-' ? 'inherit' : 'ignore', 'inherit', 'inherit'],
  };
}

function readJsonFileStrict(filePath, label) {
  try {
    return JSON.parse(readFileSync(filePath, 'utf8'));
  } catch (error) {
    throw new Error(`${label} parse failed: ${error.message}`);
  }
}

function buildFileInventoryBlock(failures) {
  try {
    const files = repoRelativeFileList(ROOT, collectFiles(ROOT, {
      includeTests: INCLUDE_TESTS,
      exclude: EFFECTIVE_EXCLUDES,
    }));
    return {
      status: 'available',
      pathMode: 'repo-relative',
      fileCount: files.length,
      files,
    };
  } catch (error) {
    const reason = error?.message?.slice(0, 400) ?? 'unknown';
    failures.push({ kind: 'file-inventory-hook-failed', reason });
    return { status: 'failed', reason };
  }
}

function buildRustPreWriteLifecycleAdvisory({
  rustArtifact,
  rustArtifactPath,
  invocationId,
  sourceCommit,
}) {
  const intent = { ...(rustArtifact.intent ?? {}), language: 'rust' };
  const failures = [];
  return {
    invocationId,
    intentHash: hashIntent(intent),
    artifactPaths: {
      invocationSpecific: path.join(OUT, `pre-write-advisory.${invocationId}.json`),
      latest: path.join(OUT, 'pre-write-advisory.latest.json'),
      rustNative: rustArtifactPath,
    },
    scanRange: {
      root: ROOT,
      output: OUT,
      includeTests: INCLUDE_TESTS,
      production: INCLUDE_TESTS === false,
      excludes: EFFECTIVE_EXCLUDES,
    },
    intent,
    intentWarnings: rustArtifact.intentWarnings ?? [],
    evidenceAvailability: {
      status: 'available',
      producer: 'lumin-rust-analyzer',
      rustNativeArtifactPath: rustArtifactPath,
    },
    lookups: rustArtifact.lookups ?? [],
    shapeLookups: rustArtifact.shapeLookups ?? [],
    fileLookups: rustArtifact.fileLookups ?? [],
    dependencyLookups: rustArtifact.dependencyLookups ?? [],
    inlinePatternLookups: rustArtifact.inlinePatternLookups ?? [],
    cueCards: rustArtifact.cueCards ?? [],
    suppressedCues: rustArtifact.suppressedCues ?? [],
    unavailableEvidence: rustArtifact.unavailableEvidence ?? [],
    cuePolicy: null,
    boundaryChecks: [],
    drift: null,
    preWrite: {
      fileInventory: buildFileInventoryBlock(failures),
      rustNativeArtifactPath: rustArtifactPath,
      sourceCommit,
    },
    rustPreWrite: {
      schemaVersion: rustArtifact.schemaVersion ?? null,
      policyVersion: rustArtifact.policyVersion ?? null,
      producer: rustArtifact.meta?.producer ?? 'lumin-rust-analyzer',
      coverage: rustArtifact.coverage ?? null,
    },
    capabilities: {
      language: 'rust',
      producer: 'lumin-rust-analyzer',
      postWriteTypeEscapes: 'not-applicable',
    },
    failures,
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
    onArtifactRead: artifactReadMetrics.observeRead,
  };
}

const PRODUCER_PERFORMANCE_LARGEST_ARTIFACT_LIMIT = 10;

function performanceCacheRoot() {
  return path.resolve(values['cache-root'] ?? path.join(ROOT, '.audit', '.cache'));
}

function memorySnapshot() {
  const usage = process.memoryUsage();
  return {
    rssBytes: usage.rss,
    heapTotalBytes: usage.heapTotal,
    heapUsedBytes: usage.heapUsed,
    externalBytes: usage.external,
    arrayBuffersBytes: usage.arrayBuffers ?? 0,
  };
}

function memoryDelta(before, after) {
  return {
    rssBytes: after.rssBytes - before.rssBytes,
    heapTotalBytes: after.heapTotalBytes - before.heapTotalBytes,
    heapUsedBytes: after.heapUsedBytes - before.heapUsedBytes,
    externalBytes: after.externalBytes - before.externalBytes,
    arrayBuffersBytes: after.arrayBuffersBytes - before.arrayBuffersBytes,
  };
}

function collectArtifactSizeSummary(artifacts = collectProducedArtifacts(OUT)) {
  const byName = Object.create(null);
  let totalBytes = 0;

  for (const name of artifacts) {
    const artifactPath = path.join(OUT, name);
    try {
      const stats = statSync(artifactPath);
      if (!stats.isFile()) continue;
      byName[name] = { bytes: stats.size };
      totalBytes += stats.size;
    } catch {
      // Artifact enumeration is best-effort: disappearing files should not
      // turn a completed audit into a failed one.
    }
  }

  const largest = Object.entries(byName)
    .map(([name, entry]) => ({ name, bytes: entry.bytes }))
    .sort((a, b) => b.bytes - a.bytes || a.name.localeCompare(b.name))
    .slice(0, PRODUCER_PERFORMANCE_LARGEST_ARTIFACT_LIMIT);

  return {
    producedCount: Object.keys(byName).length,
    totalBytes,
    largest,
    byName,
  };
}

function buildProducerPerformanceArtifact(generated, artifactsProduced) {
  const producerEvents = commandsRun.map((entry) => {
    const phaseTiming = readProducerPhaseTiming(OUT, entry.step, {
      onRead: artifactReadMetrics.observeRead,
    });
    return {
      kind: 'producer',
      name: entry.step,
      status: entry.status,
      wallMs: typeof entry.ms === 'number' ? entry.ms : null,
      ...(phaseTiming?.phases?.length > 0 ? { phases: phaseTiming.phases } : {}),
      ...(phaseTiming?.counters ? { counters: phaseTiming.counters } : {}),
      ...(entry.memory ? { memory: entry.memory } : {}),
      ...(entry.stderr ? { stderrSnippet: entry.stderr } : {}),
    };
  });
  const skippedEvents = skipped.map((entry) => ({
    kind: 'skipped',
    name: entry.step,
    reason: entry.reason,
  }));

  return buildProducerPerformanceArtifactFromLedger({
    schemaVersion: 'lumin-audit-orchestration-ledger.v1',
    generated,
    root: ROOT,
    output: OUT,
    profile: PROFILE,
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
    artifactReads: artifactReadMetrics.summary(),
    artifacts: collectArtifactSizeSummary(artifactsProduced),
    events: [...producerEvents, ...skippedEvents],
  });
}

function shortenConsoleLine(line, max = 150) {
  const normalized = String(line ?? '').replace(/\s+/g, ' ').trim();
  return normalized.length > max ? `${normalized.slice(0, max - 1)}…` : normalized;
}

function collectSummarySectionLines(markdown, heading, limit) {
  const lines = String(markdown ?? '').split(/\r?\n/);
  const start = lines.findIndex((line) => line.trim() === heading);
  if (start < 0) return [];
  const out = [];
  for (const line of lines.slice(start + 1)) {
    const trimmed = line.trim();
    if (trimmed.startsWith('## ')) break;
    if (!trimmed) continue;
    if (/^(?:-|\d+\.)\s+/.test(trimmed)) {
      out.push(shortenConsoleLine(trimmed));
      if (out.length >= limit) break;
    }
  }
  return out;
}

function renderSummaryConsolePreview(markdown) {
  const sections = [
    ['Command Result', collectSummarySectionLines(markdown, '## Command Result', 3)],
    ['Read First', collectSummarySectionLines(markdown, '## Read First', 2)],
    ['Measured Cues', collectSummarySectionLines(markdown, '## Measured Cues (Unranked)', 3)],
    ['Living Audit Tracking', collectSummarySectionLines(markdown, '## Living Audit Tracking', 2)],
    ['Guardrails', collectSummarySectionLines(markdown, '## Guardrails', 2)],
  ].filter(([, lines]) => lines.length > 0);
  if (sections.length === 0) return null;

  const out = ['[audit-repo] artifact brief preview:'];
  for (const [label, lines] of sections) {
    out.push(`[audit-repo]   ${label}:`);
    for (const line of lines) out.push(`[audit-repo]     ${line}`);
  }
  return out.join('\n');
}

function runStep(scriptRelPath, { required = false, precondition = null, reason = '' } = {}) {
  const name = path.basename(scriptRelPath);
  if (precondition) {
    const ok = precondition();
    if (!ok) {
      skipped.push({ step: name, reason });
      console.log(`[audit-repo] skip  ${name}  (${reason})`);
      return { status: 'skipped', reason };
    }
  }
  // P1-3 shell-safety: spawn via argv array. Shell-string interpolation
  // broke on paths with spaces / $ / parentheses. Keeping execSync's
  // import for any future command-line composition outside of producer
  // spawning.
  const argv = [
    path.join(__dirname, scriptRelPath),
    '--root', ROOT,
    '--output', OUT,
    ...forwardedScanArgs(),
    ...(INCREMENTAL_PRODUCER_STEPS.has(name) ? forwardedIncrementalArgs() : []),
    ...forwardedGeneratedArtifactArgs(name),
  ];
  const t0 = Date.now();
  const memoryBefore = memorySnapshot();
  clearProducerPhaseTiming(OUT, name);
  try {
    const out = execFileSync(process.execPath, argv, {
      stdio: values.verbose ? 'inherit' : ['ignore', 'pipe', 'pipe'],
      encoding: 'utf8',
    });
    const ms = Date.now() - t0;
    const memoryAfter = memorySnapshot();
    commandsRun.push({
      step: name,
      status: 'ok',
      ms,
      memory: {
        before: memoryBefore,
        after: memoryAfter,
        delta: memoryDelta(memoryBefore, memoryAfter),
      },
    });
    console.log(`[audit-repo] ok    ${name}  (${ms}ms)`);
    return { status: 'ok', out, ms };
  } catch (e) {
    const ms = Date.now() - t0;
    const memoryAfter = memorySnapshot();
    const status = required ? 'failed-required' : 'failed-optional';
    commandsRun.push({
      step: name, status, ms,
      memory: {
        before: memoryBefore,
        after: memoryAfter,
        delta: memoryDelta(memoryBefore, memoryAfter),
      },
      stderr: (e.stderr || e.message || '').toString().slice(0, 500),
    });
    console.log(`[audit-repo] ${required ? 'FAIL' : 'warn'}  ${name}  (${ms}ms) — ` +
                `${required ? 'required, aborting' : 'optional, continuing'}`);
    if (required) throw e;
    return { status };
  }
}

function runRustAnalyzerStep() {
  const triage = loadIfExists('triage.json');
  const rustFiles = rustFileCountFromTriage(triage);
  if (values['rust-analyzer'] !== true) {
    return { requested: false, ran: false, status: 'not-requested', rustFiles };
  }
  if (rustFiles <= 0) {
    const reason = 'no Rust files counted by triage';
    skipped.push({ step: 'lumin-rust-analyzer', reason });
    console.log(`[audit-repo] skip  lumin-rust-analyzer  (${reason})`);
    return { requested: true, ran: false, status: 'skipped', rustFiles, reason };
  }

  let invocation;
  try {
    invocation = rustAnalyzerInvocation();
  } catch (error) {
    const reason = error.message;
    skipped.push({ step: 'lumin-rust-analyzer', reason });
    console.log(`[audit-repo] skip  lumin-rust-analyzer  (${reason})`);
    return { requested: true, ran: false, status: 'unavailable', rustFiles, reason };
  }

  const artifact = 'rust-analyzer-health.latest.json';
  const artifactPath = path.join(OUT, artifact);
  const sourceCommit = gitHeadCommit(ROOT);
  const argv = [
    ...invocation.prefixArgs,
    '--root', ROOT,
    '--source-commit', sourceCommit,
    '--output', artifactPath,
    '--source-health-profile', 'compact',
    '--semantic-mode', 'metadata-only',
    ...forwardedRustAnalyzerArgs(),
  ];
  const t0 = Date.now();
  const memoryBefore = memorySnapshot();
  try {
    rmSync(artifactPath, { force: true });
    execFileSync(invocation.command, argv, {
      stdio: values.verbose ? 'inherit' : ['ignore', 'pipe', 'pipe'],
      encoding: 'utf8',
    });
    const ms = Date.now() - t0;
    const memoryAfter = memorySnapshot();
    const analyzerInvocation = {
      source: invocation.source,
      ...(invocation.manifestPath ? { manifestPath: invocation.manifestPath } : {}),
    };
    commandsRun.push({
      step: 'lumin-rust-analyzer',
      status: 'ok',
      ms,
      artifact,
      rustFiles,
      analyzerInvocation,
      memory: {
        before: memoryBefore,
        after: memoryAfter,
        delta: memoryDelta(memoryBefore, memoryAfter),
      },
    });
    console.log(`[audit-repo] ok    lumin-rust-analyzer  (${ms}ms)`);
    return {
      requested: true,
      ran: true,
      status: 'complete',
      rustFiles,
      artifact,
      path: artifactPath,
      sourceCommit,
      producer: 'lumin-rust-analyzer',
      analyzerInvocation,
    };
  } catch (e) {
    const ms = Date.now() - t0;
    const memoryAfter = memorySnapshot();
    const reason = Object.hasOwn(e ?? {}, 'status')
      ? `lumin-rust-analyzer exited non-zero: ${e.message}`
      : `lumin-rust-analyzer artifact refresh failed: ${e.message}`;
    commandsRun.push({
      step: 'lumin-rust-analyzer',
      status: 'failed-optional',
      ms,
      rustFiles,
      stderr: (e.stderr || e.message || '').toString().slice(0, 500),
      memory: {
        before: memoryBefore,
        after: memoryAfter,
        delta: memoryDelta(memoryBefore, memoryAfter),
      },
    });
    console.log(`[audit-repo] warn  lumin-rust-analyzer  (${ms}ms) — optional, continuing`);
    return { requested: true, ran: false, status: 'failed-optional', rustFiles, reason };
  }
}

function hasCoverage() {
  const candidates = [
    path.join(ROOT, 'coverage', 'coverage-final.json'),
    path.join(ROOT, '.nyc_output', 'coverage-final.json'),
  ];
  return candidates.some(existsSync);
}

function isGitWorkTree() {
  try {
    const out = execFileSync('git', ['rev-parse', '--is-inside-work-tree'], {
      cwd: ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
    return out === 'true';
  } catch {
    return false;
  }
}

function plannedSkip(stepName) {
  return ORCHESTRATION_PLAN.skipped?.find((entry) => entry.step === stepName) ?? null;
}

function recordPlannedSkip(stepName, fallbackReason) {
  const reason = plannedSkip(stepName)?.reason ?? fallbackReason;
  skipped.push({ step: stepName, reason });
  console.log(`[audit-repo] skip  ${stepName}  (${reason})`);
}

function plannedStepPrecondition(step) {
  switch (step.step) {
    case 'build-resolver-diagnostics.mjs':
    case 'build-entry-surface.mjs':
      return () => existsSync(path.join(OUT, 'symbols.json'));
    case 'build-module-reachability.mjs':
      return () =>
        existsSync(path.join(OUT, 'symbols.json')) &&
        existsSync(path.join(OUT, 'entry-surface.json'));
    case 'export-action-safety.mjs':
    case 'rank-fixes.mjs':
      return () => existsSync(path.join(OUT, 'dead-classify.json'));
    case 'merge-runtime-evidence.mjs':
      return hasCoverage;
    case 'measure-staleness.mjs':
      return isGitWorkTree;
    default:
      return null;
  }
}

function runPlannedBaseStep(step) {
  if (step.step === 'lumin-rust-analyzer') {
    rustAnalysisRun = runRustAnalyzerStep();
    return;
  }
  runStep(step.script, {
    required: step.required === true,
    precondition: plannedStepPrecondition(step),
    reason: step.skipReasonWhenUnmet ?? '',
  });
}

function runBasePipelineFromPlan(plan) {
  for (const step of plan.steps ?? []) {
    runPlannedBaseStep(step);
  }
}

console.log(`[audit-repo] profile=${PROFILE}  root=${ROOT}  output=${OUT}`);

if (!RUN_BASE_PIPELINE) {
  recordPlannedSkip('base-audit-profile', 'base audit profile skipped by Rust orchestration plan');
} else {
  runBasePipelineFromPlan(ORCHESTRATION_PLAN);
  if (!EMIT_SARIF && plannedSkip('emit-sarif.mjs')) {
    recordPlannedSkip('emit-sarif.mjs', 'not in --sarif mode');
  }
}

// ─── Build manifest ───────────────────────────────────────
const initialEvidence = buildManifestEvidence(manifestEvidenceOptions());

const initialRustAnalysis = mergeRustAnalysisRun({
  evidence: initialEvidence.rustAnalysis,
  run: rustAnalysisRun,
});
const manifestGenerated = new Date().toISOString();
const manifest = buildManifestRoot({
  generated: manifestGenerated,
  profile: PROFILE,
  root: ROOT,
  output: OUT,
  commandsRun,
  skipped,
  evidence: {
    scanRange: initialEvidence.scanRange,
    confidence: initialEvidence.confidence,
    blindZones: initialEvidence.blindZones,
    rustAnalysis: initialRustAnalysis,
    generatedArtifacts: initialEvidence.generatedArtifacts,
    livingAudit: initialEvidence.livingAudit,
  },
  artifactsProduced: collectProducedArtifacts(OUT, {
    rustAnalysis: initialRustAnalysis,
  }),
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
let finalExitCode = 0;
let auditSummaryPreview = null;

if (values['pre-write'] && values['post-write']) {
  const reason = '--pre-write and --post-write are mutually exclusive';
  process.stderr.write(`[audit-repo] ${reason}\n`);
  preWriteBlock = { requested: true, ran: false, reason };
  postWriteBlock = { requested: true, ran: false, reason };
  finalExitCode = 2;
} else if (values['pre-write']) {
  if (!values.intent) {
    process.stderr.write(`[audit-repo] --pre-write requested but skipped: --intent <file|-> missing\n`);
    preWriteBlock = {
      requested: true,
      ran: false,
      engine: REQUESTED_PRE_WRITE_ENGINE,
      ...(REQUESTED_PRE_WRITE_ENGINE === 'auto'
        ? {}
        : REQUESTED_PRE_WRITE_ENGINE === 'rust'
        ? { language: 'rust', producer: 'lumin-rust-analyzer' }
        : { language: 'js-ts', producer: 'pre-write.mjs' }),
      reason: '--intent missing',
    };
    finalExitCode = 2;
  } else {
    let preWriteRoute = null;
    try {
      preWriteRoute = resolvePreWriteEngineForIntent(REQUESTED_PRE_WRITE_ENGINE, values.intent);
    } catch (error) {
      preWriteBlock = {
        requested: true,
        ran: false,
        engine: REQUESTED_PRE_WRITE_ENGINE,
        reason: `pre-write engine selection failed: ${error.message}`,
      };
      finalExitCode = 2;
    }

    if (preWriteRoute?.engine === 'rust') {
      const { execFileSync: _exec } = await import('node:child_process');
      const sourceCommit = gitHeadCommit(ROOT);
      const advisoryInvocationId = generateInvocationId();
      const rustNativePath = path.join(OUT, `rust-pre-write-artifact.${advisoryInvocationId}.json`);
      const rustNativeLatestPath = path.join(OUT, 'rust-pre-write-artifact.latest.json');
      try {
        const invocation = rustAnalyzerInvocation();
        const preArgs = [
          ...invocation.prefixArgs,
          'pre-write',
          '--root', ROOT,
          '--source-commit', sourceCommit,
          '--intent', preWriteRoute.childIntentFlag,
          '--output', rustNativePath,
        ];
        if (!INCLUDE_TESTS) {
          preArgs.push('--production');
        }
        for (const pattern of EFFECTIVE_EXCLUDES) {
          preArgs.push('--exclude', pattern);
        }
        _exec(invocation.command, preArgs, childProcessOptionsForIntent(preWriteRoute, values.intent));
        const rustNativeContent = readFileSync(rustNativePath, 'utf8');
        atomicWrite(rustNativeLatestPath, rustNativeContent);
        const rustArtifact = readJsonFileStrict(rustNativePath, 'rust pre-write artifact');
        const advisory = buildRustPreWriteLifecycleAdvisory({
          rustArtifact,
          rustArtifactPath: rustNativePath,
          invocationId: advisoryInvocationId,
          sourceCommit,
        });
        const { latestPath, specificPath } = writeAdvisory(OUT, advisory);
        preWriteBlock = {
          requested: true,
          ran: true,
          engine: 'rust',
          language: 'rust',
          producer: 'lumin-rust-analyzer',
          engineSelection: preWriteRoute.engineSelection,
          advisoryPath: specificPath,
          latestAdvisoryPath: latestPath,
          advisoryInvocationId,
          rustNativeArtifactPath: rustNativePath,
          rustNativeLatestPath,
          sourceCommit,
          analyzerInvocation: {
            source: invocation.source,
            ...(invocation.manifestPath ? { manifestPath: invocation.manifestPath } : {}),
          },
        };
      } catch (e) {
        preWriteBlock = {
          requested: true,
          ran: false,
          engine: 'rust',
          language: 'rust',
          producer: 'lumin-rust-analyzer',
          engineSelection: preWriteRoute.engineSelection,
          reason: `lumin-rust-analyzer pre-write exited non-zero: ${e.message}`,
        };
        finalExitCode = typeof e.status === 'number' && e.status !== 0 ? e.status : 1;
      }
    } else if (preWriteRoute?.engine === 'js') {
      const { execFileSync: _exec } = await import('node:child_process');
      const preWritePath = path.join(__dirname, 'pre-write.mjs');
      const preArgs = [
        preWritePath,
        '--root', ROOT,
        '--output', OUT,
        '--intent', preWriteRoute.childIntentFlag,
        ...forwardedScanArgs(),
      ];
      if (values['no-fresh-audit']) preArgs.push('--no-fresh-audit');
      try {
        _exec(process.execPath, preArgs, childProcessOptionsForIntent(preWriteRoute, values.intent));
        const latestAdvisoryPath = path.join(OUT, 'pre-write-advisory.latest.json');
        let advisoryPath = latestAdvisoryPath;
        let advisoryInvocationId = null;
        let advisoryEvidenceAvailability = null;
        try {
          const advisory = JSON.parse(readFileSync(latestAdvisoryPath, 'utf8'));
          advisoryInvocationId = advisory.invocationId ?? null;
          advisoryEvidenceAvailability = advisory.evidenceAvailability ?? null;
          if (typeof advisory.artifactPaths?.invocationSpecific === 'string') {
            advisoryPath = path.resolve(advisory.artifactPaths.invocationSpecific);
          } else if (typeof advisory.invocationId === 'string') {
            advisoryPath = path.join(OUT, `pre-write-advisory.${advisory.invocationId}.json`);
          }
        } catch { /* leave latest path fallback */ }
        preWriteBlock = {
          requested: true,
          ran: true,
          engine: 'js',
          language: 'js-ts',
          producer: 'pre-write.mjs',
          engineSelection: preWriteRoute.engineSelection,
          advisoryPath,
          latestAdvisoryPath,
          advisoryInvocationId,
          evidenceAvailability: advisoryEvidenceAvailability,
        };
      } catch (e) {
        preWriteBlock = {
          requested: true,
          ran: false,
          engine: 'js',
          language: 'js-ts',
          producer: 'pre-write.mjs',
          engineSelection: preWriteRoute.engineSelection,
          reason: `pre-write.mjs exited non-zero: ${e.message}`,
        };
        finalExitCode = typeof e.status === 'number' && e.status !== 0 ? e.status : 1;
      }
    }
  }
} else if (values['post-write']) {
  if (!values['pre-write-advisory']) {
    process.stderr.write(`[audit-repo] --post-write requested but skipped: --pre-write-advisory <file> missing\n`);
    postWriteBlock = {
      requested: true,
      ran: false,
      reason: '--pre-write-advisory missing',
    };
    finalExitCode = 2;
  } else {
    const { execFileSync: _exec } = await import('node:child_process');
    const postWritePath = path.join(__dirname, 'post-write.mjs');
    const advisoryPath = path.resolve(values['pre-write-advisory']);
    const deltaOutDir = values['delta-out'] ? path.resolve(values['delta-out']) : OUT;
    const forwardedArgs = [
      postWritePath,
      '--root', ROOT,
      '--output', OUT,
      '--pre-write-advisory', advisoryPath,
    ];
    if (values['delta-out']) forwardedArgs.push('--delta-out', deltaOutDir);
    if (values['no-fresh-audit']) forwardedArgs.push('--no-fresh-audit');
    forwardedArgs.push(...forwardedScanArgs());
    forwardedArgs.push(...forwardedIncrementalArgs());

    try {
      _exec(process.execPath, forwardedArgs, { stdio: ['ignore', 'inherit', 'inherit'] });
      const deltaPath = path.join(deltaOutDir, 'post-write-delta.latest.json');
      postWriteBlock = { requested: true, ran: true, deltaPath };
      // Re-read the delta artifact to surface summary fields in the manifest.
      // Honest signal: if the delta fails to parse, summary fields stay absent
      // rather than defaulting to a "clean" value.
      try {
        const delta = JSON.parse(readFileSync(deltaPath, 'utf8'));
        postWriteBlock.silentNew = delta.summary?.silentNew ?? 0;
        postWriteBlock.requiredAcknowledgementCount =
          (delta.entries ?? []).filter((e) => e.label === 'silent-new').length;
        postWriteBlock.baselineStatus = delta.baseline?.status ?? 'missing';
        postWriteBlock.scanRangeParity = delta.scanRangeParity?.status ?? 'baseline-missing';
        postWriteBlock.typeEscapeDeltaStatus = delta.typeEscapeDelta?.status ?? 'computed';
        postWriteBlock.afterComplete = delta.inventoryCompleteness?.afterComplete ??
          (postWriteBlock.typeEscapeDeltaStatus === 'not-applicable' ? null : false);
        postWriteBlock.fileDeltaStatus = delta.fileDelta?.status ?? 'missing';
        postWriteBlock.unexpectedNewFileCount = delta.fileDelta?.summary?.unexpectedNew ?? 0;
        postWriteBlock.plannedMissingFileCount = delta.fileDelta?.summary?.plannedMissing ?? 0;
      } catch { /* delta unreadable — leave summary fields absent */ }
    } catch (e) {
      postWriteBlock = {
        requested: true,
        ran: false,
        reason: `post-write.mjs exited non-zero: ${e.message}`,
      };
    }
  }
}

manifest.preWrite = preWriteBlock;
manifest.postWrite = postWriteBlock;

// ─── P3-4-b: opt-in canon-draft orchestrator ─────────────
//
// Thin spawn wrapper. Each source runs a separate `generate-canon-draft.mjs`
// invocation; per-source outcomes populate `manifest.canonDraft.perSource`.
// Orthogonal with --pre-write / --post-write — all three can coexist on
// one invocation.
//
// Exit contract (advisory):
//   - `manifest.canonDraft.ran === true` iff ≥ 1 requested source succeeded.
//   - Orchestrator exit 0 if ran; exit 1 if every requested source failed
//     OR if --sources contained an unknown value.

if (values['canon-draft']) {
  const result = runCanonDraftLifecycle({
    sourcesValue: SOURCES_VALUE,
    root: ROOT,
    outDir: OUT,
    canonOutput: values['canon-output'],
    scriptsDir: __dirname,
    scanArgs: forwardedScanArgs(),
  });
  manifest.canonDraft = result.block;
  if (result.forceExitCode || finalExitCode === 0) finalExitCode = result.exitCode;
}

// ─── P5-4: check-canon orchestrator ──────────────────────────────
//
// Thin spawn wrapper mirroring the --canon-draft block. When every source is
// requested, spawn `check-canon.mjs --source all` once and copy its perSource
// entries into manifest.checkCanon. For subsets, spawn one child per source.
// Child exit 1 (drift) and exit 2 (attempted-but-failed-to-check, e.g.
// missing canon) are LEGITIMATE per-source outcomes recorded into manifest —
// NOT spawn failures. Only a true ENOENT-style failure produces ran=false on
// that source.
//
// Advisory default: orchestrator exit 0 if manifest.checkCanon.ran === true.
// --strict-check-canon escalates:
//   summary.driftCount > 0 → orchestrator exit 1
//   summary.sourcesChecked === 0 → orchestrator exit 2

if (values['check-canon']) {
  const result = runCheckCanonLifecycle({
    sourcesValue: SOURCES_VALUE,
    strict: !!values['strict-check-canon'],
    root: ROOT,
    outDir: OUT,
    scriptsDir: __dirname,
    scanArgs: forwardedScanArgs(),
  });
  manifest.checkCanon = result.block;
  if (finalExitCode === 0) finalExitCode = result.exitCode;
}

manifest.lifecycle = buildLifecycleSummary({
  preWrite: manifest.preWrite ?? null,
  postWrite: manifest.postWrite ?? null,
  canonDraft: manifest.canonDraft ?? null,
  checkCanon: manifest.checkCanon ?? null,
});

// Strict post-write: if --strict-post-write is set AND the post-write step
// was requested but did not run (spawn failure), escalate to exit 2. The
// mutual-exclusion and missing-advisory branches above already set
// finalExitCode=2, so this strictly targets the spawn-failure case (which
// defaults to exit 0 under advisory semantics).
if (values['strict-post-write'] && postWriteBlock?.ran === false && finalExitCode === 0) {
  process.stderr.write(`[audit-repo] --strict-post-write: post-write did not run; escalating to exit 2\n`);
  finalExitCode = 2;
}

function postWriteConfidenceLimited(block) {
  if (!block?.ran) return false;
  if (block.typeEscapeDeltaStatus === 'not-applicable') {
    return block.fileDeltaStatus !== 'computed';
  }
  return block.baselineStatus !== 'available' ||
    block.scanRangeParity !== 'ok' ||
    block.afterComplete !== true;
}

if (values['strict-post-write-confidence'] && postWriteConfidenceLimited(postWriteBlock) && finalExitCode === 0) {
  process.stderr.write(
    `[audit-repo] --strict-post-write-confidence: post-write delta confidence limited ` +
    `(baseline=${postWriteBlock.baselineStatus ?? 'unknown'}, ` +
    `scanRange=${postWriteBlock.scanRangeParity ?? 'unknown'}, ` +
    `typeEscapeDelta=${postWriteBlock.typeEscapeDeltaStatus ?? 'unknown'}, ` +
    `afterComplete=${postWriteBlock.afterComplete === true}); escalating to exit 2\n`
  );
  finalExitCode = 2;
}

refreshManifestEvidence(manifest, manifestEvidenceOptions());
manifest.rustAnalysis = mergeRustAnalysisRun({
  evidence: manifest.rustAnalysis,
  run: rustAnalysisRun,
});
const topologyArtifact = loadIfExists('topology.json');
const moduleReachabilityArtifact = loadIfExists('module-reachability.json');
if (topologyArtifact) {
  const topologyMermaidPath = path.join(OUT, 'topology.mermaid.md');
  atomicWrite(topologyMermaidPath, renderTopologyMermaid(topologyArtifact));
  manifest.topologyMermaid = {
    path: topologyMermaidPath,
    format: 'markdown',
    source: 'topology.json',
    use: 'human visual companion; topology.json remains authoritative for exact citations',
  };
  manifest.artifactsProduced = collectProducedArtifacts(OUT, {
    rustAnalysis: manifest.rustAnalysis,
  });
}
const SHOULD_WRITE_SUMMARY = (
  RUN_BASE_PIPELINE ||
  preWriteBlock?.requested ||
  postWriteBlock?.requested ||
  manifest.canonDraft?.requested ||
  manifest.checkCanon?.requested
);
if (SHOULD_WRITE_SUMMARY) {
  const auditSummaryPath = path.join(OUT, 'audit-summary.latest.md');
  const summaryMarkdown = renderAuditSummary({
    manifest,
    checklistFacts: loadIfExists('checklist-facts.json'),
    fixPlan: loadIfExists('fix-plan.json'),
    topology: topologyArtifact,
    discipline: loadIfExists('discipline.json'),
    callGraph: loadIfExists('call-graph.json'),
    functionClones: loadIfExists('function-clones.json'),
    symbols: loadIfExists('symbols.json'),
    moduleReachability: moduleReachabilityArtifact,
  });
  writeFileSync(auditSummaryPath, summaryMarkdown);
  auditSummaryPreview = renderSummaryConsolePreview(summaryMarkdown);
  manifest.auditSummary = {
    path: auditSummaryPath,
    format: 'markdown',
  };
}
if (RUN_BASE_PIPELINE && PROFILE !== 'quick') {
  const reviewPackPath = path.join(OUT, 'audit-review-pack.latest.md');
  const reviewPackMarkdown = renderAuditReviewPack({
    manifest,
    checklistFacts: loadIfExists('checklist-facts.json'),
    fixPlan: loadIfExists('fix-plan.json'),
    topology: loadIfExists('topology.json'),
    discipline: loadIfExists('discipline.json'),
    callGraph: loadIfExists('call-graph.json'),
    functionClones: loadIfExists('function-clones.json'),
    barrels: loadIfExists('barrels.json'),
    shapeIndex: loadIfExists('shape-index.json'),
    deadClassify: loadIfExists('dead-classify.json'),
    symbols: loadIfExists('symbols.json'),
    moduleReachability: moduleReachabilityArtifact,
  });
  writeFileSync(reviewPackPath, reviewPackMarkdown);
  manifest.reviewPack = {
    path: reviewPackPath,
    format: 'markdown',
    use: 'main assistant reads lanes as artifact briefs; if using built-in reviewer subagents, translate lanes into focused codebase-reading tasks with file:line evidence; the engine never calls external APIs',
  };
}
const producerPerformance = buildProducerPerformanceArtifact(
  manifest.meta.generated,
  collectProducedArtifacts(OUT, { rustAnalysis: manifest.rustAnalysis })
);
const producerPerformancePath = path.join(OUT, 'producer-performance.json');
atomicWrite(
  producerPerformancePath,
  JSON.stringify(producerPerformance, null, 2)
);
Object.assign(manifest, buildManifestFinalSummaryUpdate({
  outDir: OUT,
  producerPerformancePath,
  rustAnalysis: manifest.rustAnalysis,
}));

const manifestPath = path.join(OUT, 'manifest.json');
writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));

// ─── Console report ───────────────────────────────────────
console.log('');
console.log(`[audit-repo] wrote ${manifestPath}`);
console.log(`[audit-repo] artifacts: ${manifest.artifactsProduced.length} produced`);
if (manifest.auditSummary?.path) {
  console.log(`[audit-repo] summary: ${manifest.auditSummary.path}`);
}
if (manifest.reviewPack?.path) {
  console.log(`[audit-repo] review pack: ${manifest.reviewPack.path}`);
}
if (auditSummaryPreview) {
  console.log('');
  console.log(auditSummaryPreview);
}
if ((manifest.blindZones ?? []).length > 0) {
  console.log(`[audit-repo] ${formatBlindZonesSummary(manifest.blindZones)}`);
  for (const z of manifest.blindZones) {
    console.log(`             • ${z.area} (${z.severity}) — ${z.effect.slice(0, 80)}${z.effect.length > 80 ? '…' : ''}`);
  }
} else {
  console.log('[audit-repo] blindZones: none detected');
}
console.log('');
console.log('Next: review manifest.blindZones before making absence/removal claims.');

if (finalExitCode !== 0) process.exit(finalExitCode);
