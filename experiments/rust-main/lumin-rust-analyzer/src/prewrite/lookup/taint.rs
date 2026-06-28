use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::{AstOpaqueSurfaceVisibility, HealthResponse};

use super::model::{Locality, TaintSummary};

pub(super) fn locality(candidate_file: &str, intent_owner_file: Option<&str>) -> Locality {
    let Some(intent_owner_file) = intent_owner_file else {
        return Locality::default();
    };
    let intent_owner_file = intent_owner_file.replace('\\', "/");
    Locality {
        same_file: candidate_file == intent_owner_file,
        same_dir: dirname(candidate_file) == dirname(&intent_owner_file),
    }
}

fn dirname(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or("")
}

pub(super) fn taint_summary(syntax: &HealthResponse) -> Option<TaintSummary> {
    let mut by_kind = BTreeMap::new();
    for file in syntax.files.values() {
        for surface in &file.ast.opaque_surfaces {
            if surface.visibility == AstOpaqueSurfaceVisibility::Review {
                *by_kind.entry(surface.kind).or_insert(0) += 1;
            }
        }
    }
    if syntax.summary.parse_error_files == 0 && syntax.summary.review_opaque_surfaces == 0 {
        return None;
    }
    Some(TaintSummary {
        parse_error_files: syntax.summary.parse_error_files,
        review_opaque_surfaces: syntax.summary.review_opaque_surfaces,
        review_opaque_surfaces_by_kind: by_kind,
    })
}
