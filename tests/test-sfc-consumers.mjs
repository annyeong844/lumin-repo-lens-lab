import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  rmSync,
  readFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import {
  parseSfcGeneratedComponentManifests,
  parseSfcImportConsumers,
  parseSfcGlobalComponentRegistrations,
  parseSfcScriptSources,
  parseSfcStyleAssetReferences,
  parseSfcTemplateComponentRefs,
} from "../_lib/sfc-consumers.mjs";

const REPO_ROOT = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);

let passed = 0;
let failed = 0;
function assert(label, ok, detail = "") {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.error(`  FAIL  ${label}\n        ${detail}`);
  }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

{
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
    .map((i) => `${i.consumerFile}:${i.fromSpec}:${i.name}:${i.kind}`)
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

  assert(
    "SFC-1a. parser records Vue named imports by imported name",
    names.includes("components/App.vue:../src/card:UsedByVue:import"),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1b. parser records Vue default imports",
    names.includes("components/App.vue:../src/card:default:default"),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1c. parser records Vue namespace imports",
    names.includes("components/App.vue:../src/store:*:namespace"),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1d. parser records side-effect imports",
    names.includes(
      "components/App.vue:external-sfc-package:*:import-side-effect",
    ),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1e. parser records Svelte script imports",
    names.includes(
      "components/Page.svelte:../src/svelte-use:UsedBySvelte:import",
    ),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1f. parser records Astro frontmatter imports",
    names.includes("pages/Home.astro:../src/astro-use:UsedByAstro:import"),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1g. parser honors declared TSX dialect in SFC scripts",
    names.includes("components/Tsx.vue:../src/tsx-card:UsedByTsx:import"),
    JSON.stringify(imports),
  );
  assert(
    "SFC-1h. parser ignores template text and external script src",
    !names.some(
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
  );
  assert(
    "SFC-1i. parser records only literal relative script src as reachability candidates",
    sourceNames.length === 2 &&
      sourceNames.includes("components/App.vue:./external.ts:sfc-script-src") &&
      sourceNames.includes(
        "components/Page.svelte:../src/svelte-src.ts:sfc-script-src",
      ),
    JSON.stringify(scriptSources),
  );
  assert(
    "SFC-1j. parser records only literal relative style asset references",
    styleNames.length === 5 &&
      styleNames.includes("components/App.vue:./logo.svg:sfc-style-url:url") &&
      styleNames.includes(
        "components/App.vue:./my icon.svg:sfc-style-url:url",
      ) &&
      styleNames.includes(
        "components/App.vue:./theme.css:sfc-style-import:import",
      ) &&
      styleNames.includes(
        "components/Page.svelte:../assets/icon.svg:sfc-style-url:url",
      ) &&
      styleNames.includes(
        "pages/Home.astro:./astro.css:sfc-style-import:import",
      ) &&
      !styleNames.some(
        (name) =>
          name.includes("commented.svg") ||
          name.includes("remote.png") ||
          name.includes("template"),
      ),
    JSON.stringify(styleAssets),
  );
  assert(
    "SFC-1k. parser records explicit template component refs as review-only bindings",
    templateNames.includes("components/App.vue:Card:Card:binding:") &&
      templateNames.includes(
        "components/App.vue:user-list:UserList:binding:",
      ) &&
      templateNames.includes("components/App.vue:SfcCard:SfcCard:binding:") &&
      templateNames.includes(
        "components/Page.svelte:UsedBySvelte:UsedBySvelte:binding:",
      ) &&
      templateNames.includes(
        "pages/Home.astro:UsedByAstro:UsedByAstro:binding:",
      ) &&
      templateRefs.every(
        (entry) =>
          entry.source === "sfc-template-component-ref" &&
          entry.eligibleForFanIn === false &&
          entry.eligibleForSafeFix === false,
      ),
    JSON.stringify(templateRefs),
  );
  assert(
    "SFC-1l. parser mutes dynamic and namespace template refs without guessing globals",
    templateNames.includes(
      "components/App.vue:component:DynamicCard:muted:sfc-template-dynamic-component",
    ) &&
      templateNames.includes(
        "components/App.vue:UI.Card:UI:muted:sfc-template-namespace-component",
      ) &&
      templateNames.includes(
        "components/Page.svelte:svelte:component:UsedBySvelte:muted:sfc-template-dynamic-component",
      ) &&
      !templateNames.some(
        (name) =>
          name.includes("my-widget") ||
          name.includes("CommentedCard") ||
          name.includes("TemplateOnly"),
      ),
    JSON.stringify(templateRefs),
  );
  assert(
    "SFC-1m. parser preserves Svelte template ref source lines after script/style blocks",
    svelteStaticRef?.line === 5 && svelteDynamicRef?.line === 6,
    JSON.stringify({ svelteStaticRef, svelteDynamicRef }),
  );

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
  assert(
    "SFC-1n. parser records explicit Vue global component registrations as review-only evidence",
    registrationNames.includes(
      "src/main.ts:UserCard:UserCard:registration-syntax:",
    ) &&
      registrationNames.includes(
        "src/main.ts:admin-panel:AdminPanel:registration-syntax:",
      ) &&
      registrationNames.includes(
        "src/main.ts:SSRCard:SSRCard:registration-syntax:",
      ) &&
      registrationNames.includes(
        "src/main.ts:SharedGlobal:UserCard:registration-syntax:",
      ) &&
      registrationNames.includes(
        "src/main.ts:SharedGlobal:SSRCard:registration-syntax:",
      ) &&
      registrationNames.includes(
        "src/main.ts:ChainedCard:ChainedCard:registration-syntax:",
      ) &&
      registrationNames.includes(
        "src/main.ts:PluginCard:PluginCard:registration-syntax:",
      ) &&
      !registrationNames.some((name) => name.includes("MountedCard")) &&
      registrations.some(
        (entry) =>
          entry.componentName === "UserCard" &&
          entry.normalizedTagNames?.includes("user-card") &&
          entry.bindingSource === "./components/UserCard.vue" &&
          entry.source === "sfc-global-component-registration" &&
          entry.eligibleForFanIn === false &&
          entry.eligibleForSafeFix === false,
      ),
    JSON.stringify(registrations),
  );
  assert(
    "SFC-1o. parser mutes dynamic global registration names and unsupported values",
    registrationNames.includes(
      "src/main.ts::UserCard:muted:sfc-global-component-name-dynamic",
    ) &&
      registrationNames.includes(
        "src/main.ts:FactoryCard::muted:sfc-global-component-value-unsupported",
      ),
    JSON.stringify(registrations),
  );
  assert(
    "SFC-1p. parser records async factories and duplicate global registrations as muted evidence",
    registrationNames.includes(
      "src/main.ts:AsyncCard::muted:sfc-global-component-async-factory",
    ) &&
      registrationNames.includes(
        "src/main.ts:AsyncByLoader::muted:sfc-global-component-async-factory-nonliteral",
      ) &&
      registrationNames.includes(
        "src/main.ts:DuplicateCard:DuplicateOne:muted:sfc-global-component-duplicate-registration",
      ) &&
      registrationNames.includes(
        "src/main.ts:DuplicateCard:DuplicateTwo:muted:sfc-global-component-duplicate-registration",
      ) &&
      registrations.some(
        (entry) =>
          entry.componentName === "AsyncCard" &&
          entry.fromSpec === "./components/AsyncCard.vue" &&
          entry.factoryKind === "defineAsyncComponent" &&
          entry.eligibleForFanIn === false &&
          entry.eligibleForSafeFix === false,
      ) &&
      registrations
        .filter((entry) => entry.componentName === "DuplicateCard")
        .every((entry) => entry.ambiguityKey === "DuplicateCard"),
    JSON.stringify(registrations),
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
  const generatedManifestNames = generatedManifestRecords
    .map(
      (entry) =>
        `${entry.manifestFile}:${entry.componentName}:${entry.fromSpec}:${entry.manifestKind}`,
    )
    .sort();
  assert(
    "SFC-1q. parser records literal generated component manifest mappings",
    generatedManifestNames.includes(
      "components.d.ts:BaseButton:./components/BaseButton.vue:unplugin-vue-components-dts",
    ) &&
      generatedManifestNames.includes(
        "components.d.ts:user-card:./components/UserCard.vue:unplugin-vue-components-dts",
      ) &&
      generatedManifestNames.includes(
        "components.d.ts:SourceCard:./src/SourceCard.ts:unplugin-vue-components-dts",
      ) &&
      generatedManifestRecords.some(
        (entry) =>
          entry.componentName === "BaseButton" &&
          entry.normalizedTagNames?.includes("base-button") &&
          entry.source === "sfc-framework-generated-manifest" &&
          entry.confidence === "generated-manifest-availability" &&
          entry.eligibleForFanIn === false &&
          entry.eligibleForSafeFix === false,
      ) &&
      generatedManifestRecords.some(
        (entry) =>
          entry.componentName === "DynamicCard" &&
          entry.status === "skipped" &&
          entry.reason === "sfc-framework-generated-manifest-nonliteral" &&
          entry.fromSpec === "./components/Dynamic.vue" &&
          Array.isArray(entry.normalizedTagNames) &&
          entry.normalizedTagNames.length === 0,
      ) &&
      generatedManifestRecords.some(
        (entry) =>
          entry.componentName === "[computed]" &&
          entry.status === "skipped" &&
          entry.reason === "sfc-framework-generated-manifest-nonliteral" &&
          entry.fromSpec === "./components/Expr.vue" &&
          entry.computedKeySource === "prefix + 'Card'",
      ) &&
      !generatedManifestNames.some((name) => name.includes("RouterLink")),
    JSON.stringify(generatedManifestRecords),
  );
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "sfc-consumer-graph-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "sfc-fixture", type: "module" }),
    );
    write(
      fx,
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
    write(
      fx,
      "src/card.ts",
      "export function UsedByVue() { return null; }\n" +
        "export const TemplateOnly = 1;\n" +
        "export const Unused = 1;\n" +
        "export default function DefaultCard() { return null; }\n",
    );
    write(
      fx,
      "src/store.ts",
      "export const UsedByNamespace = 1;\n" +
        "export const AlsoProtectedByNamespace = 2;\n",
    );
    write(fx, "src/side-effect.ts", "export const SideEffectExport = 1;\n");
    write(fx, "src/svelte-use.ts", "export const UsedBySvelte = 1;\n");
    write(fx, "src/svelte-action.ts", "export function enhance() {}\n");
    write(
      fx,
      "src/svelte-store.ts",
      "export const importedCount = { subscribe() {} };\n",
    );
    write(fx, "src/astro-use.ts", "export const UsedByAstro = 1;\n");
    write(
      fx,
      "src/tsx-card.tsx",
      "export function UsedByTsx() { return null; }\n",
    );
    write(
      fx,
      "src/user-list.ts",
      "export default function UserList() { return null; }\n",
    );
    write(
      fx,
      "src/dynamic-card.ts",
      "export default function DynamicCard() { return null; }\n",
    );
    write(fx, "src/ui.ts", "export const Card = 1;\n");
    write(
      fx,
      "src/template-only-component.ts",
      "export default function TemplateOnlyComponent() { return null; }\n",
    );
    write(fx, "src/script-src-logic.ts", "export const ScriptSrcExport = 1;\n");
    write(
      fx,
      "src/svelte-src-logic.ts",
      "export const SvelteScriptSrcExport = 1;\n",
    );
    write(fx, "src/empty-src.ts", "export const EmptySrcOnly = 1;\n");
    write(
      fx,
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
    write(
      fx,
      "src/registered-source.ts",
      "export function RegisteredSource() { return null; }\n",
    );
    write(
      fx,
      "src/ssr-registered-source.ts",
      "export function SsrRegisteredSource() { return null; }\n",
    );
    write(
      fx,
      "src/chained-registered-source.ts",
      "export function ChainedRegisteredSource() { return null; }\n",
    );
    write(
      fx,
      "src/ManifestSource.ts",
      "export default function ManifestSource() { return null; }\n",
    );
    write(
      fx,
      "src/macro-alias.ts",
      "export default function MacroAlias() { return null; }\n",
    );
    write(
      fx,
      "src/options-alias.ts",
      "export default function OptionsAlias() { return null; }\n",
    );
    write(fx, "assets/logo.svg", "<svg></svg>\n");
    write(fx, "assets/my icon.svg", "<svg></svg>\n");
    write(fx, "assets/icon.svg", "<svg></svg>\n");
    write(fx, "styles/theme.css", ".theme { color: red; }\n");
    write(
      fx,
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
    write(
      fx,
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
    write(
      fx,
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
    write(
      fx,
      "components/Tsx.vue",
      [
        '<script lang="tsx">',
        "import { UsedByTsx } from '../src/tsx-card';",
        "const node = <div>{UsedByTsx}</div>;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/SfcCard.vue",
      [
        "<template><article>SFC card</article></template>",
        '<script setup lang="ts">',
        "const localOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/GlobalCard.vue",
      [
        "<template><article>Global card</article></template>",
        '<script setup lang="ts">',
        "const globalOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/PluginCard.vue",
      [
        "<template><article>Plugin card</article></template>",
        '<script setup lang="ts">',
        "const pluginOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/AsyncGlobal.vue",
      [
        "<template><article>Async global</article></template>",
        '<script setup lang="ts">',
        "const asyncOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/DuplicateOne.vue",
      [
        "<template><article>Duplicate one</article></template>",
        '<script setup lang="ts">',
        "const duplicateOneOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/DuplicateTwo.vue",
      [
        "<template><article>Duplicate two</article></template>",
        '<script setup lang="ts">',
        "const duplicateTwoOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/ManifestButton.vue",
      [
        "<template><button>Manifest</button></template>",
        '<script setup lang="ts">',
        "const manifestOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/MacroCard.vue",
      [
        "<template><article>Macro card</article></template>",
        '<script setup lang="ts">',
        "const macroCardOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/OptionsCard.vue",
      [
        "<template><article>Options card</article></template>",
        '<script setup lang="ts">',
        "const optionsCardOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
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
    write(
      fx,
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
    write(
      fx,
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
    write(
      fx,
      "components/NuxtManifest.vue",
      [
        "<template><button>Nuxt</button></template>",
        '<script setup lang="ts">',
        "const nuxtOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/ConventionOnly.vue",
      [
        "<template><button>Convention</button></template>",
        '<script setup lang="ts">',
        "const conventionOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/base/Button.vue",
      [
        "<template><button>Base button</button></template>",
        '<script setup lang="ts">',
        "const baseButton = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/user/index.vue",
      [
        "<template><section>User index</section></template>",
        '<script setup lang="ts">',
        "const userIndex = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "app/components/base/AppButton.vue",
      [
        "<template><button>App button</button></template>",
        '<script setup lang="ts">',
        "const appButton = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "shared/components/RootDecoy.vue",
      [
        "<template><section>Root decoy</section></template>",
        '<script setup lang="ts">',
        "const rootDecoy = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "app/shared/components/ConfiguredOnly.vue",
      [
        "<template><section>Configured only</section></template>",
        '<script setup lang="ts">',
        "const configuredOnly = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "local-components/LocalConfigured.vue",
      [
        "<template><section>Local configured</section></template>",
        '<script setup lang="ts">',
        "const localConfigured = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
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
    write(
      fx,
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
    write(
      fx,
      "vite.config.ts",
      [
        "import Components from 'unplugin-vue-components/vite';",
        "export default { plugins: [Components()] };",
        "",
      ].join("\n"),
    );
    write(
      fx,
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
    write(
      fx,
      "webpack.config.js",
      [
        "module.exports = {",
        "  plugins: [require('unplugin-vue-components/vite')()],",
        "};",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/ScriptSrc.vue",
      ['<script src="../src/script-src-logic.ts"></script>', ""].join("\n"),
    );
    write(
      fx,
      "components/SvelteSrc.svelte",
      ['<script src="../src/svelte-src-logic.ts"></script>', ""].join("\n"),
    );
    write(
      fx,
      "components/MissingSrc.vue",
      ['<script src="../src/missing-script-src.ts"></script>', ""].join("\n"),
    );
    write(
      fx,
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
    write(
      fx,
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

    const outDir = path.join(fx, "out");
    const run = spawnSync(
      process.execPath,
      [
        path.join(REPO_ROOT, "build-symbol-graph.mjs"),
        "--root",
        fx,
        "--output",
        outDir,
      ],
      {
        cwd: REPO_ROOT,
        encoding: "utf8",
      },
    );
    const symbolsPath = path.join(outDir, "symbols.json");
    const symbols = JSON.parse(readFileSync(symbolsPath, "utf8"));
    const dead = new Set(
      (symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`),
    );

    assert(
      "SFC-2a. build-symbol-graph succeeds on SFC fixture",
      run.status === 0,
      `stdout=${run.stdout}\nstderr=${run.stderr}`,
    );
    assert(
      "SFC-2b. SFC named import protects the referenced export",
      !dead.has("src/card.ts::UsedByVue"),
      JSON.stringify([...dead].sort()),
    );
    assert(
      "SFC-2c. SFC default import protects the default export",
      !dead.has("src/card.ts::default"),
      JSON.stringify([...dead].sort()),
    );
    assert(
      "SFC-2d. SFC namespace import protects module exports broadly",
      !dead.has("src/store.ts::UsedByNamespace") &&
        !dead.has("src/store.ts::AlsoProtectedByNamespace"),
      JSON.stringify([...dead].sort()),
    );
    assert(
      "SFC-2e. SFC TSX script import protects the referenced export",
      !dead.has("src/tsx-card.tsx::UsedByTsx"),
      JSON.stringify([...dead].sort()),
    );
    assert(
      "SFC-2f. template-only import text does not protect exports",
      dead.has("src/card.ts::TemplateOnly") &&
        dead.has("src/card.ts::Unused") &&
        dead.has("src/side-effect.ts::SideEffectExport") &&
        dead.has("src/empty-src.ts::EmptySrcOnly"),
      JSON.stringify([...dead].sort()),
    );
    assert(
      "SFC-2g. SFC script consumers are counted and advertised",
      symbols.meta?.supports?.sfcScriptImportConsumers === true &&
        symbols.uses?.sfcScriptConsumers === 14,
      JSON.stringify(symbols.uses),
    );
    assert(
      "SFC-2h. SFC external imports feed dependencyImportConsumers",
      symbols.dependencyImportConsumers?.some(
        (entry) =>
          entry.file === "components/App.vue" &&
          entry.fromSpec === "external-sfc-package" &&
          entry.source === "sfc-script-import",
      ),
      JSON.stringify(symbols.dependencyImportConsumers),
    );
    assert(
      "SFC-2i. SFC internal imports feed resolvedInternalEdges",
      symbols.resolvedInternalEdges?.some(
        (edge) =>
          edge.from === "components/App.vue" &&
          edge.to === "src/side-effect.ts" &&
          edge.kind === "import-side-effect",
      ),
      JSON.stringify(symbols.resolvedInternalEdges),
    );
    assert(
      "SFC-2j. SFC script src feeds reachability edge with a distinct kind",
      symbols.meta?.supports?.sfcScriptSrcReachability === true &&
        symbols.uses?.sfcScriptSrcReachability === 2 &&
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.from === "components/ScriptSrc.vue" &&
            edge.to === "src/script-src-logic.ts" &&
            edge.kind === "sfc-script-src" &&
            edge.source === "../src/script-src-logic.ts",
        ) &&
        symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.from === "components/SvelteSrc.svelte" &&
            edge.to === "src/svelte-src-logic.ts" &&
            edge.kind === "sfc-script-src" &&
            edge.source === "../src/svelte-src-logic.ts",
        ),
      JSON.stringify({
        uses: symbols.uses,
        edges: symbols.resolvedInternalEdges,
      }),
    );
    assert(
      "SFC-2k. SFC script src does not create named export fan-in",
      dead.has("src/script-src-logic.ts::ScriptSrcExport") &&
        dead.has("src/svelte-src-logic.ts::SvelteScriptSrcExport") &&
        symbols.fanInByIdentity?.[
          "src/script-src-logic.ts::ScriptSrcExport"
        ] === 0 &&
        symbols.fanInByIdentity?.[
          "src/svelte-src-logic.ts::SvelteScriptSrcExport"
        ] === 0,
      JSON.stringify({
        dead: [...dead].sort(),
        fanIn: symbols.fanInByIdentity,
      }),
    );
    assert(
      "SFC-2l. unsupported script src forms do not become concrete edges",
      symbols.unresolvedInternalSpecifierRecords?.some(
        (record) =>
          record.consumerFile === "components/MissingSrc.vue" &&
          record.specifier === "../src/missing-script-src.ts" &&
          record.reason === "sfc-script-src-unresolved",
      ) &&
        !symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.source === "external-package" ||
            edge.source === "https://example.test/remote.ts" ||
            edge.source === "dynamicSource",
        ),
      JSON.stringify({
        edges: symbols.resolvedInternalEdges,
        unresolved: symbols.unresolvedInternalSpecifierRecords,
      }),
    );
    assert(
      "SFC-2m. SFC style assets are asset evidence, not source graph edges",
      symbols.meta?.supports?.sfcStyleAssetReferences === true &&
        symbols.uses?.sfcStyleAssetReferences === 3 &&
        symbols.sfcStyleAssetReferences?.some(
          (entry) =>
            entry.consumerFile === "components/StyleAsset.vue" &&
            entry.fromSpec === "../assets/logo.svg" &&
            entry.resolvedFile === "assets/logo.svg" &&
            entry.source === "sfc-style-url" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcStyleAssetReferences?.some(
          (entry) =>
            entry.consumerFile === "components/StyleAsset.vue" &&
            entry.fromSpec === "../assets/my icon.svg" &&
            entry.resolvedFile === "assets/my icon.svg" &&
            entry.source === "sfc-style-url" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcStyleAssetReferences?.some(
          (entry) =>
            entry.consumerFile === "components/StyleAsset.vue" &&
            entry.fromSpec === "../styles/theme.css" &&
            entry.resolvedFile === "styles/theme.css" &&
            entry.source === "sfc-style-import" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcStyleAssetReferences?.some(
          (entry) =>
            entry.consumerFile === "components/StyleAsset.vue" &&
            entry.fromSpec === "../assets/missing.svg" &&
            entry.reason === "sfc-style-asset-unresolved" &&
            entry.status === "unresolved",
        ) &&
        !symbols.resolvedInternalEdges?.some(
          (edge) =>
            edge.source === "../assets/logo.svg" ||
            edge.source === "../assets/my icon.svg" ||
            edge.source === "../styles/theme.css" ||
            edge.source === "../assets/missing.svg",
        ) &&
        !symbols.sfcStyleAssetReferences?.some(
          (entry) =>
            entry.fromSpec.includes("commented.svg") ||
            entry.fromSpec.includes("remote.png") ||
            entry.fromSpec.includes("some-package") ||
            entry.fromSpec.includes("template-only.svg"),
        ),
      JSON.stringify({
        uses: symbols.uses,
        styleAssets: symbols.sfcStyleAssetReferences,
        edges: symbols.resolvedInternalEdges,
      }),
    );
    assert(
      "SFC-2n. SFC template component refs are review-only artifact evidence",
      symbols.meta?.supports?.sfcTemplateComponentRefs === true &&
        symbols.uses?.sfcTemplateComponentRefs === 9 &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "components/App.vue" &&
            entry.tagName === "Card" &&
            entry.bindingName === "Card" &&
            entry.bindingSource === "../src/card" &&
            entry.resolvedFile === "src/card.ts" &&
            entry.status === "resolved" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "components/App.vue" &&
            entry.tagName === "user-list" &&
            entry.normalizedTagName === "UserList" &&
            entry.resolvedFile === "src/user-list.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "components/App.vue" &&
            entry.tagName === "SfcCard" &&
            entry.bindingSource === "./SfcCard.vue" &&
            entry.resolvedFile === "components/SfcCard.vue" &&
            entry.status === "muted" &&
            entry.reason === "sfc-template-component-non-source-binding" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "pages/Home.astro" &&
            entry.tagName === "UsedByAstro" &&
            entry.resolvedFile === "src/astro-use.ts" &&
            entry.status === "resolved",
        ),
      JSON.stringify(symbols.sfcTemplateComponentRefs),
    );
    assert(
      "SFC-2o. SFC template dynamic, namespace, and missing refs stay weak",
      symbols.sfcTemplateComponentRefs?.some(
        (entry) =>
          entry.consumerFile === "components/App.vue" &&
          entry.tagName === "component" &&
          entry.bindingName === "DynamicCard" &&
          entry.status === "muted" &&
          entry.reason === "sfc-template-dynamic-component",
      ) &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "components/App.vue" &&
            entry.tagName === "UI.Card" &&
            entry.bindingName === "UI" &&
            entry.memberName === "Card" &&
            entry.status === "muted" &&
            entry.reason === "sfc-template-namespace-component",
        ) &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "components/App.vue" &&
            entry.tagName === "MissingCard" &&
            entry.status === "unresolved" &&
            entry.reason === "sfc-template-component-unresolved",
        ) &&
        symbols.sfcTemplateComponentRefs?.some(
          (entry) =>
            entry.consumerFile === "components/Page.svelte" &&
            entry.tagName === "svelte:component" &&
            entry.status === "muted" &&
            entry.reason === "sfc-template-dynamic-component",
        ),
      JSON.stringify(symbols.sfcTemplateComponentRefs),
    );
    assert(
      "SFC-2p. SFC template refs do not create graph edges or fan-in",
      !symbols.resolvedInternalEdges?.some(
        (edge) =>
          edge.kind === "sfc-template-component-ref" ||
          edge.source === "sfc-template-component-ref" ||
          edge.source === "./SfcCard.vue",
      ) &&
        !symbols.sfcTemplateComponentRefs?.some(
          (entry) => entry.tagName === "TemplateOnlyComponent",
        ) &&
        dead.has("src/template-only-component.ts::default") &&
        symbols.fanInByIdentity?.["src/template-only-component.ts::default"] ===
          0,
      JSON.stringify({
        templateRefs: symbols.sfcTemplateComponentRefs,
        edges: symbols.resolvedInternalEdges,
        dead: [...dead].sort(),
        fanIn: symbols.fanInByIdentity,
      }),
    );
    assert(
      "SFC-2q. SFC global component registrations are review-only artifact evidence",
      symbols.meta?.supports?.sfcGlobalComponentRegistrations === true &&
        symbols.uses?.sfcGlobalComponentRegistrations === 12 &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.registrationFile === "src/main.ts" &&
            entry.componentName === "GlobalCard" &&
            entry.bindingName === "GlobalCard" &&
            entry.bindingSource === "../components/GlobalCard.vue" &&
            entry.resolvedFile === "components/GlobalCard.vue" &&
            entry.status === "muted" &&
            entry.reason === "sfc-global-component-non-source-binding" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "registered-source" &&
            entry.bindingName === "RegisteredSource" &&
            entry.resolvedFile === "src/registered-source.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "ssr-registered-source" &&
            entry.bindingName === "SsrRegisteredSource" &&
            entry.resolvedFile === "src/ssr-registered-source.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.api === "app.component" &&
            entry.componentName === "shared-source" &&
            entry.bindingName === "RegisteredSource" &&
            entry.resolvedFile === "src/registered-source.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.api === "ssrApp.component" &&
            entry.componentName === "shared-source" &&
            entry.bindingName === "SsrRegisteredSource" &&
            entry.resolvedFile === "src/ssr-registered-source.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "chained-registered-source" &&
            entry.bindingName === "ChainedRegisteredSource" &&
            entry.resolvedFile === "src/chained-registered-source.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "PluginCard" &&
            entry.bindingName === "PluginCard" &&
            entry.resolvedFile === "components/PluginCard.vue" &&
            entry.status === "muted" &&
            entry.reason === "sfc-global-component-non-source-binding",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "AsyncGlobal" &&
            entry.fromSpec === "../components/AsyncGlobal.vue" &&
            entry.factoryKind === "defineAsyncComponent" &&
            entry.resolvedFile === "components/AsyncGlobal.vue" &&
            entry.status === "muted" &&
            entry.reason === "sfc-global-component-async-factory",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "DuplicateGlobal" &&
            entry.bindingName === "DuplicateOne" &&
            entry.resolvedFile === "components/DuplicateOne.vue" &&
            entry.status === "muted" &&
            entry.reason === "sfc-global-component-duplicate-registration" &&
            entry.ambiguityKey === "DuplicateGlobal",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "DuplicateGlobal" &&
            entry.bindingName === "DuplicateTwo" &&
            entry.resolvedFile === "components/DuplicateTwo.vue" &&
            entry.status === "muted" &&
            entry.reason === "sfc-global-component-duplicate-registration" &&
            entry.ambiguityKey === "DuplicateGlobal",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.bindingName === "GlobalCard" &&
            entry.status === "muted" &&
            entry.reason === "sfc-global-component-name-dynamic",
        ) &&
        symbols.sfcGlobalComponentRegistrations?.some(
          (entry) =>
            entry.componentName === "MissingGlobal" &&
            entry.status === "unresolved" &&
            entry.reason === "sfc-global-component-unresolved",
        ),
      JSON.stringify(symbols.sfcGlobalComponentRegistrations),
    );
    assert(
      "SFC-2r. SFC global component registrations do not create graph edges or fan-in",
      !symbols.resolvedInternalEdges?.some(
        (edge) =>
          edge.kind === "sfc-global-component-registration" ||
          edge.source === "sfc-global-component-registration",
      ) && dead.has("src/registered-source.ts::RegisteredSource") === false,
      JSON.stringify({
        registrations: symbols.sfcGlobalComponentRegistrations,
        edges: symbols.resolvedInternalEdges,
        dead: [...dead].sort(),
      }),
    );
    assert(
      "SFC-2s. SFC generated component manifests are review-only artifact evidence",
      symbols.meta?.supports?.sfcGeneratedComponentManifests === true &&
        symbols.uses?.sfcGeneratedComponentManifests === 6 &&
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.manifestFile === "components.d.ts" &&
            entry.manifestKind === "unplugin-vue-components-dts" &&
            entry.componentName === "ManifestButton" &&
            entry.normalizedTagNames?.includes("manifest-button") &&
            entry.bindingSource === "./components/ManifestButton.vue" &&
            entry.resolvedFile === "components/ManifestButton.vue" &&
            entry.status === "muted" &&
            entry.reason ===
              "sfc-framework-generated-manifest-non-source-binding" &&
            entry.source === "sfc-framework-generated-manifest" &&
            entry.confidence === "generated-manifest-availability" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.manifestFile === ".nuxt/components.d.ts" &&
            entry.manifestKind === "nuxt-components-dts" &&
            entry.componentName === "NuxtManifest" &&
            entry.resolvedFile === "components/NuxtManifest.vue" &&
            entry.status === "muted",
        ) &&
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.componentName === "ManifestSource" &&
            entry.resolvedFile === "src/ManifestSource.ts" &&
            entry.status === "resolved",
        ) &&
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.componentName === "MissingManifest" &&
            entry.status === "unresolved" &&
            entry.reason === "sfc-framework-generated-manifest-unresolved" &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.componentName === "DynamicManifest" &&
            entry.status === "skipped" &&
            entry.reason === "sfc-framework-generated-manifest-nonliteral" &&
            entry.bindingSource === "./components/DynamicManifest.vue" &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.componentName === "[computed]" &&
            entry.status === "skipped" &&
            entry.reason === "sfc-framework-generated-manifest-nonliteral" &&
            entry.bindingSource === "./components/ExprManifest.vue" &&
            entry.computedKeySource === "prefix + 'Manifest'" &&
            !entry.resolvedFile,
        ) &&
        !symbols.sfcGeneratedComponentManifests?.some(
          (entry) =>
            entry.componentName === "RouterLink" ||
            entry.componentName === "ConventionOnly",
        ),
      JSON.stringify(symbols.sfcGeneratedComponentManifests),
    );
    assert(
      "SFC-2t. SFC generated component manifests do not create graph edges or fan-in",
      !symbols.resolvedInternalEdges?.some(
        (edge) =>
          edge.kind === "sfc-generated-component-manifest" ||
          edge.source === "./components/ManifestButton.vue" ||
          edge.source === "./src/ManifestSource.ts" ||
          edge.source === "./components/DynamicManifest.vue" ||
          edge.source === "./components/ExprManifest.vue",
      ) &&
        dead.has("src/ManifestSource.ts::default") &&
        symbols.fanInByIdentity?.["src/ManifestSource.ts::default"] === 0,
      JSON.stringify({
        manifests: symbols.sfcGeneratedComponentManifests,
        edges: symbols.resolvedInternalEdges,
        dead: [...dead].sort(),
        fanIn: symbols.fanInByIdentity,
      }),
    );
    assert(
      "SFC-2u. Nuxt components filesystem convention is muted review-only evidence",
      symbols.meta?.supports?.sfcFrameworkConventionComponents === true &&
        symbols.uses?.sfcFrameworkConventionComponents ===
          symbols.sfcFrameworkConventionComponents?.length &&
        symbols.uses?.sfcFrameworkConventionComponents > 0 &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-directory" &&
            entry.componentName === "ConventionOnly" &&
            entry.normalizedTagNames?.includes("ConventionOnly") &&
            entry.normalizedTagNames?.includes("convention-only") &&
            entry.sourceFile === "components/ConventionOnly.vue" &&
            entry.resolvedFile === "components/ConventionOnly.vue" &&
            entry.source === "sfc-framework-nuxt-fs-convention" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-nuxt-fs-convention" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.componentName === "BaseButton" &&
            entry.normalizedTagNames?.includes("BaseButton") &&
            entry.normalizedTagNames?.includes("base-button") &&
            entry.sourceFile === "components/base/Button.vue" &&
            JSON.stringify(entry.componentPathSegments) ===
              JSON.stringify(["base", "Button"]),
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.componentName === "UserIndex" &&
            entry.normalizedTagNames?.includes("UserIndex") &&
            entry.normalizedTagNames?.includes("user-index") &&
            entry.sourceFile === "components/user/index.vue" &&
            JSON.stringify(entry.componentPathSegments) ===
              JSON.stringify(["user", "index"]),
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-app-components-directory" &&
            entry.componentName === "BaseAppButton" &&
            entry.normalizedTagNames?.includes("BaseAppButton") &&
            entry.normalizedTagNames?.includes("base-app-button") &&
            entry.sourceFile === "app/components/base/AppButton.vue" &&
            entry.resolvedFile === "app/components/base/AppButton.vue" &&
            entry.source === "sfc-framework-nuxt-app-dir-convention" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-nuxt-app-dir-convention" &&
            JSON.stringify(entry.componentPathSegments) ===
              JSON.stringify(["base", "AppButton"]) &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-alias-import" &&
            entry.consumerFile === "pages/NuxtAlias.vue" &&
            entry.componentName === "NuxtManifest" &&
            entry.normalizedTagNames?.includes("NuxtManifest") &&
            entry.normalizedTagNames?.includes("nuxt-manifest") &&
            entry.bindingName === "LocalNuxtManifest" &&
            entry.importedName === "NuxtManifest" &&
            entry.manifestFile === ".nuxt/components.d.ts" &&
            entry.manifestKind === "nuxt-components-dts" &&
            entry.bindingSource === "../components/NuxtManifest.vue" &&
            entry.fromSpec === "#components" &&
            entry.resolvedFile === "components/NuxtManifest.vue" &&
            entry.source === "sfc-framework-nuxt-components-alias" &&
            entry.confidence === "generated-manifest-availability" &&
            entry.status === "muted" &&
            entry.reason ===
              "sfc-framework-nuxt-components-alias-manifest" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-alias-import" &&
            entry.consumerFile === "pages/NuxtAlias.vue" &&
            entry.componentName === "UnknownAlias" &&
            entry.bindingName === "UnknownAlias" &&
            entry.importedName === "UnknownAlias" &&
            entry.fromSpec === "#components" &&
            entry.source === "sfc-framework-nuxt-components-alias" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "unresolved" &&
            entry.reason ===
              "sfc-framework-nuxt-components-alias-unresolved" &&
            !entry.resolvedFile &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason?.startsWith("sfc-framework-nuxt-components-alias") &&
            (entry.componentName === "TypeOnlyAlias" ||
              entry.componentName === "componentNames" ||
              entry.importedName === "componentNames" ||
              entry.bindingName === "componentNames"),
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "unplugin-vue-components" &&
            entry.conventionKind === "auto-import-plugin-config" &&
            entry.configFile === "vite.config.ts" &&
            entry.pluginName === "Components" &&
            entry.fromSpec === "unplugin-vue-components/vite" &&
            entry.source === "sfc-framework-auto-import-plugin-config" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-auto-import-plugin-config" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-dir-config" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.componentDir === "~/shared/components" &&
            entry.resolvedDir === "app/shared/components" &&
            entry.prefix === "Shared" &&
            entry.pathPrefix === false &&
            entry.global === true &&
            entry.source === "sfc-framework-nuxt-components-dir-config" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-nuxt-components-dir-config" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false &&
            !entry.componentName &&
            !entry.sourceFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-dir-config" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.componentDir === "./local-components" &&
            entry.resolvedDir === "local-components" &&
            entry.source === "sfc-framework-nuxt-components-dir-config" &&
            entry.reason === "sfc-framework-nuxt-components-dir-config" &&
            !entry.componentName &&
            !entry.sourceFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-custom-resolver-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.hookName === "components:dirs" &&
            entry.configShape === "hooks" &&
            entry.source ===
              "sfc-framework-nuxt-custom-resolver-unavailable" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "unavailable" &&
            entry.reason ===
              "sfc-framework-nuxt-custom-resolver-unavailable" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false &&
            !entry.componentName &&
            !entry.componentDir &&
            !entry.sourceFile &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-custom-resolver-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.hookName === "components:extend" &&
            entry.reason ===
              "sfc-framework-nuxt-custom-resolver-unavailable",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-layer-extends-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.configProperty === "extends" &&
            entry.configShape === "extends" &&
            entry.extendsSource === "../layer-a" &&
            entry.extendsSourceKind === "literal" &&
            entry.source === "sfc-framework-nuxt-layer-extends-unavailable" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "unavailable" &&
            entry.reason === "sfc-framework-nuxt-layer-extends-unavailable" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false &&
            !entry.componentName &&
            !entry.componentDir &&
            !entry.sourceFile &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-layer-extends-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.extendsSourceKind === "nonliteral" &&
            !entry.extendsSource &&
            entry.reason === "sfc-framework-nuxt-layer-extends-unavailable" &&
            !entry.componentName &&
            !entry.sourceFile &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-module-package-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.configProperty === "modules" &&
            entry.configShape === "modules" &&
            entry.moduleSource === "@nuxt/image" &&
            entry.moduleSourceKind === "literal" &&
            entry.source === "sfc-framework-nuxt-module-package-unavailable" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "unavailable" &&
            entry.reason === "sfc-framework-nuxt-module-package-unavailable" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false &&
            !entry.componentName &&
            !entry.componentDir &&
            !entry.sourceFile &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-module-package-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.moduleSource === "@nuxtjs/tailwindcss" &&
            entry.moduleSourceKind === "literal" &&
            entry.reason === "sfc-framework-nuxt-module-package-unavailable" &&
            !entry.componentName &&
            !entry.sourceFile &&
            !entry.resolvedFile,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-module-package-unavailable" &&
            entry.configFile === "nuxt.config.ts" &&
            entry.moduleSourceKind === "nonliteral" &&
            !entry.moduleSource &&
            entry.reason === "sfc-framework-nuxt-module-package-unavailable" &&
            !entry.componentName &&
            !entry.sourceFile &&
            !entry.resolvedFile,
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) => entry.hookName === "app:mounted",
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.sourceFile === "app/shared/components/ConfiguredOnly.vue" ||
            entry.sourceFile === "shared/components/RootDecoy.vue" ||
            entry.sourceFile === "local-components/LocalConfigured.vue",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "unplugin-vue-components" &&
            entry.conventionKind === "auto-import-plugin-config" &&
            entry.configFile === "webpack.config.cjs" &&
            entry.pluginName === "Components" &&
            entry.fromSpec === "unplugin-vue-components/webpack" &&
            entry.reason === "sfc-framework-auto-import-plugin-config",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "unplugin-vue-components" &&
            entry.conventionKind === "auto-import-plugin-config" &&
            entry.configFile === "webpack.config.cjs" &&
            entry.pluginName === "require" &&
            entry.fromSpec === "unplugin-vue-components/webpack" &&
            entry.reason === "sfc-framework-auto-import-plugin-config",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "unplugin-vue-components" &&
            entry.conventionKind === "auto-import-plugin-config" &&
            entry.configFile === "webpack.config.js" &&
            entry.pluginName === "require" &&
            entry.fromSpec === "unplugin-vue-components/vite" &&
            entry.reason === "sfc-framework-auto-import-plugin-config",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "astro" &&
            entry.conventionKind === "client-directive" &&
            entry.consumerFile === "pages/Home.astro" &&
            entry.tagName === "UsedByAstro" &&
            entry.directiveName === "client:load" &&
            entry.bindingName === "UsedByAstro" &&
            entry.bindingSource === "../src/astro-use" &&
            entry.source === "sfc-framework-astro-client-directive" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-astro-client-directive" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "svelte" &&
            entry.conventionKind === "action-directive" &&
            entry.consumerFile === "components/Page.svelte" &&
            entry.tagName === "form" &&
            entry.directiveName === "use:enhance" &&
            entry.actionName === "enhance" &&
            entry.bindingName === "enhance" &&
            entry.bindingSource === "../src/svelte-action" &&
            entry.source === "sfc-framework-svelte-action-directive" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-svelte-action-directive" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "svelte" &&
            entry.conventionKind === "action-directive" &&
            entry.consumerFile === "components/Page.svelte" &&
            entry.tagName === "div" &&
            entry.directiveName === "use:localAction" &&
            entry.actionName === "localAction" &&
            entry.bindingName === "localAction" &&
            entry.bindingSource === "components/Page.svelte" &&
            entry.bindingKind === "local-function" &&
            entry.source === "sfc-framework-svelte-action-directive" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-svelte-action-directive" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "svelte" &&
            entry.conventionKind === "action-directive" &&
            entry.consumerFile === "components/Page.svelte" &&
            entry.tagName === "section" &&
            entry.directiveName === "use:localConstAction" &&
            entry.actionName === "localConstAction" &&
            entry.bindingName === "localConstAction" &&
            entry.bindingSource === "components/Page.svelte" &&
            entry.bindingKind === "local-const-function" &&
            entry.source === "sfc-framework-svelte-action-directive" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-svelte-action-directive" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "svelte" &&
            entry.conventionKind === "store-auto-subscription" &&
            entry.consumerFile === "components/Page.svelte" &&
            entry.subscriptionName === "$importedCount" &&
            entry.storeName === "importedCount" &&
            entry.bindingName === "importedCount" &&
            entry.bindingSource === "../src/svelte-store" &&
            entry.bindingKind === "named" &&
            entry.source === "sfc-framework-svelte-store-subscription" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-svelte-store-subscription" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "svelte" &&
            entry.conventionKind === "store-auto-subscription" &&
            entry.consumerFile === "components/Page.svelte" &&
            entry.subscriptionName === "$localCount" &&
            entry.storeName === "localCount" &&
            entry.bindingName === "localCount" &&
            entry.bindingSource === "components/Page.svelte" &&
            entry.bindingKind === "local-store-factory" &&
            entry.source === "sfc-framework-svelte-store-subscription" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-svelte-store-subscription" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "vue" &&
            entry.conventionKind === "macro-registration" &&
            entry.consumerFile === "pages/Macro.vue" &&
            entry.macroName === "defineOptions" &&
            entry.componentName === "MacroCard" &&
            entry.normalizedTagNames?.includes("MacroCard") &&
            entry.normalizedTagNames?.includes("macro-card") &&
            entry.bindingName === "MacroCard" &&
            entry.bindingSource === "../components/MacroCard.vue" &&
            entry.source === "sfc-framework-vue-macro-registration" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-vue-macro-registration" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "vue" &&
            entry.conventionKind === "macro-registration" &&
            entry.consumerFile === "pages/Macro.vue" &&
            entry.componentName === "macro-alias" &&
            entry.normalizedTagNames?.includes("MacroAlias") &&
            entry.bindingName === "MacroAlias" &&
            entry.bindingSource === "../src/macro-alias",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "vue" &&
            entry.conventionKind === "options-registration" &&
            entry.consumerFile === "pages/Options.vue" &&
            entry.optionName === "components" &&
            entry.componentName === "OptionsCard" &&
            entry.normalizedTagNames?.includes("OptionsCard") &&
            entry.normalizedTagNames?.includes("options-card") &&
            entry.bindingName === "OptionsCard" &&
            entry.bindingSource === "../components/OptionsCard.vue" &&
            entry.source === "sfc-framework-vue-options-registration" &&
            entry.confidence === "framework-convention-observed" &&
            entry.status === "muted" &&
            entry.reason === "sfc-framework-vue-options-registration" &&
            entry.eligibleForFanIn === false &&
            entry.eligibleForSafeFix === false,
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "vue" &&
            entry.conventionKind === "options-registration" &&
            entry.consumerFile === "pages/Options.vue" &&
            entry.componentName === "options-alias" &&
            entry.normalizedTagNames?.includes("OptionsAlias") &&
            entry.bindingName === "OptionsAlias" &&
            entry.bindingSource === "../src/options-alias",
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-astro-client-directive" &&
            (entry.tagName === "MissingAstroClient" || entry.tagName === "div"),
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-svelte-action-directive" &&
            (entry.actionName === "missingAction" ||
              entry.actionName === "commentAction" ||
              entry.actionName === "notActionValue"),
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-svelte-store-subscription" &&
            (entry.storeName === "missingStore" ||
              entry.storeName === "commentStore" ||
              entry.storeName === "plainTextStoreMention" ||
              entry.storeName === "notStoreValue"),
        ) &&
        symbols.sfcFrameworkConventionComponents?.filter(
          (entry) =>
            entry.reason === "sfc-framework-svelte-store-subscription" &&
            entry.subscriptionName === "$localCount",
        ).length === 1 &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-vue-macro-registration" &&
            (entry.componentName === "MissingMacro" ||
              entry.componentName === "CommentOnlyMacro" ||
              entry.componentName === "dynamicMacroName"),
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.reason === "sfc-framework-vue-options-registration" &&
            (entry.componentName === "MissingOptions" ||
              entry.componentName === "CommentOnlyOptions" ||
              entry.componentName === "dynamicOptionsName"),
        ) &&
        !symbols.resolvedInternalEdges?.some(
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
        ) &&
        !symbols.unresolvedInternalSpecifierRecords?.some(
          (record) => record.specifier === "#components",
        ) &&
        !symbols.dependencyImportConsumers?.some(
          (entry) => entry.fromSpec === "#components",
        ),
      JSON.stringify({
        conventions: symbols.sfcFrameworkConventionComponents,
        edges: symbols.resolvedInternalEdges,
        unresolved: symbols.unresolvedInternalSpecifierRecords,
        dependencyImportConsumers: symbols.dependencyImportConsumers,
      }),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "sfc-convention-gate-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "plain-vue-fixture", type: "module" }),
    );
    write(
      fx,
      "components/Loose.vue",
      [
        "<template><Loose /></template>",
        '<script setup lang="ts">',
        "const loose = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/base/Button.vue",
      [
        "<template><button>Base button</button></template>",
        '<script setup lang="ts">',
        "const baseButton = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "components/user/index.vue",
      [
        "<template><section>User index</section></template>",
        '<script setup lang="ts">',
        "const userIndex = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "app/components/base/AppButton.vue",
      [
        "<template><button>App button</button></template>",
        '<script setup lang="ts">',
        "const appButton = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "pages/NoNuxtAlias.vue",
      [
        "<template><section>No Nuxt signal</section></template>",
        '<script setup lang="ts">',
        "import { Loose } from '#components';",
        "</script>",
        "",
      ].join("\n"),
    );
    const outDir = path.join(fx, "out");
    const run = spawnSync(
      process.execPath,
      [
        path.join(REPO_ROOT, "build-symbol-graph.mjs"),
        "--root",
        fx,
        "--output",
        outDir,
      ],
      { cwd: REPO_ROOT, encoding: "utf8" },
    );
    const symbols = JSON.parse(
      readFileSync(path.join(outDir, "symbols.json"), "utf8"),
    );
    assert(
      "SFC-2v. Nuxt filesystem convention evidence requires a Nuxt signal",
      run.status === 0 &&
        symbols.meta?.supports?.sfcFrameworkConventionComponents === true &&
        symbols.uses?.sfcFrameworkConventionComponents === 0 &&
        symbols.sfcFrameworkConventionComponents?.length === 0 &&
        !symbols.unresolvedInternalSpecifierRecords?.some(
          (record) => record.specifier === "#components",
        ) &&
        !symbols.dependencyImportConsumers?.some(
          (entry) => entry.fromSpec === "#components",
        ),
      JSON.stringify({
        run: run.status,
        stderr: run.stderr,
        conventions: symbols.sfcFrameworkConventionComponents,
        unresolved: symbols.unresolvedInternalSpecifierRecords,
        dependencyImportConsumers: symbols.dependencyImportConsumers,
      }),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "sfc-convention-nuxt3-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "nuxt3-app-dir-gate-fixture",
        type: "module",
        dependencies: { nuxt: "^3.12.0" },
      }),
    );
    write(
      fx,
      "components/LegacyRoot.vue",
      [
        "<template><LegacyRoot /></template>",
        '<script setup lang="ts">',
        "const legacyRoot = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "app/components/base/AppButton.vue",
      [
        "<template><button>App button</button></template>",
        '<script setup lang="ts">',
        "const appButton = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    const outDir = path.join(fx, "out");
    const run = spawnSync(
      process.execPath,
      [
        path.join(REPO_ROOT, "build-symbol-graph.mjs"),
        "--root",
        fx,
        "--output",
        outDir,
      ],
      { cwd: REPO_ROOT, encoding: "utf8" },
    );
    const symbols = JSON.parse(
      readFileSync(path.join(outDir, "symbols.json"), "utf8"),
    );
    assert(
      "SFC-2w. Nuxt 3 dependency alone does not enable app-dir convention evidence",
      run.status === 0 &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-directory" &&
            entry.componentName === "LegacyRoot",
        ) &&
        !symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.conventionKind === "nuxt-app-components-directory" ||
            entry.source === "sfc-framework-nuxt-app-dir-convention" ||
            entry.sourceFile === "app/components/base/AppButton.vue",
        ),
      JSON.stringify({
        run: run.status,
        stderr: run.stderr,
        conventions: symbols.sfcFrameworkConventionComponents,
      }),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "sfc-convention-nuxt4-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({
        name: "nuxt4-app-dir-fixture",
        type: "module",
        dependencies: { nuxt: "^4.0.0" },
      }),
    );
    write(
      fx,
      "nuxt.config.ts",
      [
        "export default defineNuxtConfig({",
        "  components: { dirs: ['~/shared/components'] },",
        "});",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "app/components/base/AppButton.vue",
      [
        "<template><button>App button</button></template>",
        '<script setup lang="ts">',
        "const appButton = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "app/shared/components/DefaultSrcDirConfigured.vue",
      [
        "<template><section>Default srcDir configured</section></template>",
        '<script setup lang="ts">',
        "const defaultSrcDirConfigured = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    write(
      fx,
      "shared/components/RootDecoy.vue",
      [
        "<template><section>Root decoy</section></template>",
        '<script setup lang="ts">',
        "const rootDecoy = true;",
        "</script>",
        "",
      ].join("\n"),
    );
    const outDir = path.join(fx, "out");
    const run = spawnSync(
      process.execPath,
      [
        path.join(REPO_ROOT, "build-symbol-graph.mjs"),
        "--root",
        fx,
        "--output",
        outDir,
      ],
      { cwd: REPO_ROOT, encoding: "utf8" },
    );
    const symbols = JSON.parse(
      readFileSync(path.join(outDir, "symbols.json"), "utf8"),
    );
    assert(
      "SFC-2x. Nuxt 4 dependency enables app-dir convention evidence",
      run.status === 0 &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-app-components-directory" &&
            entry.componentName === "BaseAppButton" &&
            entry.sourceFile === "app/components/base/AppButton.vue" &&
            entry.reason === "sfc-framework-nuxt-app-dir-convention",
        ) &&
        symbols.sfcFrameworkConventionComponents?.some(
          (entry) =>
            entry.framework === "nuxt" &&
            entry.conventionKind === "nuxt-components-dir-config" &&
            entry.componentDir === "~/shared/components" &&
            entry.resolvedDir === "app/shared/components" &&
            entry.reason === "sfc-framework-nuxt-components-dir-config",
        ),
      JSON.stringify({
        run: run.status,
        stderr: run.stderr,
        conventions: symbols.sfcFrameworkConventionComponents,
      }),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
