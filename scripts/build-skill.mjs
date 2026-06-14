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
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
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
  'canon-drift.md',
  'classification-gates.md',
  'fact-model.md',
  'identity-and-alias.md',
  'index.md',
  'invariants.md',
  'mode-contract.md',
  'pre-write-gate.md',
];

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

function buildSkillPackageJson(outDir) {
  const source = JSON.parse(readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
  const pkg = {
    name: 'lumin-repo-lens-lab-skill',
    version: source.version,
    description: 'Deployable lumin-repo-lens-lab repository evidence skill package.',
    type: 'module',
    private: true,
    license: source.license,
    luminRepoLens: {
      distribution: 'skill',
    },
    bin: {
      'lumin-repo-lens-lab': './scripts/audit-repo.mjs',
    },
    scripts: {
      audit: 'node scripts/audit-repo.mjs',
      'pre-write': 'node scripts/audit-repo.mjs --pre-write',
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

function build(outDir) {
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });

  for (const file of ROOT_FILES) copyFileRel(file, file, outDir);
  for (const file of RUNTIME_CANON_FILES) writeRuntimeCanonFile(file, outDir);
  copyDirRel('templates', 'templates', outDir);
  copyDirRel('references', 'references', outDir);
  copyDirRel('_lib', '_engine/lib', outDir);

  for (const script of PRODUCER_SCRIPTS) writeProducerScript(script, outDir);
  for (const command of PUBLIC_COMMANDS) writePublicWrapper(command, outDir);
  for (const script of PUBLIC_UTILITY_SCRIPTS) copyFileRel(script, script, outDir);

  writeEngineReadme(outDir);
  rewritePackagedSourceFiles(path.join(outDir, '_engine'));
  rewritePackagedMarkdownFiles(outDir);
  buildSkillPackageJson(outDir);
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
