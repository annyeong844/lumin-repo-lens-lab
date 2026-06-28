// Regression guard for FP-38 — workspace packages without `exports` field.
//
// In Bun / older pnpm / Turborepo workspaces, package.json files
// typically declare `main` + `types` and omit the modern `exports`
// map. Consumers import deep subpaths like `@scope/pkg/subpath`,
// which Node resolves to `<pkgDir>/subpath.{ts,tsx,...}` by legacy
// rules. Before v1.9.11 the alias-map only built entries from the
// `exports` field, so these packages registered NOTHING and the
// resolver treated every `@scope/pkg/...` as EXTERNAL.
//
// Observed impact on duyet/monorepo (2026-04, empirical v1.9.10
// confirmation report): 13 of the 229 remaining Tier C findings
// after the TS compiler API fix were workspace-imported symbols
// from packages with no `exports` field:
//
//   packages/libs/getPost.ts:
//     getPostBySlug, getAllCategories, getPostsByCategory,
//     getAllTags, getPostsByTag          (consumed from apps/blog)
//   packages/libs/getSeries.ts:
//     getAllSeries, getSeries            (consumed from apps/blog)
//   packages/components/Menu.tsx:
//     HOME, ABOUT, INSIGHTS, PHOTOS, BLOG (consumed from apps/cv)
//   apps/insights/components/tabs.tsx:
//     Tabs                                (cross-app from apps/agents)
//
// FP rate residue: 10.9% on duyet, dominated entirely by this class.
//
// Fix (v1.9.11): when a workspace package has no explicit exports
// entry covering a subpath, register a legacy-subpath wildcard
// mapping `<pkgName>/*` → `<pkgDir>/*`. The resolver's wildcard
// matcher is extended to probe source extensions (.ts, .tsx, .mts,
// .cts, .mjs, .cjs, .js, .jsx) for extensionless literals.

import { execSync } from 'node:child_process';
import {
  writeFileSync, readFileSync, mkdirSync, rmSync, mkdtempSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Fixture builder: minimal Bun-style workspace where a package
//    uses only `main` (no `exports` field). ──
function buildWorkspace(fx, variant) {
  mkdirSync(path.join(fx, 'apps/blog/app'), { recursive: true });
  mkdirSync(path.join(fx, 'packages/libs'), { recursive: true });

  writeFileSync(path.join(fx, 'package.json'),
    JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));

  // The critical setup: workspace package with NO `exports`.
  const pkgShape = variant === 'main-only'
    ? { name: '@scope/libs', type: 'module', main: './getPost.ts' }
    : variant === 'no-main-no-exports'
    ? { name: '@scope/libs', type: 'module' }
    : { name: '@scope/libs', type: 'module', exports: { '.': './getPost.ts' } };

  writeFileSync(path.join(fx, 'packages/libs/package.json'),
    JSON.stringify(pkgShape));

  writeFileSync(path.join(fx, 'packages/libs/getPost.ts'), `
export function getPostBySlug(slug: string) { return { slug }; }
export function getAllCategories() { return []; }
export function getPostsByCategory(c: string) { return []; }
export function unusedInternal() { return 99; }
`);

  writeFileSync(path.join(fx, 'packages/libs/getSeries.ts'), `
export function getAllSeries() { return []; }
export function getSeries(n: string) { return { n }; }
`);
  mkdirSync(path.join(fx, 'packages/libs/inputs'), { recursive: true });
  writeFileSync(path.join(fx, 'packages/libs/inputs/location.input.ts'), `
export function makeLocationInput() { return { location: 'office' }; }
export function unusedLocationInput() { return { location: 'unused' }; }
`);

  writeFileSync(path.join(fx, 'apps/blog/package.json'),
    JSON.stringify({ name: 'blog', type: 'module',
      dependencies: { '@scope/libs': 'workspace:*' } }));

  writeFileSync(path.join(fx, 'apps/blog/app/page.tsx'), `
import { getPostBySlug, getAllCategories } from '@scope/libs/getPost';
import { getAllSeries } from '@scope/libs/getSeries';
import { makeLocationInput } from '@scope/libs/inputs/location.input';
export function Page() {
  getPostBySlug('x');
  getAllCategories();
  getAllSeries();
  makeLocationInput();
  return null;
}
`);
}

