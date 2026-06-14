// Tests for build-plugin-package.mjs — Claude Code plugin-root package shape.

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const BUILD_PLUGIN = path.join(ROOT, 'scripts/build-plugin-package.mjs');
const COMMAND_SKILL_TARGETS = {
  'lumin-repo-lens-lab': 'skills/lumin-repo-lens-lab/SKILL.md',
  welcome: 'skills/lumin-repo-lens-lab/SKILL.md',
  full: 'skills/lumin-repo-lens-lab/SKILL.md',
  audit: 'skills/lumin-repo-lens-lab/SKILL.md',
  'refactor-plan': 'skills/lumin-repo-lens-lab/SKILL.md',
  'pre-write': 'skills/lumin-repo-lens-lab-write-gate/SKILL.md',
  'post-write': 'skills/lumin-repo-lens-lab-write-gate/SKILL.md',
  'canon-draft': 'skills/lumin-repo-lens-lab-canon/SKILL.md',
  'check-canon': 'skills/lumin-repo-lens-lab-canon/SKILL.md',
};

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function pluginRefsFromCommand(commandText) {
  return [...commandText.matchAll(/\$\{CLAUDE_PLUGIN_ROOT\}\/([^\s`,]+)/g)]
    .map((match) => match[1]);
}

const tmp = mkdtempSync(path.join(os.tmpdir(), 'plugin-package-'));
const out = path.join(tmp, 'lumin-repo-lens-lab-plugin');
const legacyOut = path.join(tmp, 'lumin-audit-plugin');

try {
  mkdirSync(legacyOut, { recursive: true });
  const build = spawnSync(NODE, [BUILD_PLUGIN, '--out', out], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert('PP1. build-plugin-package exits 0',
    build.status === 0,
    `${build.stdout}\n${build.stderr}`);
  if (build.status !== 0) {
    failed += 6;
    console.log('  SKIP  PP2-PP7 because build-plugin-package did not run');
    console.log(`\n${passed} passed, ${failed} failed`);
    process.exit(1);
  }

  assert('PP2. plugin root includes Claude Code plugin metadata and commands',
    existsSync(path.join(out, '.claude-plugin/plugin.json')) &&
    existsSync(path.join(out, '.claude-plugin/marketplace.json')) &&
    existsSync(path.join(out, 'commands/lumin-repo-lens-lab.md')) &&
    existsSync(path.join(out, 'commands/pre-write.md')) &&
    existsSync(path.join(out, 'commands/check-canon.md')),
    readdirSync(out, { recursive: true }).join('\n'));

  assert('PP2b. plugin root includes auto-hook manifest',
    existsSync(path.join(out, 'hooks/hooks.json')),
    readdirSync(out, { recursive: true }).join('\n'));

  assert('PP2c. plugin root includes auto-hook runner scripts',
    existsSync(path.join(out, 'hooks/_runner-utils.mjs')) &&
    existsSync(path.join(out, 'hooks/pre-tool-use.mjs')) &&
    existsSync(path.join(out, 'hooks/post-tool-batch.mjs')) &&
    existsSync(path.join(out, 'hooks/stop.mjs')) &&
    existsSync(path.join(out, 'hooks/user-prompt-submit.mjs')),
    readdirSync(out, { recursive: true }).join('\n'));

  assert('PP3. plugin root includes Claude Code skill surfaces with shared engine',
    existsSync(path.join(out, 'skills/lumin-repo-lens-lab/SKILL.md')) &&
    existsSync(path.join(out, 'skills/lumin-repo-lens-lab/scripts/audit-repo.mjs')) &&
    existsSync(path.join(out, 'skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs')) &&
    existsSync(path.join(out, 'skills/lumin-repo-lens-lab-write-gate/SKILL.md')) &&
    existsSync(path.join(out, 'skills/lumin-repo-lens-lab-canon/SKILL.md')),
    readdirSync(path.join(out, 'skills'), { recursive: true }).join('\n'));

  assert('PP4. plugin root excludes Codex wrapper by default to avoid Claude Code surface collision',
    !existsSync(path.join(out, 'skills/lumin-repo-lens-lab-codex')) &&
    !existsSync(path.join(out, 'skills/lumin-repo-lens-lab-codex/SKILL.md')),
    readdirSync(path.join(out, 'skills')).join(', '));

  const command = readFileSync(path.join(out, 'commands/lumin-repo-lens-lab.md'), 'utf8');
  assert('PP5. commands target packaged plugin-root skill paths',
    command.includes('${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/SKILL.md') &&
    command.includes('${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/references/command-routing.md'),
    command);

  const commandFiles = readdirSync(path.join(out, 'commands'))
    .filter((name) => name.endsWith('.md'))
    .sort();
  const unresolvedCommandRefs = [];
  const wrongSkillTargets = [];
  for (const fileName of commandFiles) {
    const commandName = fileName.replace(/\.md$/, '');
    const text = readFileSync(path.join(out, 'commands', fileName), 'utf8');
    const refs = pluginRefsFromCommand(text);
    for (const ref of refs) {
      if (!existsSync(path.join(out, ref))) unresolvedCommandRefs.push(`${fileName}: ${ref}`);
    }
    const expectedSkill = COMMAND_SKILL_TARGETS[commandName];
    if (expectedSkill && !refs.includes(expectedSkill)) {
      wrongSkillTargets.push(`${fileName}: expected ${expectedSkill}, got ${refs.join(', ')}`);
    }
  }
  assert('PP5b. every command plugin-root reference resolves inside the packaged root',
    unresolvedCommandRefs.length === 0,
    unresolvedCommandRefs.join('\n'));
  assert('PP5c. every command delegates to the expected packaged skill surface',
    wrongSkillTargets.length === 0,
    wrongSkillTargets.join('\n'));

  const plugin = JSON.parse(readFileSync(path.join(out, '.claude-plugin/plugin.json'), 'utf8'));
  const skillPkg = JSON.parse(readFileSync(path.join(out, 'skills/lumin-repo-lens-lab/package.json'), 'utf8'));
  assert('PP6. plugin package carries versioned plugin metadata plus skill distribution marker',
    plugin.name === 'lumin-repo-lens-lab' &&
    plugin.description.includes('repo structure lens') &&
    plugin.version === skillPkg.version &&
    skillPkg.luminRepoLens?.distribution === 'skill',
    `${JSON.stringify(plugin, null, 2)}\n${JSON.stringify(skillPkg, null, 2)}`);

  const packageReadme = readFileSync(path.join(out, 'README.plugin-package.md'), 'utf8');
  assert('PP6b. plugin package README names the install root and warns against installing skills alone',
    packageReadme.includes('Install this directory as the Claude Code plugin root') &&
    packageReadme.includes('Do not install `skills/` alone') &&
    packageReadme.includes('Slash command delegators resolve through `${CLAUDE_PLUGIN_ROOT}`'),
    packageReadme);

  assert('PP6c. build-plugin-package runs a plugin-root smoke check after staging',
    build.stdout.includes('[build-plugin-package] plugin-root smoke passed'),
    build.stdout);

  assert('PP6d. build-plugin-package removes stale legacy plugin output beside the current package',
    !existsSync(legacyOut) &&
    build.stdout.includes('removed legacy output'),
    build.stdout);

  const outWithCodex = path.join(tmp, 'lumin-repo-lens-lab-plugin-with-codex');
  mkdirSync(path.dirname(outWithCodex), { recursive: true });
  const buildWithCodex = spawnSync(NODE, [BUILD_PLUGIN, '--out', outWithCodex, '--include-codex'], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert('PP7. --include-codex opt-in includes Codex wrapper',
    buildWithCodex.status === 0 &&
    existsSync(path.join(outWithCodex, 'skills/lumin-repo-lens-lab-codex/SKILL.md')),
    `${buildWithCodex.stdout}\n${buildWithCodex.stderr}`);
} finally {
  rmSync(tmp, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
