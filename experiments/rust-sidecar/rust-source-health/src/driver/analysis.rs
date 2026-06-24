use anyhow::Result;

use crate::analyzer::analyze_files;
use crate::function_clones::group_function_body_fingerprints;
use crate::parallel::{build_pool, RuntimeConfig};
use crate::protocol::{
    HealthRequest, HealthResponse, InputMeta, ParserMeta, PolicyMeta, ResponseMeta, RuntimeMeta,
    SidecarMeta, SkippedFile, SourceHealthLimit, SourceHealthMode, SourceHealthProducer,
    PARSER_EDITION, PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE, PARSER_KIND, PARSER_VERSION,
    POLICY_VERSION, SCHEMA_VERSION, SIGNAL_POLICY_ID, SIGNAL_POLICY_VERSION,
};
use crate::summary::summarize;

pub struct FinalMeta {
    pub generated: String,
    pub sidecar: SidecarMeta,
    pub input: InputMeta,
}

pub fn analyze_request(
    request: HealthRequest,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
) -> Result<HealthResponse> {
    let runtime_config = RuntimeConfig::try_from(request.runtime)?;
    let pool = build_pool(runtime_config)?;
    let files = pool.install(|| analyze_files(&request.files, &request.parser))?;
    let function_clone_groups = group_function_body_fingerprints(&files);
    let mut summary = summarize(&files);
    summary.skipped_files = skipped_files.len();
    summary.function_clone_exact_body_groups = function_clone_groups.exact_body_groups.len();
    summary.function_clone_structure_groups = function_clone_groups.structure_groups.len();
    let (generated, sidecar, input) = final_meta
        .map(|meta| (Some(meta.generated), Some(meta.sidecar), Some(meta.input)))
        .unwrap_or((None, None, None));
    Ok(HealthResponse {
        schema_version: SCHEMA_VERSION,
        meta: ResponseMeta {
            producer: SourceHealthProducer::RustSourceHealth,
            mode: SourceHealthMode::SyntaxOnly,
            parser: ParserMeta {
                kind: PARSER_KIND,
                version: PARSER_VERSION.to_string(),
                edition_policy: PARSER_EDITION_POLICY,
                edition: PARSER_EDITION,
                edition_source: PARSER_EDITION_SOURCE,
            },
            policy: PolicyMeta {
                version: POLICY_VERSION.to_string(),
                signal_policy: crate::protocol::SignalPolicyMeta {
                    id: SIGNAL_POLICY_ID.to_string(),
                    version: SIGNAL_POLICY_VERSION.to_string(),
                },
            },
            runtime: RuntimeMeta {
                thread_count: pool.current_num_threads(),
                worker_stack_bytes: runtime_config.worker_stack_bytes,
            },
            limits: [
                SourceHealthLimit::SyntaxOnly,
                SourceHealthLimit::NoTypeInfo,
                SourceHealthLimit::NoTraitSolving,
                SourceHealthLimit::NoBorrowCheck,
            ],
            generated,
            sidecar,
            input,
        },
        summary,
        function_clone_groups,
        skipped_files,
        files,
    })
}
