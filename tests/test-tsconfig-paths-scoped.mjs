// Regression guard for FP-36 — monorepo-local tsconfig paths.
//
// Before v1.9.7, the resolver had no tsconfig paths support at all.
// In a monorepo where each app defined its own
// `compilerOptions.paths: { "@/*": ["./*"] }`, imports like
// `@/components/auth-control` from `apps/agents/app/x.tsx` failed to
// resolve, so `AuthControl` (defined in apps/agents/components/
// auth-control.tsx) appeared to have no consumer and got classified
// as Tier C "dead export." Observed on duyet/monorepo: 218 of 397
// Tier C findings were actually consumed via per-app `@/*` aliases.
// 73.2% FP rate driven by this single resolver blind spot.
//
// v1.9.7 introduces scoped tsconfig paths: each tsconfig.json's
// compilerOptions.paths are recorded with their scope directory, and
// the resolver applies them nearest-scope-first based on the
// importing file.
//
// Fixture: two apps that SHARE the `@/*` alias but each map to their
// own components/ folder. The critical assertion is T3 — same
// specifier, different importers, different target files. A flat
// alias map cannot pass T3; only a scope-aware resolver can.

import { execSync } from 'node:child_process';
import { writeFileSync, mkdirSync, rmSync, readFileSync, mkdtempSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const FX = mkdtempSync(path.join(tmpdir(), 'fx-tsconfig-scoped-'));
const OUT = path.join(FX, 'artifacts');
mkdirSync(OUT, { recursive: true });

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

try {
  // ── Build fixture ──
  mkdirSync(path.join(FX, 'apps/agents/components'), { recursive: true });
  mkdirSync(path.join(FX, 'apps/agents/app'),        { recursive: true });
  mkdirSync(path.join(FX, 'apps/admin/components'),  { recursive: true });
  mkdirSync(path.join(FX, 'apps/admin/app'),         { recursive: true });

  writeFileSync(path.join(FX, 'package.json'),
    JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*'] }));

  writeFileSync(path.join(FX, 'apps/agents/package.json'),
    JSON.stringify({ name: 'agents', type: 'module' }));
  // v1.9.10: fixture uses real-world JSONC shape with $schema URL and
  // a trailing line comment. Before the jsonc-parser switch, fixtures
  // were plain JSON — which let the FP-36 fix look green in tests
  // while duyet/monorepo's actual tsconfigs (full JSONC with $schema)
  // silently fell through. This shape now locks in the realistic
  // case.
  writeFileSync(path.join(FX, 'apps/agents/tsconfig.json'),
`{
  "$schema": "https://json.schemastore.org/tsconfig",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules", "**/*.test.ts"]
}
`);
  // agents app-local AuthControl
  writeFileSync(path.join(FX, 'apps/agents/components/auth-control.tsx'),
    'export function AuthControl() { return null; }\n');
  writeFileSync(path.join(FX, 'apps/agents/app/chat-top-bar.tsx'),
    "import { AuthControl } from '@/components/auth-control';\n" +
    'export function ChatTopBar() { return AuthControl(); }\n');

  writeFileSync(path.join(FX, 'apps/admin/package.json'),
    JSON.stringify({ name: 'admin', type: 'module' }));
  writeFileSync(path.join(FX, 'apps/admin/tsconfig.json'),
`{
  "$schema": "https://json.schemastore.org/tsconfig",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"]
    }
  },
  "include": ["**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules", "**/*.test.ts"]
}
`);
  // admin app-local AuthControl — same symbol name, different file
  writeFileSync(path.join(FX, 'apps/admin/components/auth-control.tsx'),
    'export function AuthControl() { return "admin"; }\n');
  writeFileSync(path.join(FX, 'apps/admin/app/sidebar.tsx'),
    "import { AuthControl } from '@/components/auth-control';\n" +
    'export function Sidebar() { return AuthControl(); }\n');

  // ── Run pipeline ──
  execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
    { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));

  // Direct unit-level check of the resolver using the same module
  // the pipeline uses, to make T1/T2/T3 explicit rather than inferred.
  const { buildAliasMap } = await import(`${pathToFileURL(DIR).href}/_lib/alias-map.mjs`);
  const { makeResolver } = await import(`${pathToFileURL(DIR).href}/_lib/resolver-core.mjs`);
  const { detectRepoMode } = await import(`${pathToFileURL(DIR).href}/_lib/repo-mode.mjs`);
  const repoMode = detectRepoMode(FX);
  const aliasMap = buildAliasMap(FX, repoMode);
  const resolve = makeResolver(FX, aliasMap);

  const agentsImporter = path.join(FX, 'apps/agents/app/chat-top-bar.tsx');
  const adminImporter = path.join(FX, 'apps/admin/app/sidebar.tsx');

  const agentsResolved = resolve(agentsImporter, '@/components/auth-control');
  const adminResolved = resolve(adminImporter, '@/components/auth-control');

  // T1: agents-side import resolves to agents' component
  assert('T1. agents @/components/auth-control → apps/agents/components/auth-control.tsx',
    typeof agentsResolved === 'string' &&
    agentsResolved.replace(/\\/g, '/').includes('apps/agents/components/auth-control.tsx'),
    `got: ${agentsResolved}`);

  // T2: admin-side import resolves to admin's component
  assert('T2. admin @/components/auth-control → apps/admin/components/auth-control.tsx',
    typeof adminResolved === 'string' &&
    adminResolved.replace(/\\/g, '/').includes('apps/admin/components/auth-control.tsx'),
    `got: ${adminResolved}`);

  // T3: CRITICAL — same specifier, different importers, DIFFERENT targets.
  // A flat alias map cannot satisfy this. Only a scope-aware resolver can.
  assert('T3. same `@/components/auth-control` spec resolves to different files ' +
         'based on importer scope (FP-36 critical invariant)',
    agentsResolved !== adminResolved &&
    !agentsResolved.includes('admin') &&
    !adminResolved.includes('agents'),
    `agents=${agentsResolved}\n        admin=${adminResolved}`);

  // T4: AuthControl must NOT appear in deadProdList (consumer found,
  // therefore not dead). This is the end-to-end outcome the duyet
  // case study measured as 218 FPs.
  const deadSymbols = new Set((syms.deadProdList ?? []).map((d) => d.symbol));
  assert('T4. AuthControl is NOT classified as dead (consumer found)',
    !deadSymbols.has('AuthControl'),
    `deadProdList: ${JSON.stringify([...deadSymbols].slice(0, 20))}`);

  // T5: unresolvedInternal is 0 for these imports — the @/* aliases
  // are supposed to resolve cleanly in both apps.
  assert('T5. uses.unresolvedInternal includes 0 for these `@/*` imports',
    syms.uses?.unresolvedInternal === 0 &&
    syms.uses?.unresolvedInternalRatio === 0,
    `uses: ${JSON.stringify(syms.uses)}`);

  // T6: EXTERNAL vs UNRESOLVED_INTERNAL sentinel distinction. If a
  // tsconfig paths pattern matches but the target file doesn't
  // exist, the resolver must return UNRESOLVED_INTERNAL rather than
  // falling through to EXTERNAL. This is the other side of FP-36:
  // an internal-looking alias is a scanner blind spot we want to
  // surface, not silently quiet.
  const bogus = resolve(agentsImporter, '@/components/does-not-exist');
  assert('T6. matched local alias with missing target → UNRESOLVED_INTERNAL (not EXTERNAL)',
    bogus === 'UNRESOLVED_INTERNAL',
    `got: ${bogus}`);

  // T7: genuine external package still goes through as EXTERNAL.
  // Sanity: we didn't accidentally broaden UNRESOLVED_INTERNAL to
  // catch all npm imports.
  const ext = resolve(agentsImporter, 'react');
  assert('T7. genuine external package (react) → EXTERNAL',
    ext === 'EXTERNAL',
    `got: ${ext}`);

  const scopedTsconfigStats = resolve.stageStats?.().scopedTsconfig ?? {};
  assert('T7b. scoped tsconfig records matched pattern attempts',
    scopedTsconfigStats.patternMatches >= 3,
    `scopedTsconfigStats=${JSON.stringify(scopedTsconfigStats)}`);
  assert('T7c. scoped tsconfig records target probe hits',
    scopedTsconfigStats.probeHits >= 2,
    `scopedTsconfigStats=${JSON.stringify(scopedTsconfigStats)}`);
  assert('T7d. scoped tsconfig records target probe misses',
    scopedTsconfigStats.probeMisses >= 1,
    `scopedTsconfigStats=${JSON.stringify(scopedTsconfigStats)}`);

  const agentsSiblingImporter = path.join(FX, 'apps/agents/app/chat-side-panel.tsx');
  const agentsSiblingResolved = resolve(agentsSiblingImporter, '@/components/auth-control');
  const scopedTsconfigStatsAfterCache = resolve.stageStats?.().scopedTsconfig ?? {};
  assert('T7e. same scoped tsconfig specifier from a sibling importer resolves to the same target',
    agentsSiblingResolved === agentsResolved,
    `first=${agentsResolved}\n        sibling=${agentsSiblingResolved}`);
  assert('T7f. sibling scoped tsconfig lookup uses the stage probe cache, not only the top-level resolver memo',
    scopedTsconfigStatsAfterCache.cacheHits > scopedTsconfigStats.cacheHits,
    `before=${JSON.stringify(scopedTsconfigStats)}\n        after=${JSON.stringify(scopedTsconfigStatsAfterCache)}`);
  assert('T7g. scoped tsconfig cache hit does not add another target probe hit',
    scopedTsconfigStatsAfterCache.probeHits === scopedTsconfigStats.probeHits,
    `before=${JSON.stringify(scopedTsconfigStats)}\n        after=${JSON.stringify(scopedTsconfigStatsAfterCache)}`);

  // T8: FP-37 — JSONC parser must handle tsconfigs containing `*/`
  // sequences inside string literals (paths like "@/*" and globs like
  // "**/*.ts" in `include`). Regex-based comment stripping matches
  // `/*` in `"@/*":` with `*/` in `"**/*.ts"` and eats the JSON
  // between them. Pre-v1.9.10 parser silently returned null for
  // these tsconfigs, so scoped paths were never registered for
  // apps/* in real-world monorepos. Direct unit test of the parser
  // so the regression is locked in independently of the pipeline.
  {
    const { discoverScopedTsconfigPaths } = await import(`${pathToFileURL(DIR).href}/_lib/tsconfig-paths.mjs`);
    const entries = discoverScopedTsconfigPaths(FX);
    // agents + admin = 2 scoped entries. If parser failed on either
    // tsconfig, count drops to 0 or 1.
    assert('T8. FP-37 — realistic tsconfigs with "@/*" paths AND "**/*.ts" include both parse ' +
           '(regex-strip parser returned 0; jsonc-parser returns 2)',
      entries.length === 2,
      `got ${entries.length} scoped entries; expected 2 (one per app)`);
  }

  // ─── T9-T11: extends-inherited paths + hoisted node_modules ───
  //
  // v1.9.10 rewrote tsconfig loading atop `ts.parseJsonConfigFileContent`
  // because hand-rolled `extends` resolution failed on Bun/pnpm
  // workspaces where node_modules is hoisted to the repo root. In that
  // layout, `apps/<X>/node_modules/@shared/tsconfig` does NOT exist —
  // only `<root>/node_modules/@shared/tsconfig` does. The old code
  // looked per-app and silently dropped the extends, losing inherited
  // `paths`.
  //
  // TypeScript's own resolver walks up from the config's directory
  // looking for `node_modules`, so it finds the root-hoisted copy.
  // These tests exercise that behavior without needing to rebuild
  // duyet/monorepo in the test harness.
  {
    const FX2 = mkdtempSync(path.join(tmpdir(), 'fx-extends-hoisted-'));
    try {
      // Workspace root with hoisted node_modules (Bun/pnpm pattern)
      mkdirSync(path.join(FX2, 'apps/agents'), { recursive: true });
      mkdirSync(path.join(FX2, 'node_modules/@shared/tsconfig'), { recursive: true });

      writeFileSync(path.join(FX2, 'package.json'),
        JSON.stringify({ name: 'root', workspaces: ['apps/*'] }));

      // Shared config in hoisted node_modules — defines BOTH baseUrl
      // and paths. Note: TS resolves these relative to wherever the
      // extending config sets baseUrl (if any).
      writeFileSync(path.join(FX2, 'node_modules/@shared/tsconfig/base.json'),
        JSON.stringify({
          compilerOptions: { paths: { '@shared/*': ['./*'] } },
        }));
      writeFileSync(path.join(FX2, 'node_modules/@shared/tsconfig/package.json'),
        JSON.stringify({ name: '@shared/tsconfig' }));

      // App that extends the shared config AND adds its own local
      // paths + baseUrl — typical real-world shape.
      writeFileSync(path.join(FX2, 'apps/agents/package.json'),
        JSON.stringify({ name: 'agents' }));
      writeFileSync(path.join(FX2, 'apps/agents/tsconfig.json'),
        JSON.stringify({
          extends: '@shared/tsconfig/base.json',
          compilerOptions: {
            baseUrl: '.',
            paths: { '@/*': ['./*'] },
          },
        }));

      const { discoverScopedTsconfigPaths } = await import(`${pathToFileURL(DIR).href}/_lib/tsconfig-paths.mjs?v=extends-hoisted`);
      const entries = discoverScopedTsconfigPaths(FX2);
      const forAgents = entries.filter((e) =>
        e.configPath.includes('apps/agents/tsconfig.json'));

      // T9: the app's tsconfig produced scoped entries
      assert('T9. app tsconfig extending a hoisted-node_modules config produces scoped entries ' +
             '(hand-rolled extends resolver used to silently drop this case)',
        forAgents.length >= 1,
        `got ${forAgents.length} entries for apps/agents; full list: ${JSON.stringify(entries.map(e => e.key))}`);

      // T10: the local `@/*` is present — proves the local config's
      // paths were preserved through the TS config resolution pass.
      assert('T10. local `@/*` alias preserved when extends resolution runs',
        forAgents.some((e) => e.key === '@/*'),
        `agents entries: ${JSON.stringify(forAgents.map(e => e.key))}`);

      // T11: TypeScript semantics — when both extending AND extended
      // configs define `paths`, the extending config's paths COMPLETELY
      // REPLACE the extended config's (they do not merge). This is
      // the rule in tsc itself. The pre-v1.9.10 hand-rolled loader
      // tried to merge, which would have caused a different kind of
      // FP (inherited aliases leaking into scopes that shouldn't have
      // them). v1.9.10 matches tsc by construction.
      assert('T11. tsc semantics: extending config\'s paths REPLACE extended config\'s (not merge) ' +
             'so `@shared/*` is NOT present when local paths is defined',
        !forAgents.some((e) => e.key === '@shared/*'),
        `agents entries: ${JSON.stringify(forAgents.map(e => e.key))} (unexpected: '@shared/*' leaked through merge)`);

      // T12: when LOCAL config has NO paths, THEN inherited paths
      // from extends should appear. This is the scenario where
      // pre-v1.9.10 would silently produce empty paths → every `@/*`
      // import falls through to EXTERNAL → FP-36 symptom on duyet.
      writeFileSync(path.join(FX2, 'apps/agents/tsconfig.json'),
        JSON.stringify({
          extends: '@shared/tsconfig/base.json',
          compilerOptions: { baseUrl: '.' },
        }));
      const { discoverScopedTsconfigPaths: discover2 } = await import(
        `${pathToFileURL(DIR).href}/_lib/tsconfig-paths.mjs?v=extends-only`);
      const entries2 = discover2(FX2);
      const forAgents2 = entries2.filter((e) =>
        e.configPath.includes('apps/agents/tsconfig.json'));
      assert('T12. extends-only config (no local paths) inherits paths from hoisted shared config ' +
             '(this was the silent-drop failure mode on duyet/monorepo)',
        forAgents2.some((e) => e.key === '@shared/*'),
        `agents entries with extends-only config: ${JSON.stringify(forAgents2.map(e => e.key))}`);
    } finally {
      rmSync(FX2, { recursive: true, force: true });
    }
  }

  // ─── T13-T18: baseUrl-only scoped imports ───
  //
  // App packages often use `compilerOptions.baseUrl: "."` without
  // a matching `paths` entry, then import `app/_types` or
  // `app/_trpc/context` from inside that app. TypeScript resolves those
  // against the app's baseUrl. The resolver must do the same; otherwise
  // the import falls through as EXTERNAL, so the dead-export classifier
  // misses real consumers and the unresolvedInternalRatio under-reports
  // the blind zone.
  {
    const FX3 = mkdtempSync(path.join(tmpdir(), 'fx-baseurl-only-'));
    const OUT3 = path.join(FX3, 'artifacts');
    try {
      mkdirSync(path.join(FX3, 'apps/web/app/_trpc'), { recursive: true });
      mkdirSync(OUT3, { recursive: true });
      writeFileSync(path.join(FX3, 'package.json'),
        JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*'] }));
      writeFileSync(path.join(FX3, 'apps/web/package.json'),
        JSON.stringify({ name: 'web', type: 'module' }));
      writeFileSync(path.join(FX3, 'apps/web/tsconfig.json'), JSON.stringify({
        compilerOptions: { baseUrl: '.' },
        include: ['**/*.ts', '**/*.tsx'],
      }));
      writeFileSync(path.join(FX3, 'apps/web/app/_types.ts'),
        'export interface PageProps { params: { slug: string } }\n');
      writeFileSync(path.join(FX3, 'apps/web/app/_trpc/context.ts'),
        'export function getTRPCContext() { return { ok: true }; }\n');
      writeFileSync(path.join(FX3, 'apps/web/app/page.ts'),
        "import type { PageProps } from 'app/_types';\n" +
        "import { getTRPCContext } from 'app/_trpc/context';\n" +
        'export function Page(_props: PageProps) { return getTRPCContext().ok; }\n');

      execSync(`node build-symbol-graph.mjs --root ${FX3} --output ${OUT3}`,
        { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

      const syms3 = JSON.parse(readFileSync(path.join(OUT3, 'symbols.json'), 'utf8'));
      const repoMode3 = detectRepoMode(FX3);
      const aliasMap3 = buildAliasMap(FX3, repoMode3);
      const resolve3 = makeResolver(FX3, aliasMap3);
      const importer = path.join(FX3, 'apps/web/app/page.ts');
      const typesResolved = resolve3(importer, 'app/_types');
      const trpcResolved = resolve3(importer, 'app/_trpc/context');
      const missingResolved = resolve3(importer, 'app/does-not-exist');
      const externalResolved = resolve3(importer, 'react');
      const deadIdentities = new Set((syms3.deadProdList ?? [])
        .map((d) => `${d.file}::${d.symbol}`));

      assert('T13. baseUrl-only `app/_types` resolves inside the importing app scope',
        typeof typesResolved === 'string' &&
          typesResolved.replace(/\\/g, '/').endsWith('apps/web/app/_types.ts'),
        `got: ${typesResolved}`);
      assert('T14. baseUrl-only `app/_trpc/context` resolves inside the importing app scope',
        typeof trpcResolved === 'string' &&
          trpcResolved.replace(/\\/g, '/').endsWith('apps/web/app/_trpc/context.ts'),
        `got: ${trpcResolved}`);
      assert('T15. baseUrl-only type import contributes fan-in and is not dead-listed',
        syms3.fanInByIdentity?.['apps/web/app/_types.ts::PageProps'] === 1 &&
          !deadIdentities.has('apps/web/app/_types.ts::PageProps'),
        `fanIn=${syms3.fanInByIdentity?.['apps/web/app/_types.ts::PageProps']}, dead=${[...deadIdentities].join(', ')}`);
      assert('T15b. baseUrl-only type import contributes type-space fan-in only',
        syms3.fanInByIdentitySpace?.['apps/web/app/_types.ts::PageProps']?.type === 1 &&
          syms3.fanInByIdentitySpace?.['apps/web/app/_types.ts::PageProps']?.value === 0 &&
          syms3.fanInByIdentitySpace?.['apps/web/app/_types.ts::PageProps']?.broad === 0,
        JSON.stringify(syms3.fanInByIdentitySpace?.['apps/web/app/_types.ts::PageProps']));
      assert('T16. baseUrl-only value import contributes fan-in and is not dead-listed',
        syms3.fanInByIdentity?.['apps/web/app/_trpc/context.ts::getTRPCContext'] === 1 &&
          !deadIdentities.has('apps/web/app/_trpc/context.ts::getTRPCContext'),
        `fanIn=${syms3.fanInByIdentity?.['apps/web/app/_trpc/context.ts::getTRPCContext']}, dead=${[...deadIdentities].join(', ')}`);
      assert('T16b. baseUrl-only value import contributes value-space fan-in only',
        syms3.fanInByIdentitySpace?.['apps/web/app/_trpc/context.ts::getTRPCContext']?.value === 1 &&
          syms3.fanInByIdentitySpace?.['apps/web/app/_trpc/context.ts::getTRPCContext']?.type === 0 &&
          syms3.fanInByIdentitySpace?.['apps/web/app/_trpc/context.ts::getTRPCContext']?.broad === 0,
        JSON.stringify(syms3.fanInByIdentitySpace?.['apps/web/app/_trpc/context.ts::getTRPCContext']));
      assert('T17. missing baseUrl-local specifier is UNRESOLVED_INTERNAL, not silent EXTERNAL',
        missingResolved === 'UNRESOLVED_INTERNAL',
        `got: ${missingResolved}`);
      assert('T18. genuine npm import still falls through as EXTERNAL under baseUrl',
        externalResolved === 'EXTERNAL' &&
          syms3.uses?.unresolvedInternal === 0 &&
          syms3.uses?.unresolvedInternalRatio === 0,
        `external=${externalResolved}, uses=${JSON.stringify(syms3.uses)}`);
    } finally {
      rmSync(FX3, { recursive: true, force: true });
    }
  }

  // T19-T20: invalid tsconfig fixtures must not crash discovery.
  //
  // Astro's repo carries deliberately invalid tsconfig fixtures under
  // package test directories. On Windows, TypeScript 5.9 can throw a
  // Debug Failure from `ts.readConfigFile` for those malformed configs
  // before returning a normal diagnostic object. The resolver must treat
  // that as "this config is not usable for alias discovery" and continue,
  // not abort the required symbol-graph step.
  {
    const FX4 = mkdtempSync(path.join(tmpdir(), 'fx-invalid-tsconfig-fixture-'));
    try {
      mkdirSync(path.join(FX4, 'app/src'), { recursive: true });
      mkdirSync(path.join(FX4, 'packages/pkg/test/fixtures/tsconfig-handling/invalid'), { recursive: true });
      writeFileSync(path.join(FX4, 'package.json'),
        JSON.stringify({ name: 'root', type: 'module' }));
      writeFileSync(path.join(FX4, 'app/tsconfig.json'), JSON.stringify({
        compilerOptions: {
          baseUrl: '.',
          paths: { '@/*': ['./src/*'] },
        },
      }));
      writeFileSync(
        path.join(FX4, 'packages/pkg/test/fixtures/tsconfig-handling/invalid/tsconfig.json'),
        '{ "compilerOptions": { "baseUrl": ".", \n',
      );

      const { discoverScopedTsconfigResolution } = await import(
        `${pathToFileURL(DIR).href}/_lib/tsconfig-paths.mjs?v=invalid-fixture`);
      let resolution;
      let error = null;
      try {
        resolution = discoverScopedTsconfigResolution(FX4);
      } catch (err) {
        error = err;
      }

      assert('T19. invalid tsconfig fixtures are skipped without crashing discovery',
        error === null,
        error?.stack ?? String(error));
      assert('T20. valid sibling tsconfig entries still load after invalid fixture is skipped',
        resolution?.paths?.some((e) => e.key === '@/*' && e.configPath.endsWith('/app/tsconfig.json')) &&
          resolution?.baseUrls?.some((e) => e.configPath.endsWith('/app/tsconfig.json')),
        `resolution=${JSON.stringify(resolution)}`);
    } finally {
      rmSync(FX4, { recursive: true, force: true });
    }
  }

  // ─── T21-T22: unresolved internal records explain WHY resolution failed ───
  //
  // Large generated/workspace-heavy monorepos can produce many
  // UNRESOLVED_INTERNAL imports. The artifact must say whether the miss came
  // from a tsconfig path target, a generated-looking target, or a workspace
  // package subpath fallback. This is intentionally generic: no repository or
  // package name is special-cased.
  {
    const FX5 = mkdtempSync(path.join(tmpdir(), 'fx-unresolved-reasons-'));
    const OUT5 = path.join(FX5, 'artifacts');
    try {
      mkdirSync(path.join(FX5, 'apps/web/src'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/generated'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/prisma-client-only'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/prisma'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/prisma-named'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/web-static/scripts'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/weak-static/scripts'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/bundle/src'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/bundle-files-only/src'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/css-output'), { recursive: true });
      mkdirSync(path.join(FX5, 'packages/types/outputs'), { recursive: true });
      mkdirSync(OUT5, { recursive: true });

      writeFileSync(path.join(FX5, 'package.json'),
        JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
      writeFileSync(path.join(FX5, 'apps/web/package.json'),
        JSON.stringify({ name: 'web', type: 'module' }));
      writeFileSync(path.join(FX5, 'apps/web/tsconfig.json'), JSON.stringify({
        compilerOptions: {
          baseUrl: '.',
          paths: {
            '@scope/generated-client': ['../../packages/generated/generated/client'],
          },
        },
      }));
      writeFileSync(path.join(FX5, 'packages/generated/package.json'),
        JSON.stringify({ name: '@scope/generated', type: 'module' }));
      writeFileSync(path.join(FX5, 'packages/prisma-client-only/package.json'),
        JSON.stringify({
          name: '@scope/prisma-client-only',
          type: 'module',
          main: 'index.ts',
          dependencies: { '@prisma/client': '1.0.0' },
        }));
      writeFileSync(path.join(FX5, 'packages/prisma-client-only/index.ts'),
        'export const prismaClientOnlyRoot = 1;\n');
      writeFileSync(path.join(FX5, 'packages/prisma/package.json'),
        JSON.stringify({
          name: '@scope/prisma',
          type: 'module',
          main: 'index.ts',
          bin: { 'prisma-enum-generator': './run-enum-generator.js' },
          prisma: { seed: 'node seed.mjs' },
          scripts: { generate: 'prisma generate' },
          dependencies: { '@prisma/client': '1.0.0' },
        }));
      writeFileSync(path.join(FX5, 'packages/prisma/index.ts'),
        'export const prismaRoot = 1;\n');
      writeFileSync(path.join(FX5, 'packages/prisma-named/package.json'),
        JSON.stringify({ name: '@scope/prisma-named', type: 'module', main: 'index.ts' }));
      writeFileSync(path.join(FX5, 'packages/prisma-named/index.ts'),
        'export const prismaNamedRoot = 1;\n');
      writeFileSync(path.join(FX5, 'packages/web-static/package.json'),
        JSON.stringify({
          name: '@scope/web-static',
          type: 'module',
          main: 'index.ts',
          scripts: { 'copy-static-assets': 'node scripts/copy-public-static.js' },
        }));
      writeFileSync(path.join(FX5, 'packages/web-static/index.ts'),
        'export const webStaticRoot = 1;\n');
      writeFileSync(path.join(FX5, 'packages/web-static/scripts/copy-public-static.js'),
        'const fs = require("node:fs");\n' +
        'const path = require("node:path");\n' +
        'const manifestPath = path.join(process.cwd(), "public", "app-store", "svg-hashes.json");\n' +
        'fs.writeFileSync(manifestPath, "{}");\n');
      writeFileSync(path.join(FX5, 'packages/weak-static/package.json'),
        JSON.stringify({
          name: '@scope/weak-static',
          type: 'module',
          main: 'index.ts',
          scripts: { 'copy-static-assets': 'node scripts/copy-static.js' },
        }));
      writeFileSync(path.join(FX5, 'packages/weak-static/index.ts'),
        'export const weakStaticRoot = 1;\n');
      writeFileSync(path.join(FX5, 'packages/weak-static/scripts/copy-static.js'),
        'console.log("copy static assets");\n');
      writeFileSync(path.join(FX5, 'packages/bundle/package.json'),
        JSON.stringify({
          name: '@scope/bundle',
          type: 'module',
          exports: { '.': { import: './dist/bundle.js', types: './dist/index.d.ts' } },
          files: ['dist'],
          scripts: { build: 'vite build' },
        }));
      writeFileSync(path.join(FX5, 'packages/bundle/src/index.ts'),
        'export const bundleSource = 1;\n');
      writeFileSync(path.join(FX5, 'packages/bundle-files-only/package.json'),
        JSON.stringify({
          name: '@scope/bundle-files-only',
          type: 'module',
          exports: { '.': './dist/bundle.js' },
          files: ['dist'],
        }));
      writeFileSync(path.join(FX5, 'packages/bundle-files-only/src/index.ts'),
        'export const bundleFilesOnlySource = 1;\n');
      writeFileSync(path.join(FX5, 'packages/css-output/package.json'),
        JSON.stringify({
          name: '@scope/css-output',
          type: 'module',
          exports: { './style.min.css': './style.min.css' },
          files: ['style.min.css'],
          scripts: { build: 'postcss ./style.css -o ./style.min.css' },
        }));
      writeFileSync(path.join(FX5, 'packages/types/package.json'),
        JSON.stringify({ name: '@scope/types', type: 'module', main: 'index.ts' }));
      writeFileSync(path.join(FX5, 'packages/types/index.ts'),
        'export const root = 1;\n');
      writeFileSync(path.join(FX5, 'packages/types/outputs/thing.ts'),
        'export interface Thing { id: string }\n');
      writeFileSync(path.join(FX5, 'apps/web/src/consumer.ts'),
        "import { missingGenerated } from '@scope/generated-client';\n" +
        "import { PrismaClientOnly } from '@scope/prisma-client-only/client';\n" +
        "import { BookingStatus } from '@scope/prisma/enums';\n" +
        "import { NamedEnums } from '@scope/prisma-named/enums';\n" +
        "import hashes from '@scope/web-static/public/app-store/svg-hashes.json';\n" +
        "import weakManifest from '@scope/weak-static/public/app-store/svg-hashes.json';\n" +
        "import { BundleRoot } from '@scope/bundle';\n" +
        "import { BundleFilesOnly } from '@scope/bundle-files-only';\n" +
        "import '@scope/css-output/style.min.css';\n" +
        "import type { Thing } from '@scope/types/thing';\n" +
        'export const uses = [missingGenerated, PrismaClientOnly, BookingStatus, NamedEnums, hashes, weakManifest, BundleRoot, BundleFilesOnly] as Thing[];\n');

      execSync(`node build-symbol-graph.mjs --root ${FX5} --output ${OUT5}`,
        { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

      const syms5 = JSON.parse(readFileSync(path.join(OUT5, 'symbols.json'), 'utf8'));
      const records = syms5.unresolvedInternalSpecifierRecords ?? [];
      const generated = records.find((r) => r.specifier === '@scope/generated-client');
      const weakDependencyOnly = records.find((r) => r.specifier === '@scope/prisma-client-only/client');
      const generatedWorkspace = records.find((r) => r.specifier === '@scope/prisma/enums');
      const weakPackageNameOnly = records.find((r) => r.specifier === '@scope/prisma-named/enums');
      const generatedStatic = records.find((r) => r.specifier === '@scope/web-static/public/app-store/svg-hashes.json');
      const weakStatic = records.find((r) => r.specifier === '@scope/weak-static/public/app-store/svg-hashes.json');
      const generatedBundle = records.find((r) => r.specifier === '@scope/bundle');
      const weakBundleFilesOnly = records.find((r) => r.specifier === '@scope/bundle-files-only');
      const generatedCssOutput = records.find((r) => r.specifier === '@scope/css-output/style.min.css');
      const workspace = records.find((r) => r.specifier === '@scope/types/thing');

      assert('T21. unresolved tsconfig path records target-missing reason and generated-artifact hint',
        generated?.reason === 'tsconfig-path-target-missing' &&
          generated?.hint === 'generated-artifact-missing' &&
          generated?.matchedPattern === '@scope/generated-client' &&
          (generated?.targetCandidates ?? []).some((p) => p.includes('packages/generated/generated/client')) &&
          generated?.generatedArtifact?.policyVersion === 'generated-artifact-policy-v1' &&
          generated?.generatedArtifact?.generatorFamily === 'path-segment' &&
          generated?.generatedArtifact?.confidence === 'supporting' &&
          generated?.generatedArtifact?.targetSubpath === 'packages/generated/generated/client' &&
          generated?.generatedArtifact?.evidence?.some((e) =>
            e.kind === 'target-path-segment' &&
            e.field === 'targetCandidates[0]' &&
            e.matched === 'generated'),
        `record=${JSON.stringify(generated)}; all=${JSON.stringify(records)}`);

      assert('T22. unresolved workspace subpath records workspace-package-subpath-target-missing reason',
        workspace?.reason === 'workspace-package-subpath-target-missing' &&
          workspace?.matchedPattern === '@scope/types/*' &&
          (workspace?.targetCandidates ?? []).some((p) => p.includes('packages/types/thing')) &&
          workspace?.typeOnly === true,
        `record=${JSON.stringify(workspace)}; all=${JSON.stringify(records)}`);

      assert('T23. generated workspace subpath records generated-artifact reason',
        generatedWorkspace?.reason === 'workspace-generated-artifact-missing' &&
          generatedWorkspace?.hint === 'generated-artifact-missing' &&
          generatedWorkspace?.matchedPattern === '@scope/prisma/*' &&
          (generatedWorkspace?.targetCandidates ?? []).some((p) => p.includes('packages/prisma/enums')) &&
          generatedWorkspace?.generatedArtifact?.policyVersion === 'generated-artifact-policy-v1' &&
          generatedWorkspace?.generatedArtifact?.generatorFamily === 'prisma' &&
          generatedWorkspace?.generatedArtifact?.confidence === 'strong' &&
          generatedWorkspace?.generatedArtifact?.matchedPackage === '@scope/prisma' &&
          generatedWorkspace?.generatedArtifact?.targetSubpath === 'enums' &&
          generatedWorkspace?.generatedArtifact?.evidence?.some((e) =>
            e.kind === 'package-bin' &&
            e.field === 'bin.prisma-enum-generator'),
        `record=${JSON.stringify(generatedWorkspace)}; all=${JSON.stringify(records)}`);

      assert('T24. weak generated signals do not promote workspace misses',
        weakDependencyOnly?.reason === 'workspace-package-subpath-target-missing' &&
          !weakDependencyOnly?.generatedArtifact &&
          weakPackageNameOnly?.reason === 'workspace-package-subpath-target-missing' &&
          !weakPackageNameOnly?.generatedArtifact &&
          weakStatic?.reason === 'workspace-package-subpath-target-missing' &&
          !weakStatic?.generatedArtifact,
        `dependencyOnly=${JSON.stringify(weakDependencyOnly)}; packageNameOnly=${JSON.stringify(weakPackageNameOnly)}; weakStatic=${JSON.stringify(weakStatic)}; all=${JSON.stringify(records)}`);

      assert('T25. generated static artifact miss uses package script output-path evidence',
        generatedStatic?.reason === 'workspace-generated-artifact-missing' &&
          generatedStatic?.hint === 'generated-artifact-missing' &&
          generatedStatic?.matchedPattern === '@scope/web-static/*' &&
          (generatedStatic?.targetCandidates ?? []).some((p) =>
            p.includes('packages/web-static/public/app-store/svg-hashes.json')) &&
          generatedStatic?.generatedArtifact?.policyVersion === 'generated-artifact-policy-v1' &&
          generatedStatic?.generatedArtifact?.generatorFamily === 'static-artifact' &&
          generatedStatic?.generatedArtifact?.confidence === 'strong' &&
          generatedStatic?.generatedArtifact?.matchedPackage === '@scope/web-static' &&
          generatedStatic?.generatedArtifact?.targetSubpath === 'public/app-store/svg-hashes.json' &&
          generatedStatic?.generatedArtifact?.evidence?.some((e) =>
            e.kind === 'script-output-path' &&
            e.field === 'scripts.copy-static-assets' &&
            e.matched === 'public/app-store/svg-hashes.json'),
        `record=${JSON.stringify(generatedStatic)}; all=${JSON.stringify(records)}`);

      assert('T25b. exact alias build-output miss uses package output evidence',
        generatedBundle?.reason === 'workspace-generated-artifact-missing' &&
          generatedBundle?.hint === 'generated-artifact-missing' &&
          generatedBundle?.matchedPattern === '@scope/bundle' &&
          (generatedBundle?.targetCandidates ?? []).some((p) =>
            p.includes('packages/bundle/dist/bundle.js')) &&
          generatedBundle?.generatedArtifact?.policyVersion === 'generated-artifact-policy-v1' &&
          generatedBundle?.generatedArtifact?.generatorFamily === 'build-output' &&
          generatedBundle?.generatedArtifact?.confidence === 'strong' &&
          generatedBundle?.generatedArtifact?.matchedPackage === '@scope/bundle' &&
          generatedBundle?.generatedArtifact?.targetSubpath === 'dist/bundle.js' &&
          generatedBundle?.generatedArtifact?.evidence?.some((e) =>
            e.kind === 'package-files' &&
            e.field === 'files' &&
            e.matched === 'dist') &&
          generatedBundle?.generatedArtifact?.evidence?.some((e) =>
            e.kind === 'package-script' &&
            e.field === 'scripts.build' &&
            e.matched === 'vite build'),
        `record=${JSON.stringify(generatedBundle)}; all=${JSON.stringify(records)}`);

      assert('T25c. exact alias static output miss uses package script output evidence',
        generatedCssOutput?.reason === 'workspace-generated-artifact-missing' &&
          generatedCssOutput?.hint === 'generated-artifact-missing' &&
          generatedCssOutput?.matchedPattern === '@scope/css-output/style.min.css' &&
          (generatedCssOutput?.targetCandidates ?? []).some((p) =>
            p.includes('packages/css-output/style.min.css')) &&
          generatedCssOutput?.generatedArtifact?.policyVersion === 'generated-artifact-policy-v1' &&
          generatedCssOutput?.generatedArtifact?.generatorFamily === 'static-artifact' &&
          generatedCssOutput?.generatedArtifact?.confidence === 'strong' &&
          generatedCssOutput?.generatedArtifact?.matchedPackage === '@scope/css-output' &&
          generatedCssOutput?.generatedArtifact?.targetSubpath === 'style.min.css' &&
          generatedCssOutput?.generatedArtifact?.evidence?.some((e) =>
            e.kind === 'script-output-path' &&
            e.field === 'scripts.build' &&
            e.matched === 'style.min.css'),
        `record=${JSON.stringify(generatedCssOutput)}; all=${JSON.stringify(records)}`);

      assert('T25d. exact alias files-only build output does not promote to generated artifact',
        weakBundleFilesOnly?.reason === 'exact-alias-target-missing' &&
          weakBundleFilesOnly?.matchedPattern === '@scope/bundle-files-only' &&
          !weakBundleFilesOnly?.generatedArtifact,
        `record=${JSON.stringify(weakBundleFilesOnly)}; all=${JSON.stringify(records)}`);

      const summary = syms5.unresolvedInternalSummaryByReason ?? {};
      assert('T26. unresolved summary counts tsconfig target misses by reason',
        summary['tsconfig-path-target-missing']?.count === 1 &&
          summary['tsconfig-path-target-missing']?.hints?.['generated-artifact-missing'] === 1 &&
          summary['tsconfig-path-target-missing']?.examples?.some((r) =>
            r.specifier === '@scope/generated-client' &&
            r.consumerFile === 'apps/web/src/consumer.ts'),
        `summary=${JSON.stringify(summary)}`);

      assert('T27. unresolved summary keeps ordinary workspace subpath misses separate',
        summary['workspace-package-subpath-target-missing']?.count === 4 &&
          summary['workspace-package-subpath-target-missing']?.spaces?.type === 1 &&
          summary['workspace-package-subpath-target-missing']?.spaces?.value === 3 &&
          summary['workspace-package-subpath-target-missing']?.spaces?.unknown === 0 &&
          summary['workspace-package-subpath-target-missing']?.examples?.some((r) =>
            r.specifier === '@scope/types/thing' &&
            r.matchedPattern === '@scope/types/*'),
        `summary=${JSON.stringify(summary)}`);

      assert('T28. unresolved summary counts generated workspace misses separately',
        summary['workspace-generated-artifact-missing']?.count === 4 &&
          summary['workspace-generated-artifact-missing']?.spaces?.type === 0 &&
          summary['workspace-generated-artifact-missing']?.spaces?.value === 4 &&
          summary['workspace-generated-artifact-missing']?.spaces?.unknown === 0 &&
          summary['workspace-generated-artifact-missing']?.hints?.['generated-artifact-missing'] === 4 &&
          summary['workspace-generated-artifact-missing']?.examples?.some((r) =>
            r.specifier === '@scope/prisma/enums' &&
            r.matchedPattern === '@scope/prisma/*') &&
          summary['workspace-generated-artifact-missing']?.examples?.some((r) =>
            r.specifier === '@scope/web-static/public/app-store/svg-hashes.json' &&
            r.matchedPattern === '@scope/web-static/*') &&
          summary['workspace-generated-artifact-missing']?.examples?.some((r) =>
            r.specifier === '@scope/bundle' &&
            r.matchedPattern === '@scope/bundle'),
        `summary=${JSON.stringify(summary)}`);
    } finally {
      rmSync(FX5, { recursive: true, force: true });
    }
  }

  // ─── T29: missing tsconfig generated target can fall back to concrete workspace source ───
  //
  // Some app-local tsconfigs point a package subpath at generated output
  // that is absent in a source checkout, while the workspace package also
  // provides a concrete source compatibility entry for the same specifier.
  // The resolver should prefer the concrete workspace source over leaving
  // the import as a tsconfig-path blind zone, but only after the tsconfig
  // target itself fails to exist.
  {
    const FX6 = mkdtempSync(path.join(tmpdir(), 'fx-tsconfig-workspace-fallback-'));
    const OUT6 = path.join(FX6, 'artifacts');
    try {
      mkdirSync(path.join(FX6, 'apps/api/src'), { recursive: true });
      mkdirSync(path.join(FX6, 'packages/orm/client'), { recursive: true });
      mkdirSync(OUT6, { recursive: true });

      writeFileSync(path.join(FX6, 'package.json'),
        JSON.stringify({ name: 'root', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
      writeFileSync(path.join(FX6, 'apps/api/package.json'),
        JSON.stringify({
          name: 'api',
          type: 'module',
          dependencies: { '@scope/orm': 'workspace:*' },
        }));
      writeFileSync(path.join(FX6, 'apps/api/tsconfig.json'), JSON.stringify({
        compilerOptions: {
          baseUrl: '.',
          paths: {
            '@scope/orm/client': ['../../packages/orm/generated/client'],
          },
        },
      }));
      writeFileSync(path.join(FX6, 'packages/orm/package.json'),
        JSON.stringify({ name: '@scope/orm', type: 'module', main: 'index.ts' }));
      writeFileSync(path.join(FX6, 'packages/orm/index.ts'),
        'export const root = 1;\n');
      writeFileSync(path.join(FX6, 'packages/orm/client/index.ts'), `
export interface OrmUser { id: string }
export function makeOrmUser(): OrmUser { return { id: 'u' }; }
export function unusedOrmClient() { return 1; }
`);
      writeFileSync(path.join(FX6, 'apps/api/src/service.ts'), `
import type { OrmUser } from '@scope/orm/client';
import { makeOrmUser } from '@scope/orm/client';
export function service(): OrmUser {
  return makeOrmUser();
}
`);

      execSync(`node build-symbol-graph.mjs --root ${FX6} --output ${OUT6}`,
        { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

      const syms6 = JSON.parse(readFileSync(path.join(OUT6, 'symbols.json'), 'utf8'));
      const unresolved = syms6.unresolvedInternalSpecifierRecords ?? [];
      const dead = new Set((syms6.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`));

      assert('T29. missing tsconfig target falls back to concrete workspace package source',
        syms6.fanInByIdentity?.['packages/orm/client/index.ts::OrmUser'] === 1 &&
          syms6.fanInByIdentity?.['packages/orm/client/index.ts::makeOrmUser'] === 1 &&
          !dead.has('packages/orm/client/index.ts::OrmUser') &&
          !dead.has('packages/orm/client/index.ts::makeOrmUser') &&
          !unresolved.some((r) => r.specifier === '@scope/orm/client') &&
          syms6.uses?.unresolvedInternal === 0,
        JSON.stringify({
          fanIn: syms6.fanInByIdentity,
          unresolved,
          uses: syms6.uses,
          dead: [...dead],
        }));
    } finally {
      rmSync(FX6, { recursive: true, force: true });
    }
  }
} finally {
  rmSync(FX, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
