import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { collectFiles } from "./collect-files.mjs";
import { JS_FAMILY_LANGS, SFC_FAMILY_LANGS } from "./lang.mjs";
import { extractSfcFileFactsForSources } from "./sfc-file-facts.mjs";

const require = createRequire(import.meta.url);
let parseSync = null;

function loadParseSync() {
  if (parseSync) return parseSync;
  const parser = require("oxc-parser");
  if (typeof parser?.parseSync !== "function") {
    throw new Error("oxc-parser parseSync export unavailable");
  }
  parseSync = parser.parseSync;
  return parseSync;
}

function lineOf(src, offset) {
  let line = 1;
  for (let i = 0; i < offset; i++) {
    if (src.charCodeAt(i) === 10) line++;
  }
  return line;
}

function isRelativeSourceSpec(spec) {
  return (
    typeof spec === "string" &&
    (spec.startsWith("./") || spec.startsWith("../"))
  );
}

function parserLangFromFile(filePath) {
  const ext = path.extname(filePath).slice(1).toLowerCase();
  if (ext === "tsx") return "tsx";
  if (ext === "jsx") return "jsx";
  if (ext === "js" || ext === "mjs" || ext === "cjs") return "js";
  return "ts";
}

function sfcLanguageForFile(filePath) {
  return path.extname(filePath).replace(/^\./, "").toLowerCase();
}

function filesForLanguages({
  root,
  includeTests = true,
  exclude = [],
  languages,
  files = null,
}) {
  if (Array.isArray(files)) {
    const allowed = new Set(languages);
    return files.filter((filePath) => allowed.has(sfcLanguageForFile(filePath)));
  }
  return collectFiles(root, {
    includeTests,
    exclude,
    languages,
  });
}

function jsFamilyFiles({
  root,
  includeTests = true,
  exclude = [],
  files = null,
}) {
  if (Array.isArray(files)) {
    const allowed = new Set(JS_FAMILY_LANGS);
    return files.filter((filePath) => allowed.has(sfcLanguageForFile(filePath)));
  }
  return collectFiles(root, {
    includeTests,
    exclude,
    languages: JS_FAMILY_LANGS,
  });
}

const GLOBAL_COMPONENT_REGISTRATION_SOURCE_RE =
  /(?:\.|\?\.)\s*component\s*\(|\[\s*["']component["']\s*\]\s*\(/;

function mayContainGlobalComponentRegistration(src) {
  return GLOBAL_COMPONENT_REGISTRATION_SOURCE_RE.test(`${src ?? ""}`);
}

function parseScriptAst(script, filePath, parserLang) {
  const parse = loadParseSync();
  const candidates = [parserLang || "ts"];
  if (parserLang === "ts") candidates.push("tsx");
  if (parserLang === "js") candidates.push("jsx");
  if (!candidates.includes("ts")) candidates.push("ts");

  for (const lang of candidates) {
    if (!["ts", "tsx", "js", "jsx"].includes(lang)) continue;
    try {
      const result = parse(filePath, script, {
        sourceType: "module",
        lang,
      });
      if (!Array.isArray(result.errors) || result.errors.length === 0) {
        return result.program;
      }
    } catch {
      // Try the next compatible dialect before giving up on the script block.
    }
  }

  return null;
}

function importedName(specifier) {
  return specifier?.imported?.name ?? specifier?.imported?.value ?? null;
}

function astPropertyName(node) {
  const key = node?.key ?? node;
  if (!key) return null;
  if (typeof key.name === "string") return key.name;
  if (typeof key.value === "string") return key.value;
  return null;
}

function computedPropertySource(node, src) {
  const key = node?.key;
  if (!node?.computed || !key) return null;
  if (!Number.isFinite(key.start) || !Number.isFinite(key.end)) return null;
  return src.slice(key.start, key.end).trim() || null;
}

function importLocalName(specifier) {
  return specifier?.local?.name ?? null;
}

function literalStringValue(node) {
  return node?.type === "Literal" && typeof node.value === "string"
    ? node.value
    : null;
}

function identifierName(node) {
  return node?.type === "Identifier" && typeof node.name === "string"
    ? node.name
    : null;
}

function memberPropertyName(node) {
  if (node?.type !== "MemberExpression") return null;
  if (node.computed) return literalStringValue(node.property);
  return identifierName(node.property);
}

function traverseAst(node, visit) {
  if (!node || typeof node !== "object") return;
  visit(node);
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) traverseAst(item, visit);
    } else if (
      value &&
      typeof value === "object" &&
      typeof value.type === "string"
    ) {
      traverseAst(value, visit);
    }
  }
}

const VUE_APP_FACTORY_NAMES = new Set(["createApp", "createSSRApp"]);
const VUE_APP_RETURNING_METHODS = new Set([
  "component",
  "directive",
  "mixin",
  "provide",
  "use",
]);

function isVueAppFactoryCall(node) {
  if (node?.type !== "CallExpression") return false;
  const callee = node.callee;
  const directName = identifierName(callee);
  if (directName && VUE_APP_FACTORY_NAMES.has(directName)) return true;
  const memberName = memberPropertyName(callee);
  return !!memberName && VUE_APP_FACTORY_NAMES.has(memberName);
}

function isVueAppReturningExpression(node) {
  if (isVueAppFactoryCall(node)) return true;
  if (node?.type !== "CallExpression") return false;
  const callee = node.callee;
  if (callee?.type !== "MemberExpression") return false;
  const methodName = memberPropertyName(callee);
  if (!methodName || !VUE_APP_RETURNING_METHODS.has(methodName)) return false;
  return isVueAppReturningExpression(callee.object);
}

function functionLikeFirstParamName(node) {
  const params = node?.params;
  if (!Array.isArray(params) || params.length === 0) return null;
  return identifierName(params[0]);
}

function collectVueComponentReceivers(program) {
  const out = new Set();
  traverseAst(program, (node) => {
    if (
      node?.type === "VariableDeclarator" &&
      isVueAppReturningExpression(node.init)
    ) {
      const name = identifierName(node.id);
      if (name) out.add(name);
      return;
    }

    if (
      node?.type === "FunctionDeclaration" &&
      identifierName(node.id) === "install"
    ) {
      const name = functionLikeFirstParamName(node);
      if (name) out.add(name);
      return;
    }

    if (node?.type === "Property" && astPropertyName(node) === "install") {
      const value = node.value;
      if (
        value?.type === "FunctionExpression" ||
        value?.type === "ArrowFunctionExpression"
      ) {
        const name = functionLikeFirstParamName(value);
        if (name) out.add(name);
      }
    }
  });
  return out;
}

