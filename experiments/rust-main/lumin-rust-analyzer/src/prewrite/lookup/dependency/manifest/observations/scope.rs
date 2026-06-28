use crate::prewrite::lookup::dependency::scope::CargoManifestScope;

pub(super) fn scope_for_file<'a>(
    scopes: &'a [CargoManifestScope],
    file: &str,
) -> Option<&'a CargoManifestScope> {
    scopes
        .iter()
        .filter(|scope| scope.file_is_in_scope(file))
        .max_by_key(|scope| scope.scope_priority_len())
}
