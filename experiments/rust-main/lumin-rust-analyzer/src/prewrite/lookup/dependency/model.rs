use serde::Serialize;

pub(in crate::prewrite) const DEPENDENCY_EXAMPLE_LIMIT: usize = 5;
pub(in crate::prewrite) const DEPENDENCY_WATCH_FOR_THRESHOLD: usize = 10;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct DependencyLookup {
    pub(super) kind: DependencyLookupKind,
    pub(in crate::prewrite) dep_name: String,
    pub(super) declared_in: Option<String>,
    pub(super) result: DependencyLookupResult,
    pub(super) existing_imports: ExistingImports,
    pub(super) citations: Vec<String>,
}

impl DependencyLookup {
    pub(in crate::prewrite) fn is_watch_for_eligible(&self) -> bool {
        self.existing_imports.count_confidence == ImportCountConfidence::Grounded
            && self
                .existing_imports
                .observed_import_count
                .is_some_and(|count| count >= DEPENDENCY_WATCH_FOR_THRESHOLD)
    }

    pub(in crate::prewrite) fn observed_import_count(&self) -> Option<usize> {
        self.existing_imports.observed_import_count
    }

    pub(in crate::prewrite) fn result(&self) -> DependencyLookupResult {
        self.result
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) enum DependencyLookupKind {
    #[serde(rename = "dependency")]
    Dependency,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(in crate::prewrite) enum DependencyLookupResult {
    #[serde(rename = "DEPENDENCY_AVAILABLE")]
    Available,
    #[serde(rename = "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS")]
    AvailableNoObservedImports,
    #[serde(rename = "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE")]
    AvailableImportGraphUnavailable,
    #[serde(rename = "DEPENDENCY_SCOPE_UNAVAILABLE")]
    ScopeUnavailable,
    #[serde(rename = "NEW_PACKAGE")]
    NewPackage,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ExistingImports {
    pub(super) examples: Vec<DependencyImportExample>,
    pub(super) observed_import_count: Option<usize>,
    pub(super) count_confidence: ImportCountConfidence,
    pub(super) unavailable_reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DependencyImportExample {
    pub(super) file: String,
    pub(super) from_spec: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ImportCountConfidence {
    Grounded,
    SampleOnly,
    Unavailable,
}
