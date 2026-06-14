#!/usr/bin/env node
// Publish the generated Claude Code plugin package to the public
// annyeong844/lumin-repo-lens-lab repository.
//
// The maintainer repo and public repo intentionally have different history.
// This script syncs only the generated plugin package surface, never the full
// maintainer checkout.

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
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const DEFAULT_REPO = 'https://github.com/annyeong844/lumin-repo-lens-lab.git';
const FORBIDDEN_STABLE_PUBLIC_REPO = 'https://github.com/annyeong844/lumin-repo-lens.git';
const DEFAULT_DIST = path.join(ROOT, 'dist', 'lumin-repo-lens-lab-plugin');
const DEFAULT_AUTHOR_NAME = 'annyeong844';
const DEFAULT_AUTHOR_EMAIL = 'annyeong844@users.noreply.github.com';
const PUBLIC_ROOT_DOCS = ['README.md', 'README.ko.md', 'LICENSE'];
const PUBLIC_WORKFLOW_SOURCE = path.join(ROOT, 'public-package/.github/workflows/ci.yml');
const PUBLIC_WORKFLOW_DEST = '.github/workflows/ci.yml';
const SYNC_DIRS = ['.claude-plugin', 'commands', 'hooks', 'skills'];
const DISALLOWED_ROOT_ENTRIES = new Set([
  '_lib',
  'audit-artifacts',
  'canonical',
  'canonical-draft',
  'docs',
  'dist',
  'output',
  'p6-corpus',
  'review-output',
  'scripts',
  'templates',
  'test-harness',
  'tests',
]);

function parseArgs(argv) {
  const out = {
    repo: DEFAULT_REPO,
    dist: DEFAULT_DIST,
    checkoutDir: null,
    push: false,
    build: true,
    keepCheckout: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--repo') {
      out.repo = argv[++i];
    } else if (arg === '--dist') {
      out.dist = path.resolve(argv[++i]);
    } else if (arg === '--checkout-dir') {
      out.checkoutDir = path.resolve(argv[++i]);
    } else if (arg === '--push') {
      out.push = true;
    } else if (arg === '--dry-run') {
      out.push = false;
    } else if (arg === '--no-build') {
      out.build = false;
    } else if (arg === '--keep-checkout') {
      out.keepCheckout = true;
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
    'usage: node scripts/publish-public-plugin.mjs [options]',
    '',
    'Options:',
    `  --repo <url-or-path>       public repo remote (default: ${DEFAULT_REPO})`,
    `  --dist <dir>               generated plugin package root (default: ${path.relative(ROOT, DEFAULT_DIST)})`,
    '  --checkout-dir <dir>       temp checkout path (must be under OS temp if it already exists)',
    '  --dry-run                  sync and validate, but do not commit or push (default)',
    '  --push                     commit and push public main',
    '  --no-build                 skip npm run build:plugin (test/advanced use)',
    '  --keep-checkout            leave the temp checkout on disk',
  ].join('\n');
}

