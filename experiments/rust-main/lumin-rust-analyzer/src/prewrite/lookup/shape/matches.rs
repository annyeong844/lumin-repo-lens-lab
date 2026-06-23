use lumin_rust_source_health::protocol::HealthResponse;

use super::candidate::{ShapeLookupMatch, ShapeMatch, SignatureMatch};

pub(super) fn shape_hash_matches(hash: &str, syntax: &HealthResponse) -> Vec<ShapeLookupMatch> {
    let mut matches = Vec::new();
    for (owner_file, file) in &syntax.files {
        for fact in &file.ast.shape_hashes {
            if fact.hash != hash {
                continue;
            }
            matches.push(ShapeLookupMatch::Shape(ShapeMatch::from_fact(
                owner_file,
                fact,
                file.path.classifications.clone(),
                file.path.suppressed,
            )));
        }
    }
    matches.sort_by(|left, right| left.identity().cmp(right.identity()));
    matches
}

pub(super) fn function_signature_matches(
    hash: &str,
    syntax: &HealthResponse,
) -> Vec<ShapeLookupMatch> {
    let mut matches = Vec::new();
    for (owner_file, file) in &syntax.files {
        for fact in &file.ast.function_signatures {
            if fact.hash != hash {
                continue;
            }
            matches.push(ShapeLookupMatch::Signature(SignatureMatch::from_fact(
                owner_file,
                fact,
                file.path.classifications.clone(),
                file.path.suppressed,
            )));
        }
    }
    matches.sort_by(|left, right| left.identity().cmp(right.identity()));
    matches
}