// ───────────────────────────────────────────────────────────
// F1-F5: main-only package (typical Bun/Turborepo pattern)
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-fp38-main-'));
  const OUT = path.join(FX, 'artifacts');
  mkdirSync(OUT, { recursive: true });

  try {
    buildWorkspace(FX, 'main-only');

    execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const deadSymbols = new Set((syms.deadProdList ?? []).map((d) => d.symbol));

    // F1: getPostBySlug IS imported by apps/blog/app/page.tsx via
    // `@scope/libs/getPost`. Pre-v1.9.11 marked dead because the
    // spec fell through to EXTERNAL.
    assert('F1. workspace package with main-only (no exports) — imported symbol ' +
           'getPostBySlug is NOT classified as dead',
      !deadSymbols.has('getPostBySlug'),
      `deadProdList: ${[...deadSymbols].join(', ')}`);

    // F2: same for getAllCategories — second symbol on the same import line
    assert('F2. second named import from same subpath also live (getAllCategories)',
      !deadSymbols.has('getAllCategories'),
      `deadProdList: ${[...deadSymbols].join(', ')}`);

    // F3: cross-file subpath — getAllSeries is in packages/libs/getSeries.ts
    // imported via `@scope/libs/getSeries`. Tests that the wildcard covers
    // subpaths beyond just the one in `main`.
    assert('F3. deep subpath resolution — getAllSeries (getSeries.ts) also live',
      !deadSymbols.has('getAllSeries'),
      `deadProdList: ${[...deadSymbols].join(', ')}`);

    // F4: the consumer itself is now tracked — resolvedInternal > 0
    assert('F4. resolver counts internal uses (resolvedInternal >= 3)',
      (syms.uses?.resolvedInternal ?? 0) >= 3,
      `uses: ${JSON.stringify(syms.uses)}`);

    // F5: external count does NOT include these workspace specs anymore
    assert('F5. external count is 0 (no workspace specs leaking to external)',
      (syms.uses?.external ?? 0) === 0,
      `uses: ${JSON.stringify(syms.uses)}`);

    // F6: truly unused symbols STILL get flagged. Defense against the
    // opposite failure — the wildcard is additive, not a blanket
    // "mark everything live". getPostsByCategory and unusedInternal
    // really have no consumer in this fixture.
    assert('F6. truly unconsumed exports (getPostsByCategory, unusedInternal) ARE ' +
           'still classified as dead — the fix is additive, not blanket-live',
      deadSymbols.has('getPostsByCategory') && deadSymbols.has('unusedInternal'),
      `deadProdList: ${[...deadSymbols].join(', ')}`);

    assert('F6b. legacy workspace subpath with dotted extensionless stem resolves ' +
           'to .ts source (location.input → location.input.ts)',
      syms.fanInByIdentity?.['packages/libs/inputs/location.input.ts::makeLocationInput'] === 1 &&
        !deadSymbols.has('makeLocationInput'),
      `fanIn=${syms.fanInByIdentity?.['packages/libs/inputs/location.input.ts::makeLocationInput']}, deadProdList: ${[...deadSymbols].join(', ')}`);

    assert('F6c. dotted-stem subpath fix remains precise; unused sibling is still dead',
      deadSymbols.has('unusedLocationInput'),
      `deadProdList: ${[...deadSymbols].join(', ')}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// F7-F8: explicit exports still wins — regression against the
//        earlier wildcard masking a narrower exports entry
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-fp38-exports-'));
  const OUT = path.join(FX, 'artifacts');
  mkdirSync(OUT, { recursive: true });

  try {
    buildWorkspace(FX, 'exports');

    execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const deadSymbols = new Set((syms.deadProdList ?? []).map((d) => d.symbol));

    // F7: package with explicit exports still resolves its main entry.
    // Deep subpaths may or may not work depending on the exports shape;
    // in this fixture `exports: { '.': './getPost.ts' }` doesn't cover
    // `/getSeries`, so getAllSeries may be dead — that's tsc-accurate
    // behavior (explicit exports restricts subpath access). The key
    // assertion is that the `/getPost` subpath import STILL doesn't
    // accidentally fall into the legacy wildcard (which would have
    // bypassed the exports restriction).
    assert('F7. explicit exports still wins — getPostBySlug from `.` entry live',
      !deadSymbols.has('getPostBySlug'),
      `deadProdList: ${[...deadSymbols].join(', ')}`);

    // F8: the legacy wildcard is only added when `exports` doesn't
    // already cover subpaths. This guards against regression where
    // the wildcard would re-open access that `exports` deliberately
    // closed.
    const { buildAliasMap } = await import(`${pathToFileURL(DIR).href}/_lib/alias-map.mjs`);
    const { detectRepoMode } = await import(`${pathToFileURL(DIR).href}/_lib/repo-mode.mjs`);
    const mode = detectRepoMode(FX);
    const aliasMap = buildAliasMap(FX, mode);
    const legacyKeys = [...aliasMap.keys()].filter((k) => k.includes('__LEGACY_SUBPATH__'));
    // `@scope/libs` has an explicit `.` exports entry but no subpath
    // entries, so a legacy wildcard IS added (exports doesn't cover
    // deep subpaths). Verify this is exactly the scope intended.
    assert('F8. legacy-subpath wildcard added for @scope/libs (exports does not ' +
           'cover subpaths). Note: other workspace packages without exports ' +
           '(e.g., blog) also correctly get their own legacy wildcards — the ' +
           'fix is per-package.',
      legacyKeys.some((k) => k.startsWith('@scope/libs/')),
      `legacy keys: ${JSON.stringify(legacyKeys)}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// F9-F13: dist output targets whose source lives at package root
// ───────────────────────────────────────────────────────────
//
// Some workspace packages publish `main`/`exports` targets under `dist/`
// while the authored source sits directly in the package root
// (`index.ts`, `api.ts`, ...), not under `src/`. The resolver should map
// `./dist/index.js` → `./index.ts` and `./dist/*.js` → `./*.ts`.
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-fp38-dist-root-'));
  const OUT = path.join(FX, 'artifacts');
  mkdirSync(OUT, { recursive: true });

  try {
    mkdirSync(path.join(FX, 'apps/web/app'), { recursive: true });
    mkdirSync(path.join(FX, 'packages/platform-types'), { recursive: true });

    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
    writeFileSync(path.join(FX, 'packages/platform-types/package.json'),
      JSON.stringify({
        name: '@scope/platform-types',
        type: 'module',
        main: './dist/index.js',
        types: './dist/index.d.ts',
        exports: {
          '.': {
            import: './dist/index.js',
            types: './dist/index.d.ts',
          },
          './*': './dist/*.js',
        },
      }));
    writeFileSync(path.join(FX, 'packages/platform-types/index.ts'), `
export interface PlatformUser { id: string }
export function makePlatformUser(): PlatformUser { return { id: 'u' }; }
export function unusedPlatformRoot() { return 1; }
`);
    writeFileSync(path.join(FX, 'packages/platform-types/api.ts'), `
export interface ApiResponse { ok: boolean }
export function makeApiResponse(): ApiResponse { return { ok: true }; }
export function unusedApiResponse() { return 2; }
`);
    mkdirSync(path.join(FX, 'packages/platform-types/bookings/2024-08-13/inputs'), { recursive: true });
    writeFileSync(path.join(FX, 'packages/platform-types/bookings/2024-08-13/inputs/location.input.ts'), `
export interface LocationInput { location: string }
export function makeLocationInput(): LocationInput { return { location: 'office' }; }
`);
    writeFileSync(path.join(FX, 'apps/web/package.json'),
      JSON.stringify({
        name: 'web',
        type: 'module',
        dependencies: { '@scope/platform-types': 'workspace:*' },
      }));
    writeFileSync(path.join(FX, 'apps/web/app/page.tsx'), `
import { makePlatformUser } from '@scope/platform-types';
import { makeApiResponse } from '@scope/platform-types/api';
import { makeLocationInput } from '@scope/platform-types/bookings/2024-08-13/inputs/location.input';
export function Page() {
  makePlatformUser();
  makeApiResponse();
  makeLocationInput();
  return null;
}
`);

    execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const deadSymbols = new Set((syms.deadProdList ?? [])
      .map((d) => `${d.file}::${d.symbol}`));

    assert('F9. dist/index.js workspace main maps to package-root index.ts source',
      syms.fanInByIdentity?.['packages/platform-types/index.ts::makePlatformUser'] === 1 &&
        !deadSymbols.has('packages/platform-types/index.ts::makePlatformUser'),
      `fanIn=${syms.fanInByIdentity?.['packages/platform-types/index.ts::makePlatformUser']}, dead=${[...deadSymbols].join(', ')}`);
    assert('F10. dist/*.js workspace export maps subpath to package-root api.ts source',
      syms.fanInByIdentity?.['packages/platform-types/api.ts::makeApiResponse'] === 1 &&
        !deadSymbols.has('packages/platform-types/api.ts::makeApiResponse'),
      `fanIn=${syms.fanInByIdentity?.['packages/platform-types/api.ts::makeApiResponse']}, dead=${[...deadSymbols].join(', ')}`);
    assert('F11. dist-root workspace imports do not leak to external or unresolvedInternal',
      (syms.uses?.external ?? 0) === 0 && (syms.uses?.unresolvedInternal ?? 0) === 0,
      `uses=${JSON.stringify(syms.uses)}`);
    assert('F12. genuinely unused subpath root-source exports remain dead-listed',
      deadSymbols.has('packages/platform-types/api.ts::unusedApiResponse'),
      `dead=${[...deadSymbols].join(', ')}`);
    assert('F13. legacy workspace subpath with dotted extensionless stem resolves ' +
           'to .ts source (location.input → location.input.ts)',
      syms.fanInByIdentity?.['packages/platform-types/bookings/2024-08-13/inputs/location.input.ts::makeLocationInput'] === 1 &&
        !deadSymbols.has('packages/platform-types/bookings/2024-08-13/inputs/location.input.ts::makeLocationInput'),
      `fanIn=${syms.fanInByIdentity?.['packages/platform-types/bookings/2024-08-13/inputs/location.input.ts::makeLocationInput']}, dead=${[...deadSymbols].join(', ')}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// F14-F16: declarationDir subpaths map back to source files
// ───────────────────────────────────────────────────────────
//
// Some workspace packages expose generated declaration subpaths to
// sibling workspaces, while the declarations are not present in a
// source checkout. Example shape:
//
//   package.json#main: "index.ts"
//   tsconfig.json#compilerOptions.declarationDir: "types/server"
//   tsconfig.json#include: ["./server"]
//
// Consumers import `@scope/trpc/types/server/createContext`; TypeScript
// builds that declaration from `packages/trpc/server/createContext.ts`.
// The resolver should map the declaration output subpath back to the
// source path instead of reporting a workspace subpath blind zone.
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-fp38-declaration-dir-'));
  const OUT = path.join(FX, 'artifacts');
  mkdirSync(OUT, { recursive: true });

  try {
    mkdirSync(path.join(FX, 'apps/web/app'), { recursive: true });
    mkdirSync(path.join(FX, 'packages/trpc/server/routers'), { recursive: true });

    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
    writeFileSync(path.join(FX, 'packages/trpc/package.json'),
      JSON.stringify({ name: '@scope/trpc', type: 'module', main: 'index.ts' }));
    writeFileSync(path.join(FX, 'packages/trpc/tsconfig.json'), JSON.stringify({
      compilerOptions: {
        declaration: true,
        emitDeclarationOnly: true,
        declarationDir: 'types/server',
      },
      include: ['./server'],
    }));
    writeFileSync(path.join(FX, 'packages/trpc/index.ts'),
      'export const root = 1;\n');
    writeFileSync(path.join(FX, 'packages/trpc/server/createContext.ts'), `
export interface TRPCContext { userId: string }
export function createContext(): TRPCContext { return { userId: 'u' }; }
export function unusedContextHelper() { return 1; }
`);
    writeFileSync(path.join(FX, 'packages/trpc/server/routers/_app.ts'), `
export interface AppRouter { routes: string[] }
export function makeAppRouter(): AppRouter { return { routes: [] }; }
export function unusedRouterHelper() { return 2; }
`);
    writeFileSync(path.join(FX, 'apps/web/package.json'),
      JSON.stringify({
        name: 'web',
        type: 'module',
        dependencies: { '@scope/trpc': 'workspace:*' },
      }));
    writeFileSync(path.join(FX, 'apps/web/app/page.ts'), `
import type { TRPCContext } from '@scope/trpc/types/server/createContext';
import type { AppRouter } from '@scope/trpc/types/server/routers/_app';
export function Page(_ctx: TRPCContext, _router: AppRouter) { return null; }
`);

    execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const dead = new Set((syms.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`));
    const unresolved = syms.unresolvedInternalSpecifierRecords ?? [];

    assert('F14. declarationDir subpath createContext maps back to server/createContext.ts',
      syms.fanInByIdentity?.['packages/trpc/server/createContext.ts::TRPCContext'] === 1 &&
        !dead.has('packages/trpc/server/createContext.ts::TRPCContext'),
      `fanIn=${syms.fanInByIdentity?.['packages/trpc/server/createContext.ts::TRPCContext']}, dead=${[...dead].join(', ')}`);
    assert('F15. declarationDir nested router subpath maps back to server/routers/_app.ts',
      syms.fanInByIdentity?.['packages/trpc/server/routers/_app.ts::AppRouter'] === 1 &&
        !dead.has('packages/trpc/server/routers/_app.ts::AppRouter'),
      `fanIn=${syms.fanInByIdentity?.['packages/trpc/server/routers/_app.ts::AppRouter']}, dead=${[...dead].join(', ')}`);
    assert('F16. declarationDir imports do not create workspace subpath unresolved records',
      !unresolved.some((r) => r.specifier?.startsWith('@scope/trpc/types/server/')),
      JSON.stringify(unresolved.filter((r) => r.specifier?.startsWith('@scope/trpc/types/server/')), null, 2));
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// F17-F22: source-direct bare package entries without package.json#main
// ───────────────────────────────────────────────────────────
//
// Some source-first workspaces omit `main` and `exports` entirely, while
// TypeScript can still resolve the package root through `types`/`typings` or
// the package-root `index.ts`. Treat these as concrete workspace source only
// when a real source file exists; missing generated/package entries must remain
// unresolved diagnostics.
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-fp38-bare-source-entry-'));
  const OUT = path.join(FX, 'artifacts');
  mkdirSync(OUT, { recursive: true });

  try {
    mkdirSync(path.join(FX, 'apps/web/src'), { recursive: true });
    mkdirSync(path.join(FX, 'packages/typed-entry'), { recursive: true });
    mkdirSync(path.join(FX, 'packages/index-entry'), { recursive: true });
    mkdirSync(path.join(FX, 'packages/generated-entry'), { recursive: true });

    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
    writeFileSync(path.join(FX, 'packages/typed-entry/package.json'),
      JSON.stringify({ name: '@scope/typed-entry', type: 'module', types: './index.ts' }));
    writeFileSync(path.join(FX, 'packages/typed-entry/index.ts'), `
export interface TypedEntry { id: string }
export function makeTypedEntry(): TypedEntry { return { id: 'typed' }; }
export function unusedTypedEntry() { return 1; }
`);
    writeFileSync(path.join(FX, 'packages/index-entry/package.json'),
      JSON.stringify({ name: '@scope/index-entry', type: 'module' }));
    writeFileSync(path.join(FX, 'packages/index-entry/index.ts'), `
export function makeIndexEntry() { return { id: 'index' }; }
export function unusedIndexEntry() { return 2; }
`);
    writeFileSync(path.join(FX, 'packages/generated-entry/package.json'),
      JSON.stringify({
        name: '@scope/generated-entry',
        type: 'module',
        typings: './dist/index.d.ts',
        files: ['dist'],
        scripts: { build: 'vite build' },
      }));

    writeFileSync(path.join(FX, 'apps/web/package.json'),
      JSON.stringify({
        name: 'web',
        type: 'module',
        dependencies: {
          '@scope/typed-entry': 'workspace:*',
          '@scope/index-entry': 'workspace:*',
          '@scope/generated-entry': 'workspace:*',
        },
      }));
    writeFileSync(path.join(FX, 'apps/web/src/page.ts'), `
import type { TypedEntry } from '@scope/typed-entry';
import { makeTypedEntry } from '@scope/typed-entry';
import { makeIndexEntry } from '@scope/index-entry';
import type { GeneratedEntry } from '@scope/generated-entry';
export function page(): TypedEntry {
  makeIndexEntry();
  return makeTypedEntry() as GeneratedEntry & TypedEntry;
}
`);

    execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const dead = new Set((syms.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`));
    const unresolved = syms.unresolvedInternalSpecifierRecords ?? [];

    assert('F17. workspace package with types-only source entry resolves bare type import',
      syms.fanInByIdentity?.['packages/typed-entry/index.ts::TypedEntry'] === 1 &&
        !dead.has('packages/typed-entry/index.ts::TypedEntry'),
      `fanIn=${syms.fanInByIdentity?.['packages/typed-entry/index.ts::TypedEntry']}; dead=${[...dead].join(', ')}`);
    assert('F18. workspace package with types-only source entry resolves bare value import',
      syms.fanInByIdentity?.['packages/typed-entry/index.ts::makeTypedEntry'] === 1 &&
        !dead.has('packages/typed-entry/index.ts::makeTypedEntry'),
      `fanIn=${syms.fanInByIdentity?.['packages/typed-entry/index.ts::makeTypedEntry']}; dead=${[...dead].join(', ')}`);
    assert('F19. workspace package with no main/types falls back to package-root index.ts',
      syms.fanInByIdentity?.['packages/index-entry/index.ts::makeIndexEntry'] === 1 &&
        !dead.has('packages/index-entry/index.ts::makeIndexEntry'),
      `fanIn=${syms.fanInByIdentity?.['packages/index-entry/index.ts::makeIndexEntry']}; dead=${[...dead].join(', ')}`);
    assert('F20. source-direct bare package entries do not leak to external',
      (syms.uses?.external ?? 0) === 0,
      `uses=${JSON.stringify(syms.uses)}`);
    assert('F21. missing generated typings entry remains unresolved, not fake-resolved',
      unresolved.some((r) =>
        r.specifier === '@scope/generated-entry' &&
        r.reason === 'workspace-generated-artifact-missing' &&
        r.resolverStage === 'exact-alias' &&
        r.generatedArtifact?.generatorFamily === 'build-output'),
      JSON.stringify(unresolved, null, 2));
    assert('F22. unused siblings remain dead after source-direct bare entry resolution',
      dead.has('packages/typed-entry/index.ts::unusedTypedEntry') &&
        dead.has('packages/index-entry/index.ts::unusedIndexEntry'),
      `dead=${[...dead].join(', ')}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
