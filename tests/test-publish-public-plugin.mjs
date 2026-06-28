// Tests for scripts/publish-public-plugin.mjs.
//
// The public marketplace repo is a generated plugin package, not the
// maintainer checkout. These tests use local git repositories so the publish
// workflow is covered without touching GitHub.

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const SCRIPT = path.join(ROOT, 'scripts/publish-public-plugin.mjs');
const PACKAGE = JSON.parse(readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
const CURRENT_VERSION = PACKAGE.version;

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function run(cmd, args, cwd = ROOT, options = {}) {
  return spawnSync(cmd, args, {
    cwd,
    env: { ...process.env, ...(options.env ?? {}) },
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function git(args, cwd) {
  const result = run('git', args, cwd);
  if (result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed\n${result.stdout}\n${result.stderr}`);
  }
  return result.stdout.trim();
}

function writeJson(file, value) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function writeText(file, text) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, text);
}

const commandNames = [
  'audit',
  'canon-draft',
  'check-canon',
  'full',
  'lumin-repo-lens-lab',
  'post-write',
  'pre-write',
  'refactor-plan',
  'welcome',
];

function createDistPlugin(root, version = CURRENT_VERSION) {
  writeJson(path.join(root, '.claude-plugin/plugin.json'), {
    name: 'lumin-repo-lens-lab',
    version,
    description: 'Lumin Repo Lens: evidence-backed TS/JS repo structure lens for Claude Code.',
    repository: 'https://github.com/annyeong844/lumin-repo-lens-lab',
    license: 'MIT',
  });
  writeJson(path.join(root, '.claude-plugin/marketplace.json'), {
    name: 'annyeong844-marketplace',
    owner: { name: 'annyeong844' },
    metadata: {
      description: 'Public beta marketplace for annyeong844 Claude Code plugins.',
      version,
    },
    plugins: [
      {
        name: 'lumin-repo-lens-lab',
        source: './',
        description: 'Lumin Repo Lens: evidence-backed TS/JS repo structure lens for Claude Code.',
      },
    ],
  });

  for (const commandName of commandNames) {
    writeText(
      path.join(root, 'commands', `${commandName}.md`),
      `---\ndescription: ${commandName}\n---\n\nUse \${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/SKILL.md\n`,
    );
  }

  for (const skill of ['lumin-repo-lens-lab', 'lumin-repo-lens-lab-write-gate', 'lumin-repo-lens-lab-canon']) {
    writeText(path.join(root, 'skills', skill, 'SKILL.md'), `---\nname: ${skill}\n---\n`);
  }
  writeJson(path.join(root, 'hooks/hooks.json'), {
    hooks: {
      PreToolUse: [
        {
          matcher: '*',
          hooks: [
            {
              type: 'command',
              command: 'node "${CLAUDE_PLUGIN_ROOT}/hooks/pre-tool-use.mjs"',
              timeout: 2,
            },
          ],
        },
      ],
    },
  });
  writeText(path.join(root, 'hooks/_runner-utils.mjs'), 'export const runnerUtils = true;\n');
  writeText(path.join(root, 'hooks/pre-tool-use.mjs'), '#!/usr/bin/env node\n');
  writeText(path.join(root, 'hooks/post-tool-batch.mjs'), '#!/usr/bin/env node\n');
  writeText(path.join(root, 'hooks/stop.mjs'), '#!/usr/bin/env node\n');
  writeText(path.join(root, 'hooks/user-prompt-submit.mjs'), '#!/usr/bin/env node\n');
  writeJson(path.join(root, 'skills/lumin-repo-lens-lab/package.json'), {
    name: 'lumin-repo-lens-lab-skill',
    version,
    luminRepoLens: { distribution: 'skill' },
  });
  writeJson(path.join(root, 'skills/lumin-repo-lens-lab/package-lock.json'), {
    name: 'lumin-repo-lens-lab-skill',
    version,
    lockfileVersion: 3,
    packages: { '': { name: 'lumin-repo-lens-lab-skill', version } },
  });
  writeText(
    path.join(root, 'skills/lumin-repo-lens-lab/_engine/producers/emit-sarif.mjs'),
    `const TOOL_VERSION = '${version}';\n`,
  );
  writeText(path.join(root, 'README.plugin-package.md'), '# Package root\n');
}

function seedPublicRepo(workDir, bareDir) {
  mkdirSync(workDir, { recursive: true });
  git(['init', '-b', 'main'], workDir);
  git(['config', 'user.name', 'annyeong844'], workDir);
  git(['config', 'user.email', 'annyeong844@users.noreply.github.com'], workDir);

  writeJson(path.join(workDir, '.claude-plugin/plugin.json'), {
    name: 'lumin-repo-lens-lab',
    version: '0.9.0-beta.6',
  });
  writeJson(path.join(workDir, '.claude-plugin/marketplace.json'), {
    name: 'annyeong844-marketplace',
    metadata: { version: '0.9.0-beta.6' },
  });
  writeJson(path.join(workDir, 'skills/lumin-repo-lens-lab/package.json'), {
    name: 'lumin-repo-lens-lab-skill',
    version: '0.9.0-beta.6',
  });
  writeText(path.join(workDir, 'CHANGELOG.md'), [
    '# Changelog',
    '',
    '## 0.9.0-beta.6',
    '',
    '- Existing public beta6 entry.',
    '',
  ].join('\n'));
  writeText(path.join(workDir, 'README.md'), '# Old public README\n');
  writeText(path.join(workDir, 'README.ko.md'), '# 오래된 공개 README\n');
  writeText(path.join(workDir, 'LICENSE'), 'MIT\n');
  writeText(path.join(workDir, '.gitignore'), 'node_modules/\n.audit/\n');
  git(['add', '-A'], workDir);
  git(['commit', '-m', 'seed public package'], workDir);
  git(['clone', '--bare', workDir, bareDir], path.dirname(bareDir));
}

const tmp = mkdtempSync(path.join(os.tmpdir(), 'publish-public-plugin-test-'));
try {
  const dist = path.join(tmp, 'dist-plugin');
  const seed = path.join(tmp, 'seed-public');
  const bare = path.join(tmp, 'public.git');
  const checkout = path.join(tmp, 'checkout');
  createDistPlugin(dist);
  seedPublicRepo(seed, bare);

  const dry = run(NODE, [
    SCRIPT,
    '--repo', bare,
    '--dist', dist,
    '--checkout-dir', checkout,
    '--no-build',
    '--dry-run',
    '--keep-checkout',
  ]);
  assert('PPUB1. dry-run exits 0',
    dry.status === 0,
    `${dry.stdout}\n${dry.stderr}`);
  assert('PPUB2. dry-run stages plugin package version without committing',
    JSON.parse(readFileSync(path.join(checkout, '.claude-plugin/plugin.json'), 'utf8')).version === CURRENT_VERSION &&
    JSON.parse(readFileSync(path.join(checkout, 'skills/lumin-repo-lens-lab/package.json'), 'utf8')).version === CURRENT_VERSION &&
    git(['rev-parse', 'HEAD'], checkout) === git(['rev-parse', 'main'], seed),
    dry.stdout);
  assert('PPUB3. dry-run does not leak maintainer-only root directories',
    !existsSync(path.join(checkout, 'docs')) &&
    !existsSync(path.join(checkout, 'tests')) &&
    !existsSync(path.join(checkout, '_lib')) &&
    !existsSync(path.join(checkout, 'skills/lumin-repo-lens-lab-codex')),
    'maintainer-only path exists in public checkout');
  const dryChangelog = readFileSync(path.join(checkout, 'CHANGELOG.md'), 'utf8');
  assert('PPUB4. dry-run prepends internal beta entries before existing public beta6 entry',
    dryChangelog.indexOf(`## ${CURRENT_VERSION}`) < dryChangelog.indexOf('## 0.9.0-beta.6') &&
    dryChangelog.includes('## 0.9.0-beta.10') &&
    dryChangelog.includes('- Existing public beta6 entry.'),
    dryChangelog.slice(0, 800));
  const workflowPath = path.join(checkout, '.github/workflows/ci.yml');
  const workflowText = existsSync(workflowPath) ? readFileSync(workflowPath, 'utf8') : '';
  assert('PPUB4b. dry-run syncs public package CI workflow',
    existsSync(workflowPath) &&
      workflowText.includes('npm ci') &&
      workflowText.includes('npm run smoke') &&
      workflowText.includes('node skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --help') &&
      workflowText.includes('node hooks/pre-tool-use.mjs'),
    workflowText || 'missing .github/workflows/ci.yml');
  assert('PPUB4c. public package CI workflow does not reference maintainer-only paths',
    !/\b(tests|test-harness|docs\/spec|_lib|p6-corpus)\b/.test(workflowText),
    workflowText);
  assert('PPUB4d. dry-run syncs auto-hook manifest and runner scripts',
    existsSync(path.join(checkout, 'hooks/hooks.json')) &&
      existsSync(path.join(checkout, 'hooks/_runner-utils.mjs')) &&
      existsSync(path.join(checkout, 'hooks/pre-tool-use.mjs')) &&
      existsSync(path.join(checkout, 'hooks/post-tool-batch.mjs')) &&
      existsSync(path.join(checkout, 'hooks/stop.mjs')) &&
      existsSync(path.join(checkout, 'hooks/user-prompt-submit.mjs')),
    'missing public hook files');

  rmSync(checkout, { recursive: true, force: true });
  const pushed = run(NODE, [
    SCRIPT,
    '--repo', bare,
    '--dist', dist,
    '--checkout-dir', checkout,
    '--no-build',
    '--push',
  ], ROOT, {
    env: {
      LUMIN_REPO_LENS_PUBLISH_AUTHOR_NAME: 'Lumin Publish Bot',
      LUMIN_REPO_LENS_PUBLISH_AUTHOR_EMAIL: 'lumin-publish@example.test',
    },
  });
  assert('PPUB5. --push commits and pushes to public main',
    pushed.status === 0 &&
    pushed.stdout.includes('pushed public package'),
    `${pushed.stdout}\n${pushed.stderr}`);
  const pushedAuthor = git(['--git-dir', bare, 'log', '-1', '--format=%an <%ae>'], ROOT);
  assert('PPUB5b. --push honors explicit publish author environment',
    pushedAuthor === 'Lumin Publish Bot <lumin-publish@example.test>',
    pushedAuthor);
  const pushedPlugin = git(['--git-dir', bare, 'show', 'main:.claude-plugin/plugin.json'], ROOT);
  const pushedSkillPkg = git(['--git-dir', bare, 'show', 'main:skills/lumin-repo-lens-lab/package.json'], ROOT);
  assert('PPUB6. pushed public repo exposes current plugin and skill metadata',
    JSON.parse(pushedPlugin).version === CURRENT_VERSION &&
    JSON.parse(pushedSkillPkg).version === CURRENT_VERSION,
    `${pushedPlugin}\n${pushedSkillPkg}`);
  const pushedWorkflowResult = run('git', ['--git-dir', bare, 'show', 'main:.github/workflows/ci.yml'], ROOT);
  const pushedWorkflow = pushedWorkflowResult.status === 0 ? pushedWorkflowResult.stdout : '';
  assert('PPUB6b. pushed public repo includes package CI workflow',
    pushedWorkflowResult.status === 0 &&
      pushedWorkflow.includes('name: Public Package CI') &&
      pushedWorkflow.includes('working-directory: skills/lumin-repo-lens-lab') &&
      pushedWorkflow.includes('npm run smoke') &&
      pushedWorkflow.includes('node hooks/pre-tool-use.mjs'),
    pushedWorkflow || pushedWorkflowResult.stderr);
  const pushedHooksResult = run('git', ['--git-dir', bare, 'show', 'main:hooks/hooks.json'], ROOT);
  assert('PPUB6c. pushed public repo includes auto-hook manifest',
    pushedHooksResult.status === 0 &&
      JSON.parse(pushedHooksResult.stdout).hooks?.PreToolUse,
    pushedHooksResult.stdout || pushedHooksResult.stderr);

  assert('PPUB7. package.json exposes check and push scripts for public plugin publishing',
    PACKAGE.scripts['check:public-plugin'] === 'node scripts/publish-public-plugin.mjs --dry-run' &&
    PACKAGE.scripts['publish:public-plugin'] === 'node scripts/publish-public-plugin.mjs --push',
    JSON.stringify(PACKAGE.scripts, null, 2));
} finally {
  rmSync(tmp, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