function collectImportBindings(program, src) {
  const out = new Map();
  for (const node of program.body ?? []) {
    if (node?.type !== "ImportDeclaration") continue;
    const fromSpec = node.source?.value;
    if (typeof fromSpec !== "string" || fromSpec.length === 0) continue;
    if (node.importKind === "type") continue;
    for (const specifier of node.specifiers ?? []) {
      if (specifier.importKind === "type") continue;
      const bindingName = importLocalName(specifier);
      if (!bindingName) continue;
      if (
        specifier.type !== "ImportDefaultSpecifier" &&
        specifier.type !== "ImportSpecifier"
      ) {
        continue;
      }
      out.set(bindingName, {
        bindingName,
        bindingSource: fromSpec,
        bindingKind:
          specifier.type === "ImportDefaultSpecifier" ? "default" : "named",
        importedName:
          specifier.type === "ImportDefaultSpecifier"
            ? "default"
            : importedName(specifier),
        line: lineOf(src, node.start),
      });
    }
  }
  return out;
}

function kebabFromPascal(value) {
  if (!/^[A-Z][A-Za-z0-9]*$/.test(value)) return null;
  return value
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1-$2")
    .toLowerCase();
}

function normalizedGlobalComponentNames(componentName) {
  const names = [];
  if (componentName) names.push(componentName);
  const pascal = pascalFromKebab(componentName);
  if (pascal) names.push(pascal);
  const kebab = kebabFromPascal(componentName);
  if (kebab) names.push(kebab);
  return [...new Set(names)];
}

function globalRegistrationRecord({
  filePath,
  api,
  componentName = null,
  binding = null,
  fromSpec = null,
  factoryKind = null,
  ambiguityKey = null,
  line,
  status = "registration-syntax",
  reason = null,
}) {
  const explicitFromSpec = binding?.bindingSource ?? fromSpec;
  return {
    registrationFile: filePath,
    framework: "vue",
    api,
    ...(componentName
      ? {
          componentName,
          normalizedTagNames: normalizedGlobalComponentNames(componentName),
        }
      : {}),
    ...(binding
      ? {
          bindingName: binding.bindingName,
          bindingSource: binding.bindingSource,
          fromSpec: explicitFromSpec,
          bindingKind: binding.bindingKind,
          ...(binding.importedName
            ? { importedName: binding.importedName }
            : {}),
        }
      : explicitFromSpec
        ? { fromSpec: explicitFromSpec }
        : {}),
    source: "sfc-global-component-registration",
    status,
    confidence: status === "muted" ? "muted-review" : "registration-review",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    ...(reason ? { reason } : {}),
    ...(factoryKind ? { factoryKind } : {}),
    ...(ambiguityKey ? { ambiguityKey } : {}),
    line,
  };
}

function importExpressionLiteralSource(node) {
  if (node?.type !== "ImportExpression") return null;
  return literalStringValue(node.source);
}

function asyncLoaderImportSource(node) {
  if (
    node?.type !== "ArrowFunctionExpression" &&
    node?.type !== "FunctionExpression"
  ) {
    return null;
  }
  const direct = importExpressionLiteralSource(node.body);
  if (direct) return direct;
  if (node.body?.type !== "BlockStatement") return null;
  for (const statement of node.body.body ?? []) {
    if (statement?.type !== "ReturnStatement") continue;
    const returned = importExpressionLiteralSource(statement.argument);
    if (returned) return returned;
  }
  return null;
}

function defineAsyncComponentFactory(node) {
  if (node?.type !== "CallExpression") return null;
  if (identifierName(node.callee) !== "defineAsyncComponent") return null;
  return {
    factoryKind: "defineAsyncComponent",
    fromSpec: asyncLoaderImportSource(node.arguments?.[0]),
  };
}

function markDuplicateGlobalRegistrations(records) {
  const byName = new Map();
  for (const record of records) {
    if (!record.componentName) continue;
    if (!record.bindingName && !record.fromSpec) continue;
    const key = `${record.api}:${record.componentName}`;
    const group = byName.get(key) ?? [];
    group.push(record);
    byName.set(key, group);
  }
  const duplicateRecords = new Set(
    [...byName.entries()]
      .filter(([, group]) => group.length > 1)
      .flatMap(([, group]) => group.map((record) => record)),
  );
  if (duplicateRecords.size === 0) return records;
  return records.map((record) => {
    if (!duplicateRecords.has(record)) return record;
    if (!record.bindingName && !record.fromSpec) return record;
    return {
      ...record,
      status: "muted",
      confidence: "muted-review",
      reason: "sfc-global-component-duplicate-registration",
      ambiguityKey: record.componentName,
    };
  });
}

const GENERATED_COMPONENT_MANIFESTS = Object.freeze([
  {
    relPath: "components.d.ts",
    manifestKind: "unplugin-vue-components-dts",
  },
  {
    relPath: ".nuxt/components.d.ts",
    manifestKind: "nuxt-components-dts",
  },
]);

const NUXT_COMPONENTS_ALIAS_SPEC = "#components";
const NUXT_COMPONENTS_ALIAS_SOURCE =
  "sfc-framework-nuxt-components-alias";
const NUXT_COMPONENTS_ALIAS_MANIFEST_REASON =
  "sfc-framework-nuxt-components-alias-manifest";
const NUXT_COMPONENTS_ALIAS_UNRESOLVED_REASON =
  "sfc-framework-nuxt-components-alias-unresolved";
const NUXT_COMPONENTS_ALIAS_HELPER_EXPORTS = new Set(["componentNames"]);
const NUXT_COMPONENTS_DIR_CONFIG_REASON =
  "sfc-framework-nuxt-components-dir-config";
const NUXT_CUSTOM_RESOLVER_UNAVAILABLE_REASON =
  "sfc-framework-nuxt-custom-resolver-unavailable";
const NUXT_LAYER_EXTENDS_UNAVAILABLE_REASON =
  "sfc-framework-nuxt-layer-extends-unavailable";
const NUXT_MODULE_PACKAGE_UNAVAILABLE_REASON =
  "sfc-framework-nuxt-module-package-unavailable";
const NUXT_CUSTOM_RESOLVER_HOOKS = new Set([
  "components:dirs",
  "components:extend",
]);

const NUXT_ROOT_CONFIGS = Object.freeze([
  "nuxt.config.ts",
  "nuxt.config.mts",
  "nuxt.config.cts",
  "nuxt.config.js",
  "nuxt.config.mjs",
  "nuxt.config.cjs",
]);

const NUXT_COMPONENT_CONVENTION_ROOTS = Object.freeze([
  {
    relPath: "components",
    conventionKind: "nuxt-components-directory",
    reason: "sfc-framework-nuxt-fs-convention",
  },
  {
    relPath: "app/components",
    conventionKind: "nuxt-app-components-directory",
    reason: "sfc-framework-nuxt-app-dir-convention",
    requiresAppSrcDirSignal: true,
  },
]);

