use super::super::input::SourceUseAssemblyRecord;
use super::generated::handle_generated_virtual_record;
use super::glob_record::handle_import_meta_glob_record;
use super::internal::{handle_projection_only_target, handle_resolved_internal_record};
use super::support::{
    is_projection_only_consumer_source, is_relative_spec, looks_like_non_source_asset, skip,
    AssemblyState,
};
use super::terminal::{
    handle_external_record, handle_non_source_asset_record, handle_relative_target_missing,
    handle_unresolved_record,
};

pub(super) fn assemble_record(state: &mut AssemblyState, record: SourceUseAssemblyRecord) {
    let resolver_stage = record.resolver_stage.as_deref();
    let rust_resolved_relative = resolver_stage == Some("relative");
    let has_pre_resolved_file = record
        .resolved_file
        .as_deref()
        .is_some_and(|path| !path.is_empty());
    if resolver_stage == Some("external") {
        handle_external_record(state, record);
        return;
    }
    if resolver_stage == Some("generated-virtual") {
        handle_generated_virtual_record(state, record);
        return;
    }
    if resolver_stage == Some("non-source-asset") {
        handle_non_source_asset_record(state, record);
        return;
    }
    let track_unresolved_prefix = resolver_stage == Some("unresolved-internal");
    if matches!(
        resolver_stage,
        Some("unresolved-internal" | "unresolved-relative")
    ) {
        handle_unresolved_record(state, record, track_unresolved_prefix);
        return;
    }
    let kind = record.kind.as_deref().unwrap_or("import");
    let supported_stage = match resolver_stage {
        Some("relative") => true,
        Some("resolved-internal") => has_pre_resolved_file,
        Some("import-meta-glob") if kind == "import-meta-glob" => true,
        Some(_) => false,
        None => true,
    };
    if !supported_stage {
        skip(state, record.record_id, "non-relative-resolver-stage");
        return;
    }
    let from_spec = record.from_spec.as_deref().unwrap_or_default();
    if kind == "import-meta-glob" {
        handle_import_meta_glob_record(state, record);
        return;
    }
    if is_projection_only_consumer_source(record.consumer_source.as_deref())
        && has_pre_resolved_file
    {
        let resolved_file = record.resolved_file.clone().unwrap_or_default();
        handle_projection_only_target(state, record, &resolved_file);
        return;
    }
    if !has_pre_resolved_file && !is_relative_spec(from_spec) {
        skip(state, record.record_id, "non-relative-specifier");
        return;
    }
    if !has_pre_resolved_file && looks_like_non_source_asset(from_spec) {
        skip(state, record.record_id, "non-source-asset-specifier");
        return;
    }

    let resolved_file = record
        .resolved_file
        .as_deref()
        .filter(|path| !path.is_empty())
        .map(ToString::to_string)
        .or_else(|| state.resolver.resolve(&record.consumer_file, from_spec));
    let Some(resolved_file) = resolved_file else {
        if state.options.relative_target_missing_is_unresolved {
            handle_relative_target_missing(state, record);
        } else {
            skip(state, record.record_id, "relative-target-missing");
        }
        return;
    };

    if is_projection_only_consumer_source(record.consumer_source.as_deref()) {
        handle_projection_only_target(state, record, &resolved_file);
        return;
    }

    handle_resolved_internal_record(state, record, resolved_file, rust_resolved_relative);
}
