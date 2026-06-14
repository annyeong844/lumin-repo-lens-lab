import { mkdtempSync, mkdirSync, writeFileSync, rmSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { parseMdxImportConsumers } from '../_lib/mdx-consumers.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.error(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

{
  const src = [
    "import DefaultCard, { UsedByMdx as Card, type Props } from '../src/card';",
    "import * as Widgets from '../src/widgets';",
    "import DefaultWidget, * as WidgetNamespace from '../src/widget-namespace';",
    '```tsx',
    "import { ExampleOnly } from '../src/example';",
    '```',
    '```md',
    '```tsx',
    "import { NestedExampleOnly } from '../src/nested-example';",
    '```',
    '',
    '<Card />',
  ].join('\n');
  const imports = parseMdxImportConsumers(src, 'content/page.mdx');
  const names = imports.map((i) => `${i.fromSpec}:${i.name}:${i.kind}`).sort();

  assert('MX-1a. MDX parser records named imports by imported name',
    names.includes('../src/card:UsedByMdx:import'),
    JSON.stringify(imports));
  assert('MX-1b. MDX parser records default imports',
    names.includes('../src/card:default:default'),
    JSON.stringify(imports));
  assert('MX-1c. MDX parser records namespace imports as whole-file consumers',
    names.includes('../src/widgets:*:namespace'),
    JSON.stringify(imports));
  assert('MX-1d. MDX parser records default+namespace default side',
    names.includes('../src/widget-namespace:default:default'),
    JSON.stringify(imports));
  assert('MX-1e. MDX parser records default+namespace namespace side',
    names.includes('../src/widget-namespace:*:namespace'),
    JSON.stringify(imports));
  assert('MX-1f. MDX parser ignores fenced example imports',
    !names.some((name) => name.includes('../src/example')),
    JSON.stringify(imports));
  assert('MX-1g. MDX parser does not treat info-string fence content as a closing fence',
    !names.some((name) => name.includes('../src/nested-example')),
    JSON.stringify(imports));
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'mdx-consumer-graph-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'mdx-fixture', type: 'module' }));
    write(fx, 'src/card.tsx',
      'export function UsedByMdx() { return null; }\n' +
      'export const Unused = 1;\n' +
      'export default function DefaultCard() { return null; }\n');
    write(fx, 'src/widgets.tsx',
      'export function UsedByNamespace() { return null; }\n' +
      'export default function DefaultWidget() { return null; }\n');
    write(fx, 'content/page.mdx',
      "import DefaultCard, { UsedByMdx as Card } from '../src/card';\n" +
      "import DefaultWidget, * as Widgets from '../src/widgets';\n" +
      '```md\n' +
      '```tsx\n' +
      "import { Unused } from '../src/card';\n" +
      '```\n' +
      '\n' +
      '<DefaultCard />\n' +
      '<Card />\n');

    const outDir = path.join(fx, 'out');
    const run = spawnSync(process.execPath, [
      path.join(REPO_ROOT, 'build-symbol-graph.mjs'),
      '--root', fx,
      '--output', outDir,
    ], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
    });
    const symbolsPath = path.join(outDir, 'symbols.json');
    const symbols = JSON.parse(readFileSync(symbolsPath, 'utf8'));
    const dead = new Set((symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`));

    assert('MX-2a. build-symbol-graph succeeds on MDX fixture',
      run.status === 0,
      `stdout=${run.stdout}\nstderr=${run.stderr}`);
    assert('MX-2b. MDX named import protects the referenced export',
      !dead.has('src/card.tsx::UsedByMdx'),
      JSON.stringify([...dead].sort()));
    assert('MX-2c. MDX default import protects the default export',
      !dead.has('src/card.tsx::default'),
      JSON.stringify([...dead].sort()));
    assert('MX-2d. unrelated export remains a dead candidate',
      dead.has('src/card.tsx::Unused'),
      JSON.stringify([...dead].sort()));
    assert('MX-2e. MDX default+namespace default side is protected',
      !dead.has('src/widgets.tsx::default'),
      JSON.stringify([...dead].sort()));
    assert('MX-2f. MDX default+namespace namespace side protects module exports',
      !dead.has('src/widgets.tsx::UsedByNamespace'),
      JSON.stringify([...dead].sort()));
    assert('MX-2g. fenced import does not contribute fan-in evidence',
      symbols.fanInByIdentity?.['src/card.tsx::Unused'] === 0,
      JSON.stringify({
        fanIn: symbols.fanInByIdentity?.['src/card.tsx::Unused'],
        mdxConsumers: symbols.uses?.mdxConsumers,
      }));
    assert('MX-2h. MDX consumer contributes identity fan-in evidence',
      symbols.fanInByIdentity?.['src/card.tsx::UsedByMdx'] === 1 &&
        symbols.uses?.mdxConsumers === 4,
      JSON.stringify({
        fanIn: symbols.fanInByIdentity?.['src/card.tsx::UsedByMdx'],
        mdxConsumers: symbols.uses?.mdxConsumers,
      }));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
