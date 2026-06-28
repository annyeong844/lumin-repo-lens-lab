#!/usr/bin/env node
// Build a Claude Code plugin-root package from the generated skill surfaces.
//
// `scripts/build-skill.mjs` owns the skill-only package. This script stages
// the Claude Code plugin shell around those generated surfaces so the output
// root can be zipped or installed as a plugin package.

import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const DEFAULT_OUT = path.join(ROOT, 'dist', 'lumin-repo-lens-lab-plugin');
const DEFAULT_SKILL_OUT = path.join(ROOT, 'skills', 'lumin-repo-lens-lab');
const CURRENT_PLUGIN_DIR_NAME = 'lumin-repo-lens-lab-plugin';
const LEGACY_PLUGIN_DIR_NAMES = [
  'lumin-audit-plugin',
];
const CLAUDE_SKILLS = [
  'lumin-repo-lens-lab',
  'lumin-repo-lens-lab-write-gate',
  'lumin-repo-lens-lab-canon',
];
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

function parseArgs(argv) {
  const out = {
    output: DEFAULT_OUT,
    includeCodex: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--out' || arg === '--output') {
      out.output = argv[++i];
    } else if (arg === '--include-codex') {
      out.includeCodex = true;
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
    'usage: node scripts/build-plugin-package.mjs [--out <dir>] [--include-codex]',
    '',
    'Default output:',
    `  ${path.relative(ROOT, DEFAULT_OUT)}`,
    '',
    'Default package includes Claude Code surfaces only:',
    '  lumin-repo-lens-lab, lumin-repo-lens-lab-write-gate, lumin-repo-lens-lab-canon',
    '',
    'Use --include-codex only for a mixed Codex/Claude local bundle.',
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

function copyRel(srcRel, destRel, outDir) {
  const src = path.join(ROOT, srcRel);
  const dest = path.join(outDir, destRel);
  if (!existsSync(src)) throw new Error(`missing source path: ${srcRel}`);
  mkdirSync(path.dirname(dest), { recursive: true });
  cpSync(src, dest, { recursive: true });
}

function cleanLegacyPluginOutputs(outDir) {
  if (path.basename(outDir) !== CURRENT_PLUGIN_DIR_NAME) return [];
  const parent = path.dirname(outDir);
  const removed = [];
  for (const dirName of LEGACY_PLUGIN_DIR_NAMES) {
    const legacyDir = path.join(parent, dirName);
    if (!existsSync(legacyDir)) continue;
    rmSync(legacyDir, { recursive: true, force: true });
    removed.push(path.relative(ROOT, legacyDir) || legacyDir);
  }
  return removed;
}

function runBuildSkill() {
  const result = spawnSync(process.execPath, [
    path.join(ROOT, 'scripts', 'build-skill.mjs'),
    '--out', DEFAULT_SKILL_OUT,
  ], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  if (result.status !== 0) {
    throw new Error(
      `build-skill failed with exit ${result.status}\n${result.stdout}\n${result.stderr}`.trim()
    );
  }
}

function writePackageReadme(outDir, { includeCodex }) {
  const dest = path.join(outDir, 'README.plugin-package.md');
  const skills = includeCodex
    ? [...CLAUDE_SKILLS, 'lumin-repo-lens-lab-codex']
    : CLAUDE_SKILLS;
  writeFileSync(dest, [
    '# Lumin Repo Lens Claude Code Plugin Package',
    '',
    'Install this directory as the Claude Code plugin root. Do not install `skills/` alone;',
    'the slash command delegators and plugin metadata live at this package root.',
    '',
    'This directory is a plugin-root package. It includes Claude Code plugin',
    'metadata, slash-command delegators, and generated skill surfaces.',
    '',
    'Slash command delegators resolve through `${CLAUDE_PLUGIN_ROOT}` and',
    'point at the generated skill surfaces below.',
    '',
    'Included skill surfaces:',
    '',
    ...skills.map((skill) => `- \`skills/${skill}/\``),
    '',
    includeCodex
      ? 'The Codex wrapper is included because `--include-codex` was passed.'
      : 'The Codex wrapper is excluded by default to avoid Claude Code implicit-invocation overlap.',
    '',
  ].join('\n'));
}

function pluginRefsFromCommand(commandText) {
  return [...commandText.matchAll(/\$\{CLAUDE_PLUGIN_ROOT\}\/([^\s`,]+)/g)]
    .map((match) => match[1]);
}

function verifyPluginRoot(outDir, { includeCodex }) {
  const requiredRootFiles = [
    '.claude-plugin/plugin.json',
    '.claude-plugin/marketplace.json',
    'README.plugin-package.md',
  ];
  for (const rel of requiredRootFiles) {
    if (!existsSync(path.join(outDir, rel))) {
      throw new Error(`plugin-root smoke failed: missing ${rel}`);
    }
  }

  const commandDir = path.join(outDir, 'commands');
  const commandFiles = readdirSync(commandDir)
    .filter((name) => name.endsWith('.md'))
    .sort();
  const expectedCommands = Object.keys(COMMAND_SKILL_TARGETS)
    .map((name) => `${name}.md`)
    .sort();
  if (JSON.stringify(commandFiles) !== JSON.stringify(expectedCommands)) {
    throw new Error(
      `plugin-root smoke failed: command set mismatch; expected ${expectedCommands.join(', ')}, got ${commandFiles.join(', ')}`
    );
  }

  let refCount = 0;
  for (const fileName of commandFiles) {
    const commandName = fileName.replace(/\.md$/, '');
    const text = readFileSync(path.join(commandDir, fileName), 'utf8');
    const refs = pluginRefsFromCommand(text);
    if (refs.length === 0) {
      throw new Error(`plugin-root smoke failed: ${fileName} has no CLAUDE_PLUGIN_ROOT references`);
    }
    refCount += refs.length;
    for (const rel of refs) {
      if (!existsSync(path.join(outDir, rel))) {
        throw new Error(`plugin-root smoke failed: ${fileName} points at missing ${rel}`);
      }
    }

    const expectedSkill = COMMAND_SKILL_TARGETS[commandName];
    if (!refs.includes(expectedSkill)) {
      throw new Error(
        `plugin-root smoke failed: ${fileName} should delegate to ${expectedSkill}; saw ${refs.join(', ')}`
      );
    }
  }

  for (const skill of CLAUDE_SKILLS) {
    if (!existsSync(path.join(outDir, 'skills', skill, 'SKILL.md'))) {
      throw new Error(`plugin-root smoke failed: missing packaged skill ${skill}`);
    }
  }
  const codexPath = path.join(outDir, 'skills', 'lumin-repo-lens-lab-codex', 'SKILL.md');
  if (includeCodex && !existsSync(codexPath)) {
    throw new Error('plugin-root smoke failed: --include-codex did not package lumin-repo-lens-lab-codex');
  }
  if (!includeCodex && existsSync(codexPath)) {
    throw new Error('plugin-root smoke failed: default package unexpectedly includes lumin-repo-lens-lab-codex');
  }

  return { commandCount: commandFiles.length, refCount };
}

function build(outDir, { includeCodex }) {
  const removedLegacyOutputs = cleanLegacyPluginOutputs(outDir);
  runBuildSkill();
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });

  copyRel('.claude-plugin', '.claude-plugin', outDir);
  copyRel('commands', 'commands', outDir);
  if (existsSync(path.join(ROOT, 'hooks'))) {
    copyRel('hooks', 'hooks', outDir);
  }

  const skills = includeCodex
    ? [...CLAUDE_SKILLS, 'lumin-repo-lens-lab-codex']
    : CLAUDE_SKILLS;
  for (const skill of skills) {
    copyRel(path.join('skills', skill), path.join('skills', skill), outDir);
  }
  writePackageReadme(outDir, { includeCodex });
  return { ...verifyPluginRoot(outDir, { includeCodex }), removedLegacyOutputs };
}

try {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(usage());
    process.exit(0);
  }
  const outDir = guardOutputPath(args.output);
  const smoke = build(outDir, { includeCodex: args.includeCodex });
  console.log(`[build-plugin-package] wrote ${path.relative(ROOT, outDir) || outDir}`);
  for (const legacyDir of smoke.removedLegacyOutputs) {
    console.log(`[build-plugin-package] removed legacy output ${legacyDir}`);
  }
  console.log(
    `[build-plugin-package] plugin-root smoke passed (${smoke.commandCount} commands, ${smoke.refCount} plugin refs)`
  );
  if (!args.includeCodex) {
    console.log('[build-plugin-package] omitted skills/lumin-repo-lens-lab-codex by default');
  }
} catch (e) {
  console.error(`[build-plugin-package] ${e.message}`);
  process.exit(1);
}
