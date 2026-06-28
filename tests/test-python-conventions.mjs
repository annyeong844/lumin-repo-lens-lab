// Regression tests for Python-specific convention support added in v1.7.2.
//
// Four concerns, each a separate set of assertions:
//   A. Self-reference import resolution — `--root /pkg` + `import pkg.x` must
//      resolve to `/pkg/x.py`, not probe `/pkg/pkg/x.py`.
//   B. __all__ awareness — modules declaring `__all__ = [...]` expose only
//      listed names; other top-level names are module-private and must not
//      appear in the dead-list.
//   C. Framework-registered functions — decorators like `@app.command()`,
//      `@app.route()`, `@task`, `@fixture` mean the function is dispatched
//      by the framework; must not appear in the dead-list.
//   D. Dunder methods — `__getattr__`, `__dir__`, `__init__`, etc. are
//      runtime-dispatched; must not enter the def list at all.

import { execSync, execFileSync } from 'node:child_process';
import { writeFileSync, mkdirSync, rmSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// Skip the whole suite if python3 isn't available — the extractor requires it.
try { execFileSync('python3', ['-c', 'pass'], { stdio: 'ignore' }); }
catch {
  console.log('  SKIP  python3 not available — Python convention tests skipped');
  console.log('\n0 passed, 0 failed');
  process.exit(0);
}

function buildSymbols(root, out) {
  rmSync(out, { recursive: true, force: true });
  execSync(`node build-symbol-graph.mjs --root ${root} --output ${out}`,
    { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });
  return JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
}

// ─── A. Self-reference import ────────────────────────────────
{
  const ROOT = '/tmp/fx-py-self-ref';
  rmSync(ROOT, { recursive: true, force: true });
  mkdirSync(path.join(ROOT, 'agents'), { recursive: true });

  writeFileSync(path.join(ROOT, '__init__.py'), '');
  writeFileSync(path.join(ROOT, 'agents/__init__.py'), '');
  writeFileSync(path.join(ROOT, 'agents/loader.py'),
    'def load_agent() -> str:\n    return "x"\n'
  );
  // Consumer uses self-qualified import `<root-basename>.agents.loader`
  // where the root basename is the Python package name.
  const pkgName = path.basename(ROOT); // 'fx-py-self-ref' — dash is illegal in Python identifier, rename
  rmSync(ROOT, { recursive: true, force: true });
  // Use a valid Python pkg name — must match directory basename.
  const ROOT2 = '/tmp/fxpyselfref';
  mkdirSync(path.join(ROOT2, 'agents'), { recursive: true });
  mkdirSync(path.join(ROOT2, 'consumers'), { recursive: true });
  writeFileSync(path.join(ROOT2, '__init__.py'), '');
  writeFileSync(path.join(ROOT2, 'agents/__init__.py'), '');
  writeFileSync(path.join(ROOT2, 'consumers/__init__.py'), '');
  writeFileSync(path.join(ROOT2, 'agents/loader.py'),
    'def load_agent() -> str:\n    return "x"\n'
  );
  writeFileSync(path.join(ROOT2, 'consumers/app.py'),
    'from fxpyselfref.agents.loader import load_agent\n' +
    '\n' +
    'def main():\n' +
    '    return load_agent()\n'
  );

  const d = buildSymbols(ROOT2, '/tmp/fxpyselfref-out');
  const deadNames = new Set(d.deadProdList.map((x) => x.symbol));

  assert('A1. self-reference import resolves — load_agent not in dead list',
    !deadNames.has('load_agent'),
    `deadProdList contained: ${[...deadNames].slice(0, 10).join(', ')}`);
  assert('A2. consumer main still appears (dead — only self-references)',
    deadNames.has('main'),
    `main should be in dead list (no cross-file consumer)`);
}

// ─── B. __all__ awareness ────────────────────────────────────
{
  const ROOT = '/tmp/fxpydunderall';
  rmSync(ROOT, { recursive: true, force: true });
  mkdirSync(ROOT, { recursive: true });

  writeFileSync(path.join(ROOT, '__init__.py'), '');
  // Module declares __all__ — only Foo is public; _helper and internal_util
  // are implicitly private and MUST NOT be flagged dead even with no
  // cross-file uses.
  writeFileSync(path.join(ROOT, 'module.py'),
    '__all__ = ["Foo"]\n' +
    '\n' +
    'class Foo:\n' +
    '    def method(self):\n' +
    '        return self._helper()\n' +
    '\n' +
    'def _helper():\n' +
    '    return 1\n' +
    '\n' +
    'def internal_util():\n' +
    '    return 2\n'
  );
  // Ensure there's another file so symbol graph has something to compare
  writeFileSync(path.join(ROOT, 'dummy.py'), 'x = 1\n');

  const d = buildSymbols(ROOT, '/tmp/fxpydunderall-out');
  const dead = d.deadProdList.filter((x) => x.file.endsWith('module.py'));
  const deadNames = new Set(dead.map((x) => x.symbol));

  assert('B1. __all__ itself NOT in dead list',
    !deadNames.has('__all__'),
    `__all__ appeared in dead list`);
  assert('B2. Foo (listed in __all__) IS a candidate',
    deadNames.has('Foo'),
    `Foo should be in dead list (in __all__, no cross-file consumer)`);
  assert('B3. _helper (NOT in __all__) NOT in dead list (module-private by convention)',
    !deadNames.has('_helper'),
    `_helper leaked into dead list`);
  assert('B4. internal_util (NOT in __all__) NOT in dead list',
    !deadNames.has('internal_util'),
    `internal_util leaked into dead list`);
}

// ─── C. Framework-registered decorators ──────────────────────
{
  const ROOT = '/tmp/fxpytyper';
  rmSync(ROOT, { recursive: true, force: true });
  mkdirSync(ROOT, { recursive: true });
  writeFileSync(path.join(ROOT, '__init__.py'), '');
  writeFileSync(path.join(ROOT, 'dummy.py'), 'x = 1\n');
  writeFileSync(path.join(ROOT, 'cli.py'),
    'import typer\n' +
    'app = typer.Typer()\n' +
    '\n' +
    '@app.command()\n' +
    'def subcommand_a():\n' +
    '    return 1\n' +
    '\n' +
    '@app.command(name="list")\n' +
    'def list_items():\n' +
    '    return 2\n' +
    '\n' +
    '@app.callback()\n' +
    'def callback_entry():\n' +
    '    return 3\n' +
    '\n' +
    'def actually_unused():\n' +
    '    return 4\n'
  );

  const d = buildSymbols(ROOT, '/tmp/fxpytyper-out');
  const dead = d.deadProdList.filter((x) => x.file.endsWith('cli.py'));
  const deadNames = new Set(dead.map((x) => x.symbol));

  assert('C1. @app.command decorated function NOT dead',
    !deadNames.has('subcommand_a'),
    `subcommand_a leaked into dead list (decorator-registered)`);
  assert('C2. @app.command(name=...) decorated function NOT dead',
    !deadNames.has('list_items'),
    `list_items leaked into dead list`);
  assert('C3. @app.callback decorated function NOT dead',
    !deadNames.has('callback_entry'),
    `callback_entry leaked into dead list`);
  assert('C4. undecorated function IS still a dead candidate',
    deadNames.has('actually_unused'),
    `actually_unused should be in dead list`);
}

// ─── D. Dunder methods never enter defs ──────────────────────
{
  const ROOT = '/tmp/fxpydunder';
  rmSync(ROOT, { recursive: true, force: true });
  mkdirSync(ROOT, { recursive: true });
  writeFileSync(path.join(ROOT, '__init__.py'),
    '# Module-level __getattr__ is a lazy-loading hook\n' +
    'def __getattr__(name):\n' +
    '    raise AttributeError(name)\n' +
    '\n' +
    'def __dir__():\n' +
    '    return []\n' +
    '\n' +
    'def regular_fn():\n' +
    '    return 1\n'
  );
  writeFileSync(path.join(ROOT, 'dummy.py'), 'x = 1\n');

  const d = buildSymbols(ROOT, '/tmp/fxpydunder-out');
  const initDead = d.deadProdList.filter((x) => x.file.endsWith('__init__.py'));
  const deadNames = new Set(initDead.map((x) => x.symbol));

  assert('D1. __getattr__ NOT in dead list',
    !deadNames.has('__getattr__'),
    `__getattr__ leaked`);
  assert('D2. __dir__ NOT in dead list',
    !deadNames.has('__dir__'),
    `__dir__ leaked`);
  assert('D3. regular_fn IS a dead candidate (no consumer)',
    deadNames.has('regular_fn'),
    `regular_fn missing from dead list`);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
