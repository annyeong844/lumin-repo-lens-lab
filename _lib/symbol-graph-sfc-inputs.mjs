import { sourceUseProjectionRecordId } from "./source-use-assembly-request.mjs";

function finiteLine(value) {
  return Number.isFinite(value) ? value : undefined;
}

function linkedRecordId(root, recordIds, source, index, consumerFile, fromSpec) {
  if (
    typeof consumerFile !== "string" ||
    consumerFile.length === 0 ||
    typeof fromSpec !== "string" ||
    fromSpec.length === 0
  ) {
    return undefined;
  }
  const recordId = sourceUseProjectionRecordId(root, source, index, {
    consumerFile,
    fromSpec,
  });
  return recordIds.has(recordId) ? recordId : undefined;
}

export function sfcGlobalComponentResolutionSpec(use) {
  if (use?.status !== "muted") return use?.bindingSource;
  if (use.reason === "sfc-global-component-async-factory") {
    return use.fromSpec;
  }
  if (use.reason === "sfc-global-component-duplicate-registration") {
    return use.bindingSource;
  }
  return null;
}

function styleAssetInput(use) {
  return {
    consumerFile: use.consumerFile,
    fromSpec: use.fromSpec,
    source: use.source,
    kind: use.kind,
    styleKind: use.styleKind,
    confidence: use.confidence,
    importSyntax: use.importSyntax,
    line: finiteLine(use.line),
    sfcBlockKind: use.sfcBlockKind,
    sfcLanguage: use.sfcLanguage,
  };
}

function templateComponentInput(root, recordIds, use, index) {
  return {
    consumerFile: use.consumerFile,
    tagName: use.tagName,
    normalizedTagName: use.normalizedTagName,
    bindingName: use.bindingName,
    bindingSource: use.bindingSource,
    source: use.source,
    language: use.language,
    templateKind: use.templateKind,
    confidence: use.confidence,
    ...(use.status === "muted"
      ? {
          status: "muted",
          reason: use.reason ?? "sfc-template-component-muted",
        }
      : {}),
    sourceUseRecordId: linkedRecordId(
      root,
      recordIds,
      "sfc-template-component-ref",
      index,
      use.consumerFile,
      use.bindingSource,
    ),
    bindingKind: use.bindingKind,
    importedName: use.importedName,
    memberName: use.memberName,
    line: finiteLine(use.line),
    sfcBlockKind: use.sfcBlockKind,
  };
}

function globalComponentInput(root, recordIds, use, index) {
  const fromSpec = sfcGlobalComponentResolutionSpec(use);
  return {
    registrationFile: use.registrationFile,
    framework: use.framework,
    api: use.api,
    componentName: use.componentName,
    normalizedTagNames: Array.isArray(use.normalizedTagNames)
      ? [...use.normalizedTagNames]
      : undefined,
    bindingName: use.bindingName,
    bindingSource: use.bindingSource,
    fromSpec: use.fromSpec,
    source: use.source,
    ...(use.status === "muted"
      ? {
          status: "muted",
          reason: use.reason ?? "sfc-global-component-muted",
        }
      : {}),
    sourceUseRecordId: linkedRecordId(
      root,
      recordIds,
      "sfc-global-component-registration",
      index,
      use.registrationFile,
      fromSpec,
    ),
    bindingKind: use.bindingKind,
    importedName: use.importedName,
    factoryKind: use.factoryKind,
    ambiguityKey: use.ambiguityKey,
    line: finiteLine(use.line),
  };
}

function generatedManifestInput(root, recordIds, use, index) {
  return {
    manifestFile: use.manifestFile,
    manifestKind: use.manifestKind,
    componentName: use.componentName,
    normalizedTagNames: Array.isArray(use.normalizedTagNames)
      ? [...use.normalizedTagNames]
      : [],
    bindingSource: use.bindingSource,
    fromSpec: use.fromSpec,
    computedKeySource: use.computedKeySource,
    source: use.source,
    confidence: use.confidence,
    ...(use.status === "skipped"
      ? {
          status: "skipped",
          reason:
            use.reason ?? "sfc-framework-generated-manifest-nonliteral",
        }
      : {}),
    sourceUseRecordId: linkedRecordId(
      root,
      recordIds,
      "sfc-generated-component-manifest",
      index,
      use.manifestFile,
      use.bindingSource,
    ),
    line: finiteLine(use.line),
  };
}

export function buildSymbolGraphSfcInputs({
  root,
  styleAssetReferences,
  templateComponentRefs,
  globalComponentRegistrations,
  generatedComponentManifests,
  frameworkConventionComponents,
  templateRecordIds,
  globalRecordIds,
  generatedManifestRecordIds,
}) {
  return {
    styleAssetReferences: styleAssetReferences.map(styleAssetInput),
    templateComponentRefs: templateComponentRefs.map((use, index) =>
      templateComponentInput(root, templateRecordIds, use, index),
    ),
    globalComponentRegistrations: globalComponentRegistrations.map(
      (use, index) => globalComponentInput(root, globalRecordIds, use, index),
    ),
    generatedComponentManifests: generatedComponentManifests.map((use, index) =>
      generatedManifestInput(
        root,
        generatedManifestRecordIds,
        use,
        index,
      ),
    ),
    generatedManifestExternalUses: 0,
    frameworkConventionComponents: [...frameworkConventionComponents],
  };
}
