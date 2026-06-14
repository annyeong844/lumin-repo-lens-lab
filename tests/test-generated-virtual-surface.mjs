import { execSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

import { parsePrismaEnums, schemaUsesPrismaEnumGenerator } from '../_lib/generated-virtual-surface.mjs';

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
  const schema =
    'generator enums {\n' +
    '  provider = "prisma-enum-generator"\n' +
    '}\n\n' +
    'enum BookingStatus {\n' +
    '  /// accepted booking\n' +
    '  ACCEPTED @map("accepted")\n' +
    '  CANCELLED\n' +
    '}\n';
  const enums = parsePrismaEnums(schema);
  assert('GVS0. prisma enum parser extracts enum names and values without value attributes',
    schemaUsesPrismaEnumGenerator(schema) &&
      enums.length === 1 &&
      enums[0]?.name === 'BookingStatus' &&
      enums[0]?.values?.join(',') === 'ACCEPTED,CANCELLED',
    JSON.stringify(enums));
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'fx-generated-virtual-prisma-'));
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
    writeFileSync(path.join(fx, 'packages/prisma/schema.prisma'),
      'generator enums {\n' +
      '  provider = "prisma-enum-generator"\n' +
      '}\n\n' +
      'enum BookingStatus {\n' +
      '  ACCEPTED @map("accepted")\n' +
      '  CANCELLED\n' +
      '}\n');
    writeFileSync(path.join(fx, 'apps/web/src/consumer.ts'),
      "import { BookingStatus } from '@scope/prisma/enums';\n" +
      'export const status = BookingStatus.ACCEPTED;\n');

    execSync(`node build-symbol-graph.mjs --root ${fx} --output ${out}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    const unresolved = syms.unresolvedInternalSpecifierRecords ?? [];
    const surface = (syms.generatedVirtualSurfaces ?? [])
      .find((item) => item.matchedPackage === '@scope/prisma' && item.targetSubpath === 'enums');
    const consumer = (syms.generatedVirtualImportConsumers ?? [])
      .find((item) => item.specifier === '@scope/prisma/enums' && item.name === 'BookingStatus');

    assert('GVS1. prisma enum generated virtual surface resolves known enum import',
      syms.uses?.unresolvedInternal === 0 &&
        !unresolved.some((item) => item.specifier === '@scope/prisma/enums') &&
        syms.uses?.resolvedGeneratedVirtual === 1 &&
        syms.meta?.supports?.generatedVirtualSurfaces === true &&
        surface?.source === 'generated-virtual' &&
        surface?.virtual === true &&
        surface?.runtimeEquivalence === false &&
        surface?.surfaceCompleteness === 'partial' &&
        surface?.exports?.some((item) =>
          item.name === 'BookingStatus' &&
          item.spaces?.includes('value') &&
          item.spaces?.includes('type')) &&
        consumer?.surfaceId === surface?.id &&
        consumer?.consumerFile === 'apps/web/src/consumer.ts',
      `uses=${JSON.stringify(syms.uses)} unresolved=${JSON.stringify(unresolved)} surfaces=${JSON.stringify(syms.generatedVirtualSurfaces)} consumers=${JSON.stringify(syms.generatedVirtualImportConsumers)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'fx-generated-virtual-prisma-no-provider-'));
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
        dependencies: { '@prisma/client': '1.0.0' },
      }));
    writeFileSync(path.join(fx, 'packages/prisma/index.ts'),
      'export const prismaRoot = 1;\n');
    writeFileSync(path.join(fx, 'packages/prisma/schema.prisma'),
      'generator client {\n' +
      '  provider = "prisma-client-js"\n' +
      '}\n\n' +
      'enum BookingStatus {\n' +
      '  ACCEPTED\n' +
      '}\n');
    writeFileSync(path.join(fx, 'apps/web/src/consumer.ts'),
      "import { BookingStatus } from '@scope/prisma/enums';\n" +
      'export const status = BookingStatus.ACCEPTED;\n');

    execSync(`node build-symbol-graph.mjs --root ${fx} --output ${out}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    assert('GVS2. prisma enum virtual surface requires schema generator provider evidence',
      syms.uses?.unresolvedInternal === 1 &&
        (syms.generatedVirtualSurfaces ?? []).length === 0 &&
        (syms.generatedVirtualImportConsumers ?? []).length === 0,
      `uses=${JSON.stringify(syms.uses)} surfaces=${JSON.stringify(syms.generatedVirtualSurfaces)} consumers=${JSON.stringify(syms.generatedVirtualImportConsumers)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'fx-generated-virtual-prisma-missing-export-'));
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
    writeFileSync(path.join(fx, 'packages/prisma/schema.prisma'),
      'generator enums {\n' +
      '  provider = "prisma-enum-generator"\n' +
      '}\n\n' +
      'enum BookingStatus {\n' +
      '  ACCEPTED\n' +
      '}\n');
    writeFileSync(path.join(fx, 'apps/web/src/consumer.ts'),
      "import { MissingEnum } from '@scope/prisma/enums';\n" +
      'export const status = MissingEnum.ACCEPTED;\n');

    execSync(`node build-symbol-graph.mjs --root ${fx} --output ${out}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
    assert('GVS3. prisma virtual surface does not resolve imports absent from schema enum surface',
      syms.uses?.unresolvedInternal === 1 &&
        (syms.generatedVirtualSurfaces ?? []).some((item) =>
          item.matchedPackage === '@scope/prisma' &&
          item.exports?.some((entry) => entry.name === 'BookingStatus')) &&
        !(syms.generatedVirtualImportConsumers ?? []).some((item) => item.name === 'MissingEnum'),
      `uses=${JSON.stringify(syms.uses)} unresolved=${JSON.stringify(syms.unresolvedInternalSpecifierRecords)} surfaces=${JSON.stringify(syms.generatedVirtualSurfaces)} consumers=${JSON.stringify(syms.generatedVirtualImportConsumers)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
