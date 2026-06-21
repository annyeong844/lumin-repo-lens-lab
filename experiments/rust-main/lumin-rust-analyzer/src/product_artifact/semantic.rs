use lumin_rust_cargo_oracle::protocol::{
    CargoCheckMode, CargoTargetKind, ClaimKind, CleanKind, CleanScope, CoverageEntry, CoverageId,
    CoverageKind, CoverageStatus, OracleCfgSetSource, OracleId, OraclePlan, OraclePlanReason,
    OraclePlanSelectedPackage, OraclePlanStatus, OracleScope, OracleScopeFeatureSelection,
    OracleScopeKind, OracleScopeProfile, OracleScopeTarget, OracleScopeTargetSource,
    OracleTargetTripleSource, StreamParseStatus,
};
use serde::Serialize;

use crate::policy::{ProductCoverageUnavailableReason, ORACLE_SCOPE_SAMPLE_LIMIT};

pub(super) fn coverage_projection(entries: &[CoverageEntry]) -> ProductCoverageProjection<'_> {
    ProductCoverageProjection {
        entries: entries
            .iter()
            .map(ProductCoverageEntryProjection::from_entry)
            .collect(),
    }
}

pub(super) fn oracle_plan_projection(plan: &OraclePlan) -> ProductOraclePlanProjection<'_> {
    ProductOraclePlanProjection::from_plan(plan)
}

fn is_empty<T>(items: &[T]) -> bool {
    items.is_empty()
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(super) struct ProductCoverageProjection<'a> {
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductOraclePlanProjection<'a> {
    schema_version: &'static str,
    mode: CargoCheckMode,
    status: OraclePlanStatus,
    reason: OraclePlanReason,
    target_path_count: usize,
    target_path_examples: &'a [String],
    selected_target_path_count: usize,
    omitted_target_path_count: usize,
    candidate_package_count: usize,
    selected_package_count: usize,
    targeted_package_cap: usize,
    selected_package_examples: Vec<ProductOraclePlanSelectedPackageProjection<'a>>,
    omitted_package_count: usize,
    omitted_package_examples: &'a [String],
    unmatched_target_path_count: usize,
    unmatched_target_path_examples: &'a [String],
}

impl<'a> ProductOraclePlanProjection<'a> {
    fn from_plan(plan: &'a OraclePlan) -> Self {
        Self {
            schema_version: plan.schema_version,
            mode: plan.mode,
            status: plan.status,
            reason: plan.reason,
            target_path_count: plan.target_path_count,
            target_path_examples: &plan.target_path_examples,
            selected_target_path_count: plan.selected_target_path_count,
            omitted_target_path_count: plan.omitted_target_path_count,
            candidate_package_count: plan.candidate_package_count,
            selected_package_count: plan.selected_package_count,
            targeted_package_cap: plan.targeted_package_cap,
            selected_package_examples: plan
                .selected_packages
                .iter()
                .take(ORACLE_SCOPE_SAMPLE_LIMIT)
                .map(ProductOraclePlanSelectedPackageProjection::from_selected_package)
                .collect(),
            omitted_package_count: plan.omitted_package_count,
            omitted_package_examples: &plan.omitted_package_examples,
            unmatched_target_path_count: plan.unmatched_target_path_count,
            unmatched_target_path_examples: &plan.unmatched_target_paths[..plan
                .unmatched_target_paths
                .len()
                .min(ORACLE_SCOPE_SAMPLE_LIMIT)],
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductOraclePlanSelectedPackageProjection<'a> {
    package_name: &'a str,
    reason: OraclePlanReason,
    target_path_count: usize,
    target_path_examples: &'a [String],
}

impl<'a> ProductOraclePlanSelectedPackageProjection<'a> {
    fn from_selected_package(package: &'a OraclePlanSelectedPackage) -> Self {
        Self {
            package_name: &package.package_name,
            reason: package.reason,
            target_path_count: package.target_path_count,
            target_path_examples: &package.target_path_examples,
        }
    }
}