const UNPLUGIN_COMPONENT_CONFIGS = Object.freeze([
  "vite.config.ts",
  "vite.config.mts",
  "vite.config.cts",
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.cjs",
  "webpack.config.ts",
  "webpack.config.mts",
  "webpack.config.cts",
  "webpack.config.js",
  "webpack.config.mjs",
  "webpack.config.cjs",
]);

function packageJsonNuxtDependencyRange(root) {
  const filePath = path.join(root, "package.json");
  if (!existsSync(filePath)) return null;
  try {
    const pkg = JSON.parse(readFileSync(filePath, "utf8"));
    for (const field of [
      "dependencies",
      "devDependencies",
      "peerDependencies",
      "optionalDependencies",
    ]) {
      if (pkg?.[field]?.nuxt) return pkg[field].nuxt;
    }
  } catch {
    return null;
  }
  return null;
}

function packageJsonHasNuxtDependency(root) {
  return typeof packageJsonNuxtDependencyRange(root) === "string";
}

function isNuxtFourDependencyRange(value) {
  if (typeof value !== "string") return false;
  const spec = value.trim().replace(/^npm:nuxt@/, "");
  const match = spec.match(/(?:^|[^\d])([0-9]+)(?:\b|[.\-xX*])/);
  return match ? Number(match[1]) >= 4 : false;
}

function packageJsonHasNuxtFourDependency(root) {
  return isNuxtFourDependencyRange(packageJsonNuxtDependencyRange(root));
}

