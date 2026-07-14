mod generated_blind_zones;
mod indexes;
mod ordering;
mod paths;
mod surfaces;
mod unresolved;

pub(super) use generated_blind_zones::build_generated_consumer_blind_zones;
pub(super) use indexes::{
    build_class_method_index, build_pre_write_local_operation_index, build_re_exports_by_file,
};
pub(super) use ordering::{
    dependency_consumer_key, generated_blind_zone_key, generated_import_consumer_key,
    namespace_re_export_key, resolved_internal_edge_key, sfc_framework_convention_key,
    sfc_generated_manifest_key, sfc_global_registration_key, sfc_style_asset_key,
    sfc_template_ref_key, sort_generated_virtual_surfaces, sort_values_by_key, sorted_strings,
    unresolved_record_key, value_string,
};
pub(super) use paths::{is_absolute_like_path, normalize_path_segments, rel_path};
pub(super) use surfaces::{
    build_cjs_export_surface_by_file, build_cjs_require_opacity, build_dynamic_import_opacity,
    files_with_parse_errors,
};
pub(super) use unresolved::{top_unresolved_specifiers, unresolved_summary_by_reason};
