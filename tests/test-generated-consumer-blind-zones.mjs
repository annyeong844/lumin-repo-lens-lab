import { execSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'fx-generated-consumer-zone-'));
  const out = path.join(fx, 'artifacts');
  try {
    mkdirSync(path.join(fx, 'apps/web/src'), { recursive: true });
    mkdirSync(path.join(fx, 'packages/prisma'), { recursive: true });
    mkdirSync(out, { recursive: true });

    writeFileSync(path.join(fx, 'package.json'),
      JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
    writeFileSync(path.join(fx, 'apps/web/package.json'),
      JSON.stringify({ name: 'web', type: 'module' }));
    writeFileSync(path.join(fx, 'packages/prisma/package.json'),
      JSON.stringify({
        name: '@scope/prisma',
        type: 'module',
        main: 'index.ts',
        bin: { 'prisma-enum-generator': './run-enum-generator.js' },
        scripts: { generate: 'prisma generate' },
        dependencies: { '@prisma/client': '1.0.0' },
      }));
    writeFileSync(path.join(fx, 'packages/prisma/index.ts'),
      'export const prismaRoot = 1;\n');
    writeFileSync(path.join(fx, 'apps/web/src/consumer.ts'),
      "import { BookingStatus } from '@scope/prisma/enums';\n" +
      'export const status = BookingStatus.ACCEPTED;\n');

    execSync(`node build-symbol-graph.mjs --root ${fx} --output ${out}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    const zone = syms.generatedConsumerBlindZones?.[0];

    assert('GCBZ1. symbols artifact exposes generated consumer blind-zone support',
      syms.meta?.supports?.generatedConsumerBlindZones === true,
      JSON.stringify(syms.meta?.supports));
    assert('GCBZ2. missing generated workspace subpath emits consumer blind-zone inventory',
      zone?.reason === 'generated-consumer-blind-zone' &&
        zone?.sourceReason === 'workspace-generated-artifact-missing' &&
        zone?.specifier === '@scope/prisma/enums' &&
        zone?.consumerFile === 'apps/web/src/consumer.ts' &&
        zone?.matchedPackage === '@scope/prisma' &&
        zone?.targetSubpath === 'enums' &&
        zone?.status === 'missing' &&
        zone?.scopePackageRoot === 'packages/prisma',
      JSON.stringify(syms.generatedConsumerBlindZones, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'fx-generated-consumer-zone-prepared-'));
  const out = path.join(fx, 'artifacts');
  try {
    mkdirSync(path.join(fx, 'apps/web/src'), { recursive: true });
    mkdirSync(path.join(fx, 'packages/prisma'), { recursive: true });
    mkdirSync(out, { recursive: true });

    writeFileSync(path.join(fx, 'package.json'),
      JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
    writeFileSync(path.join(fx, 'apps/web/package.json'),
      JSON.stringify({ name: 'web', type: 'module' }));
    writeFileSync(path.join(fx, 'packages/prisma/package.json'),
      JSON.stringify({
        name: '@scope/prisma',
        type: 'module',
        main: 'index.ts',
        bin: { 'prisma-enum-generator': './run-enum-generator.js' },
        scripts: { generate: 'prisma generate' },
        dependencies: { '@prisma/client': '1.0.0' },
      }));
    writeFileSync(path.join(fx, 'packages/prisma/index.ts'),
      'export const prismaRoot = 1;\n');
    writeFileSync(path.join(fx, 'apps/web/src/consumer.ts'),
      "import { BookingStatus } from '@scope/prisma/enums';\n" +
      'export const status = BookingStatus.ACCEPTED;\n');

    execSync(
      `node build-symbol-graph.mjs --root ${fx} --output ${out} --generated-artifacts prepared`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] },
    );

    const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    const zone = syms.generatedConsumerBlindZones?.[0];

    assert('GCBZ3. build-symbol-graph forwards prepared mode to generated consumer zones',
      zone?.status === 'missing' &&
        zone?.mode === 'prepared' &&
        zone?.staleStatus === 'unknown' &&
        zone?.staleReason === 'generator-input-hash-not-recorded',
      JSON.stringify(syms.generatedConsumerBlindZones, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
