import { readFileSync } from "node:fs";
import path from "node:path";

import {
  collectSfcFrameworkConventionComponents,
  collectSfcGeneratedComponentManifests,
  collectSfcGlobalComponentRegistrations,
  collectSfcImportConsumers,
  collectSfcScriptSources,
  collectSfcStyleAssetReferences,
  collectSfcTemplateComponentRefs,
} from "./sfc-consumers.mjs";

const SFC_PACKAGE_ROOTS = new Set([
  "astro",
  "nuxt",
  "svelte",
  "unplugin-vue-components",
  "vue",
]);

function packageRootFromSpecifier(spec) {
  if (
    typeof spec !== "string" ||
    spec.length === 0 ||
    spec.startsWith(".") ||
    spec.startsWith("/") ||
    spec.startsWith("#")
  ) {
    return null;
  }
  const parts = spec.split("/");
  if (spec.startsWith("@")) {
    return parts.length >= 2 ? `${parts[0]}/${parts[1]}` : spec;
  }
  return parts[0] ?? null;
}

function isSfcPackageRoot(name) {
  if (typeof name !== "string" || name.length === 0) return false;
  return (
    SFC_PACKAGE_ROOTS.has(name) ||
    name.startsWith("@astrojs/") ||
    name.startsWith("@nuxt/") ||
    name.startsWith("@sveltejs/") ||
    name.startsWith("@vitejs/plugin-vue") ||
    name.startsWith("@vue/")
  );
}

function packageJsonHasSfcDependency(packageJson) {
  return [
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
  ].some((field) =>
    Object.keys(packageJson?.[field] ?? {}).some(isSfcPackageRoot),
  );
}

function readPackageJsonAtDir(directory) {
  try {
    return JSON.parse(readFileSync(path.join(directory, "package.json"), "utf8"));
  } catch {
    return null;
  }
}

function repoHasSfcPackageDependency(repoMode) {
  if (packageJsonHasSfcDependency(repoMode.rootPkgJson)) return true;
  return (repoMode.workspaceDirs ?? []).some((directory) =>
    packageJsonHasSfcDependency(readPackageJsonAtDir(directory)),
  );
}

function specifierHasSfcSignal(spec) {
  if (typeof spec !== "string" || spec.length === 0) return false;
  const withoutQuery = spec.split("?")[0] ?? spec;
  if (/\.(?:astro|svelte|vue)$/i.test(withoutQuery)) return true;
  return isSfcPackageRoot(packageRootFromSpecifier(spec));
}

function fileDataHasSfcImportSignal(fileData) {
  for (const info of fileData.values()) {
    for (const use of info.uses ?? []) {
      if (specifierHasSfcSignal(use?.fromSpec)) return true;
    }
  }
  return false;
}

function collectTimed(phaseTimer, phase, collect) {
  const started = Date.now();
  try {
    return collect();
  } finally {
    phaseTimer.recordPhase(phase, Date.now() - started);
  }
}

export function discoverSymbolGraphSfcFacts({
  root,
  includeTests,
  exclude,
  files,
  sfcSourceFiles,
  fileData,
  repoMode,
  phaseTimer,
}) {
  const frameworkSignalDetected =
    sfcSourceFiles.length > 0 ||
    repoHasSfcPackageDependency(repoMode) ||
    fileDataHasSfcImportSignal(fileData);

  const scopedCollectorInput = {
    root,
    includeTests,
    exclude,
    files: sfcSourceFiles,
  };
  const collectWhenSfcFilesExist = (phase, collect) =>
    collectTimed(phaseTimer, phase, () =>
      sfcSourceFiles.length > 0 ? collect(scopedCollectorInput) : [],
    );

  const scriptImportConsumers = collectWhenSfcFilesExist(
    "collect-sfc-script-imports",
    collectSfcImportConsumers,
  );
  const scriptSources = collectWhenSfcFilesExist(
    "collect-sfc-script-src",
    collectSfcScriptSources,
  );
  const styleAssetReferences = collectWhenSfcFilesExist(
    "collect-sfc-style-assets",
    collectSfcStyleAssetReferences,
  );
  const templateComponentRefs = collectWhenSfcFilesExist(
    "collect-sfc-template-component-refs",
    collectSfcTemplateComponentRefs,
  );
  const globalComponentRegistrations = collectTimed(
    phaseTimer,
    "collect-sfc-global-component-registrations",
    () =>
      frameworkSignalDetected
        ? collectSfcGlobalComponentRegistrations({
            root,
            includeTests,
            exclude,
            files,
          })
        : [],
  );
  const generatedComponentManifests = collectTimed(
    phaseTimer,
    "collect-sfc-generated-component-manifests",
    () => collectSfcGeneratedComponentManifests({ root }),
  );
  const frameworkConventionComponents = collectTimed(
    phaseTimer,
    "collect-sfc-framework-convention-components",
    () => collectSfcFrameworkConventionComponents(scopedCollectorInput),
  );

  phaseTimer.setCounter(
    "sfcFrameworkSignalDetected",
    frameworkSignalDetected ? 1 : 0,
  );
  phaseTimer.setCounter(
    "sfcFrameworkSignalScanSkipped",
    frameworkSignalDetected ? 0 : 1,
  );
  phaseTimer.setCounter(
    "sfcScriptImportConsumerCandidateCount",
    scriptImportConsumers.length,
  );
  phaseTimer.setCounter("sfcScriptSrcCandidateCount", scriptSources.length);
  phaseTimer.setCounter(
    "sfcStyleAssetCandidateCount",
    styleAssetReferences.length,
  );
  phaseTimer.setCounter(
    "sfcTemplateComponentRefCandidateCount",
    templateComponentRefs.length,
  );
  phaseTimer.setCounter(
    "sfcGlobalComponentRegistrationCandidateCount",
    globalComponentRegistrations.length,
  );
  phaseTimer.setCounter(
    "sfcGlobalComponentRegistrationScanSkipped",
    frameworkSignalDetected ? 0 : 1,
  );
  phaseTimer.setCounter(
    "sfcGeneratedComponentManifestCandidateCount",
    generatedComponentManifests.length,
  );
  phaseTimer.setCounter(
    "sfcFrameworkConventionComponentCandidateCount",
    frameworkConventionComponents.length,
  );

  return {
    frameworkSignalDetected,
    scriptImportConsumers,
    scriptSources,
    styleAssetReferences,
    templateComponentRefs,
    globalComponentRegistrations,
    generatedComponentManifests,
    frameworkConventionComponents,
  };
}
