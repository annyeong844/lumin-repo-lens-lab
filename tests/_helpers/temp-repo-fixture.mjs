import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

const DEFAULT_PREFIX = 'lrl-fixture-';
const DEFAULT_PACKAGE_JSON = {
  name: 'fixture',
  private: true,
  type: 'module',
};
const DEFAULT_OUTPUT_DIR_NAME = '.audit';

function assertSafePrefix(prefix) {
  if (typeof prefix !== 'string' || prefix.length === 0) {
    throw new Error('fixture prefix must be a non-empty string');
  }
  if (prefix.includes('\0') || /[\\/]/.test(prefix) || /^[A-Za-z]:/.test(prefix)) {
    throw new Error(`fixture prefix is unsafe: ${JSON.stringify(prefix)}`);
  }
  return prefix;
}

function normalizeFixturePath(relPath) {
  if (typeof relPath !== 'string' || relPath.length === 0) {
    throw new Error('fixture path must be a non-empty relative path');
  }
  if (relPath.includes('\0')) {
    throw new Error(`fixture path contains NUL: ${JSON.stringify(relPath)}`);
  }
  if (
    path.isAbsolute(relPath) ||
    path.win32.isAbsolute(relPath) ||
    relPath.startsWith('/') ||
    relPath.startsWith('\\') ||
    /^[A-Za-z]:/.test(relPath)
  ) {
    throw new Error(`fixture path must be relative: ${JSON.stringify(relPath)}`);
  }

  const slashPath = relPath.replace(/\\/g, '/');
  const normalized = path.posix.normalize(slashPath);
  if (normalized === '.' || normalized === '..' || normalized.startsWith('../')) {
    throw new Error(`fixture path must stay inside fixture root: ${JSON.stringify(relPath)}`);
  }
  return normalized;
}

function resolveInside(base, relPath) {
  const normalized = normalizeFixturePath(relPath);
  const target = path.resolve(base, ...normalized.split('/'));
  const relative = path.relative(base, target);
  if (relative === '' || relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`fixture path resolved outside fixture root: ${JSON.stringify(relPath)}`);
  }
  return target;
}

function assertSafeCleanupTarget(root, prefix) {
  const tmpRoot = path.resolve(tmpdir());
  const resolvedRoot = path.resolve(root);
  const relative = path.relative(tmpRoot, resolvedRoot);
  if (relative === '' || relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`refusing to clean fixture outside temp dir: ${resolvedRoot}`);
  }
  if (!path.basename(resolvedRoot).startsWith(prefix)) {
    throw new Error(`refusing to clean fixture without expected prefix: ${resolvedRoot}`);
  }
}

function selectBase(root, output, options = {}, optionName = 'from') {
  const location = options?.[optionName] ?? 'root';
  if (location === 'root') return root;
  if (location === 'output') return output;
  throw new Error(`unsupported fixture location: ${String(location)}`);
}

export function createTempRepoFixture(options = {}) {
  const prefix = assertSafePrefix(options.prefix ?? DEFAULT_PREFIX);
  const packageJson = options.packageJson ?? DEFAULT_PACKAGE_JSON;
  const outputDirName = options.outputDirName ?? DEFAULT_OUTPUT_DIR_NAME;
  const root = mkdtempSync(path.join(tmpdir(), prefix));
  const output = resolveInside(root, outputDirName);

  mkdirSync(output, { recursive: true });
  writeFileSync(
    resolveInside(root, 'package.json'),
    `${JSON.stringify(packageJson, null, 2)}\n`
  );

  const fixture = {
    root,
    output,
    path(relPath) {
      return resolveInside(root, relPath);
    },
    outputPath(relPath) {
      return resolveInside(output, relPath);
    },
    mkdir(relPath) {
      const target = resolveInside(root, relPath);
      mkdirSync(target, { recursive: true });
      return target;
    },
    write(relPath, text, writeOptions = {}) {
      const base = selectBase(root, output, writeOptions, 'to');
      const target = resolveInside(base, relPath);
      mkdirSync(path.dirname(target), { recursive: true });
      writeFileSync(target, text);
      return target;
    },
    writeJson(relPath, value, writeOptions = {}) {
      return fixture.write(relPath, `${JSON.stringify(value, null, 2)}\n`, writeOptions);
    },
    read(relPath, readOptions = {}) {
      const base = selectBase(root, output, readOptions, 'from');
      return readFileSync(resolveInside(base, relPath), 'utf8');
    },
    readJson(relPath, readOptions = {}) {
      return JSON.parse(fixture.read(relPath, readOptions));
    },
    cleanup() {
      assertSafeCleanupTarget(root, prefix);
      rmSync(root, { recursive: true, force: true });
    },
  };

  return fixture;
}
