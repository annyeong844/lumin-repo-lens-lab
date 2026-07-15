use std::collections::{BTreeMap, BTreeSet};

use crate::relative_source_resolver::RelativeSourceResolver;

use super::super::namespace::NamespaceReExportResolver;
use super::super::protocol::{
    ResolvedRecordTarget, SkippedSourceUseRecord, SourceUseAssemblyCounters,
    SourceUseAssemblyResponse, SourceUseAssemblySummary,
    SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
};

#[derive(Clone, Copy)]
pub(super) struct SourceUseAssemblyBuildOptions {
    pub(super) emit_standalone_transport: bool,
    pub(super) relative_target_missing_is_unresolved: bool,
}

pub(super) const STANDALONE_BUILD_OPTIONS: SourceUseAssemblyBuildOptions =
    SourceUseAssemblyBuildOptions {
        emit_standalone_transport: true,
        relative_target_missing_is_unresolved: false,
    };

pub(super) const EMBEDDED_BUILD_OPTIONS: SourceUseAssemblyBuildOptions =
    SourceUseAssemblyBuildOptions {
        emit_standalone_transport: false,
        relative_target_missing_is_unresolved: true,
    };

pub(super) struct AssemblyState {
    pub(super) response: SourceUseAssemblyResponse,
    pub(super) root: String,
    pub(super) resolver: RelativeSourceResolver,
    pub(super) namespace_resolver: NamespaceReExportResolver,
    pub(super) namespace_users_seen: BTreeSet<(String, String)>,
    pub(super) import_meta_glob_cap: usize,
    pub(super) options: SourceUseAssemblyBuildOptions,
}

impl AssemblyState {
    pub(super) fn new(
        response_root: String,
        root: String,
        resolver: RelativeSourceResolver,
        namespace_resolver: NamespaceReExportResolver,
        import_meta_glob_cap: usize,
        options: SourceUseAssemblyBuildOptions,
        record_count: usize,
    ) -> Self {
        Self {
            response: SourceUseAssemblyResponse {
                schema_version: SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
                root: response_root,
                summary: SourceUseAssemblySummary {
                    record_count,
                    ..SourceUseAssemblySummary::default()
                },
                handled_record_ids: Vec::new(),
                resolved_record_targets: Vec::new(),
                external_record_ids: Vec::new(),
                non_source_asset_record_ids: Vec::new(),
                non_source_asset_record_targets: Vec::new(),
                generated_virtual_record_ids: Vec::new(),
                skipped_records: Vec::new(),
                counters: SourceUseAssemblyCounters::default(),
                branch_counts: BTreeMap::new(),
                resolved_internal_edges: Vec::new(),
                dependency_import_consumers: Vec::new(),
                unresolved_internal_by_prefix: BTreeMap::new(),
                prefix_examples: BTreeMap::new(),
                unresolved_internal_specifiers: BTreeSet::new(),
                unresolved_internal_specifier_records: Vec::new(),
                direct_consumers: Vec::new(),
                namespace_users: Vec::new(),
                namespace_re_export_diagnostics: Vec::new(),
                generated_virtual_surfaces: Vec::new(),
                generated_virtual_import_consumers: Vec::new(),
            },
            root,
            resolver,
            namespace_resolver,
            namespace_users_seen: BTreeSet::new(),
            import_meta_glob_cap,
            options,
        }
    }

    pub(super) fn into_response(self) -> SourceUseAssemblyResponse {
        self.response
    }
}

pub(super) fn mark_handled(state: &mut AssemblyState, record_id: String) {
    state.response.summary.handled_count += 1;
    if state.options.emit_standalone_transport {
        state.response.handled_record_ids.push(record_id);
    }
}

pub(super) fn skip(state: &mut AssemblyState, record_id: String, reason: &'static str) {
    state.response.summary.skipped_count += 1;
    state
        .response
        .skipped_records
        .push(SkippedSourceUseRecord { record_id, reason });
}