function isNuxtAppSrcDirValue(value) {
  if (typeof value !== "string") return false;
  const normalized = value
    .replace(/\\/g, "/")
    .replace(/^\.\//, "")
    .replace(/\/+$/, "");
  return normalized === "app";
}

function nuxtConfigHasAppSrcDir(root) {
  for (const rel of NUXT_ROOT_CONFIGS) {
    const filePath = path.join(root, rel);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
    if (!program) continue;
    let found = false;
    traverseAst(program, (node) => {
      if (found) return;
      if (node?.type !== "Property") return;
      if (node.computed) return;
      if (astPropertyName(node) !== "srcDir") return;
      if (isNuxtAppSrcDirValue(literalStringValue(node.value))) found = true;
    });
    if (found) return true;
  }
  return false;
}

function hasNuxtConventionSignal(root) {
  if (existsSync(path.join(root, ".nuxt", "components.d.ts"))) return true;
  if (NUXT_ROOT_CONFIGS.some((rel) => existsSync(path.join(root, rel)))) {
    return true;
  }
  return packageJsonHasNuxtDependency(root);
}

function hasNuxtAppDirConventionSignal(root) {
  return packageJsonHasNuxtFourDependency(root) || nuxtConfigHasAppSrcDir(root);
}

function nuxtConfigObjectExpression(node) {
  if (node?.type === "ObjectExpression") return node;
  if (
    node?.type === "CallExpression" &&
    identifierName(node.callee) === "defineNuxtConfig" &&
    node.arguments?.[0]?.type === "ObjectExpression"
  ) {
    return node.arguments[0];
  }
  return null;
}

function nuxtConfigRootObjectExpressions(program) {
  const out = [];
  for (const node of program?.body ?? []) {
    if (node?.type === "ExportDefaultDeclaration") {
      const configObject = nuxtConfigObjectExpression(node.declaration);
      if (configObject) out.push(configObject);
      continue;
    }
    const expr = node?.type === "ExpressionStatement" ? node.expression : null;
    if (
      expr?.type === "AssignmentExpression" &&
      expr.operator === "=" &&
      expr.left?.type === "MemberExpression" &&
      identifierName(expr.left.object) === "module" &&
      memberPropertyName(expr.left) === "exports"
    ) {
      const configObject = nuxtConfigObjectExpression(expr.right);
      if (configObject) out.push(configObject);
    }
  }
  return out;
}

function nuxtConfigSrcDirValue(configObject) {
  for (const property of configObject?.properties ?? []) {
    if (property?.type !== "Property" || property.computed) continue;
    if (astPropertyName(property) !== "srcDir") continue;
    return literalStringValue(property.value);
  }
  return null;
}

function nuxtRootRelativePath(root, configuredPath) {
  if (typeof configuredPath !== "string" || configuredPath.length === 0) {
    return null;
  }
  const normalized = configuredPath.replace(/\\/g, "/");
  if (path.isAbsolute(normalized)) return normalized;
  return path.join(root, normalized);
}

function nuxtConfigSrcDirRoot(root, configObject) {
  const srcDir = nuxtConfigSrcDirValue(configObject);
  if (srcDir) return nuxtRootRelativePath(root, srcDir);
  if (packageJsonHasNuxtFourDependency(root)) return path.join(root, "app");
  return root;
}

function nuxtConfigPath(root, configFile, srcDirRoot, configuredPath) {
  if (typeof configuredPath !== "string" || configuredPath.length === 0) {
    return null;
  }
  const normalized = configuredPath.replace(/\\/g, "/");
  if (normalized.startsWith("~/") || normalized.startsWith("@/")) {
    return path.join(srcDirRoot || root, normalized.slice(2));
  }
  if (normalized.startsWith("./") || normalized.startsWith("../")) {
    return path.resolve(path.dirname(configFile), normalized);
  }
  if (path.isAbsolute(normalized)) return normalized;
  return path.join(root, normalized);
}

function nuxtComponentsDirConfigRecord({
  root,
  configFile,
  srcDirRoot,
  componentDir,
  prefix = null,
  pathPrefix = null,
  global = null,
  line,
}) {
  const resolved = nuxtConfigPath(root, configFile, srcDirRoot, componentDir);
  return {
    framework: "nuxt",
    conventionKind: "nuxt-components-dir-config",
    configFile,
    componentDir,
    ...(resolved && existsSync(resolved) ? { resolvedDir: resolved } : {}),
    ...(typeof prefix === "string" && prefix.length > 0 ? { prefix } : {}),
    ...(typeof pathPrefix === "boolean" || typeof pathPrefix === "string"
      ? { pathPrefix }
      : {}),
    ...(typeof global === "boolean" ? { global } : {}),
    source: NUXT_COMPONENTS_DIR_CONFIG_REASON,
    confidence: "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status: "muted",
    reason: NUXT_COMPONENTS_DIR_CONFIG_REASON,
    ...(Number.isFinite(line) ? { line } : {}),
  };
}

function nuxtCustomResolverUnavailableRecord({ configFile, hookName, line }) {
  return {
    framework: "nuxt",
    conventionKind: "nuxt-custom-resolver-unavailable",
    configFile,
    hookName,
    configShape: "hooks",
    source: NUXT_CUSTOM_RESOLVER_UNAVAILABLE_REASON,
    confidence: "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status: "unavailable",
    reason: NUXT_CUSTOM_RESOLVER_UNAVAILABLE_REASON,
    ...(Number.isFinite(line) ? { line } : {}),
  };
}

function nuxtLayerExtendsUnavailableRecord({
  configFile,
  extendsSource = null,
  extendsSourceKind,
  line,
}) {
  return {
    framework: "nuxt",
    conventionKind: "nuxt-layer-extends-unavailable",
    configFile,
    configProperty: "extends",
    configShape: "extends",
    ...(typeof extendsSource === "string" && extendsSource.length > 0
      ? { extendsSource }
      : {}),
    extendsSourceKind,
    source: NUXT_LAYER_EXTENDS_UNAVAILABLE_REASON,
    confidence: "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status: "unavailable",
    reason: NUXT_LAYER_EXTENDS_UNAVAILABLE_REASON,
    ...(Number.isFinite(line) ? { line } : {}),
  };
}

function nuxtModulePackageUnavailableRecord({
  configFile,
  moduleSource = null,
  moduleSourceKind,
  line,
}) {
  return {
    framework: "nuxt",
    conventionKind: "nuxt-module-package-unavailable",
    configFile,
    configProperty: "modules",
    configShape: "modules",
    ...(typeof moduleSource === "string" && moduleSource.length > 0
      ? { moduleSource }
      : {}),
    moduleSourceKind,
    source: NUXT_MODULE_PACKAGE_UNAVAILABLE_REASON,
    confidence: "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status: "unavailable",
    reason: NUXT_MODULE_PACKAGE_UNAVAILABLE_REASON,
    ...(Number.isFinite(line) ? { line } : {}),
  };
}

function booleanLiteralValue(node) {
  return node?.type === "Literal" && typeof node.value === "boolean"
    ? node.value
    : null;
}

function nuxtComponentsDirObjectRecord({ root, configFile, srcDirRoot, node, src }) {
  let componentDir = null;
  let prefix = null;
  let pathPrefix = null;
  let global = null;
  for (const property of node?.properties ?? []) {
    if (property?.type !== "Property" || property.computed) continue;
    const name = astPropertyName(property);
    if (name === "path") componentDir = literalStringValue(property.value);
    if (name === "prefix") prefix = literalStringValue(property.value);
    if (name === "pathPrefix") {
      pathPrefix =
        booleanLiteralValue(property.value) ?? literalStringValue(property.value);
    }
    if (name === "global") global = booleanLiteralValue(property.value);
  }
  if (!componentDir) return null;
  return nuxtComponentsDirConfigRecord({
    root,
    configFile,
    srcDirRoot,
    componentDir,
    prefix,
    pathPrefix,
    global,
    line: lineOf(src, node.start),
  });
}

function nuxtComponentsDirConfigRecordsFromValue({
  root,
  configFile,
  srcDirRoot,
  value,
  src,
}) {
  const out = [];
  const stringValue = literalStringValue(value);
  if (stringValue) {
    out.push(
      nuxtComponentsDirConfigRecord({
        root,
        configFile,
        srcDirRoot,
        componentDir: stringValue,
        line: lineOf(src, value.start),
      }),
    );
    return out;
  }
  if (value?.type === "ArrayExpression") {
    for (const element of value.elements ?? []) {
      if (!element) continue;
      out.push(
        ...nuxtComponentsDirConfigRecordsFromValue({
          root,
          configFile,
          srcDirRoot,
          value: element,
          src,
        }),
      );
    }
    return out;
  }
  if (value?.type === "ObjectExpression") {
    const direct = nuxtComponentsDirObjectRecord({
      root,
      configFile,
      srcDirRoot,
      node: value,
      src,
    });
    if (direct) out.push(direct);
    for (const property of value.properties ?? []) {
      if (property?.type !== "Property" || property.computed) continue;
      if (astPropertyName(property) !== "dirs") continue;
      out.push(
        ...nuxtComponentsDirConfigRecordsFromValue({
          root,
          configFile,
          srcDirRoot,
          value: property.value,
          src,
        }),
      );
    }
  }
  return out;
}

function collectSfcNuxtComponentsDirConfigConventions({ root }) {
  if (!hasNuxtConventionSignal(root)) return [];
  const out = [];
  for (const rel of NUXT_ROOT_CONFIGS) {
    const filePath = path.join(root, rel);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
    if (!program) continue;
    for (const configObject of nuxtConfigRootObjectExpressions(program)) {
      const srcDirRoot = nuxtConfigSrcDirRoot(root, configObject);
      for (const property of configObject.properties ?? []) {
        if (property?.type !== "Property" || property.computed) continue;
        if (astPropertyName(property) !== "components") continue;
        out.push(
          ...nuxtComponentsDirConfigRecordsFromValue({
            root,
            configFile: filePath,
            srcDirRoot,
            value: property.value,
            src,
          }),
        );
      }
    }
  }
  return out;
}

function isFunctionLikeExpression(node) {
  return (
    node?.type === "FunctionExpression" ||
    node?.type === "ArrowFunctionExpression"
  );
}

function collectSfcNuxtCustomResolverUnavailableConventions({ root }) {
  if (!hasNuxtConventionSignal(root)) return [];
  const out = [];
  for (const rel of NUXT_ROOT_CONFIGS) {
    const filePath = path.join(root, rel);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
    if (!program) continue;
    for (const configObject of nuxtConfigRootObjectExpressions(program)) {
      for (const property of configObject.properties ?? []) {
        if (property?.type !== "Property" || property.computed) continue;
        if (astPropertyName(property) !== "hooks") continue;
        if (property.value?.type !== "ObjectExpression") continue;
        for (const hookProperty of property.value.properties ?? []) {
          if (hookProperty?.type !== "Property" || hookProperty.computed) {
            continue;
          }
          const hookName = astPropertyName(hookProperty);
          if (!NUXT_CUSTOM_RESOLVER_HOOKS.has(hookName)) continue;
          if (!isFunctionLikeExpression(hookProperty.value)) continue;
          out.push(
            nuxtCustomResolverUnavailableRecord({
              configFile: filePath,
              hookName,
              line: lineOf(src, hookProperty.start ?? property.start),
            }),
          );
        }
      }
    }
  }
  return out;
}

function nuxtLayerExtendsRecordsFromValue({ configFile, value, src }) {
  const out = [];
  const stringValue = literalStringValue(value);
  if (stringValue) {
    out.push(
      nuxtLayerExtendsUnavailableRecord({
        configFile,
        extendsSource: stringValue,
        extendsSourceKind: "literal",
        line: lineOf(src, value.start),
      }),
    );
    return out;
  }
  if (value?.type === "ArrayExpression") {
    for (const element of value.elements ?? []) {
      if (!element) continue;
      out.push(
        ...nuxtLayerExtendsRecordsFromValue({
          configFile,
          value: element,
          src,
        }),
      );
    }
    return out;
  }
  out.push(
    nuxtLayerExtendsUnavailableRecord({
      configFile,
      extendsSourceKind: "nonliteral",
      line: lineOf(src, value?.start),
    }),
  );
  return out;
}

function collectSfcNuxtLayerExtendsUnavailableConventions({ root }) {
  if (!hasNuxtConventionSignal(root)) return [];
  const out = [];
  for (const rel of NUXT_ROOT_CONFIGS) {
    const filePath = path.join(root, rel);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
    if (!program) continue;
    for (const configObject of nuxtConfigRootObjectExpressions(program)) {
      for (const property of configObject.properties ?? []) {
        if (property?.type !== "Property" || property.computed) continue;
        if (astPropertyName(property) !== "extends") continue;
        out.push(
          ...nuxtLayerExtendsRecordsFromValue({
            configFile: filePath,
            value: property.value,
            src,
          }),
        );
      }
    }
  }
  return out;
}

function nuxtModulePackageRecordFromEntry({ configFile, value, src }) {
  const entry =
    value?.type === "ArrayExpression" ? (value.elements ?? [])[0] : value;
  const stringValue = literalStringValue(entry);
  if (stringValue) {
    return nuxtModulePackageUnavailableRecord({
      configFile,
      moduleSource: stringValue,
      moduleSourceKind: "literal",
      line: lineOf(src, entry.start),
    });
  }
  return nuxtModulePackageUnavailableRecord({
    configFile,
    moduleSourceKind: "nonliteral",
    line: lineOf(src, entry?.start ?? value?.start),
  });
}

function nuxtModulePackageRecordsFromValue({ configFile, value, src }) {
  if (value?.type !== "ArrayExpression") {
    return [nuxtModulePackageRecordFromEntry({ configFile, value, src })];
  }
  const out = [];
  for (const element of value.elements ?? []) {
    if (!element) continue;
    out.push(nuxtModulePackageRecordFromEntry({ configFile, value: element, src }));
  }
  return out;
}

function collectSfcNuxtModulePackageUnavailableConventions({ root }) {
  if (!hasNuxtConventionSignal(root)) return [];
  const out = [];
  for (const rel of NUXT_ROOT_CONFIGS) {
    const filePath = path.join(root, rel);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
    if (!program) continue;
    for (const configObject of nuxtConfigRootObjectExpressions(program)) {
      for (const property of configObject.properties ?? []) {
        if (property?.type !== "Property" || property.computed) continue;
        if (astPropertyName(property) !== "modules") continue;
        out.push(
          ...nuxtModulePackageRecordsFromValue({
            configFile: filePath,
            value: property.value,
            src,
          }),
        );
      }
    }
  }
  return out;
}

function stripNuxtComponentModeSuffix(value) {
  return `${value ?? ""}`.replace(/\.(client|server|global)$/i, "");
}

function pascalFromNuxtConventionSegment(value) {
  const cleaned = stripNuxtComponentModeSuffix(value);
  const parts = cleaned.split(/[-_\s.]+/).filter(Boolean);
  if (parts.length === 0) return null;
  return parts
    .map((part) => `${part[0] ?? ""}`.toUpperCase() + part.slice(1))
    .join("");
}

function pathInsideRoot(rootRel, conventionRootRelPath) {
  const rootParts = conventionRootRelPath.split("/").filter(Boolean);
  const fileParts = rootRel.split(path.sep).filter(Boolean);
  if (fileParts.length < rootParts.length) return false;
  return rootParts.every((part, index) => fileParts[index] === part);
}

function nuxtConventionRootDir(root, conventionRoot) {
  return path.join(root, ...conventionRoot.relPath.split("/").filter(Boolean));
}

function nuxtConventionRootForRelPath(rootRel) {
  return NUXT_COMPONENT_CONVENTION_ROOTS.find((conventionRoot) =>
    pathInsideRoot(rootRel, conventionRoot.relPath),
  );
}

function nuxtConventionPathSegments({ root, filePath, conventionRoot }) {
  return path
    .relative(nuxtConventionRootDir(root, conventionRoot), filePath)
    .split(path.sep)
    .filter(Boolean)
    .map((segment) =>
      stripNuxtComponentModeSuffix(segment.replace(/\.[^.]+$/, "")),
    );
}

function nuxtConventionComponentName({ root, filePath, conventionRoot }) {
  const parts = nuxtConventionPathSegments({ root, filePath, conventionRoot })
    .map((segment) => pascalFromNuxtConventionSegment(segment))
    .filter(Boolean);
  const deduped = [];
  for (const part of parts) {
    if (deduped.at(-1)?.toLowerCase() === part.toLowerCase()) continue;
    deduped.push(part);
  }
  return deduped.join("");
}

function nuxtConventionRecord({ root, filePath, conventionRoot }) {
  const componentName = nuxtConventionComponentName({
    root,
    filePath,
    conventionRoot,
  });
  return {
    framework: "nuxt",
    conventionKind: conventionRoot.conventionKind,
    componentName,
    normalizedTagNames: normalizedGlobalComponentNames(componentName),
    sourceFile: filePath,
    resolvedFile: filePath,
    source: conventionRoot.reason,
    confidence: "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status: "muted",
    reason: conventionRoot.reason,
    componentPathSegments: nuxtConventionPathSegments({
      root,
      filePath,
      conventionRoot,
    }),
  };
}

function generatedManifestTargetFile(manifest) {
  const fromSpec = manifest?.bindingSource ?? manifest?.fromSpec;
  if (!isRelativeSourceSpec(fromSpec)) return null;
  const target = path.resolve(path.dirname(manifest.manifestFile), fromSpec);
  return existsSync(target) ? target : null;
}

function nuxtGeneratedManifestLookup(root) {
  const byName = new Map();
  for (const record of collectSfcGeneratedComponentManifests({ root })) {
    if (record.manifestKind !== "nuxt-components-dts") continue;
    if (record.status === "skipped") continue;
    const names = [
      record.componentName,
      ...(Array.isArray(record.normalizedTagNames)
        ? record.normalizedTagNames
        : []),
    ].filter(Boolean);
    for (const name of names) byName.set(name, record);
  }
  return byName;
}

function nuxtComponentsAliasRecord({ use, manifest = null }) {
  const resolvedFile = manifest ? generatedManifestTargetFile(manifest) : null;
  const status = manifest && resolvedFile ? "muted" : "unresolved";
  const reason =
    status === "muted"
      ? NUXT_COMPONENTS_ALIAS_MANIFEST_REASON
      : NUXT_COMPONENTS_ALIAS_UNRESOLVED_REASON;
  const componentName = manifest?.componentName ?? use.name;
  return {
    framework: "nuxt",
    conventionKind: "nuxt-components-alias-import",
    consumerFile: use.consumerFile,
    componentName,
    normalizedTagNames:
      manifest?.normalizedTagNames ?? normalizedGlobalComponentNames(componentName),
    bindingName: use.localName ?? use.name,
    importedName: use.name,
    ...(manifest
      ? {
          manifestFile: manifest.manifestFile,
          manifestKind: manifest.manifestKind,
          bindingSource: manifest.bindingSource ?? manifest.fromSpec,
        }
      : {}),
    fromSpec: NUXT_COMPONENTS_ALIAS_SPEC,
    ...(resolvedFile ? { resolvedFile } : {}),
    source: NUXT_COMPONENTS_ALIAS_SOURCE,
    confidence: manifest
      ? "generated-manifest-availability"
      : "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status,
    reason,
    ...(use.sfcBlockKind ? { sfcBlockKind: use.sfcBlockKind } : {}),
    ...(Number.isFinite(use.line) ? { line: use.line } : {}),
  };
}

function collectSfcNuxtComponentsAliasConventions({
  root,
  scriptImports,
}) {
  if (!hasNuxtConventionSignal(root)) return [];
  const out = [];
  const manifestByName = nuxtGeneratedManifestLookup(root);
  for (const use of scriptImports) {
    if (use.fromSpec !== NUXT_COMPONENTS_ALIAS_SPEC) continue;
    if (use.kind !== "import" || use.typeOnly) continue;
    if (NUXT_COMPONENTS_ALIAS_HELPER_EXPORTS.has(use.name)) continue;
    out.push(
      nuxtComponentsAliasRecord({
        use,
        manifest: manifestByName.get(use.name) ?? null,
      }),
    );
  }

  return out;
}

function isUnpluginVueComponentsSpec(fromSpec) {
  return (
    typeof fromSpec === "string" &&
    (fromSpec === "unplugin-vue-components" ||
      fromSpec.startsWith("unplugin-vue-components/"))
  );
}

function unpluginRequireSource(node) {
  if (node?.type !== "CallExpression") return null;
  if (identifierName(node.callee) !== "require") return null;
  const fromSpec = literalStringValue(node.arguments?.[0]);
  return isUnpluginVueComponentsSpec(fromSpec) ? fromSpec : null;
}

function unpluginConfigRecord({ filePath, pluginName, fromSpec, line }) {
  return {
    framework: "unplugin-vue-components",
    conventionKind: "auto-import-plugin-config",
    configFile: filePath,
    pluginName,
    fromSpec,
    source: "sfc-framework-auto-import-plugin-config",
    confidence: "framework-convention-observed",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    status: "muted",
    reason: "sfc-framework-auto-import-plugin-config",
    line,
  };
}

function parseUnpluginVueComponentsConfig(src, filePath) {
  const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
  if (!program) return [];

  const bindings = new Map();
  for (const node of program.body ?? []) {
    if (node?.type !== "ImportDeclaration") continue;
    const fromSpec = node.source?.value;
    if (!isUnpluginVueComponentsSpec(fromSpec)) continue;
    if (node.importKind === "type") continue;
    for (const specifier of node.specifiers ?? []) {
      if (specifier.importKind === "type") continue;
      if (
        specifier.type !== "ImportDefaultSpecifier" &&
        specifier.type !== "ImportSpecifier"
      ) {
        continue;
      }
      const pluginName = importLocalName(specifier);
      if (pluginName) bindings.set(pluginName, fromSpec);
    }
  }
  for (const node of program.body ?? []) {
    if (node?.type !== "VariableDeclaration") continue;
    for (const declaration of node.declarations ?? []) {
      const pluginName = identifierName(declaration.id);
      if (!pluginName) continue;
      const fromSpec = unpluginRequireSource(declaration.init);
      if (fromSpec) bindings.set(pluginName, fromSpec);
    }
  }
  const out = [];
  const seen = new Set();
  traverseAst(program, (node) => {
    if (node?.type !== "CallExpression") return;
    const directRequireSpec = unpluginRequireSource(node.callee);
    const pluginName = directRequireSpec
      ? "require"
      : identifierName(node.callee);
    const fromSpec = directRequireSpec ?? bindings.get(pluginName);
    if (!pluginName || !fromSpec) return;
    const key = `${pluginName}|${node.start ?? ""}`;
    if (seen.has(key)) return;
    seen.add(key);
    out.push(
      unpluginConfigRecord({
        filePath,
        pluginName,
        fromSpec,
        line: lineOf(src, node.start ?? 0),
      }),
    );
  });

  return out;
}

function collectUnpluginVueComponentsConfigs(root) {
  const out = [];
  for (const relPath of UNPLUGIN_COMPONENT_CONFIGS) {
    const filePath = path.join(root, relPath);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    out.push(...parseUnpluginVueComponentsConfig(src, filePath));
  }
  return out;
}

function generatedManifestImportSource(node) {
  const annotation = node?.typeAnnotation?.typeAnnotation;
  if (annotation?.type !== "TSIndexedAccessType") return null;

  const indexLiteral = annotation.indexType?.literal;
  if (indexLiteral?.type !== "Literal" || indexLiteral.value !== "default") {
    return null;
  }

  const objectType = annotation.objectType;
  const importType =
    objectType?.type === "TSTypeQuery" ? objectType.exprName : null;
  const source = importType?.type === "TSImportType" ? importType.source : null;
  return typeof source?.value === "string" ? source.value : null;
}

function generatedManifestRawImportSource(node) {
  const annotation = node?.typeAnnotation?.typeAnnotation;
  if (annotation?.type !== "TSIndexedAccessType") return null;
  const objectType = annotation.objectType;
  const importType =
    objectType?.type === "TSTypeQuery" ? objectType.exprName : null;
  const source = importType?.type === "TSImportType" ? importType.source : null;
  return typeof source?.value === "string" ? source.value : null;
}

function generatedComponentManifestRecord({
  manifestFile,
  manifestKind,
  componentName,
  fromSpec,
  status = null,
  reason = null,
  normalizedTagNames = null,
  computedKeySource = null,
  line,
}) {
  return {
    manifestFile,
    manifestKind,
    componentName,
    normalizedTagNames:
      normalizedTagNames ?? normalizedGlobalComponentNames(componentName),
    ...(fromSpec ? { bindingSource: fromSpec, fromSpec } : {}),
    ...(computedKeySource ? { computedKeySource } : {}),
    source: "sfc-framework-generated-manifest",
    confidence: "generated-manifest-availability",
    eligibleForFanIn: false,
    eligibleForSafeFix: false,
    ...(status ? { status } : {}),
    ...(reason ? { reason } : {}),
    line,
  };
}

function parseGeneratedComponentManifestInterface(
  node,
  { manifestFile, manifestKind, fileSource },
) {
  const out = [];
  if (node?.type !== "TSInterfaceDeclaration") return out;
  if (identifierName(node.id) !== "GlobalComponents") return out;

  for (const property of node.body?.body ?? []) {
    if (property?.type !== "TSPropertySignature") continue;
    const propertyName = astPropertyName(property);
    const componentName =
      propertyName ?? (property.computed ? "[computed]" : null);
    if (!componentName) continue;
    const fromSpec = generatedManifestImportSource(property);
    const rawFromSpec = fromSpec ?? generatedManifestRawImportSource(property);
    if (rawFromSpec && !isRelativeSourceSpec(rawFromSpec)) continue;
    if (property.computed || !fromSpec) {
      out.push(
        generatedComponentManifestRecord({
          manifestFile,
          manifestKind,
          componentName,
          fromSpec: rawFromSpec,
          status: "skipped",
          reason: "sfc-framework-generated-manifest-nonliteral",
          normalizedTagNames: [],
          computedKeySource: computedPropertySource(property, fileSource),
          line: lineOf(fileSource, property.start ?? 0),
        }),
      );
      continue;
    }
    if (!isRelativeSourceSpec(fromSpec)) continue;
    out.push(
      generatedComponentManifestRecord({
        manifestFile,
        manifestKind,
        componentName,
        fromSpec,
        line: lineOf(fileSource, property.start ?? 0),
      }),
    );
  }

  return out;
}

function parseGeneratedComponentManifestProgram(
  program,
  { manifestFile, manifestKind, fileSource },
) {
  const out = [];
  for (const node of program.body ?? []) {
    if (node?.type !== "TSModuleDeclaration") continue;
    const moduleName =
      node.id?.type === "Literal" && typeof node.id.value === "string"
        ? node.id.value
        : identifierName(node.id);
    if (moduleName !== "vue") continue;
    traverseAst(node.body, (candidate) => {
      if (candidate?.type === "TSInterfaceDeclaration") {
        out.push(
          ...parseGeneratedComponentManifestInterface(candidate, {
            manifestFile,
            manifestKind,
            fileSource,
          }),
        );
      }
    });
  }
  return out;
}

function parseGlobalComponentRegistrations(program, { filePath, fileSource }) {
  const out = [];
  const imports = collectImportBindings(program, fileSource);
  const receivers = collectVueComponentReceivers(program);
  if (receivers.size === 0) return out;

  traverseAst(program, (node) => {
    if (node?.type !== "CallExpression") return;
    const callee = node.callee;
    if (callee?.type !== "MemberExpression") return;
    if (memberPropertyName(callee) !== "component") return;
    const receiverName = identifierName(callee.object);
    if (!receiverName || !receivers.has(receiverName)) return;

    const args = node.arguments ?? [];
    const componentName = literalStringValue(args[0]);
    const asyncFactory = defineAsyncComponentFactory(args[1]);
    const bindingName = identifierName(args[1]);
    const binding = bindingName ? imports.get(bindingName) : null;
    const line = lineOf(fileSource, node.start);
    const api = `${receiverName}.component`;

    if (!componentName) {
      if (!binding) return;
      out.push(
        globalRegistrationRecord({
          filePath,
          api,
          binding,
          line,
          status: "muted",
          reason: "sfc-global-component-name-dynamic",
        }),
      );
      return;
    }

    if (asyncFactory) {
      out.push(
        globalRegistrationRecord({
          filePath,
          api,
          componentName,
          fromSpec: asyncFactory.fromSpec,
          factoryKind: asyncFactory.factoryKind,
          line,
          status: "muted",
          reason: asyncFactory.fromSpec
            ? "sfc-global-component-async-factory"
            : "sfc-global-component-async-factory-nonliteral",
        }),
      );
      return;
    }

    if (!binding) {
      out.push(
        globalRegistrationRecord({
          filePath,
          api,
          componentName,
          line,
          status: "muted",
          reason: "sfc-global-component-value-unsupported",
        }),
      );
      return;
    }

    out.push(
      globalRegistrationRecord({
        filePath,
        api,
        componentName,
        binding,
        line,
      }),
    );
  });

  return markDuplicateGlobalRegistrations(out);
}

function pascalFromKebab(value) {
  if (!/^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/.test(value)) return null;
  return value
    .split("-")
    .filter(Boolean)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("");
}

export function parseSfcTemplateComponentRefs(src, filePath = "<sfc>") {
  return rustFileFacts(src, filePath).templateComponentRefs;
}

export function parseSfcGlobalComponentRegistrations(
  src,
  filePath = "<source>",
) {
  const program = parseScriptAst(src, filePath, parserLangFromFile(filePath));
  if (!program) return [];
  return parseGlobalComponentRegistrations(program, {
    filePath,
    fileSource: src,
  });
}

export function parseSfcGeneratedComponentManifests(
  src,
  filePath = "<manifest>",
  manifestKind = "unplugin-vue-components-dts",
) {
  const program = parseScriptAst(src, filePath, "ts");
  if (!program) return [];
  return parseGeneratedComponentManifestProgram(program, {
    manifestFile: filePath,
    manifestKind,
    fileSource: src,
  });
}

export function parseSfcScriptSources(src, filePath = "<sfc>") {
  return rustFileFacts(src, filePath).scriptSources;
}

export function parseSfcStyleAssetReferences(src, filePath = "<sfc>") {
  return rustFileFacts(src, filePath).styleAssetReferences;
}

export function parseSfcImportConsumers(src, filePath = "<sfc>") {
  return rustFileFacts(src, filePath).scriptImportConsumers;
}

function rustFileFacts(src, filePath) {
  if (!SFC_FAMILY_LANGS.includes(sfcLanguageForFile(filePath))) {
    return {
      scriptImportConsumers: [],
      scriptSources: [],
      styleAssetReferences: [],
      templateComponentRefs: [],
      frameworkConventionComponents: [],
    };
  }
  return extractSfcFileFactsForSources([{ filePath, source: src }])[0];
}

export function collectSfcImportConsumers({
  root,
  includeTests = true,
  exclude = [],
  files = null,
}) {
  const out = [];
  const sfcFiles = filesForLanguages({
    root,
    includeTests,
    exclude,
    languages: SFC_FAMILY_LANGS,
    files,
  });

  for (const filePath of sfcFiles) {
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    out.push(
      ...parseSfcImportConsumers(src, filePath).filter(
        (record) => record.fromSpec !== NUXT_COMPONENTS_ALIAS_SPEC,
      ),
    );
  }

  return out;
}

export function collectSfcTemplateComponentRefs({
  root,
  includeTests = true,
  exclude = [],
  files = null,
}) {
  const out = [];
  const sfcFiles = filesForLanguages({
    root,
    includeTests,
    exclude,
    languages: SFC_FAMILY_LANGS,
    files,
  });

  for (const filePath of sfcFiles) {
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    out.push(...parseSfcTemplateComponentRefs(src, filePath));
  }

  return out;
}

export function collectSfcGlobalComponentRegistrations({
  root,
  includeTests = true,
  exclude = [],
  files = null,
}) {
  const out = [];
  const sourceFiles = jsFamilyFiles({
    root,
    includeTests,
    exclude,
    files,
  });

  for (const filePath of sourceFiles) {
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    if (!mayContainGlobalComponentRegistration(src)) continue;
    out.push(...parseSfcGlobalComponentRegistrations(src, filePath));
  }

  return out;
}

export function collectSfcGeneratedComponentManifests({ root }) {
  const out = [];
  for (const manifest of GENERATED_COMPONENT_MANIFESTS) {
    const filePath = path.join(root, manifest.relPath);
    if (!existsSync(filePath)) continue;
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    out.push(
      ...parseSfcGeneratedComponentManifests(
        src,
        filePath,
        manifest.manifestKind,
      ),
    );
  }
  return out;
}

const PER_FILE_FRAMEWORK_CONVENTION_ORDER = [
  ["astro", "client-directive"],
  ["svelte", "action-directive"],
  ["svelte", "store-auto-subscription"],
  ["vue", "macro-registration"],
  ["vue", "options-registration"],
];

function orderedPerFileFrameworkConventions(rows) {
  const ordered = [];
  const recognized = new Set();
  for (const [framework, conventionKind] of PER_FILE_FRAMEWORK_CONVENTION_ORDER) {
    for (const [index, row] of rows.entries()) {
      if (
        row?.framework !== framework ||
        row?.conventionKind !== conventionKind
      ) {
        continue;
      }
      recognized.add(index);
      ordered.push(row);
    }
  }
  if (recognized.size !== rows.length) {
    throw new Error(
      "collectSfcFrameworkConventionComponents: Rust returned an unknown per-file convention kind",
    );
  }
  return ordered;
}

export function collectSfcFrameworkConventionComponents({
  root,
  includeTests = true,
  exclude = [],
  files = null,
  perFileConventions = null,
  perFileScriptImports = null,
}) {
  const out = [];
  const resolvedRoot = path.resolve(root);
  out.push(...collectUnpluginVueComponentsConfigs(resolvedRoot));
  let scriptImports = perFileScriptImports;
  if (perFileConventions === null && perFileScriptImports === null) {
    const sfcFiles = filesForLanguages({
      root: resolvedRoot,
      includeTests,
      exclude,
      languages: SFC_FAMILY_LANGS,
      files,
    });
    const sourceInputs = [];
    for (const filePath of sfcFiles) {
      try {
        sourceInputs.push({ filePath, source: readFileSync(filePath, "utf8") });
      } catch {
        continue;
      }
    }
    const fileFacts = extractSfcFileFactsForSources(sourceInputs);
    out.push(
      ...orderedPerFileFrameworkConventions(
        fileFacts.flatMap((file) => file.frameworkConventionComponents),
      ),
    );
    scriptImports = fileFacts.flatMap((file) => file.scriptImportConsumers);
  } else {
    if (
      !Array.isArray(perFileConventions) ||
      !Array.isArray(perFileScriptImports)
    ) {
      throw new TypeError(
        "collectSfcFrameworkConventionComponents: per-file facts must be arrays",
      );
    }
    out.push(...orderedPerFileFrameworkConventions(perFileConventions));
  }
  out.push(
    ...collectSfcNuxtComponentsAliasConventions({
      root: resolvedRoot,
      scriptImports,
    }),
  );
  out.push(...collectSfcNuxtComponentsDirConfigConventions({ root: resolvedRoot }));
  out.push(
    ...collectSfcNuxtCustomResolverUnavailableConventions({
      root: resolvedRoot,
    }),
  );
  out.push(
    ...collectSfcNuxtLayerExtendsUnavailableConventions({
      root: resolvedRoot,
    }),
  );
  out.push(
    ...collectSfcNuxtModulePackageUnavailableConventions({
      root: resolvedRoot,
    }),
  );

  if (hasNuxtConventionSignal(resolvedRoot)) {
    const hasAppDirConventionSignal =
      hasNuxtAppDirConventionSignal(resolvedRoot);
    const vueFiles = filesForLanguages({
      root: resolvedRoot,
      includeTests,
      exclude,
      languages: ["vue"],
      files,
    });

    for (const filePath of vueFiles) {
      const rel = path.relative(resolvedRoot, filePath);
      const conventionRoot = nuxtConventionRootForRelPath(rel);
      if (!conventionRoot) continue;
      if (
        conventionRoot.requiresAppSrcDirSignal &&
        !hasAppDirConventionSignal
      ) {
        continue;
      }
      out.push(
        nuxtConventionRecord({
          root: resolvedRoot,
          filePath,
          conventionRoot,
        }),
      );
    }
  }

  return out;
}

export function collectSfcStyleAssetReferences({
  root,
  includeTests = true,
  exclude = [],
  files = null,
}) {
  const out = [];
  const sfcFiles = filesForLanguages({
    root,
    includeTests,
    exclude,
    languages: SFC_FAMILY_LANGS,
    files,
  });

  for (const filePath of sfcFiles) {
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    out.push(...parseSfcStyleAssetReferences(src, filePath));
  }

  return out;
}

export function collectSfcScriptSources({
  root,
  includeTests = true,
  exclude = [],
  files = null,
}) {
  const out = [];
  const sfcFiles = filesForLanguages({
    root,
    includeTests,
    exclude,
    languages: SFC_FAMILY_LANGS,
    files,
  });

  for (const filePath of sfcFiles) {
    let src;
    try {
      src = readFileSync(filePath, "utf8");
    } catch {
      continue;
    }
    out.push(...parseSfcScriptSources(src, filePath));
  }

  return out;
}
