#!/usr/bin/env node
// Build the deployable skill surface from the maintainer repo.
//
// The source repo intentionally keeps tests, research notes, and lab
// artifacts. The generated skill package keeps only the user-facing
// contract, public wrappers, internal engine code, runtime canon,
// templates, and selected references.

import {
  chmodSync,
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

import {
  AUDIT_CORE_REQUIRED_FEATURES,
  AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION,
  auditCoreBinaryReportsCurrentContract,
  auditCoreBinarySupportsFixtureContract,
} from '../_lib/audit-core.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const DEFAULT_OUT = path.join(ROOT, 'skills', 'lumin-repo-lens-lab');
const MAX_PACKAGED_LINUX_GLIBC = { major: 2, minor: 31 };

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
oxc_allocator = "0.139.0"
oxc_ast = "0.139.0"
oxc_ast_visit = "0.139.0"
oxc_parser = "0.139.0"
oxc_span = "0.139.0"
oxc_syntax = "0.139.0"
rayon = "1"
serde = "1"
serde_json = "1"
sha2 = "0.10"
tempfile = "3"

[workspace.lints]
rust = {}

[workspace.lints.clippy]
await_holding_invalid_type = "deny"
await_holding_lock = "deny"
undocumented_unsafe_blocks = "deny"
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
    '--release',
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
  return path.join(targetDir, 'release', exe);
}

function validateRunnableAuditCoreBinary(binaryPath) {
  if (!auditCoreBinaryReportsCurrentContract(binaryPath, { cwd: ROOT })) {
    throw new Error(`built lumin-audit-core at ${binaryPath} reports a stale runtime contract`);
  }
  if (!auditCoreBinarySupportsFixtureContract(binaryPath, { cwd: ROOT })) {
    throw new Error(
      `built lumin-audit-core at ${binaryPath} does not satisfy the executable runtime contract probe`
    );
  }
}

function validatePackagedAuditCoreBinaryMetadata(source) {
  const binary = readFileSync(source.path);
  const key = auditCorePlatformKey(source.platform, source.arch);
  if (source.platform === 'linux') {
    if (binary.length < 20 || !binary.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46]))) {
      throw new Error(`configured ${key} audit-core is not an ELF binary`);
    }
    const expectedMachine = source.arch === 'x64' ? 62 : source.arch === 'arm64' ? 183 : null;
    if (expectedMachine !== null && binary.readUInt16LE(18) !== expectedMachine) {
      throw new Error(`configured ${key} audit-core has the wrong ELF architecture`);
    }
  } else if (source.platform === 'win32') {
    if (binary.length < 0x40 || binary.subarray(0, 2).toString('ascii') !== 'MZ') {
      throw new Error(`configured ${key} audit-core is not a PE binary`);
    }
    const peOffset = binary.readUInt32LE(0x3c);
    if (
      peOffset > binary.length - 6 ||
      !binary.subarray(peOffset, peOffset + 4).equals(Buffer.from([0x50, 0x45, 0, 0]))
    ) {
      throw new Error(`configured ${key} audit-core has an invalid PE header`);
    }
    const expectedMachine = {
      ia32: 0x014c,
      x64: 0x8664,
      arm64: 0xaa64,
    }[source.arch];
    if (expectedMachine === undefined) {
      throw new Error(`configured ${key} audit-core uses an unsupported Windows architecture`);
    }
    if (binary.readUInt16LE(peOffset + 4) !== expectedMachine) {
      throw new Error(`configured ${key} audit-core has the wrong PE architecture`);
    }
  }

  for (const marker of [
    AUDIT_CORE_RUNTIME_BRIDGE_CONTRACT_VERSION,
    ...AUDIT_CORE_REQUIRED_FEATURES,
  ]) {
    if (!binary.includes(Buffer.from(marker))) {
      throw new Error(`configured ${key} audit-core is missing embedded contract marker ${marker}`);
    }
  }

  if (source.platform !== 'linux') return;
  const text = binary.toString('latin1');
  let maxGlibc = null;
  for (const match of text.matchAll(/GLIBC_(\d+)\.(\d+)/g)) {
    const version = { major: Number(match[1]), minor: Number(match[2]) };
    if (
      maxGlibc === null ||
      version.major > maxGlibc.major ||
      (version.major === maxGlibc.major && version.minor > maxGlibc.minor)
    ) {
      maxGlibc = version;
    }
  }
  if (
    maxGlibc !== null &&
    (
      maxGlibc.major > MAX_PACKAGED_LINUX_GLIBC.major ||
      (
        maxGlibc.major === MAX_PACKAGED_LINUX_GLIBC.major &&
        maxGlibc.minor > MAX_PACKAGED_LINUX_GLIBC.minor
      )
    )
  ) {
    throw new Error(
      `configured ${key} audit-core requires GLIBC_${maxGlibc.major}.${maxGlibc.minor}; maximum packaged baseline is GLIBC_${MAX_PACKAGED_LINUX_GLIBC.major}.${MAX_PACKAGED_LINUX_GLIBC.minor}`
    );
  }
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

function ensurePackagedAuditCoreMode(dest) {
  chmodSync(dest, 0o755);
}

function copyAuditCoreSourceFallback(outDir) {
  const rustRoot = path.join(outDir, '_engine', 'rust');
  mkdirSync(rustRoot, { recursive: true });
  writeFileSync(path.join(rustRoot, 'Cargo.toml'), AUDIT_CORE_SOURCE_WORKSPACE.endsWith('\n')
    ? AUDIT_CORE_SOURCE_WORKSPACE
    : `${AUDIT_CORE_SOURCE_WORKSPACE}\n`);
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
    validatePackagedAuditCoreBinaryMetadata(source);
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
    ensurePackagedAuditCoreMode(dest);
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
      crossPlatformValidation: 'binary-format-architecture-contract-markers-and-linux-glibc-floor',
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
