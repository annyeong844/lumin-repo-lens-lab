// Product package contract: the repo may keep maintainer/lab files, but
// build-skill emits a small deployable skill surface.

import { spawnSync } from 'node:child_process';
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CAPABILITIES = ['audit', 'pre-write', 'post-write', 'canon-draft', 'check-canon'];
const COMMANDS = ['lumin-repo-lens-lab', 'welcome', 'full', ...CAPABILITIES, 'refactor-plan'];
const KOREAN_EPISTEMIC_TOKENS = ['[확인 불가]', '확인 불가 / unknown', '확인 불가', '~인 것 같아요'];
const COMMAND_SKILL_TARGETS = {
  'lumin-repo-lens-lab': 'lumin-repo-lens-lab',
  welcome: 'lumin-repo-lens-lab',
  full: 'lumin-repo-lens-lab',
  audit: 'lumin-repo-lens-lab',
  'refactor-plan': 'lumin-repo-lens-lab',
  'pre-write': 'lumin-repo-lens-lab-write-gate',
  'post-write': 'lumin-repo-lens-lab-write-gate',
  'canon-draft': 'lumin-repo-lens-lab-canon',
  'check-canon': 'lumin-repo-lens-lab-canon',
};

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function readJson(rel) {
  return JSON.parse(readFileSync(path.join(OUT, rel), 'utf8'));
}

function listFiles(dir, suffix, acc = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) listFiles(full, suffix, acc);
    else if (entry.isFile() && entry.name.endsWith(suffix)) acc.push(full);
  }
  return acc;
}

function collectMarkdownFiles(dir) {
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...collectMarkdownFiles(full));
    else if (entry.isFile() && entry.name.endsWith('.md')) out.push(full);
  }
  return out;
}

function linkOrCopyDir(src, dest) {
  mkdirSync(path.dirname(dest), { recursive: true });
  cpSync(src, dest, { recursive: true });
}

function installTreeSitterDepsForGeneratedPackage(outDir) {
  const nodeModules = path.join(outDir, 'node_modules');
  linkOrCopyDir(
    path.join(ROOT, 'node_modules/oxc-parser'),
    path.join(nodeModules, 'oxc-parser'),
  );
  linkOrCopyDir(
    path.join(ROOT, 'node_modules/@oxc-parser'),
    path.join(nodeModules, '@oxc-parser'),
  );
  linkOrCopyDir(
    path.join(ROOT, 'node_modules/@oxc-project'),
    path.join(nodeModules, '@oxc-project'),
  );
  linkOrCopyDir(
    path.join(ROOT, 'node_modules/typescript'),
    path.join(nodeModules, 'typescript'),
  );
  linkOrCopyDir(
    path.join(ROOT, 'node_modules/web-tree-sitter'),
    path.join(nodeModules, 'web-tree-sitter'),
  );
  linkOrCopyDir(
    path.join(ROOT, 'node_modules/@vscode/tree-sitter-wasm'),
    path.join(nodeModules, '@vscode/tree-sitter-wasm'),
  );
}

const TMP = mkdtempSync(path.join(os.tmpdir(), 'skill-package-'));
const OUT = path.join(TMP, 'lumin-repo-lens-lab');
const SKILLS_ROOT = path.dirname(OUT);