function runGit(args, cwd, { allowFailure = false } = {}) {
  const result = spawnSync('git', args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (!allowFailure && result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed in ${cwd}\n${result.stdout}\n${result.stderr}`);
  }
  return result;
}

function normalizeRepoTarget(value) {
  return String(value ?? '')
    .trim()
    .replace(/^git@github\.com:/u, 'https://github.com/')
    .replace(/\.git$/u, '')
    .replace(/\/+$/u, '')
    .toLowerCase();
}

function assertAllowedPublishRepo(repo) {
  if (normalizeRepoTarget(repo) === normalizeRepoTarget(FORBIDDEN_STABLE_PUBLIC_REPO)) {
    throw new Error(
      `refusing to publish lab package to stable public repo: ${FORBIDDEN_STABLE_PUBLIC_REPO}`
    );
  }
}

function runNode(args, cwd) {
  const result = spawnSync(process.execPath, args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  if (result.status !== 0) {
    throw new Error(`node ${args.join(' ')} failed\n${result.stdout}\n${result.stderr}`);
  }
  return result;
}

function assertTempPath(dir) {
  const resolved = path.resolve(dir);
  const temp = path.resolve(os.tmpdir());
  if (!resolved.toLowerCase().startsWith(temp.toLowerCase() + path.sep)) {
    throw new Error(`refusing to clean checkout outside OS temp: ${resolved}`);
  }
}

function prepareCheckoutDir(requested) {
  const checkoutDir = requested ?? mkdtempSync(path.join(os.tmpdir(), 'lumin-repo-lens-lab-public-'));
  const resolved = path.resolve(checkoutDir);
  if (existsSync(resolved)) {
    assertTempPath(resolved);
    rmSync(resolved, { recursive: true, force: true });
  } else {
    mkdirSync(path.dirname(resolved), { recursive: true });
  }
  return resolved;
}

function readJson(file) {
  return JSON.parse(readFileSync(file, 'utf8'));
}

function readOptionalJson(file) {
  try {
    return readJson(file);
  } catch (error) {
    if (error?.code === 'ENOENT') return null;
    throw error;
  }
}

function copyPath(src, dest) {
  if (!existsSync(src)) throw new Error(`missing source path: ${src}`);
  rmSync(dest, { recursive: true, force: true });
  mkdirSync(path.dirname(dest), { recursive: true });
  cpSync(src, dest, { recursive: true });
}

function copyFileIfExists(src, dest) {
  if (!existsSync(src)) return;
  mkdirSync(path.dirname(dest), { recursive: true });
  cpSync(src, dest, { force: true });
}

function parseChangelogEntries(text) {
  const lines = text.split(/\r?\n/);
  const headers = [];
  for (let i = 0; i < lines.length; i++) {
    const match = lines[i].match(/^##\s+(\S+)/);
    if (match) headers.push({ line: i, version: match[1] });
  }
  return { lines, headers };
}

function buildPublicChangelog({ internalPath, publicPath }) {
  const internal = parseChangelogEntries(readFileSync(internalPath, 'utf8'));
  const publicText = existsSync(publicPath) ? readFileSync(publicPath, 'utf8') : null;
  if (internal.headers.length === 0) {
    throw new Error('internal CHANGELOG.md has no version entries');
  }
  if (publicText === null) {
    const first = internal.headers[0];
    const next = internal.headers[1]?.line ?? internal.lines.length;
    const newEntryLines = internal.lines.slice(first.line, next);
    return `${['# Changelog', '', ...newEntryLines].join('\n').replace(/\n+$/u, '')}\n`;
  }

  const pub = parseChangelogEntries(publicText);
  if (pub.headers.length === 0) {
    const first = internal.headers[0];
    const next = internal.headers[1]?.line ?? internal.lines.length;
    const newEntryLines = internal.lines.slice(first.line, next);
    return `${['# Changelog', '', ...newEntryLines].join('\n').replace(/\n+$/u, '')}\n`;
  }

  const publicTop = pub.headers[0].version;
  const matchingInternal = internal.headers.find((entry) => entry.version === publicTop);
  if (!matchingInternal) {
    throw new Error(`public top changelog version ${publicTop} was not found in internal CHANGELOG.md`);
  }

  const newEntryLines = internal.lines.slice(2, matchingInternal.line);
  const existingPublicLines = pub.lines.slice(pub.headers[0].line);
  const merged = ['# Changelog', '', ...newEntryLines];
  if (newEntryLines.length > 0 && newEntryLines.at(-1) !== '') merged.push('');
  merged.push(...existingPublicLines);
  return `${merged.join('\n').replace(/\n+$/u, '')}\n`;
}

function validatePackageSurface(checkoutDir, expectedVersion) {
  const plugin = readJson(path.join(checkoutDir, '.claude-plugin/plugin.json'));
  const marketplace = readJson(path.join(checkoutDir, '.claude-plugin/marketplace.json'));
  const skillPkg = readJson(path.join(checkoutDir, 'skills/lumin-repo-lens-lab/package.json'));
  const skillLock = readOptionalJson(path.join(checkoutDir, 'skills/lumin-repo-lens-lab/package-lock.json'));

  const versions = [
    ['plugin.json', plugin.version],
    ['marketplace.json metadata.version', marketplace.metadata?.version],
    ['skills/lumin-repo-lens-lab/package.json', skillPkg.version],
  ];
  if (skillLock) {
    versions.push(['skills/lumin-repo-lens-lab/package-lock.json', skillLock.version]);
    versions.push(['skills/lumin-repo-lens-lab/package-lock.json packages[""].version', skillLock.packages?.['']?.version]);
  }
  for (const [label, actual] of versions) {
    if (actual !== expectedVersion) {
      throw new Error(`${label} version ${JSON.stringify(actual)} does not match ${expectedVersion}`);
    }
  }

  const sarifProducer = path.join(checkoutDir, 'skills/lumin-repo-lens-lab/_engine/producers/emit-sarif.mjs');
  if (existsSync(sarifProducer)) {
    const text = readFileSync(sarifProducer, 'utf8');
    const match = text.match(/TOOL_VERSION\s*=\s*['"]([^'"]+)['"]/);
    if (match?.[1] !== expectedVersion) {
      throw new Error(`packaged emit-sarif TOOL_VERSION ${JSON.stringify(match?.[1])} does not match ${expectedVersion}`);
    }
  }

  if (existsSync(path.join(checkoutDir, 'skills/lumin-repo-lens-lab-codex'))) {
    throw new Error('public plugin package must not include skills/lumin-repo-lens-lab-codex');
  }
  for (const rel of [
    '.claude-plugin/plugin.json',
    '.claude-plugin/marketplace.json',
    'commands/lumin-repo-lens-lab.md',
    'hooks/hooks.json',
    'hooks/_runner-utils.mjs',
    'hooks/pre-tool-use.mjs',
    'hooks/post-tool-batch.mjs',
    'hooks/stop.mjs',
    'hooks/user-prompt-submit.mjs',
    'skills/lumin-repo-lens-lab/SKILL.md',
    'skills/lumin-repo-lens-lab-write-gate/SKILL.md',
    'skills/lumin-repo-lens-lab-canon/SKILL.md',
    'README.plugin-package.md',
    PUBLIC_WORKFLOW_DEST,
  ]) {
    if (!existsSync(path.join(checkoutDir, rel))) {
      throw new Error(`public plugin package missing required file: ${rel}`);
    }
  }

  for (const entry of readdirSync(checkoutDir, { withFileTypes: true })) {
    if (entry.name === '.git') continue;
    if (DISALLOWED_ROOT_ENTRIES.has(entry.name)) {
      throw new Error(`public plugin package contains maintainer-only root entry: ${entry.name}`);
    }
  }
}

function hasActualGitChanges(checkoutDir) {
  const diff = runGit(['diff', '--quiet', '--exit-code'], checkoutDir, { allowFailure: true });
  if (diff.status === 1) return true;
  if (diff.status !== 0) {
    throw new Error(`git diff --quiet failed in ${checkoutDir}\n${diff.stdout}\n${diff.stderr}`);
  }
  const untracked = runGit(['ls-files', '--others', '--exclude-standard'], checkoutDir).stdout.trim();
  return untracked.length > 0;
}

function syncPublicCheckout({ checkoutDir, distDir }) {
  const sourcePkg = readJson(path.join(ROOT, 'package.json'));
  const expectedVersion = sourcePkg.version;
  const distPlugin = readJson(path.join(distDir, '.claude-plugin/plugin.json'));
  if (distPlugin.version !== expectedVersion) {
    throw new Error(
      `dist plugin version ${distPlugin.version} does not match package.json ${expectedVersion}; run npm run build:plugin`
    );
  }

  for (const rel of SYNC_DIRS) {
    copyPath(path.join(distDir, rel), path.join(checkoutDir, rel));
  }
  copyPath(
    path.join(distDir, 'README.plugin-package.md'),
    path.join(checkoutDir, 'README.plugin-package.md'),
  );
  for (const rel of PUBLIC_ROOT_DOCS) {
    copyFileIfExists(path.join(ROOT, rel), path.join(checkoutDir, rel));
  }
  copyFileIfExists(PUBLIC_WORKFLOW_SOURCE, path.join(checkoutDir, PUBLIC_WORKFLOW_DEST));
  writeFileSync(
    path.join(checkoutDir, 'CHANGELOG.md'),
    buildPublicChangelog({
      internalPath: path.join(ROOT, 'CHANGELOG.md'),
      publicPath: path.join(checkoutDir, 'CHANGELOG.md'),
    }),
  );

  validatePackageSurface(checkoutDir, expectedVersion);
  return expectedVersion;
}

function configureGitAuthor(checkoutDir) {
  const configuredName = runGit(['config', '--get', 'user.name'], ROOT, { allowFailure: true }).stdout.trim();
  const configuredEmail = runGit(['config', '--get', 'user.email'], ROOT, { allowFailure: true }).stdout.trim();
  const githubActor = process.env.GITHUB_ACTOR?.trim();
  const name =
    process.env.LUMIN_REPO_LENS_PUBLISH_AUTHOR_NAME?.trim() ||
    process.env.GIT_AUTHOR_NAME?.trim() ||
    configuredName ||
    githubActor ||
    DEFAULT_AUTHOR_NAME;
  const email =
    process.env.LUMIN_REPO_LENS_PUBLISH_AUTHOR_EMAIL?.trim() ||
    process.env.GIT_AUTHOR_EMAIL?.trim() ||
    configuredEmail ||
    (githubActor ? `${githubActor}@users.noreply.github.com` : DEFAULT_AUTHOR_EMAIL);
  runGit(['config', 'user.name', name], checkoutDir);
  runGit(['config', 'user.email', email], checkoutDir);
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(usage());
    return;
  }
  assertAllowedPublishRepo(args.repo);

  const checkoutDir = prepareCheckoutDir(args.checkoutDir);
  const removeCheckout = !args.keepCheckout;
  try {
    if (args.build) {
      runNode([path.join(ROOT, 'scripts/build-plugin-package.mjs')], ROOT);
    }
    if (!existsSync(args.dist)) {
      throw new Error(`plugin dist not found: ${args.dist}`);
    }

    runGit(['clone', args.repo, checkoutDir], ROOT);
    runGit(['checkout', 'main'], checkoutDir);
    const version = syncPublicCheckout({ checkoutDir, distDir: args.dist });
    if (!hasActualGitChanges(checkoutDir)) {
      console.log(`[publish-public-plugin] public package already up to date at ${version}`);
      return;
    }

    console.log(`[publish-public-plugin] prepared public package ${version}`);
    console.log(runGit(['diff', '--stat'], checkoutDir).stdout.trim());

    if (!args.push) {
      console.log('[publish-public-plugin] dry-run only; rerun with --push to commit and publish');
      return;
    }

    configureGitAuthor(checkoutDir);
    runGit(['add', '-A'], checkoutDir);
    runGit(['commit', '-m', `Publish lumin repo lens ${version} package`], checkoutDir);
    runGit(['push', 'origin', 'HEAD:main'], checkoutDir);
    const sha = runGit(['rev-parse', 'HEAD'], checkoutDir).stdout.trim();
    console.log(`[publish-public-plugin] pushed public package ${version} at ${sha}`);
  } finally {
    if (removeCheckout && existsSync(checkoutDir)) {
      assertTempPath(checkoutDir);
      rmSync(checkoutDir, { recursive: true, force: true });
    }
  }
}

try {
  main();
} catch (error) {
  console.error(`[publish-public-plugin] ${error.message}`);
  process.exit(1);
}
