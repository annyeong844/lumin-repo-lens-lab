use lumin_rust_cargo_oracle::protocol::{
    CargoTargetKind, ClaimKind, CleanKind, CleanScope, CoverageEntry, CoverageId, CoverageKind,
    CoverageStatus, OracleCfgSetSource, OracleId, OracleScope, OracleScopeFeatureSelection,
    OracleScopeKind, OracleScopeProfile, OracleScopeTarget, OracleScopeTargetSource,
    OracleTargetTripleSource, StreamParseStatus,
};
use serde::Serialize;

use crate::policy::{ProductCoverageUnavailableReason, ORACLE_SCOPE_SAMPLE_LIMIT};

pub(in crate::product_artifact) fn coverage_projection(
    entries: &[CoverageEntry],
) -> ProductCoverageProjection<'_> {
    ProductCoverageProjection {
        entries: entries
            .iter()
            .map(ProductCoverageEntryProjection::from_entry)
            .collect(),
    }
}

fn is_empty<T>(items: &[T]) -> bool {
    items.is_empty()
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(in crate::product_artifact) struct ProductCoverageProjection<'a> {
    entries: Vec<ProductCoverageEntryProjection<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductCoverageEntryProjection<'a> {
    id: CoverageId,
    oracle_id: OracleId,
    coverage_kind: CoverageKind,
    status: CoverageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_parse_status: Option<StreamParseStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invalid_json_line_count: Option<usize>,
    scope: ProductOracleScopeProjection<'a>,
    command_arg_count: usize,
    exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<ProductCoverageUnavailableReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clean_kind: Option<CleanKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clean_scope: Option<CleanScope>,
    #[serde(skip_serializing_if = "is_empty")]
    absence_of_claim_kinds: &'a [ClaimKind],
    #[serde(skip_serializing_if = "is_empty")]
    allows_concurrent_claim_kinds: &'a [ClaimKind],
    #[serde(skip_serializing_if = "Option::is_none")]
    clean: Option<bool>,
}

impl<'a> ProductCoverageEntryProjection<'a> {
    fn from_entry(entry: &'a CoverageEntry) -> Self {
        Self {
            id: entry.id,
            oracle_id: entry.oracle_id,
            coverage_kind: entry.coverage_kind,
            status: entry.status,
            stream_parse_status: entry.stream_parse_status,
            invalid_json_line_count: entry.invalid_json_line_count,
            scope: ProductOracleScopeProjection::from_scope(&entry.scope),
            command_arg_count: entry.command_args.len(),
            exit_code: entry.exit_code,
            reason: entry
                .reason
                .as_ref()
                .map(ProductCoverageUnavailableReason::from_reason),
            clean_kind: entry.clean_kind,
            clean_scope: entry.clean_scope,
            absence_of_claim_kinds: &entry.absence_of_claim_kinds,
            allows_concurrent_claim_kinds: &entry.allows_concurrent_claim_kinds,
            clean: entry.clean,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductOracleScopeProjection<'a> {
    kind: OracleScopeKind,
    package: &'a str,
    package_name_count: usize,
    package_name_examples: &'a [String],
    target: &'a str,
    target_count: usize,
    target_examples: Vec<ProductOracleScopeTargetProjection<'a>>,
    feature_count: usize,
    feature_examples: &'a [String],
    feature_selection: &'a OracleScopeFeatureSelection,
    target_triple: &'a str,
    target_triple_count: usize,
    target_triple_examples: &'a [String],
    target_triple_source: &'a OracleTargetTripleSource,
    cfg_set_count: usize,
    cfg_set_source: &'a OracleCfgSetSource,
    cfg_set_complete: bool,
    profile: OracleScopeProfile,
}

impl<'a> ProductOracleScopeProjection<'a> {
    fn from_scope(scope: &'a OracleScope) -> Self {
        Self {
            kind: scope.kind,
            package: &scope.package,
            package_name_count: scope.package_names.len(),
            package_name_examples: &scope.package_names
                [..scope.package_names.len().min(ORACLE_SCOPE_SAMPLE_LIMIT)],
            target: &scope.target,
            target_count: scope.targets.len(),
            target_examples: scope
                .targets
                .iter()
                .take(ORACLE_SCOPE_SAMPLE_LIMIT)
                .map(ProductOracleScopeTargetProjection::from_target)
                .collect(),
            feature_count: scope.feature_set.len(),
            feature_examples: &scope.feature_set
                [..scope.feature_set.len().min(ORACLE_SCOPE_SAMPLE_LIMIT)],
            feature_selection: &scope.feature_selection,
            target_triple: &scope.target_triple,
            target_triple_count: scope.target_triples.len(),
            target_triple_examples: &scope.target_triples
                [..scope.target_triples.len().min(ORACLE_SCOPE_SAMPLE_LIMIT)],
            target_triple_source: &scope.target_triple_source,
            cfg_set_count: scope.cfg_set.len(),
            cfg_set_source: &scope.cfg_set_source,
            cfg_set_complete: scope.cfg_set_complete,
            profile: scope.profile,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductOracleScopeTargetProjection<'a> {
    package_name: Option<&'a str>,
    target_name: &'a str,
    target_kinds: &'a [CargoTargetKind],
    source: OracleScopeTargetSource,
}

impl<'a> ProductOracleScopeTargetProjection<'a> {
    fn from_target(target: &'a OracleScopeTarget) -> Self {
        Self {
            package_name: target.package_name.as_deref(),
            target_name: &target.target_name,
            target_kinds: &target.target_kinds,
            source: target.source,
        }
    }
}
