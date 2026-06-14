import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  parseSfcGeneratedComponentManifests,
  parseSfcGlobalComponentRegistrations,
  parseSfcImportConsumers,
  parseSfcScriptSources,
  parseSfcStyleAssetReferences,
  parseSfcTemplateComponentRefs,
} from "../_lib/sfc-consumers.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function runSymbolGraph(fixture) {
  execFileSync(
    process.execPath,
    [
      path.join(REPO_ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    {
      cwd: REPO_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  return fixture.readJson("symbols.json", { from: "output" });
}

describe("SFC consumers", () => {
  it("SFC-1. parses Vue/Svelte/Astro script imports and ignores template text", () => {
    const vue = [
      "<template>",
      "  import { TemplateOnly } from '../src/card';",
      "  <Card />",
      "  <user-list />",
      "  <SfcCard />",
      '  <component :is="DynamicCard" />',
      "  <UI.Card />",
      "  <my-widget />",
      "  <!-- <CommentedCard /> -->",
      "</template>",
      '<script setup lang="ts">',
      "import DefaultCard, { UsedByVue as Card, type Props } from '../src/card';",
      "import UserList from '../src/user-list';",
      "import SfcCard from './SfcCard.vue';",
      "import DynamicCard from '../src/dynamic-card';",
      "import * as UI from '../src/ui';",
      "import * as Store from '../src/store';",
      "import 'external-sfc-package';",
      "const example = `",
      "import { StringOnly } from '../src/string-only';",
      "`;",
      "/*",
      "import { CommentOnly } from '../src/comment-only';",
      "*/",
      "</script>",
      '<script src="./external.ts"></script>',
      '<script :src="dynamicSource"></script>',
      '<script src="external-package"></script>',
      '<script src="">',
      "import { EmptySrcOnly } from '../src/empty-src';",
      "</script>",
      "<script src=''>",
      "import { SingleQuotedEmptySrcOnly } from '../src/single-empty-src';",
      "</script>",
      "<style>",
      '.hero { background-image: url("./logo.svg"); }',
      ".escaped { background: url(./my\\ icon.svg); }",
      '@import "./theme.css";',
      '/* url("./commented.svg") */',
      "</style>",
    ].join("\n");
    const svelte = [
      '<script context="module" lang="ts">',
      "import { UsedBySvelte } from '../src/svelte-use';",
      "</script>",
      '<script src="../src/svelte-src.ts"></script>',
      "<UsedBySvelte />",
      "<svelte:component this={UsedBySvelte} />",
      "<div>import { SvelteTemplateOnly } from '../src/svelte-template';</div>",
      "<style>",
      ".icon { background: url('../assets/icon.svg'); }",
      "</style>",
    ].join("\n");
    const astro = [
      "---",
      "import { UsedByAstro } from '../src/astro-use';",
      "---",
      "<UsedByAstro client:load />",
      "<div>import { AstroTemplateOnly } from '../src/astro-template';</div>",
      "<style>",
      '@import url("./astro.css");',
      '.remote { background: url("https://example.test/remote.png"); }',
      "</style>",
    ].join("\n");
    const vueTsx = [
      '<script lang="tsx">',
      "import { UsedByTsx } from '../src/tsx-card';",
      "const node = <div>{UsedByTsx}</div>;",
      "</script>",
    ].join("\n");

    const imports = [
      ...parseSfcImportConsumers(vue, "components/App.vue"),
      ...parseSfcImportConsumers(svelte, "components/Page.svelte"),
      ...parseSfcImportConsumers(astro, "pages/Home.astro"),
      ...parseSfcImportConsumers(vueTsx, "components/Tsx.vue"),
    ];
    const names = imports
      .map(
        (entry) =>
          `${entry.consumerFile}:${entry.fromSpec}:${entry.name}:${entry.kind}`,
      )
      .sort();
    const scriptSources = [
      ...parseSfcScriptSources(vue, "components/App.vue"),
      ...parseSfcScriptSources(svelte, "components/Page.svelte"),
      ...parseSfcScriptSources(astro, "pages/Home.astro"),
      ...parseSfcScriptSources(vueTsx, "components/Tsx.vue"),
    ];
    const sourceNames = scriptSources
      .map((entry) => `${entry.consumerFile}:${entry.fromSpec}:${entry.kind}`)
      .sort();
    const styleAssets = [
      ...parseSfcStyleAssetReferences(vue, "components/App.vue"),
      ...parseSfcStyleAssetReferences(svelte, "components/Page.svelte"),
      ...parseSfcStyleAssetReferences(astro, "pages/Home.astro"),
      ...parseSfcStyleAssetReferences(vueTsx, "components/Tsx.vue"),
    ];
    const styleNames = styleAssets
      .map(
        (entry) =>
          `${entry.consumerFile}:${entry.fromSpec}:${entry.source}:${entry.styleKind}`,
      )
      .sort();
    const templateRefs = [
      ...parseSfcTemplateComponentRefs(vue, "components/App.vue"),
      ...parseSfcTemplateComponentRefs(svelte, "components/Page.svelte"),
      ...parseSfcTemplateComponentRefs(astro, "pages/Home.astro"),
      ...parseSfcTemplateComponentRefs(vueTsx, "components/Tsx.vue"),
    ];
    const templateNames = templateRefs
      .map(
        (entry) =>
          `${entry.consumerFile}:${entry.tagName}:${entry.bindingName}:${entry.status}:${entry.reason ?? ""}`,
      )
      .sort();
    const svelteStaticRef = templateRefs.find(
      (entry) =>
        entry.consumerFile === "components/Page.svelte" &&
        entry.tagName === "UsedBySvelte",
    );
    const svelteDynamicRef = templateRefs.find(
      (entry) =>
        entry.consumerFile === "components/Page.svelte" &&
        entry.tagName === "svelte:component",
    );

    expect(names).toContain("components/App.vue:../src/card:UsedByVue:import");
    expect(names).toContain("components/App.vue:../src/card:default:default");
    expect(names).toContain("components/App.vue:../src/store:*:namespace");
    expect(names).toContain(
      "components/App.vue:external-sfc-package:*:import-side-effect",
    );
    expect(names).toContain(
      "components/Page.svelte:../src/svelte-use:UsedBySvelte:import",
    );
    expect(names).toContain(
      "pages/Home.astro:../src/astro-use:UsedByAstro:import",
    );
    expect(names).toContain(
      "components/Tsx.vue:../src/tsx-card:UsedByTsx:import",
    );
    expect(
      names.some(
        (name) =>
          name.includes("TemplateOnly") ||
          name.includes("SvelteTemplateOnly") ||
          name.includes("AstroTemplateOnly") ||
          name.includes("StringOnly") ||
          name.includes("CommentOnly") ||
          name.includes("EmptySrcOnly") ||
          name.includes("SingleQuotedEmptySrcOnly") ||
          name.includes("./external.ts"),
      ),
      JSON.stringify(imports),
    ).toBe(false);
    expect(sourceNames).toEqual([
      "components/App.vue:./external.ts:sfc-script-src",
      "components/Page.svelte:../src/svelte-src.ts:sfc-script-src",
    ]);
    expect(styleNames).toEqual([
      "components/App.vue:./logo.svg:sfc-style-url:url",
      "components/App.vue:./my icon.svg:sfc-style-url:url",
      "components/App.vue:./theme.css:sfc-style-import:import",
      "components/Page.svelte:../assets/icon.svg:sfc-style-url:url",
      "pages/Home.astro:./astro.css:sfc-style-import:import",
    ]);
    expect(templateNames).toEqual(
      expect.arrayContaining([
        "components/App.vue:Card:Card:binding:",
        "components/App.vue:user-list:UserList:binding:",
        "components/App.vue:SfcCard:SfcCard:binding:",
        "components/Page.svelte:UsedBySvelte:UsedBySvelte:binding:",
        "pages/Home.astro:UsedByAstro:UsedByAstro:binding:",
        "components/App.vue:component:DynamicCard:muted:sfc-template-dynamic-component",
        "components/App.vue:UI.Card:UI:muted:sfc-template-namespace-component",
        "components/Page.svelte:svelte:component:UsedBySvelte:muted:sfc-template-dynamic-component",
      ]),
    );
    expect(
      templateRefs.every(
        (entry) =>
          entry.source === "sfc-template-component-ref" &&
          entry.eligibleForFanIn === false &&
          entry.eligibleForSafeFix === false,
      ),
    ).toBe(true);
    expect(
      templateNames.some(
        (name) =>
          name.includes("my-widget") ||
          name.includes("CommentedCard") ||
          name.includes("TemplateOnly"),
      ),
    ).toBe(false);
    expect(svelteStaticRef?.line).toBe(5);
    expect(svelteDynamicRef?.line).toBe(6);

    const registrationScript = [
      "import UserCard from './components/UserCard.vue';",
      "import { AdminPanel } from './components/AdminPanel';",
      "import SSRCard from './components/SSRCard.vue';",
      "import ChainedCard from './components/ChainedCard.vue';",
      "import MountedCard from './components/MountedCard.vue';",
      "import PluginCard from './components/PluginCard.vue';",
      "import DuplicateOne from './components/DuplicateOne.vue';",
      "import DuplicateTwo from './components/DuplicateTwo.vue';",
      "const dynamicName = 'DynamicCard';",
      "const asyncLoader = () => import('./components/AsyncByLoader.vue');",
      "const app = createApp({});",
      "const ssrApp = createSSRApp({});",
      "const chainedApp = createApp({}).use(router);",
      "const mounted = createApp({}).mount('#app');",
      "export default {",
      "  install(pluginApp) {",
      "    pluginApp.component('PluginCard', PluginCard);",
      "  },",
      "};",
      "app.component('UserCard', UserCard);",
      "app.component('admin-panel', AdminPanel);",
      "ssrApp.component('SSRCard', SSRCard);",
      "app.component('SharedGlobal', UserCard);",
      "ssrApp.component('SharedGlobal', SSRCard);",
      "chainedApp.component('ChainedCard', ChainedCard);",
      "mounted.component('MountedCard', MountedCard);",
      "app.component(dynamicName, UserCard);",
      "app.component('FactoryCard', resolveComponent());",
      "app.component('AsyncCard', defineAsyncComponent(() => import('./components/AsyncCard.vue')));",
      "app.component('AsyncByLoader', defineAsyncComponent(asyncLoader));",
      "app.component('DuplicateCard', DuplicateOne);",
      "app.component('DuplicateCard', DuplicateTwo);",
    ].join("\n");
    const registrations = parseSfcGlobalComponentRegistrations(
      registrationScript,
      "src/main.ts",
    );
    const registrationNames = registrations
      .map(
        (entry) =>
          `${entry.registrationFile}:${entry.componentName ?? ""}:${entry.bindingName ?? ""}:${entry.status}:${entry.reason ?? ""}`,
      )
      .sort();

    expect(registrationNames).toEqual(
      expect.arrayContaining([
        "src/main.ts:UserCard:UserCard:registration-syntax:",
        "src/main.ts:admin-panel:AdminPanel:registration-syntax:",
        "src/main.ts:SSRCard:SSRCard:registration-syntax:",
        "src/main.ts:SharedGlobal:UserCard:registration-syntax:",
        "src/main.ts:SharedGlobal:SSRCard:registration-syntax:",
        "src/main.ts:ChainedCard:ChainedCard:registration-syntax:",
        "src/main.ts:PluginCard:PluginCard:registration-syntax:",
        "src/main.ts::UserCard:muted:sfc-global-component-name-dynamic",
        "src/main.ts:FactoryCard::muted:sfc-global-component-value-unsupported",
        "src/main.ts:AsyncCard::muted:sfc-global-component-async-factory",
        "src/main.ts:AsyncByLoader::muted:sfc-global-component-async-factory-nonliteral",
        "src/main.ts:DuplicateCard:DuplicateOne:muted:sfc-global-component-duplicate-registration",
        "src/main.ts:DuplicateCard:DuplicateTwo:muted:sfc-global-component-duplicate-registration",
      ]),
    );
    expect(registrationNames.some((name) => name.includes("MountedCard"))).toBe(
      false,
    );
    expect(registrations).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          componentName: "UserCard",
          normalizedTagNames: expect.arrayContaining(["user-card"]),
          bindingSource: "./components/UserCard.vue",
          source: "sfc-global-component-registration",
          eligibleForFanIn: false,
          eligibleForSafeFix: false,
        }),
      ]),
    );
    expect(registrations).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          componentName: "AsyncCard",
          fromSpec: "./components/AsyncCard.vue",
          factoryKind: "defineAsyncComponent",
          eligibleForFanIn: false,
          eligibleForSafeFix: false,
        }),
        expect.objectContaining({
          componentName: "DuplicateCard",
          bindingName: "DuplicateOne",
          ambiguityKey: "DuplicateCard",
        }),
        expect.objectContaining({
          componentName: "DuplicateCard",
          bindingName: "DuplicateTwo",
          ambiguityKey: "DuplicateCard",
        }),
      ]),
    );

    const generatedManifest = [
      "declare module 'vue' {",
      "  export interface GlobalComponents {",
      "    BaseButton: typeof import('./components/BaseButton.vue')['default']",
      '    \'user-card\': typeof import("./components/UserCard.vue")["default"]',
      "    SourceCard: typeof import('./src/SourceCard.ts')['default']",
      "    RouterLink: typeof import('vue-router')['RouterLink']",
      "    [DynamicCard]: typeof import('./components/Dynamic.vue')['default']",
      "    [prefix + 'Card']: typeof import('./components/Expr.vue')['default']",
      "  }",
      "}",
      "",
    ].join("\n");
    const generatedManifestRecords = parseSfcGeneratedComponentManifests(
      generatedManifest,
      "components.d.ts",
      "unplugin-vue-components-dts",
    );
    expect(generatedManifestRecords).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          manifestFile: "components.d.ts",
          manifestKind: "unplugin-vue-components-dts",
          componentName: "BaseButton",
          fromSpec: "./components/BaseButton.vue",
          normalizedTagNames: expect.arrayContaining(["base-button"]),
          source: "sfc-framework-generated-manifest",
          confidence: "generated-manifest-availability",
          eligibleForFanIn: false,
          eligibleForSafeFix: false,
        }),
        expect.objectContaining({
          componentName: "user-card",
          fromSpec: "./components/UserCard.vue",
        }),
        expect.objectContaining({
          componentName: "SourceCard",
          fromSpec: "./src/SourceCard.ts",
        }),
        expect.objectContaining({
          componentName: "DynamicCard",
          fromSpec: "./components/Dynamic.vue",
          status: "skipped",
          reason: "sfc-framework-generated-manifest-nonliteral",
          normalizedTagNames: [],
        }),
        expect.objectContaining({
          componentName: "[computed]",
          fromSpec: "./components/Expr.vue",
          status: "skipped",
          reason: "sfc-framework-generated-manifest-nonliteral",
          computedKeySource: "prefix + 'Card'",
        }),
      ]),
    );
    expect(
      generatedManifestRecords.some(
        (entry) => entry.componentName === "RouterLink",
      ),
    ).toBe(false);
  });

  it("SFC-2. contributes script import fan-in evidence without protecting templates", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-sfc-consumer-graph-",
      packageJson: { name: "sfc-fixture", type: "module" },
      outputDirName: "out",
    });
    try {
      fixture.write(
        "nuxt.config.ts",
        [
          "const layerPreset = '../dynamic-layer';",
          "const customModule = () => {};",
          "export default defineNuxtConfig({",
          "  extends: ['../layer-a', layerPreset],",
          "  modules: ['@nuxt/image', ['@nuxtjs/tailwindcss', { exposeConfig: true }], customModule],",
          "  srcDir: 'app/',",
          "  components: {",
          "    dirs: [",
          "      { path: '~/shared/components', prefix: 'Shared', pathPrefix: false, global: true },",
          "      './local-components',",
          "    ],",
          "  },",
          "  hooks: {",
          "    'components:dirs'(dirs) { dirs.push({ path: '~/runtime-components' }); },",
          "    'components:extend': (components) => components.push({ pascalName: 'RuntimeCard' }),",
          "    'app:mounted': () => {},",
          "  },",
          "});",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/card.ts",
        [
          "export function UsedByVue() { return null; }",
          "export const TemplateOnly = 1;",
          "export const Unused = 1;",
          "export default function DefaultCard() { return null; }",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/store.ts",
        [
          "export const UsedByNamespace = 1;",
          "export const AlsoProtectedByNamespace = 2;",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/side-effect.ts",
        "export const SideEffectExport = 1;\n",
      );
      fixture.write("src/svelte-use.ts", "export const UsedBySvelte = 1;\n");
      fixture.write("src/svelte-action.ts", "export function enhance() {}\n");
      fixture.write(
        "src/svelte-store.ts",
        "export const importedCount = { subscribe() {} };\n",
      );
      fixture.write("src/astro-use.ts", "export const UsedByAstro = 1;\n");
      fixture.write(
        "src/tsx-card.tsx",
        "export function UsedByTsx() { return null; }\n",
      );
      fixture.write(
        "src/user-list.ts",
        "export default function UserList() { return null; }\n",
      );
      fixture.write(
        "src/dynamic-card.ts",
        "export default function DynamicCard() { return null; }\n",
      );
      fixture.write("src/ui.ts", "export const Card = 1;\n");
      fixture.write(
        "src/template-only-component.ts",
        "export default function TemplateOnlyComponent() { return null; }\n",
      );
      fixture.write(
        "src/script-src-logic.ts",
        "export const ScriptSrcExport = 1;\n",
      );
      fixture.write(
        "src/svelte-src-logic.ts",
        "export const SvelteScriptSrcExport = 1;\n",
      );
      fixture.write("src/empty-src.ts", "export const EmptySrcOnly = 1;\n");
      fixture.write(
        "src/main.ts",
        [
          "import { createApp, createSSRApp, defineAsyncComponent } from 'vue';",
          "import App from '../components/App.vue';",
          "import GlobalCard from '../components/GlobalCard.vue';",
          "import PluginCard from '../components/PluginCard.vue';",
          "import DuplicateOne from '../components/DuplicateOne.vue';",
          "import DuplicateTwo from '../components/DuplicateTwo.vue';",
          "import { RegisteredSource } from './registered-source';",
          "import { SsrRegisteredSource } from './ssr-registered-source';",
          "import { ChainedRegisteredSource } from './chained-registered-source';",
          "import MissingGlobal from './missing-global';",
          "const dynamicName = 'DynamicGlobal';",
          "const router = {};",
          "const app = createApp(App);",
          "const ssrApp = createSSRApp(App);",
          "const chainedApp = createApp(App).use(router);",
          "export default {",
          "  install(pluginApp) {",
          "    pluginApp.component('PluginCard', PluginCard);",
          "  },",
          "};",
          "app.component('GlobalCard', GlobalCard);",
          "app.component('registered-source', RegisteredSource);",
          "ssrApp.component('ssr-registered-source', SsrRegisteredSource);",
          "app.component('shared-source', RegisteredSource);",
          "ssrApp.component('shared-source', SsrRegisteredSource);",
          "chainedApp.component('chained-registered-source', ChainedRegisteredSource);",
          "app.component('AsyncGlobal', defineAsyncComponent(() => import('../components/AsyncGlobal.vue')));",
          "app.component('DuplicateGlobal', DuplicateOne);",
          "app.component('DuplicateGlobal', DuplicateTwo);",
          "app.component(dynamicName, GlobalCard);",
          "app.component('MissingGlobal', MissingGlobal);",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/registered-source.ts",
        "export function RegisteredSource() { return null; }\n",
      );
      fixture.write(
        "src/ssr-registered-source.ts",
        "export function SsrRegisteredSource() { return null; }\n",
      );
      fixture.write(
        "src/chained-registered-source.ts",
        "export function ChainedRegisteredSource() { return null; }\n",
      );
      fixture.write(
        "src/ManifestSource.ts",
        "export default function ManifestSource() { return null; }\n",
      );
      fixture.write(
        "src/macro-alias.ts",
        "export default function MacroAlias() { return null; }\n",
      );
      fixture.write(
        "src/options-alias.ts",
        "export default function OptionsAlias() { return null; }\n",
      );
      fixture.write("assets/logo.svg", "<svg></svg>\n");
      fixture.write("assets/my icon.svg", "<svg></svg>\n");
      fixture.write("assets/icon.svg", "<svg></svg>\n");
      fixture.write("styles/theme.css", ".theme { color: red; }\n");
      fixture.write(
        "components/App.vue",
        [
          "<template>",
          "  import { TemplateOnly } from '../src/card';",
          "  <Card />",
          "  <user-list />",
          "  <SfcCard />",
          '  <component :is="DynamicCard" />',
          "  <UI.Card />",
          "  <MissingCard />",
          "  <TemplateOnlyComponent />",
          "</template>",
          '<script setup lang="ts">',
          "import DefaultCard, { UsedByVue as Card } from '../src/card';",
          "import UserList from '../src/user-list';",
          "import SfcCard from './SfcCard.vue';",
          "import DynamicCard from '../src/dynamic-card';",
          "import * as UI from '../src/ui';",
          "import MissingCard from '../src/missing-card';",
          "import * as Store from '../src/store';",
          "import 'external-sfc-package';",
          "import '../src/side-effect';",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/Page.svelte",
        [
          '<script context="module" lang="ts">',
          "import { UsedBySvelte } from '../src/svelte-use';",
          "import { enhance } from '../src/svelte-action';",
          "</script>",
          '<script lang="ts">',
          "import { derived, writable } from 'svelte/store';",
          "import { importedCount } from '../src/svelte-store';",
          "function localAction(node) { return { destroy() {} }; }",
          "const localConstAction = (node) => ({ destroy() {} });",
          "const localCount = writable(0);",
          "const derivedCount = derived(localCount, ($localCount) => $localCount * 2);",
          "const notActionValue = 1;",
          "const notStoreValue = 1;",
          "$: doubled = $localCount * 2;",
          "</script>",
          "<UsedBySvelte />",
          "<svelte:component this={UsedBySvelte} />",
          "<form use:enhance></form>",
          "<div use:localAction></div>",
          "<section use:localConstAction></section>",
          "<p>{$importedCount}</p>",
          "<p>$plainTextStoreMention</p>",
          "<p>{$missingStore}</p>",
          "<p>{$notStoreValue}</p>",
          "<button use:notActionValue>Non-function local stays silent</button>",
          "<button use:missingAction>Missing action stays silent</button>",
          "<!-- <div use:commentAction></div> -->",
          "<!-- <p>{$commentStore}</p> -->",
          "<p>import { Fake } from '../src/fake';</p>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "pages/Home.astro",
        [
          "---",
          "import { UsedByAstro } from '../src/astro-use';",
          "---",
          "<UsedByAstro client:load />",
          "<MissingAstroClient client:load />",
          "<div client:load>native client directive stays unsupported</div>",
          "<div>import { FakeAstro } from '../src/fake-astro';</div>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/Tsx.vue",
        [
          '<script lang="tsx">',
          "import { UsedByTsx } from '../src/tsx-card';",
          "const node = <div>{UsedByTsx}</div>;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/SfcCard.vue",
        [
          "<template><article>SFC card</article></template>",
          '<script setup lang="ts">',
          "const localOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/GlobalCard.vue",
        [
          "<template><article>Global card</article></template>",
          '<script setup lang="ts">',
          "const globalOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/PluginCard.vue",
        [
          "<template><article>Plugin card</article></template>",
          '<script setup lang="ts">',
          "const pluginOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/AsyncGlobal.vue",
        [
          "<template><article>Async global</article></template>",
          '<script setup lang="ts">',
          "const asyncOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/DuplicateOne.vue",
        [
          "<template><article>Duplicate one</article></template>",
          '<script setup lang="ts">',
          "const duplicateOneOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/DuplicateTwo.vue",
        [
          "<template><article>Duplicate two</article></template>",
          '<script setup lang="ts">',
          "const duplicateTwoOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/ManifestButton.vue",
        [
          "<template><button>Manifest</button></template>",
          '<script setup lang="ts">',
          "const manifestOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/MacroCard.vue",
        [
          "<template><article>Macro card</article></template>",
          '<script setup lang="ts">',
          "const macroCardOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/OptionsCard.vue",
        [
          "<template><article>Options card</article></template>",
          '<script setup lang="ts">',
          "const optionsCardOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "pages/Macro.vue",
        [
          "<template>",
          "  <!-- defineOptions({ components: { CommentOnlyMacro } }) -->",
          "  <section>Macro page</section>",
          "</template>",
          '<script setup lang="ts">',
          "import MacroCard from '../components/MacroCard.vue';",
          "import MacroAlias from '../src/macro-alias';",
          "const dynamicMacroName = 'DynamicMacro';",
          "defineOptions({",
          "  components: {",
          "    MacroCard,",
          "    'macro-alias': MacroAlias,",
          "    MissingMacro,",
          "    [dynamicMacroName]: MacroCard,",
          "  },",
          "});",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "pages/Options.vue",
        [
          "<template>",
          "  <!-- export default { components: { CommentOnlyOptions } } -->",
          "  <section>Options page</section>",
          "</template>",
          '<script lang="ts">',
          "import OptionsCard from '../components/OptionsCard.vue';",
          "import OptionsAlias from '../src/options-alias';",
          "const dynamicOptionsName = 'DynamicOptions';",
          "export default {",
          "  components: {",
          "    OptionsCard,",
          "    'options-alias': OptionsAlias,",
          "    MissingOptions,",
          "    [dynamicOptionsName]: OptionsCard,",
          "  },",
          "};",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "pages/NuxtAlias.vue",
        [
          "<template><section>Nuxt alias</section></template>",
          '<script setup lang="ts">',
          "import { NuxtManifest as LocalNuxtManifest, UnknownAlias } from '#components';",
          "import { componentNames } from '#components';",
          "import type { TypeOnlyAlias } from '#components';",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/NuxtManifest.vue",
        [
          "<template><button>Nuxt</button></template>",
          '<script setup lang="ts">',
          "const nuxtOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/ConventionOnly.vue",
        [
          "<template><button>Convention</button></template>",
          '<script setup lang="ts">',
          "const conventionOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/base/Button.vue",
        [
          "<template><button>Base button</button></template>",
          '<script setup lang="ts">',
          "const baseButton = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/user/index.vue",
        [
          "<template><section>User index</section></template>",
          '<script setup lang="ts">',
          "const userIndex = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "app/components/base/AppButton.vue",
        [
          "<template><button>App button</button></template>",
          '<script setup lang="ts">',
          "const appButton = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "shared/components/RootDecoy.vue",
        [
          "<template><section>Root decoy</section></template>",
          '<script setup lang="ts">',
          "const rootDecoy = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "app/shared/components/ConfiguredOnly.vue",
        [
          "<template><section>Configured only</section></template>",
          '<script setup lang="ts">',
          "const configuredOnly = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "local-components/LocalConfigured.vue",
        [
          "<template><section>Local configured</section></template>",
          '<script setup lang="ts">',
          "const localConfigured = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components.d.ts",
        [
          "declare module 'vue' {",
          "  export interface GlobalComponents {",
          "    ManifestButton: typeof import('./components/ManifestButton.vue')['default']",
          "    ManifestSource: typeof import('./src/ManifestSource.ts')['default']",
          "    MissingManifest: typeof import('./components/MissingManifest.vue')['default']",
          "    RouterLink: typeof import('vue-router')['RouterLink']",
          "    [DynamicManifest]: typeof import('./components/DynamicManifest.vue')['default']",
          "    [prefix + 'Manifest']: typeof import('./components/ExprManifest.vue')['default']",
          "  }",
          "}",
          "",
        ].join("\n"),
      );
      fixture.write(
        ".nuxt/components.d.ts",
        [
          "declare module 'vue' {",
          "  export interface GlobalComponents {",
          "    NuxtManifest: typeof import('../components/NuxtManifest.vue')['default']",
          "  }",
          "}",
          "",
        ].join("\n"),
      );
      fixture.write(
        "vite.config.ts",
        [
          "import Components from 'unplugin-vue-components/vite';",
          "export default { plugins: [Components()] };",
          "",
        ].join("\n"),
      );
      fixture.write(
        "webpack.config.cjs",
        [
          "const Components = require('unplugin-vue-components/webpack');",
          "module.exports = {",
          "  plugins: [",
          "    Components(),",
          "    require('unplugin-vue-components/webpack')(),",
          "  ],",
          "};",
          "",
        ].join("\n"),
      );
      fixture.write(
        "webpack.config.js",
        [
          "module.exports = {",
          "  plugins: [require('unplugin-vue-components/vite')()],",
          "};",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/ScriptSrc.vue",
        ['<script src="../src/script-src-logic.ts"></script>', ""].join("\n"),
      );
      fixture.write(
        "components/SvelteSrc.svelte",
        ['<script src="../src/svelte-src-logic.ts"></script>', ""].join("\n"),
      );
      fixture.write(
        "components/MissingSrc.vue",
        ['<script src="../src/missing-script-src.ts"></script>', ""].join("\n"),
      );
      fixture.write(
        "components/IgnoredSrc.vue",
        [
          '<script src="external-package"></script>',
          '<script src="https://example.test/remote.ts"></script>',
          '<script :src="dynamicSource"></script>',
          '<script src="">',
          "import { EmptySrcOnly } from '../src/empty-src';",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/StyleAsset.vue",
        [
          "<template>",
          "  <div style=\"background:url('../assets/template-only.svg')\"></div>",
          "</template>",
          "<style>",
          '.logo { background: url("../assets/logo.svg"); }',
          ".escaped { background: url(../assets/my\\ icon.svg); }",
          '@import "../styles/theme.css";',
          '/* url("../assets/commented.svg") */',
          '.remote { background: url("https://example.test/remote.png"); }',
          '.pkg { background: url("some-package/icon.svg"); }',
          ".dynamic { background: url(var(--asset)); }",
          '.missing { background: url("../assets/missing.svg"); }',
          "</style>",
          "",
        ].join("\n"),
      );

      const symbols = runSymbolGraph(fixture);
      const dead = new Set(
        (symbols.deadProdList ?? []).map(
          (entry) => `${entry.file}::${entry.symbol}`,
        ),
      );

      expect(dead).not.toContain("src/card.ts::UsedByVue");
      expect(dead).not.toContain("src/card.ts::default");
      expect(dead).not.toContain("src/store.ts::UsedByNamespace");
      expect(dead).not.toContain("src/store.ts::AlsoProtectedByNamespace");
      expect(dead).not.toContain("src/tsx-card.tsx::UsedByTsx");
      expect(dead).toContain("src/card.ts::TemplateOnly");
      expect(dead).toContain("src/card.ts::Unused");
      expect(dead).toContain("src/side-effect.ts::SideEffectExport");
      expect(dead).toContain("src/empty-src.ts::EmptySrcOnly");
      expect(symbols.meta?.supports?.sfcScriptImportConsumers).toBe(true);
      expect(symbols.uses?.sfcScriptConsumers).toBe(14);
      expect(symbols.dependencyImportConsumers).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            file: "components/App.vue",
            fromSpec: "external-sfc-package",
            source: "sfc-script-import",
          }),
        ]),
      );
      expect(symbols.resolvedInternalEdges).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            from: "components/App.vue",
            to: "src/side-effect.ts",
            kind: "import-side-effect",
          }),
        ]),
      );
      expect(symbols.meta?.supports?.sfcScriptSrcReachability).toBe(true);
      expect(symbols.uses?.sfcScriptSrcReachability).toBe(2);
      expect(symbols.resolvedInternalEdges).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            from: "components/ScriptSrc.vue",
            to: "src/script-src-logic.ts",
            kind: "sfc-script-src",
            source: "../src/script-src-logic.ts",
          }),
          expect.objectContaining({
            from: "components/SvelteSrc.svelte",
            to: "src/svelte-src-logic.ts",
            kind: "sfc-script-src",
            source: "../src/svelte-src-logic.ts",
          }),
        ]),
      );
      expect(dead).toContain("src/script-src-logic.ts::ScriptSrcExport");
      expect(dead).toContain("src/svelte-src-logic.ts::SvelteScriptSrcExport");
      expect(
        symbols.fanInByIdentity?.["src/script-src-logic.ts::ScriptSrcExport"],
      ).toBe(0);
      expect(
        symbols.fanInByIdentity?.[
          "src/svelte-src-logic.ts::SvelteScriptSrcExport"
        ],
      ).toBe(0);
      expect(symbols.unresolvedInternalSpecifierRecords).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            consumerFile: "components/MissingSrc.vue",
            specifier: "../src/missing-script-src.ts",
            reason: "sfc-script-src-unresolved",
          }),
        ]),
      );
      expect(
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.source === "external-package" ||
            edge.source === "https://example.test/remote.ts" ||
            edge.source === "dynamicSource",
        ),
      ).toBe(false);
      expect(symbols.meta?.supports?.sfcStyleAssetReferences).toBe(true);
      expect(symbols.uses?.sfcStyleAssetReferences).toBe(3);
      expect(symbols.sfcStyleAssetReferences).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            consumerFile: "components/StyleAsset.vue",
            fromSpec: "../assets/logo.svg",
            resolvedFile: "assets/logo.svg",
            source: "sfc-style-url",
            status: "resolved",
          }),
          expect.objectContaining({
            consumerFile: "components/StyleAsset.vue",
            fromSpec: "../assets/my icon.svg",
            resolvedFile: "assets/my icon.svg",
            source: "sfc-style-url",
            status: "resolved",
          }),
          expect.objectContaining({
            consumerFile: "components/StyleAsset.vue",
            fromSpec: "../styles/theme.css",
            resolvedFile: "styles/theme.css",
            source: "sfc-style-import",
            status: "resolved",
          }),
          expect.objectContaining({
            consumerFile: "components/StyleAsset.vue",
            fromSpec: "../assets/missing.svg",
            reason: "sfc-style-asset-unresolved",
            status: "unresolved",
          }),
        ]),
      );
      expect(
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.source === "../assets/logo.svg" ||
            edge.source === "../assets/my icon.svg" ||
            edge.source === "../styles/theme.css" ||
            edge.source === "../assets/missing.svg",
        ),
      ).toBe(false);
      expect(
        symbols.sfcStyleAssetReferences?.some(
          (entry) =>
            entry.fromSpec.includes("commented.svg") ||
            entry.fromSpec.includes("remote.png") ||
            entry.fromSpec.includes("some-package") ||
            entry.fromSpec.includes("template-only.svg"),
        ),
      ).toBe(false);
      expect(symbols.meta?.supports?.sfcTemplateComponentRefs).toBe(true);
      expect(symbols.uses?.sfcTemplateComponentRefs).toBe(9);
      expect(symbols.sfcTemplateComponentRefs).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            consumerFile: "components/App.vue",
            tagName: "Card",
            bindingName: "Card",
            bindingSource: "../src/card",
            resolvedFile: "src/card.ts",
            status: "resolved",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            consumerFile: "components/App.vue",
            tagName: "user-list",
            normalizedTagName: "UserList",
            resolvedFile: "src/user-list.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            consumerFile: "components/App.vue",
            tagName: "SfcCard",
            bindingSource: "./SfcCard.vue",
            resolvedFile: "components/SfcCard.vue",
            status: "muted",
            reason: "sfc-template-component-non-source-binding",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            consumerFile: "pages/Home.astro",
            tagName: "UsedByAstro",
            resolvedFile: "src/astro-use.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            consumerFile: "components/App.vue",
            tagName: "component",
            bindingName: "DynamicCard",
            status: "muted",
            reason: "sfc-template-dynamic-component",
          }),
          expect.objectContaining({
            consumerFile: "components/App.vue",
            tagName: "UI.Card",
            bindingName: "UI",
            memberName: "Card",
            status: "muted",
            reason: "sfc-template-namespace-component",
          }),
          expect.objectContaining({
            consumerFile: "components/App.vue",
            tagName: "MissingCard",
            status: "unresolved",
            reason: "sfc-template-component-unresolved",
          }),
          expect.objectContaining({
            consumerFile: "components/Page.svelte",
            tagName: "svelte:component",
            status: "muted",
            reason: "sfc-template-dynamic-component",
          }),
        ]),
      );
      expect(
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.kind === "sfc-template-component-ref" ||
            edge.source === "sfc-template-component-ref" ||
            edge.source === "./SfcCard.vue",
        ),
      ).toBe(false);
      expect(
        symbols.sfcTemplateComponentRefs?.some(
          (entry) => entry.tagName === "TemplateOnlyComponent",
        ),
      ).toBe(false);
      expect(dead).toContain("src/template-only-component.ts::default");
      expect(
        symbols.fanInByIdentity?.["src/template-only-component.ts::default"],
      ).toBe(0);
      expect(symbols.meta?.supports?.sfcGlobalComponentRegistrations).toBe(
        true,
      );
      expect(symbols.uses?.sfcGlobalComponentRegistrations).toBe(12);
      expect(symbols.sfcGlobalComponentRegistrations).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            registrationFile: "src/main.ts",
            componentName: "GlobalCard",
            bindingName: "GlobalCard",
            bindingSource: "../components/GlobalCard.vue",
            resolvedFile: "components/GlobalCard.vue",
            status: "muted",
            reason: "sfc-global-component-non-source-binding",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            componentName: "registered-source",
            bindingName: "RegisteredSource",
            resolvedFile: "src/registered-source.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            componentName: "ssr-registered-source",
            bindingName: "SsrRegisteredSource",
            resolvedFile: "src/ssr-registered-source.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            api: "app.component",
            componentName: "shared-source",
            bindingName: "RegisteredSource",
            resolvedFile: "src/registered-source.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            api: "ssrApp.component",
            componentName: "shared-source",
            bindingName: "SsrRegisteredSource",
            resolvedFile: "src/ssr-registered-source.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            componentName: "chained-registered-source",
            bindingName: "ChainedRegisteredSource",
            resolvedFile: "src/chained-registered-source.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            componentName: "PluginCard",
            bindingName: "PluginCard",
            resolvedFile: "components/PluginCard.vue",
            status: "muted",
            reason: "sfc-global-component-non-source-binding",
          }),
          expect.objectContaining({
            componentName: "AsyncGlobal",
            fromSpec: "../components/AsyncGlobal.vue",
            factoryKind: "defineAsyncComponent",
            resolvedFile: "components/AsyncGlobal.vue",
            status: "muted",
            reason: "sfc-global-component-async-factory",
          }),
          expect.objectContaining({
            componentName: "DuplicateGlobal",
            bindingName: "DuplicateOne",
            resolvedFile: "components/DuplicateOne.vue",
            status: "muted",
            reason: "sfc-global-component-duplicate-registration",
            ambiguityKey: "DuplicateGlobal",
          }),
          expect.objectContaining({
            componentName: "DuplicateGlobal",
            bindingName: "DuplicateTwo",
            resolvedFile: "components/DuplicateTwo.vue",
            status: "muted",
            reason: "sfc-global-component-duplicate-registration",
            ambiguityKey: "DuplicateGlobal",
          }),
          expect.objectContaining({
            bindingName: "GlobalCard",
            status: "muted",
            reason: "sfc-global-component-name-dynamic",
          }),
          expect.objectContaining({
            componentName: "MissingGlobal",
            status: "unresolved",
            reason: "sfc-global-component-unresolved",
          }),
        ]),
      );
      expect(
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.kind === "sfc-global-component-registration" ||
            edge.source === "sfc-global-component-registration",
        ),
      ).toBe(false);
      expect(dead).not.toContain("src/registered-source.ts::RegisteredSource");
      expect(symbols.meta?.supports?.sfcGeneratedComponentManifests).toBe(true);
      expect(symbols.uses?.sfcGeneratedComponentManifests).toBe(6);
      expect(symbols.sfcGeneratedComponentManifests).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            manifestFile: "components.d.ts",
            manifestKind: "unplugin-vue-components-dts",
            componentName: "ManifestButton",
            normalizedTagNames: expect.arrayContaining(["manifest-button"]),
            bindingSource: "./components/ManifestButton.vue",
            resolvedFile: "components/ManifestButton.vue",
            status: "muted",
            reason: "sfc-framework-generated-manifest-non-source-binding",
            source: "sfc-framework-generated-manifest",
            confidence: "generated-manifest-availability",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            manifestFile: ".nuxt/components.d.ts",
            manifestKind: "nuxt-components-dts",
            componentName: "NuxtManifest",
            resolvedFile: "components/NuxtManifest.vue",
            status: "muted",
          }),
          expect.objectContaining({
            componentName: "ManifestSource",
            resolvedFile: "src/ManifestSource.ts",
            status: "resolved",
          }),
          expect.objectContaining({
            componentName: "MissingManifest",
            status: "unresolved",
            reason: "sfc-framework-generated-manifest-unresolved",
          }),
          expect.objectContaining({
            componentName: "DynamicManifest",
            bindingSource: "./components/DynamicManifest.vue",
            status: "skipped",
            reason: "sfc-framework-generated-manifest-nonliteral",
          }),
          expect.objectContaining({
            componentName: "[computed]",
            bindingSource: "./components/ExprManifest.vue",
            status: "skipped",
            reason: "sfc-framework-generated-manifest-nonliteral",
            computedKeySource: "prefix + 'Manifest'",
          }),
        ]),
      );
      expect(
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.componentName === "RouterLink" ||
            entry.componentName === "ConventionOnly",
        ),
      ).toBe(false);
      expect(
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.kind === "sfc-generated-component-manifest" ||
            edge.source === "./components/ManifestButton.vue" ||
            edge.source === "./src/ManifestSource.ts" ||
            edge.source === "./components/DynamicManifest.vue" ||
            edge.source === "./components/ExprManifest.vue",
        ),
      ).toBe(false);
      expect(dead).toContain("src/ManifestSource.ts::default");
      expect(symbols.fanInByIdentity?.["src/ManifestSource.ts::default"]).toBe(
        0,
      );
      expect(symbols.meta?.supports?.sfcFrameworkConventionComponents).toBe(
        true,
      );
      expect(symbols.uses?.sfcFrameworkConventionComponents).toBe(
        symbols.sfcFrameworkConventionComponents?.length,
      );
      expect(symbols.uses?.sfcFrameworkConventionComponents).toBeGreaterThan(0);
      expect(symbols.sfcFrameworkConventionComponents).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-directory",
            componentName: "ConventionOnly",
            normalizedTagNames: expect.arrayContaining([
              "ConventionOnly",
              "convention-only",
            ]),
            sourceFile: "components/ConventionOnly.vue",
            resolvedFile: "components/ConventionOnly.vue",
            source: "sfc-framework-nuxt-fs-convention",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-nuxt-fs-convention",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            componentName: "BaseButton",
            normalizedTagNames: expect.arrayContaining([
              "BaseButton",
              "base-button",
            ]),
            sourceFile: "components/base/Button.vue",
            componentPathSegments: ["base", "Button"],
          }),
          expect.objectContaining({
            componentName: "UserIndex",
            normalizedTagNames: expect.arrayContaining([
              "UserIndex",
              "user-index",
            ]),
            sourceFile: "components/user/index.vue",
            componentPathSegments: ["user", "index"],
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-app-components-directory",
            componentName: "BaseAppButton",
            normalizedTagNames: expect.arrayContaining([
              "BaseAppButton",
              "base-app-button",
            ]),
            sourceFile: "app/components/base/AppButton.vue",
            resolvedFile: "app/components/base/AppButton.vue",
            source: "sfc-framework-nuxt-app-dir-convention",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-nuxt-app-dir-convention",
            componentPathSegments: ["base", "AppButton"],
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-alias-import",
            consumerFile: "pages/NuxtAlias.vue",
            componentName: "NuxtManifest",
            normalizedTagNames: expect.arrayContaining([
              "NuxtManifest",
              "nuxt-manifest",
            ]),
            bindingName: "LocalNuxtManifest",
            importedName: "NuxtManifest",
            manifestFile: ".nuxt/components.d.ts",
            manifestKind: "nuxt-components-dts",
            bindingSource: "../components/NuxtManifest.vue",
            fromSpec: "#components",
            resolvedFile: "components/NuxtManifest.vue",
            source: "sfc-framework-nuxt-components-alias",
            confidence: "generated-manifest-availability",
            status: "muted",
            reason: "sfc-framework-nuxt-components-alias-manifest",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-alias-import",
            consumerFile: "pages/NuxtAlias.vue",
            componentName: "UnknownAlias",
            bindingName: "UnknownAlias",
            importedName: "UnknownAlias",
            fromSpec: "#components",
            source: "sfc-framework-nuxt-components-alias",
            confidence: "framework-convention-observed",
            status: "unresolved",
            reason: "sfc-framework-nuxt-components-alias-unresolved",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-dir-config",
            configFile: "nuxt.config.ts",
            componentDir: "~/shared/components",
            resolvedDir: "app/shared/components",
            prefix: "Shared",
            pathPrefix: false,
            global: true,
            source: "sfc-framework-nuxt-components-dir-config",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-nuxt-components-dir-config",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-dir-config",
            configFile: "nuxt.config.ts",
            componentDir: "./local-components",
            resolvedDir: "local-components",
            source: "sfc-framework-nuxt-components-dir-config",
            reason: "sfc-framework-nuxt-components-dir-config",
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-custom-resolver-unavailable",
            configFile: "nuxt.config.ts",
            hookName: "components:dirs",
            configShape: "hooks",
            source: "sfc-framework-nuxt-custom-resolver-unavailable",
            confidence: "framework-convention-observed",
            status: "unavailable",
            reason: "sfc-framework-nuxt-custom-resolver-unavailable",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-custom-resolver-unavailable",
            configFile: "nuxt.config.ts",
            hookName: "components:extend",
            reason: "sfc-framework-nuxt-custom-resolver-unavailable",
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-layer-extends-unavailable",
            configFile: "nuxt.config.ts",
            configProperty: "extends",
            configShape: "extends",
            extendsSource: "../layer-a",
            extendsSourceKind: "literal",
            source: "sfc-framework-nuxt-layer-extends-unavailable",
            confidence: "framework-convention-observed",
            status: "unavailable",
            reason: "sfc-framework-nuxt-layer-extends-unavailable",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-layer-extends-unavailable",
            configFile: "nuxt.config.ts",
            extendsSourceKind: "nonliteral",
            reason: "sfc-framework-nuxt-layer-extends-unavailable",
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-module-package-unavailable",
            configFile: "nuxt.config.ts",
            configProperty: "modules",
            configShape: "modules",
            moduleSource: "@nuxt/image",
            moduleSourceKind: "literal",
            source: "sfc-framework-nuxt-module-package-unavailable",
            confidence: "framework-convention-observed",
            status: "unavailable",
            reason: "sfc-framework-nuxt-module-package-unavailable",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-module-package-unavailable",
            configFile: "nuxt.config.ts",
            moduleSource: "@nuxtjs/tailwindcss",
            moduleSourceKind: "literal",
            reason: "sfc-framework-nuxt-module-package-unavailable",
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-module-package-unavailable",
            configFile: "nuxt.config.ts",
            moduleSourceKind: "nonliteral",
            reason: "sfc-framework-nuxt-module-package-unavailable",
          }),
          expect.objectContaining({
            framework: "unplugin-vue-components",
            conventionKind: "auto-import-plugin-config",
            configFile: "vite.config.ts",
            pluginName: "Components",
            fromSpec: "unplugin-vue-components/vite",
            source: "sfc-framework-auto-import-plugin-config",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-auto-import-plugin-config",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "unplugin-vue-components",
            conventionKind: "auto-import-plugin-config",
            configFile: "webpack.config.cjs",
            pluginName: "Components",
            fromSpec: "unplugin-vue-components/webpack",
            reason: "sfc-framework-auto-import-plugin-config",
          }),
          expect.objectContaining({
            framework: "unplugin-vue-components",
            conventionKind: "auto-import-plugin-config",
            configFile: "webpack.config.cjs",
            pluginName: "require",
            fromSpec: "unplugin-vue-components/webpack",
            reason: "sfc-framework-auto-import-plugin-config",
          }),
          expect.objectContaining({
            framework: "unplugin-vue-components",
            conventionKind: "auto-import-plugin-config",
            configFile: "webpack.config.js",
            pluginName: "require",
            fromSpec: "unplugin-vue-components/vite",
            reason: "sfc-framework-auto-import-plugin-config",
          }),
          expect.objectContaining({
            framework: "astro",
            conventionKind: "client-directive",
            consumerFile: "pages/Home.astro",
            tagName: "UsedByAstro",
            directiveName: "client:load",
            bindingName: "UsedByAstro",
            bindingSource: "../src/astro-use",
            source: "sfc-framework-astro-client-directive",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-astro-client-directive",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "svelte",
            conventionKind: "action-directive",
            consumerFile: "components/Page.svelte",
            tagName: "form",
            directiveName: "use:enhance",
            actionName: "enhance",
            bindingName: "enhance",
            bindingSource: "../src/svelte-action",
            source: "sfc-framework-svelte-action-directive",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-svelte-action-directive",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "svelte",
            conventionKind: "action-directive",
            consumerFile: "components/Page.svelte",
            tagName: "div",
            directiveName: "use:localAction",
            actionName: "localAction",
            bindingName: "localAction",
            bindingSource: "components/Page.svelte",
            bindingKind: "local-function",
            source: "sfc-framework-svelte-action-directive",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-svelte-action-directive",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "svelte",
            conventionKind: "action-directive",
            consumerFile: "components/Page.svelte",
            tagName: "section",
            directiveName: "use:localConstAction",
            actionName: "localConstAction",
            bindingName: "localConstAction",
            bindingSource: "components/Page.svelte",
            bindingKind: "local-const-function",
            source: "sfc-framework-svelte-action-directive",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-svelte-action-directive",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "svelte",
            conventionKind: "store-auto-subscription",
            consumerFile: "components/Page.svelte",
            subscriptionName: "$importedCount",
            storeName: "importedCount",
            bindingName: "importedCount",
            bindingSource: "../src/svelte-store",
            bindingKind: "named",
            source: "sfc-framework-svelte-store-subscription",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-svelte-store-subscription",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "svelte",
            conventionKind: "store-auto-subscription",
            consumerFile: "components/Page.svelte",
            subscriptionName: "$localCount",
            storeName: "localCount",
            bindingName: "localCount",
            bindingSource: "components/Page.svelte",
            bindingKind: "local-store-factory",
            source: "sfc-framework-svelte-store-subscription",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-svelte-store-subscription",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "vue",
            conventionKind: "macro-registration",
            consumerFile: "pages/Macro.vue",
            macroName: "defineOptions",
            componentName: "MacroCard",
            normalizedTagNames: expect.arrayContaining([
              "MacroCard",
              "macro-card",
            ]),
            bindingName: "MacroCard",
            bindingSource: "../components/MacroCard.vue",
            source: "sfc-framework-vue-macro-registration",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-vue-macro-registration",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "vue",
            conventionKind: "macro-registration",
            consumerFile: "pages/Macro.vue",
            componentName: "macro-alias",
            normalizedTagNames: expect.arrayContaining(["MacroAlias"]),
            bindingName: "MacroAlias",
            bindingSource: "../src/macro-alias",
          }),
          expect.objectContaining({
            framework: "vue",
            conventionKind: "options-registration",
            consumerFile: "pages/Options.vue",
            optionName: "components",
            componentName: "OptionsCard",
            normalizedTagNames: expect.arrayContaining([
              "OptionsCard",
              "options-card",
            ]),
            bindingName: "OptionsCard",
            bindingSource: "../components/OptionsCard.vue",
            source: "sfc-framework-vue-options-registration",
            confidence: "framework-convention-observed",
            status: "muted",
            reason: "sfc-framework-vue-options-registration",
            eligibleForFanIn: false,
            eligibleForSafeFix: false,
          }),
          expect.objectContaining({
            framework: "vue",
            conventionKind: "options-registration",
            consumerFile: "pages/Options.vue",
            componentName: "options-alias",
            normalizedTagNames: expect.arrayContaining(["OptionsAlias"]),
            bindingName: "OptionsAlias",
            bindingSource: "../src/options-alias",
          }),
        ]),
      );
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) => entry.hookName === "app:mounted",
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents
          ?.filter(
            (entry) =>
              entry.reason ===
              "sfc-framework-nuxt-custom-resolver-unavailable",
          )
          .every(
            (entry) =>
              !entry.componentName &&
              !entry.componentDir &&
              !entry.sourceFile &&
              !entry.resolvedFile,
          ),
      ).toBe(true);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.sourceFile === "app/shared/components/ConfiguredOnly.vue" ||
            entry.sourceFile === "shared/components/RootDecoy.vue" ||
            entry.sourceFile === "local-components/LocalConfigured.vue",
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason?.startsWith("sfc-framework-nuxt-components-alias") &&
            (entry.componentName === "TypeOnlyAlias" ||
              entry.componentName === "componentNames" ||
              entry.importedName === "componentNames" ||
              entry.bindingName === "componentNames"),
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-astro-client-directive" &&
            (entry.tagName === "MissingAstroClient" || entry.tagName === "div"),
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-svelte-action-directive" &&
            (entry.actionName === "missingAction" ||
              entry.actionName === "commentAction" ||
              entry.actionName === "notActionValue"),
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-svelte-store-subscription" &&
            (entry.storeName === "missingStore" ||
              entry.storeName === "commentStore" ||
              entry.storeName === "plainTextStoreMention" ||
              entry.storeName === "notStoreValue"),
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents?.filter(
          (entry) =>
            entry.reason === "sfc-framework-svelte-store-subscription" &&
            entry.subscriptionName === "$localCount",
        ),
      ).toHaveLength(1);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-vue-macro-registration" &&
            (entry.componentName === "MissingMacro" ||
              entry.componentName === "CommentOnlyMacro" ||
              entry.componentName === "dynamicMacroName"),
        ),
      ).toBe(false);
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-vue-options-registration" &&
            (entry.componentName === "MissingOptions" ||
              entry.componentName === "CommentOnlyOptions" ||
              entry.componentName === "dynamicOptionsName"),
        ),
      ).toBe(false);
      expect(
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.kind === "sfc-framework-nuxt-fs-convention" ||
            edge.source === "sfc-framework-nuxt-fs-convention" ||
            edge.kind === "sfc-framework-nuxt-app-dir-convention" ||
            edge.source === "sfc-framework-nuxt-app-dir-convention" ||
            edge.kind === "sfc-framework-nuxt-components-alias" ||
            edge.source === "sfc-framework-nuxt-components-alias" ||
            edge.source === "#components" ||
            edge.kind === "sfc-framework-nuxt-components-dir-config" ||
            edge.source === "sfc-framework-nuxt-components-dir-config" ||
            edge.kind ===
              "sfc-framework-nuxt-custom-resolver-unavailable" ||
            edge.source ===
              "sfc-framework-nuxt-custom-resolver-unavailable" ||
            edge.kind === "sfc-framework-nuxt-layer-extends-unavailable" ||
            edge.source === "sfc-framework-nuxt-layer-extends-unavailable" ||
            edge.kind === "sfc-framework-nuxt-module-package-unavailable" ||
            edge.source === "sfc-framework-nuxt-module-package-unavailable" ||
            edge.kind === "sfc-framework-auto-import-plugin-config" ||
            edge.source === "sfc-framework-auto-import-plugin-config" ||
            edge.kind === "sfc-framework-astro-client-directive" ||
            edge.source === "sfc-framework-astro-client-directive" ||
            edge.kind === "sfc-framework-svelte-action-directive" ||
            edge.source === "sfc-framework-svelte-action-directive" ||
            edge.kind === "sfc-framework-svelte-store-subscription" ||
            edge.source === "sfc-framework-svelte-store-subscription" ||
            edge.kind === "sfc-framework-vue-macro-registration" ||
            edge.source === "sfc-framework-vue-macro-registration" ||
            edge.kind === "sfc-framework-vue-options-registration" ||
            edge.source === "sfc-framework-vue-options-registration",
        ),
      ).toBe(false);
      expect(
        symbols.unresolvedInternalSpecifierRecords?.some(
          (record) => record.specifier === "#components",
        ),
      ).toBe(false);
      expect(
        symbols.dependencyImportConsumers?.some(
          (entry) => entry.fromSpec === "#components",
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  }, 15000);

  it("SFC-3. requires a Nuxt signal before emitting filesystem convention evidence", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-sfc-convention-gate-",
      packageJson: { name: "plain-vue-fixture", type: "module" },
      outputDirName: "out",
    });
    try {
      fixture.write(
        "components/Loose.vue",
        [
          "<template><Loose /></template>",
          '<script setup lang="ts">',
          "const loose = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/base/Button.vue",
        [
          "<template><button>Base button</button></template>",
          '<script setup lang="ts">',
          "const baseButton = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "components/user/index.vue",
        [
          "<template><section>User index</section></template>",
          '<script setup lang="ts">',
          "const userIndex = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "app/components/base/AppButton.vue",
        [
          "<template><button>App button</button></template>",
          '<script setup lang="ts">',
          "const appButton = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "pages/NoNuxtAlias.vue",
        [
          "<template><section>No Nuxt signal</section></template>",
          '<script setup lang="ts">',
          "import { Loose } from '#components';",
          "</script>",
          "",
        ].join("\n"),
      );
      const symbols = runSymbolGraph(fixture);
      expect(symbols.meta?.supports?.sfcFrameworkConventionComponents).toBe(
        true,
      );
      expect(symbols.uses?.sfcFrameworkConventionComponents).toBe(0);
      expect(symbols.sfcFrameworkConventionComponents).toEqual([]);
      expect(
        symbols.unresolvedInternalSpecifierRecords?.some(
          (record) => record.specifier === "#components",
        ),
      ).toBe(false);
      expect(
        symbols.dependencyImportConsumers?.some(
          (entry) => entry.fromSpec === "#components",
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });

  it("SFC-4. keeps app-dir convention evidence behind a Nuxt 4 or srcDir app signal", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-sfc-nuxt3-app-gate-",
      packageJson: {
        name: "nuxt3-app-dir-gate-fixture",
        type: "module",
        dependencies: { nuxt: "^3.12.0" },
      },
      outputDirName: "out",
    });
    try {
      fixture.write(
        "components/LegacyRoot.vue",
        [
          "<template><LegacyRoot /></template>",
          '<script setup lang="ts">',
          "const legacyRoot = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "app/components/base/AppButton.vue",
        [
          "<template><button>App button</button></template>",
          '<script setup lang="ts">',
          "const appButton = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      const symbols = runSymbolGraph(fixture);
      expect(symbols.sfcFrameworkConventionComponents).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-directory",
            componentName: "LegacyRoot",
          }),
        ]),
      );
      expect(
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.conventionKind === "nuxt-app-components-directory" ||
            entry.source === "sfc-framework-nuxt-app-dir-convention" ||
            entry.sourceFile === "app/components/base/AppButton.vue",
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });

  it("SFC-5. accepts Nuxt 4 dependency ranges for app-dir convention evidence", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-sfc-nuxt4-app-gate-",
      packageJson: {
        name: "nuxt4-app-dir-fixture",
        type: "module",
        dependencies: { nuxt: "^4.0.0" },
      },
      outputDirName: "out",
    });
    try {
      fixture.write(
        "nuxt.config.ts",
        [
          "export default defineNuxtConfig({",
          "  components: { dirs: ['~/shared/components'] },",
          "});",
          "",
        ].join("\n"),
      );
      fixture.write(
        "app/components/base/AppButton.vue",
        [
          "<template><button>App button</button></template>",
          '<script setup lang="ts">',
          "const appButton = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "app/shared/components/DefaultSrcDirConfigured.vue",
        [
          "<template><section>Default srcDir configured</section></template>",
          '<script setup lang="ts">',
          "const defaultSrcDirConfigured = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      fixture.write(
        "shared/components/RootDecoy.vue",
        [
          "<template><section>Root decoy</section></template>",
          '<script setup lang="ts">',
          "const rootDecoy = true;",
          "</script>",
          "",
        ].join("\n"),
      );
      const symbols = runSymbolGraph(fixture);
      expect(symbols.sfcFrameworkConventionComponents).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-app-components-directory",
            componentName: "BaseAppButton",
            sourceFile: "app/components/base/AppButton.vue",
            reason: "sfc-framework-nuxt-app-dir-convention",
          }),
          expect.objectContaining({
            framework: "nuxt",
            conventionKind: "nuxt-components-dir-config",
            componentDir: "~/shared/components",
            resolvedDir: "app/shared/components",
            reason: "sfc-framework-nuxt-components-dir-config",
          }),
        ]),
      );
    } finally {
      fixture.cleanup();
    }
  });
});