try {
  const build = spawnSync(NODE, [
    path.join(ROOT, 'scripts/build-skill.mjs'),
    '--out', OUT,
  ], {
    cwd: ROOT,
    encoding: 'utf8',
  });

  assert('SP1. build-skill exits 0',
    build.status === 0,
    `${build.stdout}\n${build.stderr}`);

  const publicScripts = existsSync(path.join(OUT, 'scripts'))
    ? readdirSync(path.join(OUT, 'scripts')).filter((f) => f.endsWith('.mjs')).sort()
    : [];

  assert('SP2. generated skill exposes public script wrappers plus smoke test',
    JSON.stringify(publicScripts) === JSON.stringify([
      'audit-repo.mjs',
      'check-canon.mjs',
      'generate-canon-draft.mjs',
      'post-write.mjs',
      'pre-write.mjs',
      'smoke-test.mjs',
    ]),
    JSON.stringify(publicScripts));

  assert('SP3. generated skill includes shipping contract files',
    existsSync(path.join(OUT, 'SKILL.md')) &&
    existsSync(path.join(path.dirname(OUT), 'lumin-repo-lens-lab-codex/SKILL.md')) &&
    existsSync(path.join(path.dirname(OUT), 'lumin-repo-lens-lab-write-gate/SKILL.md')) &&
    existsSync(path.join(path.dirname(OUT), 'lumin-repo-lens-lab-canon/SKILL.md')) &&
    existsSync(path.join(OUT, 'README.md')) &&
    existsSync(path.join(OUT, 'canonical/index.md')) &&
    existsSync(path.join(OUT, 'canonical/evidence-ladder.md')) &&
    existsSync(path.join(OUT, 'canonical/oracle-registry.json')) &&
    existsSync(path.join(OUT, 'templates/README.md')) &&
    existsSync(path.join(OUT, 'templates/report-template.md')) &&
    existsSync(path.join(OUT, 'templates/refactor-plan-template.md')) &&
    existsSync(path.join(OUT, 'templates/REVIEW_CHECKLIST_SHORT.md')) &&
    existsSync(path.join(OUT, 'templates/REVIEW_CHECKLIST.md')) &&
    existsSync(path.join(OUT, 'templates/REVIEW_CHECKLIST_RUST.md')) &&
    !existsSync(path.join(OUT, 'templates/SELF_AUDIT_HANDBOOK.md')) &&
    existsSync(path.join(OUT, 'references/false-positive-index.md')) &&
    existsSync(path.join(OUT, 'references/false-positive-patterns.md')) &&
    existsSync(path.join(OUT, 'references/pre-write-intent-shape.md')) &&
    existsSync(path.join(OUT, 'references/glossary.md')) &&
    existsSync(path.join(OUT, 'references/refactor-plan-policy.md')) &&
    existsSync(path.join(OUT, 'references/lifecycle-modes.md')) &&
    existsSync(path.join(OUT, 'references/cli-options.md')) &&
    existsSync(path.join(OUT, 'references/command-routing.md')) &&
    existsSync(path.join(OUT, 'references/operational-gates.md')) &&
    existsSync(path.join(OUT, 'references/language-support.md')),
    OUT);

  const generatedRefactorTemplate = readFileSync(path.join(OUT, 'templates/refactor-plan-template.md'), 'utf8');
  const generatedRefactorPolicy = readFileSync(path.join(OUT, 'references/refactor-plan-policy.md'), 'utf8');
  assert('SP3c. generated refactor-plan policy and template split behavior from output shape',
    generatedRefactorPolicy.includes('audit evidence -> LLM-authored plan -> implementation by user/agent -> scoped quick audit -> closeout') &&
    generatedRefactorPolicy.includes('has no CLI flag, no') &&
    generatedRefactorPolicy.includes('## Tone Contract') &&
    generatedRefactorPolicy.includes('## Phase Scope Rules') &&
    generatedRefactorPolicy.includes('## Lifecycle Integration') &&
    generatedRefactorPolicy.includes('## Ripple-Aware Changes') &&
    generatedRefactorPolicy.includes('phrase criticism as actionable improvement') &&
    generatedRefactorPolicy.includes('pre-write intent for Phase 1') &&
    generatedRefactorPolicy.includes('Semantic phase grouping is allowed because it is LLM judgment over') &&
    generatedRefactorTemplate.includes('## SHORT Mode') &&
    generatedRefactorTemplate.includes('## FULL Mode') &&
    generatedRefactorTemplate.includes('Ask the coding agent:') &&
    generatedRefactorTemplate.includes('Default length target: 20 to 35 lines') &&
    generatedRefactorTemplate.includes('Only include machine-readable scope JSON when the user asks for it') &&
    !generatedRefactorTemplate.includes('imply the user should have known better'),
    `${generatedRefactorPolicy}\n${generatedRefactorTemplate}`);

  const generatedShortChecklist = readFileSync(path.join(OUT, 'templates/REVIEW_CHECKLIST_SHORT.md'), 'utf8');
  assert('SP3d. generated skill includes short chat-facing checklist',
    generatedShortChecklist.includes('Gentle Structural Review') &&
    generatedShortChecklist.includes('Worth Smoothing Next') &&
    generatedShortChecklist.includes('Keep As-Is For Now') &&
    generatedShortChecklist.includes('Why I think this:') &&
    generatedShortChecklist.includes('Ask the coding agent:') &&
    generatedShortChecklist.includes('Truth Before Warmth') &&
    generatedShortChecklist.includes('Current State') &&
    generatedShortChecklist.includes('Review lenses: C boundaries') &&
    generatedShortChecklist.includes('output density, not analysis depth') &&
    generatedShortChecklist.includes('internal checklist triage pass') &&
    generatedShortChecklist.includes('required feature-discovery tail') &&
    generatedShortChecklist.includes('full checklist walk, formal report, or') &&
    generatedShortChecklist.includes('full checklist로 펼쳐줘') &&
    generatedShortChecklist.includes('formal report로 써줘') &&
    generatedShortChecklist.includes('due-diligence handoff로 정리해줘') &&
    /Do not omit\s+it after full-profile short answers/.test(generatedShortChecklist) &&
    generatedShortChecklist.includes('Use the long `templates/REVIEW_CHECKLIST.md` only') &&
    generatedShortChecklist.includes('chat-facing structural reviews'),
    generatedShortChecklist);

  const generatedLongChecklist = readFileSync(path.join(OUT, 'templates/REVIEW_CHECKLIST.md'), 'utf8');
  const generatedRustChecklist = readFileSync(path.join(OUT, 'templates/REVIEW_CHECKLIST_RUST.md'), 'utf8');
  assert('SP3e. generated long checklist is repo-neutral and excludes maintainer-only self-audit notes',
    !generatedLongChecklist.includes('For THIS repo') &&
    !generatedLongChecklist.includes('_lib/vocab.mjs is the single source') &&
    generatedLongChecklist.includes('This checklist is repo-neutral') &&
    generatedLongChecklist.includes('intentionally does not include') &&
    !generatedLongChecklist.includes('docs/maintainer/SELF_AUDIT_HANDBOOK.md') &&
    !existsSync(path.join(OUT, 'docs/maintainer/SELF_AUDIT_HANDBOOK.md')) &&
    !existsSync(path.join(OUT, 'templates/SELF_AUDIT_HANDBOOK.md')),
    generatedLongChecklist);

  assert('SP3e2. generated Rust checklist keeps emitted evidence paths and AI adjudication honest',
    generatedRustChecklist.includes('rust-analyzer-health.latest.json.summary.syntaxReviewOpaqueSurfaces') &&
    generatedRustChecklist.includes('No checked artifact emits a JSON field named') &&
    generatedRustChecklist.includes('### Layer 3: AI review-model judgment') &&
    generatedRustChecklist.includes('must not defer a source-readable decision') &&
    !generatedRustChecklist.includes('files.<path>.astSummary.compilerOracleOpaqueSurfaces'),
    generatedRustChecklist);

  const skillText = readFileSync(path.join(OUT, 'SKILL.md'), 'utf8');
  const skillLines = skillText.split('\n').length;
  assert('SP3b. generated SKILL.md stays slim enough for progressive disclosure',
    skillLines <= 165,
    `line count = ${skillLines}`);

  const generatedSkill = skillText;
  const generatedCommandRouting = readFileSync(path.join(OUT, 'references/command-routing.md'), 'utf8');
  const generatedReviewWorkflow = readFileSync(path.join(OUT, 'references/structural-review-workflow.md'), 'utf8');
  const generatedFpStub = readFileSync(path.join(OUT, 'references/false-positive-patterns.md'), 'utf8');
  assert('SP3f. generated audit surface preserves full-baseline then quick-incremental cadence',
    generatedSkill.includes('references/structural-review-workflow.md') &&
    generatedSkill.includes('NO STRUCTURAL REVIEW WITHOUT A CHECKLIST GATE') &&
    !generatedSkill.includes('## Standard Audit Workflow') &&
    !generatedSkill.includes('## Structural Review Mode') &&
    generatedReviewWorkflow.includes('Audit cadence:') &&
    generatedReviewWorkflow.includes('First pass on a repo') &&
    generatedReviewWorkflow.includes('Short chat output is still allowed after a full run') &&
    generatedReviewWorkflow.includes('Do not delegate this closeout to a string heuristic') &&
    generatedReviewWorkflow.includes('Checklist gate and output density are separate') &&
    generatedReviewWorkflow.includes('The Core Contract makes') &&
    generatedReviewWorkflow.includes('required review step, not merely a template') &&
    generatedCommandRouting.includes('choose the profile by cadence') &&
    generatedCommandRouting.includes('--root . --output .audit --profile full') &&
    generatedCommandRouting.includes('--root . --output .audit --profile quick') &&
    generatedCommandRouting.includes('Checklist gate: the checklist is a required review step') &&
    generatedCommandRouting.includes('Short output is not permission to skip it') &&
    generatedCommandRouting.includes('triage C/D/E/A/B/F') &&
    generatedCommandRouting.includes('Do not replace this with a string') &&
    generatedCommandRouting.includes('open `templates/REVIEW_CHECKLIST.md` and walk it before drafting'),
    `${generatedSkill}\n${generatedCommandRouting}`);

  assert('SP3f3. generated package keeps the historical FP ledger out of normal context',
    existsSync(path.join(OUT, 'references/false-positive-index.md')) &&
    existsSync(path.join(OUT, 'references/false-positive-patterns.md')) &&
    !existsSync(path.join(OUT, 'docs/maintainer/false-positive-patterns-ledger.md')) &&
    generatedFpStub.length < 1200 &&
    generatedSkill.includes('The long FP case ledger is maintainer-only') &&
    generatedCommandRouting.includes('the long FP case ledger is') &&
    generatedCommandRouting.includes('do not load historical FP case'),
    `${generatedFpStub}\n${generatedSkill}\n${generatedCommandRouting}`);

  const packagedCanonText = collectMarkdownFiles(path.join(OUT, 'canonical'))
    .map((file) => readFileSync(file, 'utf8'))
    .join('\n');
  assert('SP3f4. generated canonical spine omits maintainer patch-note metadata',
    !/Last updated|docs\/history|docs\/spec|rustlike3-clone|Generated:|v\d+(?:\.\d+)? change|promoted \d{4}-\d{2}-\d{2}|Previous sessions/.test(packagedCanonText) &&
    packagedCanonText.includes('NO STRUCTURAL CLAIM WITHOUT MACHINE EVIDENCE') &&
    packagedCanonText.includes('Drift records carry three classifying fields'),
    packagedCanonText);

  assert('SP3f2. generated routing offers full expansion after short full-profile output',
    generatedCommandRouting.includes('one required feature-discovery tail') &&
    /full checklist walk, formal report, or due-diligence\s+handoff/.test(generatedCommandRouting) &&
    generatedCommandRouting.includes('This tail is not decoration') &&
    generatedCommandRouting.includes('full checklist로 펼쳐줘') &&
    generatedCommandRouting.includes('formal report로 써줘') &&
    generatedCommandRouting.includes('due-diligence handoff로 정리해줘') &&
    /Do not\s+omit it after full-profile short answers/.test(generatedCommandRouting) &&
    /Do not add this tail after\s+quick incremental checks/.test(generatedCommandRouting) &&
    generatedCommandRouting.includes('Do not treat that offer as a required next step'),
    generatedCommandRouting);

  assert('SP3g. generated routing explains first-run dependency setup without making users write install steps',
    generatedCommandRouting.includes('First audit may install parser dependencies locally once') &&
    generatedCommandRouting.includes('LUMIN_REPO_LENS_NO_AUTO_INSTALL=1') &&
    generatedCommandRouting.includes('Do not repeat this note') &&
    generatedCommandRouting.includes('Do not tell first-time users that they must write intent JSON') &&
    generatedCommandRouting.includes('npm ci --omit=dev --ignore-scripts --no-audit --fund=false') &&
    /Do not ask[\s\S]*install packages unless that\s+guard still fails/.test(generatedCommandRouting),
    generatedCommandRouting);

  assert('SP4. generated skill moves implementation under _engine',
    existsSync(path.join(OUT, '_engine/_README.md')) &&
    readFileSync(path.join(OUT, '_engine/_README.md'), 'utf8').includes('LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>') &&
    readFileSync(path.join(OUT, '_engine/_README.md'), 'utf8').includes('`lumin-audit-core` / `lumin-audit-core.exe` on `PATH`') &&
    readFileSync(path.join(OUT, '_engine/_README.md'), 'utf8').includes('LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1') &&
    readFileSync(path.join(OUT, 'README.md'), 'utf8').includes('minimal packaged Cargo source fallback') &&
    readFileSync(path.join(OUT, 'README.md'), 'utf8').includes('_engine/rust/Cargo.toml') &&
    existsSync(path.join(OUT, '_engine/lib/cli.mjs')) &&
    existsSync(path.join(OUT, '_engine/lib/audit-core.mjs')) &&
    existsSync(path.join(OUT, '_engine/lib/dependency-guard.mjs')) &&
    readFileSync(path.join(OUT, '_engine/lib/audit-core.mjs'), 'utf8').includes('process.env.LUMIN_AUDIT_CORE_BIN') &&
    readFileSync(path.join(OUT, '_engine/lib/audit-core.mjs'), 'utf8').includes('LUMIN_AUDIT_CORE_BIN_') &&
    readFileSync(path.join(OUT, '_engine/lib/audit-core.mjs'), 'utf8').includes('LUMIN_AUDIT_CORE_NO_AUTO_BUILD') &&
    readFileSync(path.join(OUT, '_engine/lib/audit-core.mjs'), 'utf8').includes('audit-core-platforms.json') &&
    readFileSync(path.join(OUT, '_engine/lib/audit-core.mjs'), 'utf8').includes('executableOnPath') &&
    readFileSync(path.join(OUT, '_engine/lib/audit-manifest.mjs'), 'utf8').includes("from './audit-core.mjs'") &&
    existsSync(path.join(
      OUT,
      '_engine/bin',
      `${process.platform}-${process.arch}`,
      process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core',
    )) &&
    existsSync(path.join(OUT, '_engine/rust/Cargo.toml')) &&
    existsSync(path.join(OUT, '_engine/rust/Cargo.lock')) &&
    existsSync(path.join(OUT, '_engine/rust/rust-common/Cargo.toml')) &&
    existsSync(path.join(OUT, '_engine/rust/rust-main/lumin-audit-core/Cargo.toml')) &&
    existsSync(path.join(OUT, '_engine/producers/audit-repo.mjs')) &&
    existsSync(path.join(OUT, '_engine/producers/build-framework-resource-surfaces.mjs')) &&
    existsSync(path.join(OUT, '_engine/producers/build-symbol-graph.mjs')) &&
    existsSync(path.join(OUT, '_engine/producers/rank-fixes.mjs')),
    OUT);

  const auditCorePlatformKey = `${process.platform}-${process.arch}`;
  const auditCorePlatformManifest = JSON.parse(
    readFileSync(path.join(OUT, '_engine/bin/audit-core-platforms.json'), 'utf8'));
  const packageJson = JSON.parse(readFileSync(path.join(OUT, 'package.json'), 'utf8'));
  assert('SP4a. generated skill records packaged audit-core source fallback',
    auditCorePlatformManifest.schemaVersion === 'lumin-audit-core-packaged-platforms.v1' &&
    auditCorePlatformManifest.packageScope === 'current-platform-binary-with-source-fallback' &&
    auditCorePlatformManifest.binaryPackageScope === auditCorePlatformKey &&
    auditCorePlatformManifest.fallback?.kind === 'packaged-source-build-env-or-path' &&
    auditCorePlatformManifest.fallback?.requiredWhenRuntimePlatformMissing === true &&
    auditCorePlatformManifest.runtimeResolution?.packageBinaryLayout === '_engine/bin/<platform>-<arch>/<executable>' &&
    auditCorePlatformManifest.runtimeResolution?.missingPlatformBinaryBehavior === 'build-packaged-source-with-cargo-or-use-env-or-path-override' &&
    auditCorePlatformManifest.runtimeResolution?.requiresCargoWhenPackagedBinaryIsMissing === true &&
    auditCorePlatformManifest.sourceFallback?.kind === 'packaged-cargo-workspace' &&
    auditCorePlatformManifest.sourceFallback?.manifest === '_engine/rust/Cargo.toml' &&
    auditCorePlatformManifest.buildPolicy?.currentPlatformBinary === 'rebuilt-before-copy' &&
    auditCorePlatformManifest.buildPolicy?.contractValidation === 'required-cli-commands-before-copy' &&
    auditCorePlatformManifest.platforms.some((platform) =>
      platform.key === auditCorePlatformKey &&
      platform.executable === (process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core')) &&
    packageJson.os === undefined &&
    packageJson.cpu === undefined &&
    packageJson.luminRepoLens?.auditCore?.packagedPlatforms?.includes(auditCorePlatformKey) &&
    packageJson.luminRepoLens?.auditCore?.platformScope === 'current-platform-binary-with-source-fallback' &&
    packageJson.luminRepoLens?.auditCore?.binaryPlatformScope === auditCorePlatformKey &&
    packageJson.luminRepoLens?.auditCore?.sourceFallback === true &&
    packageJson.luminRepoLens?.auditCore?.sourceFallbackManifest === '_engine/rust/Cargo.toml' &&
    packageJson.luminRepoLens?.auditCore?.platformOverrideEnv === 'LUMIN_AUDIT_CORE_BIN_<PLATFORM>_<ARCH>' &&
    packageJson.luminRepoLens?.auditCore?.genericOverrideEnv === 'LUMIN_AUDIT_CORE_BIN' &&
    packageJson.luminRepoLens?.auditCore?.pathFallback === true,
    JSON.stringify({ auditCorePlatformManifest, packageJson }, null, 2));

  const generatedDocs = collectMarkdownFiles(OUT);
  const staleLibRefs = generatedDocs
    .filter((file) => !file.includes(`${path.sep}_engine${path.sep}`))
    .flatMap((file) => {
      const text = readFileSync(file, 'utf8');
      return text.includes('_lib/')
        ? [path.relative(OUT, file)]
        : [];
    });
  assert('SP4b. generated package markdown normalizes maintainer _lib/ paths to _engine/lib/',
    staleLibRefs.length === 0 &&
    readFileSync(path.join(OUT, 'canonical/any-contamination.md'), 'utf8').includes('_engine/lib/extract-ts.mjs'),
    JSON.stringify(staleLibRefs.slice(0, 20)));

  const generatedLanguageSupport = readFileSync(
    path.join(OUT, 'references/language-support.md'),
    'utf8',
  );
  assert('SP4c. generated language support documents Rust blind-zone area as rs',
    generatedLanguageSupport.includes('area: "rs"') &&
    !generatedLanguageSupport.includes('area: "rust"'),
    generatedLanguageSupport);

  assert('SP5. generated skill excludes maintainer/lab surfaces',
    !existsSync(path.join(OUT, 'tests')) &&
    !existsSync(path.join(OUT, 'CHANGELOG.md')) &&
    !existsSync(path.join(OUT, 'docs/history')) &&
    !existsSync(path.join(OUT, 'docs/lab')) &&
    !existsSync(path.join(OUT, 'canonical-draft')) &&
    !existsSync(path.join(OUT, 'review-output')) &&
    !existsSync(path.join(OUT, 'p6-corpus')) &&
    !existsSync(path.join(OUT, 'test-harness')) &&
    !existsSync(path.join(OUT, 'scripts/run-tests.mjs')) &&
    !existsSync(path.join(OUT, 'scripts/update-test-doc.mjs')),
    OUT);

  const pkg = readJson('package.json');
  assert('SP6. generated package bin points at public audit wrapper',
    pkg.name === 'lumin-repo-lens-lab-skill' &&
    pkg.luminRepoLens?.distribution === 'skill' &&
    pkg.bin?.['lumin-repo-lens-lab'] === './scripts/audit-repo.mjs' &&
    !Object.hasOwn(pkg.bin ?? {}, 'lumin-audit') &&
    !Object.hasOwn(pkg.bin ?? {}, 'grounded-audit') &&
    pkg.scripts?.audit === 'node scripts/audit-repo.mjs' &&
    pkg.scripts?.['pre-write'] === 'node scripts/audit-repo.mjs --pre-write --pre-write-engine auto' &&
    pkg.scripts?.['post-write'] === 'node scripts/audit-repo.mjs --post-write' &&
    pkg.scripts?.['canon-draft'] === 'node scripts/audit-repo.mjs --canon-draft' &&
    pkg.scripts?.['check-canon'] === 'node scripts/audit-repo.mjs --check-canon' &&
    pkg.scripts?.smoke === 'node scripts/smoke-test.mjs' &&
    !pkg.scripts?.test,
    JSON.stringify(pkg, null, 2));

  assert('SP6a. generated package pins Node to parser-supported engines',
    pkg.engines?.node === '^20.19.0 || >=22.12.0',
    JSON.stringify(pkg.engines));

  assert('SP6b. generated package does not expose platform-specific oxc binding as a direct dependency',
    !Object.hasOwn(pkg.dependencies ?? {}, '@oxc-parser/binding-linux-x64-gnu') &&
    Object.hasOwn(pkg.dependencies ?? {}, 'oxc-parser') &&
    !Object.hasOwn(pkg.dependencies ?? {}, 'jsonc-parser'),
    JSON.stringify(pkg.dependencies, null, 2));

  const generatedLock = readJson('package-lock.json');
  const lockPackages = Object.keys(generatedLock.packages ?? {});
  assert('SP6c. generated package-lock excludes maintainer-only dev dependencies',
    generatedLock.packages?.['']?.engines?.node === '^20.19.0 || >=22.12.0' &&
    generatedLock.packages?.['']?.bin?.['lumin-repo-lens-lab'] === 'scripts/audit-repo.mjs' &&
    !Object.hasOwn(generatedLock.packages?.['']?.bin ?? {}, 'lumin-audit') &&
    !Object.hasOwn(generatedLock.packages?.['']?.bin ?? {}, 'grounded-audit') &&
    !lockPackages.includes('node_modules/eslint') &&
    !lockPackages.some((name) => name.startsWith('node_modules/@eslint/')),
    lockPackages.filter((name) => name.includes('eslint')).slice(0, 20).join('\n'));

  const auditProducer = readFileSync(path.join(OUT, '_engine/producers/audit-repo.mjs'), 'utf8');
  assert('SP7. generated producer imports are rewritten from ./_lib to ../lib',
    auditProducer.includes("from '../lib/blind-zones.mjs'") &&
    auditProducer.includes("from '../lib/dependency-guard.mjs'") &&
    auditProducer.includes("from '../lib/audit-manifest.mjs'") &&
    auditProducer.includes("executeCanonDraftLifecycle") &&
    auditProducer.includes("executeCheckCanonLifecycle") &&
    !auditProducer.includes("audit-check-canon.mjs") &&
    !auditProducer.includes("from './_lib/") &&
    !auditProducer.includes("import('./_lib/"),
    auditProducer.slice(0, 1200));

  const checkWrapper = spawnSync(NODE, ['--check', path.join(OUT, 'scripts/audit-repo.mjs')], {
    encoding: 'utf8',
  });
  const checkProducer = spawnSync(NODE, ['--check', path.join(OUT, '_engine/producers/audit-repo.mjs')], {
    encoding: 'utf8',
  });
  assert('SP8. generated public wrapper and rewritten producer pass node --check',
    checkWrapper.status === 0 &&
      checkProducer.status === 0,
    `${checkWrapper.stderr}\n${checkProducer.stderr}`);

  const help = spawnSync(NODE, [path.join(OUT, 'scripts/audit-repo.mjs'), '--help'], {
    cwd: OUT,
    encoding: 'utf8',
  });
  assert('SP9. generated audit wrapper reaches the engine help text',
    help.status === 0 &&
    help.stdout.includes('lumin-repo-lens-lab public CLI') &&
    help.stdout.includes('Stable capabilities:'),
    `${help.stdout}\n${help.stderr}`);

  writeFileSync(path.join(OUT, 'package.json'), `${JSON.stringify({
    ...pkg,
    name: 'forked-lumin-repo-lens-lab-skill',
  }, null, 2)}\n`);
  const missingDeps = spawnSync(NODE, [
    path.join(OUT, 'scripts/audit-repo.mjs'),
    '--root', OUT,
    '--output', path.join(TMP, 'missing-deps-audit'),
  ], {
    cwd: OUT,
    encoding: 'utf8',
    env: {
      ...process.env,
      LUMIN_REPO_LENS_NO_AUTO_INSTALL: '1',
    },
  });
  assert('SP9a. generated audit wrapper uses luminRepoLens metadata for skill-safe dependency setup',
    missingDeps.status === 2 &&
    missingDeps.stderr.includes('setup required: runtime dependencies are not ready') &&
    !missingDeps.stderr.includes('GROUNDED_AUDIT') &&
    missingDeps.stderr.includes('npm ci --omit=dev --ignore-scripts') &&
    missingDeps.stderr.includes('--no-audit --fund=false') &&
    missingDeps.stderr.includes('oxc-parser'),
    `${missingDeps.stdout}\n${missingDeps.stderr}`);

  const packagedCanonFiles = readdirSync(path.join(OUT, 'canonical'))
    .filter((name) => name.endsWith('.md'))
    .sort();
  const packagedCanonJsonFiles = readdirSync(path.join(OUT, 'canonical'))
    .filter((name) => name.endsWith('.json'))
    .sort();
  const selfAuditCanonFacts = ['helper-registry.md', 'naming.md', 'topology.md', 'type-ownership.md'];
  assert('SP9b. generated package excludes maintainer self-audit canonical fact snapshots',
    packagedCanonFiles.length === 11 &&
    selfAuditCanonFacts.every((name) => !packagedCanonFiles.includes(name)) &&
    packagedCanonFiles.includes('audit-core.md') &&
    packagedCanonFiles.includes('index.md') &&
    packagedCanonFiles.includes('invariants.md') &&
    packagedCanonFiles.includes('mode-contract.md') &&
    packagedCanonFiles.includes('pre-write-gate.md') &&
    packagedCanonFiles.includes('evidence-ladder.md') &&
    packagedCanonFiles.includes('fact-model.md') &&
    packagedCanonFiles.includes('identity-and-alias.md') &&
    packagedCanonFiles.includes('classification-gates.md') &&
    packagedCanonFiles.includes('any-contamination.md') &&
    packagedCanonFiles.includes('canon-drift.md') &&
    JSON.stringify(packagedCanonJsonFiles) === JSON.stringify(['oracle-registry.json']),
    JSON.stringify({ packagedCanonFiles, packagedCanonJsonFiles }, null, 2));

  const selfAuditOut = path.join(TMP, 'generated-self-audit');
  const selfCheck = spawnSync(NODE, [
    path.join(ROOT, 'audit-repo.mjs'),
    '--root', OUT,
    '--output', selfAuditOut,
    '--check-canon',
    '--sources', 'all',
  ], { cwd: OUT, encoding: 'utf8' });
  let selfDrift = null;
  try {
    selfDrift = JSON.parse(readFileSync(path.join(selfAuditOut, 'canon-drift.json'), 'utf8'));
  } catch { /* assertion below reports stdout/stderr */ }
  assert('SP9b2. generated package check-canon skips absent self-audit fact canon instead of shipping it',
    selfCheck.status === 0 &&
    selfDrift?.summary?.sourcesSkipped === 4 &&
    selfDrift?.summary?.driftCount === 0 &&
    Object.values(selfDrift?.perSource ?? {}).every((entry) => entry.status === 'skipped-missing-canon'),
    `${selfCheck.stdout}\n${selfCheck.stderr}\n${JSON.stringify(selfDrift?.summary ?? null)}`);

  installTreeSitterDepsForGeneratedPackage(OUT);
  const smoke = spawnSync(NODE, [path.join(OUT, 'scripts/smoke-test.mjs')], {
    cwd: OUT,
    encoding: 'utf8',
  });
  assert('SP9f. generated package includes a runnable smoke test',
    smoke.status === 0 &&
    smoke.stdout.includes('[smoke-test] ok') &&
    /manifest\.json/.test(smoke.stdout),
    `${smoke.stdout}\n${smoke.stderr}`);

  const goRepo = path.join(TMP, 'tiny-go-repo');
  const goFile = path.join(goRepo, 'src/main.go');
  mkdirSync(path.dirname(goFile), { recursive: true });
  writeFileSync(path.join(goRepo, 'go.mod'), 'module example.com/tiny\n\ngo 1.22\n');
  writeFileSync(goFile, [
    'package main',
    '',
    'const Version = "1"',
    '',
    'func main() {}',
    '',
  ].join('\n'));

  const treeSitterModuleUrl = pathToFileURL(path.join(OUT, '_engine/lib/tree-sitter-langs.mjs')).href;
  const tsCheck = spawnSync(NODE, [
    '--input-type=module',
    '-e',
    [
      `import { isTreeSitterAvailable, extractTreeSitterBatch } from ${JSON.stringify(treeSitterModuleUrl)};`,
      `const file = ${JSON.stringify(goFile)};`,
      'const available = await isTreeSitterAvailable();',
      'const batch = await extractTreeSitterBatch([file]);',
      'const rec = batch?.get(file) ?? null;',
      'console.log(JSON.stringify({ available, files: batch?.size ?? null, defs: rec?.defs?.map((d) => d.name) ?? [], error: rec?.error ?? null }));',
    ].join('\n'),
  ], { cwd: OUT, encoding: 'utf8' });
  let tsResult = null;
  try {
    tsResult = JSON.parse(tsCheck.stdout);
  } catch { /* assertion below reports stdout/stderr */ }
  assert('SP9c. generated package resolves tree-sitter WASM dependencies from package root',
    tsCheck.status === 0 && tsResult?.available === true,
    `${tsCheck.stdout}\n${tsCheck.stderr}`);

  assert('SP9d. generated package extracts Go symbols through tree-sitter',
    tsCheck.status === 0 &&
    tsResult?.files === 1 &&
    tsResult?.defs?.includes('Version') &&
    tsResult?.defs?.includes('main'),
    `${tsCheck.stdout}\n${tsCheck.stderr}`);

  const shippedHistoryRefs = listFiles(path.join(OUT, '_engine'), '.mjs')
    .flatMap((file) => {
      const text = readFileSync(file, 'utf8');
      return text.includes('docs/history/') || text.includes('docs/spec/')
        ? [path.relative(OUT, file)]
        : [];
    });
  assert('SP9e. generated engine comments do not point at maintainer-only docs/history or docs/spec paths',
    shippedHistoryRefs.length === 0,
    shippedHistoryRefs.join('\n'));

  const plugin = JSON.parse(readFileSync(path.join(ROOT, '.claude-plugin/plugin.json'), 'utf8'));
  const marketplace = JSON.parse(readFileSync(path.join(ROOT, '.claude-plugin/marketplace.json'), 'utf8'));
  assert('SP10. plugin wrapper uses default skills/commands discovery and marketplace source',
    plugin.name === 'lumin-repo-lens-lab' &&
    plugin.description.includes('repo structure lens') &&
    plugin.author?.name === 'annyeong844' &&
    plugin.repository === 'https://github.com/annyeong844/lumin-repo-lens-lab' &&
    !Object.hasOwn(plugin, 'skills') &&
    !Object.hasOwn(plugin, 'commands') &&
    plugin.version === pkg.version &&
    marketplace.name === 'annyeong844-marketplace' &&
    marketplace.owner?.name === 'annyeong844' &&
    marketplace.plugins?.[0]?.source === './',
    `${JSON.stringify(plugin, null, 2)}\n${JSON.stringify(marketplace, null, 2)}`);

  const commands = existsSync(path.join(ROOT, 'commands'))
    ? readdirSync(path.join(ROOT, 'commands')).filter((f) => f.endsWith('.md')).sort()
    : [];
  assert('SP11. plugin exposes default command, welcome, stable capability commands, and refactor-plan coaching command',
    JSON.stringify(commands) === JSON.stringify(COMMANDS.map((c) => `${c}.md`).sort()) &&
    COMMANDS.every((name) => {
      const text = readFileSync(path.join(ROOT, 'commands', `${name}.md`), 'utf8');
      const skillTarget = COMMAND_SKILL_TARGETS[name];
      const expectedMode = name === 'lumin-repo-lens-lab' ? 'default'
        : name;
      return text.includes(`\${CLAUDE_PLUGIN_ROOT}/skills/${skillTarget}/SKILL.md`) &&
        text.includes('${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/references/command-routing.md') &&
        text.includes(`Mode: \`${expectedMode}\``);
    }) &&
    readFileSync(path.join(ROOT, 'references/command-routing.md'), 'utf8').includes('do not ask which mode') &&
    readFileSync(path.join(ROOT, 'references/command-routing.md'), 'utf8').includes('Do not run a scan immediately') &&
    /short four-section\s+chat output/.test(readFileSync(path.join(ROOT, 'references/command-routing.md'), 'utf8')),
    JSON.stringify(commands));

  const mainYaml = readFileSync(path.join(OUT, 'agents/openai.yaml'), 'utf8');
  const codexYaml = readFileSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-codex', 'agents/openai.yaml'), 'utf8');
  const codexSkill = readFileSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-codex', 'SKILL.md'), 'utf8');
  const writeGateYaml = readFileSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-write-gate', 'agents/openai.yaml'), 'utf8');
  const canonYaml = readFileSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-canon', 'agents/openai.yaml'), 'utf8');
  const generatedReadme = readFileSync(path.join(OUT, 'README.md'), 'utf8');
  assert('SP12. build-skill keeps Codex discovery thin by adding metadata to shared generated skill surfaces',
    existsSync(path.join(OUT, 'agents/openai.yaml')) &&
    existsSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-codex', 'agents/openai.yaml')) &&
    existsSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-write-gate', 'agents/openai.yaml')) &&
    existsSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-canon', 'agents/openai.yaml')) &&
    !existsSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-codex', '_engine')) &&
    !existsSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-codex', 'scripts')) &&
    !existsSync(path.join(TMP, 'codex')) &&
    !existsSync(path.join(TMP, '.codex')),
    JSON.stringify(readdirSync(TMP).sort()));

  assert('SP12b. Codex metadata names the shared surfaces and thin Codex wrapper without duplicating the engine',
    mainYaml.includes('display_name: "Lumin Repo Lens"') &&
    mainYaml.includes('default_prompt: "Use $lumin-repo-lens-lab') &&
    codexYaml.includes('display_name: "Lumin Repo Lens Codex"') &&
    codexYaml.includes('default_prompt: "Use $lumin-repo-lens-lab-codex') &&
    codexSkill.includes('Codex-native wrapper') &&
    codexSkill.includes('Set `<audit-repo>` to this path resolved relative') &&
    codexSkill.includes('../lumin-repo-lens-lab/scripts/audit-repo.mjs') &&
    codexSkill.includes('node <audit-repo> --root . --output .audit --profile full') &&
    codexSkill.includes('NO STRUCTURAL REVIEW WITHOUT A CHECKLIST GATE') &&
    codexSkill.includes('required feature-discovery tail') &&
    codexSkill.includes('full checklist로') &&
    codexSkill.includes('Do not omit it after full-profile short answers') &&
    !codexSkill.includes('slash command') &&
    writeGateYaml.includes('display_name: "Lumin Repo Lens Write Gate"') &&
    writeGateYaml.includes('default_prompt: "Use $lumin-repo-lens-lab-write-gate') &&
    canonYaml.includes('display_name: "Lumin Repo Lens Canon"') &&
    canonYaml.includes('default_prompt: "Use $lumin-repo-lens-lab-canon'),
    `${mainYaml}\n${codexYaml}\n${codexSkill}\n${writeGateYaml}\n${canonYaml}`);

  const generatedEnglishPublicDocs = [
    ['README.md', generatedReadme],
    ['SKILL.md', generatedSkill],
    ['../lumin-repo-lens-lab-write-gate/SKILL.md', readFileSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-write-gate/SKILL.md'), 'utf8')],
    ['../lumin-repo-lens-lab-canon/SKILL.md', readFileSync(path.join(SKILLS_ROOT, 'lumin-repo-lens-lab-canon/SKILL.md'), 'utf8')],
    ['references/command-routing.md', generatedCommandRouting],
    ['references/glossary.md', readFileSync(path.join(OUT, 'references/glossary.md'), 'utf8')],
    ['references/refactor-plan-policy.md', generatedRefactorPolicy],
    ['references/structural-review-workflow.md', generatedReviewWorkflow],
    ['templates/report-template.md', readFileSync(path.join(OUT, 'templates/report-template.md'), 'utf8')],
    ['templates/REVIEW_CHECKLIST.md', generatedLongChecklist],
    ['templates/REVIEW_CHECKLIST_RUST.md', generatedRustChecklist],
    ['templates/REVIEW_CHECKLIST_SHORT.md', generatedShortChecklist],
  ];
  const generatedKoreanEpistemicOffenders = [];
  for (const [name, text] of generatedEnglishPublicDocs) {
    for (const token of KOREAN_EPISTEMIC_TOKENS) {
      if (text.includes(token)) generatedKoreanEpistemicOffenders.push(`${name}: ${token}`);
    }
  }
  assert('SP12b2. generated public English docs use the English `unknown` evidence label',
    generatedKoreanEpistemicOffenders.length === 0 &&
      generatedLongChecklist.includes('[unknown, scan range:') &&
      generatedShortChecklist.includes('[unknown, scan range:'),
    generatedKoreanEpistemicOffenders.join('\n'));

  assert('SP12c0. shipped README carries Claude Code marketplace install instructions before Codex link install',
    generatedReadme.includes('## First useful run') &&
    generatedReadme.includes('/plugin marketplace add annyeong844/lumin-repo-lens-lab') &&
    generatedReadme.includes('/plugin install lumin-repo-lens-lab@annyeong844-marketplace') &&
    generatedReadme.includes('/reload-plugins') &&
    generatedReadme.includes('/lumin-repo-lens-lab') &&
    generatedReadme.includes('### Codex-native install') &&
    generatedReadme.indexOf('## First useful run') >= 0 &&
    generatedReadme.indexOf('## First useful run') < generatedReadme.indexOf('### Codex-native install'),
    generatedReadme);

  assert('SP12c1. shipped README states conservative clone and shape evidence boundaries',
    generatedReadme.includes('Function-clone cues are review cues') &&
    generatedReadme.includes('not semantic-equivalence claims') &&
    generatedReadme.includes('Shape index is exact') &&
    generatedReadme.includes('nullable or widened types') &&
    generatedReadme.includes('Start from `audit-summary.latest.md`, `manifest.json`, and `checklist-facts.json`') &&
    generatedReadme.includes('open raw JSON artifacts only for the claim being cited'),
    generatedReadme);

  assert('SP12c. shipped README carries Codex link-install instructions',
    generatedReadme.includes('### Codex-native install') &&
    generatedReadme.includes('$lumin-repo-lens-lab-codex') &&
    generatedReadme.includes('ln -sfn ~/.codex/lumin-repo-lens-lab/skills/lumin-repo-lens-lab-codex') &&
    generatedReadme.includes('cmd /c mklink /J "%USERPROFILE%\\.codex\\skills\\lumin-repo-lens-lab-codex"') &&
    generatedReadme.includes('ln -sfn ~/.codex/lumin-repo-lens-lab/skills/lumin-repo-lens-lab') &&
    generatedReadme.includes('cmd /c mklink /J "%USERPROFILE%\\.codex\\skills\\lumin-repo-lens-lab"') &&
    generatedReadme.includes('skills/lumin-repo-lens-lab-write-gate') &&
    generatedReadme.includes('skills/lumin-repo-lens-lab-canon'),
    generatedReadme);

  assert('SP12d. shipped README warns that default .audit artifacts may be commit-sensitive',
    generatedReadme.includes('under `<repo>/.audit/`') &&
    generatedReadme.includes('file paths, symbol names') &&
    generatedReadme.includes('Add `.audit/` to `.gitignore`'),
    generatedReadme);
} finally {
  rmSync(TMP, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
