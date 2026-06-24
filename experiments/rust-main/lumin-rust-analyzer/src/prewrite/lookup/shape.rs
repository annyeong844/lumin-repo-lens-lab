mod candidate;
mod evidence;
mod matches;
mod model;

use crate::prewrite::intent::{NormalizedIntent, ShapeIntent};
use lumin_rust_source_health::protocol::HealthResponse;

pub(in crate::prewrite) use candidate::{ShapeLookupMatch, SignatureVisibility};
pub(in crate::prewrite) use evidence::unavailable_evidence_from_shape_lookups;
pub(in crate::prewrite) use model::ShapeLookup;
use model::{ShapeHashSource, ShapeLookupResult};

const FIELD_ONLY_UNAVAILABLE_CITATION: &str =
    "[확인 불가, shape intent lacks exact sha256 shape hash or typeLiteral; field names alone are not structural equality evidence for P4 shape-hash lookup]";
const TYPE_LITERAL_UNAVAILABLE_CITATION: &str =
    "[확인 불가, Rust pre-write shape lookup does not normalize TS/JS shape.typeLiteral; provide an exact Rust source-health shape hash]";

pub(in crate::prewrite) fn lookup_shapes(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
) -> Vec<ShapeLookup> {
    intent
        .shapes
        .iter()
        .map(|shape| lookup_shape(shape, syntax))
        .collect()
}

fn lookup_shape(shape: &ShapeIntent, syntax: &HealthResponse) -> ShapeLookup {
    if let Some(hash) = &shape.hash {
        let source_health_complete =
            syntax.summary.parse_error_files == 0 && syntax.skipped_files.is_empty();

        let matches = matches::shape_hash_matches(hash, syntax);
        if !matches.is_empty() {
            return ShapeLookup::matched(
                shape,
                ShapeLookupResult::ShapeMatch,
                hash,
                ShapeHashSource::Hash,
                matches,
                vec![format!(
                    "[grounded, rust-source-health files[*].ast.shapeHashes matched exact hash {hash}]"
                )],
            );
        }

        let matches = matches::function_signature_matches(hash, syntax);
        if !matches.is_empty() {
            return ShapeLookup::matched(
                shape,
                ShapeLookupResult::SignatureMatch,
                hash,
                ShapeHashSource::FunctionSignature,
                matches,
                vec![format!(
                    "[grounded, rust-source-health files[*].ast.functionSignatures matched exact hash {hash}]"
                )],
            );
        }

        if source_health_complete {
            return ShapeLookup::not_observed(
                shape,
                hash,
                ShapeHashSource::Hash,
                vec![format!(
                    "[grounded, complete rust-source-health files[*].ast.shapeHashes and files[*].ast.functionSignatures have no exact match for {hash}]"
                )],
            );
        }

        return unavailable(
            shape,
            Some(hash.clone()),
            Some(ShapeHashSource::Hash),
            vec![format!(
                "[확인 불가, rust-source-health is incomplete; files[*].ast.shapeHashes and files[*].ast.functionSignatures have no exact match for {hash}, but absence is not grounded]"
            )],
        );
    }

    if shape.type_literal.is_some() {
        return unavailable(
            shape,
            None,
            None,
            vec![TYPE_LITERAL_UNAVAILABLE_CITATION.to_string()],
        );
    }

    unavailable(
        shape,
        None,
        None,
        vec![FIELD_ONLY_UNAVAILABLE_CITATION.to_string()],
    )
}

fn unavailable(
    shape: &ShapeIntent,
    shape_hash: Option<String>,
    shape_hash_source: Option<ShapeHashSource>,
    citations: Vec<String>,
) -> ShapeLookup {
    ShapeLookup::unavailable(shape, shape_hash, shape_hash_source, citations)
}