pub(super) fn push_resolved_record_target(
    state: &mut AssemblyState,
    record_id: &str,
    resolved_file: &str,
) {
    state
        .response
        .resolved_record_targets
        .push(ResolvedRecordTarget {
            record_id: record_id.to_string(),
            resolved_file: resolved_file.to_string(),
        });
}

pub(super) fn increment_branch(state: &mut AssemblyState, name: &str) {
    *state
        .response
        .branch_counts
        .entry(name.to_string())
        .or_insert(0) += 1;
}

pub(super) fn increment_out_of_band_consumer_counter(
    counters: &mut SourceUseAssemblyCounters,
    consumer_source: Option<&str>,
) {
    match consumer_source {
        Some("mdx-import") => counters.mdx_consumer_uses += 1,
        Some("sfc-script-import") => counters.sfc_script_consumer_uses += 1,
        Some("sfc-script-src") => counters.sfc_script_src_reachability_uses += 1,
        _ => {}
    }
}

pub(super) fn is_projection_only_consumer_source(consumer_source: Option<&str>) -> bool {
    matches!(
        consumer_source,
        Some(
            "sfc-template-component-ref"
                | "sfc-global-component-registration"
                | "sfc-generated-component-manifest"
        )
    )
}

pub(super) fn is_namespace_reexport_use(kind: &str) -> bool {
    kind == "imported-namespace-member" || kind == "imported-namespace-escape"
}

pub(super) fn is_relative_spec(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../")
}

pub(super) fn looks_like_non_source_asset(spec: &str) -> bool {
    let stripped = strip_resource_query(spec);
    has_extension(stripped) && !js_source_extension(stripped)
}

fn strip_resource_query(spec: &str) -> &str {
    let query = spec.find('?');
    let fragment = spec.find('#').filter(|index| *index > 0);
    match (query, fragment) {
        (Some(left), Some(right)) => &spec[..left.min(right)],
        (Some(index), None) | (None, Some(index)) => &spec[..index],
        (None, None) => spec,
    }
}

fn has_extension(spec: &str) -> bool {
    let file_name = spec.rsplit('/').next().unwrap_or(spec);
    file_name
        .rfind('.')
        .is_some_and(|index| index > 0 && index + 1 < file_name.len())
}

fn js_source_extension(spec: &str) -> bool {
    let lower = spec.to_ascii_lowercase();
    [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext))
}

pub(super) fn is_broad_namespace_use(kind: &str) -> bool {
    matches!(
        kind,
        "namespace"
            | "reExportAll"
            | "dynamic"
            | "cjs-namespace-escape"
            | "cjs-reexport-broad"
            | "dynamic-import-meta-glob"
    )
}

pub(super) fn requires_symbol_name(kind: &str) -> bool {
    !matches!(
        kind,
        "cjs-side-effect-only"
            | "import-side-effect"
            | "reExportNamespace"
            | "sfc-script-src"
            | "namespace"
            | "reExportAll"
            | "dynamic"
            | "import-meta-glob"
            | "dynamic-import-meta-glob"
            | "cjs-namespace-escape"
            | "cjs-reexport-broad"
    )
}

pub(super) fn edge_kind_for_use(kind: &str) -> &str {
    match kind {
        "import" => "import-named",
        "default" => "import-default",
        "namespace" | "namespace-member" => "import-namespace",
        "import-side-effect" => "import-side-effect",
        "reExport" => "reexport-named",
        "reExportAll" => "reexport-broad",
        "reExportNamespace" => "reexport-namespace",
        "imported-namespace-member" => "reexport-namespace-member",
        "imported-namespace-escape" => "reexport-namespace-escape",
        "dynamic" | "dynamic-member" => "dynamic-literal",
        "cjs-side-effect-only" => "cjs-side-effect",
        "cjs-require-exact" => "cjs-require-exact",
        "cjs-namespace-member" => "cjs-namespace-member",
        "cjs-namespace-escape" => "cjs-namespace-escape",
        "cjs-reexport-broad" => "cjs-reexport-broad",
        "sfc-script-src" => "sfc-script-src",
        other => other,
    }
}
