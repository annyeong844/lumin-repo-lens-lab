use std::path::PathBuf;

use anyhow::Result;
use lumin_rust_common::{
    atomic_write_json_pretty, parse_min_usize, parse_nonzero_usize, take_path, take_string,
    CliAction,
};
use serde::Serialize;
use std::collections::BTreeMap;

use crate::protocol::{
    AstFacts, AstOpaqueMuteReason, AstOpaqueSurface, AstOpaqueSurfaceVisibility, Facts, FileHealth,
    HealthResponse, ParseStatus, PathMeta, ResponseMeta, Signal, SkippedFile, Summary,
    DEFAULT_WORKER_STACK_BYTES,
};
use crate::usage_error;

use super::request::{analyze_root, RustSourceHealthOptions};

const REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT: usize = 10;

pub fn run_cli(args: Vec<String>) -> Result<()> {
    let options = match WrapperOptions::parse(args)? {
        CliAction::Run(options) => options,
        CliAction::Help => return Ok(()),
    };
    let output = options.output.clone();
    let response = analyze_root(RustSourceHealthOptions {
        root: options.root,
        source_commit: options.source_commit,
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    })?;
    match options.artifact_profile {
        ArtifactProfile::Compact => {
            let artifact = CompactHealthResponse::from_response(&response);
            atomic_write_json_pretty(&output, &artifact)?;
        }
        ArtifactProfile::Full => {
            atomic_write_json_pretty(&output, &response)?;
        }
    }
    println!("[rust-source-health] wrote {}", output.display());
    println!(
        "[rust-source-health] profile={} files={} skipped={} signals={}",
        options.artifact_profile.as_str(),
        response.summary.files,
        response.summary.skipped_files,
        response.summary.signals
    );
    Ok(())
}

#[derive(Debug)]
struct WrapperOptions {
    root: PathBuf,
    output: PathBuf,
    source_commit: String,
    thread_count: Option<usize>,
    worker_stack_bytes: usize,
    artifact_profile: ArtifactProfile,
}

impl WrapperOptions {
    fn parse(args: Vec<String>) -> Result<CliAction<Self>> {
        let mut root = None;
        let mut output = None;
        let mut source_commit = None;
        let mut thread_count = None;
        let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;
        let mut artifact_profile = ArtifactProfile::Compact;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--root" => root = Some(take_path(&mut iter, "--root")?),
                "--output" => output = Some(take_path(&mut iter, "--output")?),
                "--source-commit" | "--sidecar-source-commit" => {
                    source_commit = Some(take_string(&mut iter, "--source-commit")?)
                }
                "--threads" => {
                    let raw = take_string(&mut iter, "--threads")?;
                    thread_count = Some(parse_nonzero_usize(&raw, "--threads")?);
                }
                "--worker-stack-bytes" => {
                    let raw = take_string(&mut iter, "--worker-stack-bytes")?;
                    worker_stack_bytes =
                        parse_min_usize(&raw, "--worker-stack-bytes", DEFAULT_WORKER_STACK_BYTES)?;
                }
                "--artifact-profile" => {
                    let raw = take_string(&mut iter, "--artifact-profile")?;
                    artifact_profile = ArtifactProfile::parse(&raw)?;
                }
                "--help" | "-h" => {
                    print_usage();
                    return Ok(CliAction::Help);
                }
                unknown => return Err(usage_error(format!("unknown argument: {unknown}"))),
            }
        }

        Ok(CliAction::Run(Self {
            root: root.ok_or_else(|| usage_error("--root is required"))?,
            output: output.ok_or_else(|| usage_error("--output is required"))?,
            source_commit: source_commit
                .ok_or_else(|| usage_error("--source-commit is required"))?,
            thread_count,
            worker_stack_bytes,
            artifact_profile,
        }))
    }
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-source-health --root <path> --output <path> --source-commit <sha> [--artifact-profile compact|full] [--threads <n>] [--worker-stack-bytes <bytes>]"
    );
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ArtifactProfile {
    Compact,
    Full,
}

impl ArtifactProfile {
    fn parse(raw: &str) -> Result<Self> {
        match raw {
            "compact" => Ok(Self::Compact),
            "full" => Ok(Self::Full),
            _ => Err(usage_error(format!(
                "invalid --artifact-profile: expected compact or full, got {raw}"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Full => "full",
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactHealthResponse<'a> {
    schema_version: u32,
    artifact_profile: &'static str,
    meta: &'a ResponseMeta,
    summary: &'a Summary,
    skipped_files: &'a [SkippedFile],
    files: BTreeMap<&'a str, CompactFileHealth<'a>>,
}

impl<'a> CompactHealthResponse<'a> {
    fn from_response(response: &'a HealthResponse) -> Self {
        let files = response
            .files
            .iter()
            .map(|(path, file)| (path.as_str(), CompactFileHealth::from_file(file)))
            .collect();

        Self {
            schema_version: response.schema_version,
            artifact_profile: "compact",
            meta: &response.meta,
            summary: &response.summary,
            skipped_files: &response.skipped_files,
            files,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactFileHealth<'a> {
    sha256: &'a str,
    facts: &'a Facts,
    ast_summary: CompactAstSummary<'a>,
    signals: &'a [Signal],
    parse: &'a ParseStatus,
    path: &'a PathMeta,
}

impl<'a> CompactFileHealth<'a> {
    fn from_file(file: &'a FileHealth) -> Self {
        Self {
            sha256: &file.sha256,
            facts: &file.facts,
            ast_summary: CompactAstSummary::from_ast(&file.ast),
            signals: &file.signals,
            parse: &file.parse,
            path: &file.path,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompactAstSummary<'a> {
    definitions: usize,
    shape_hashes: usize,
    impl_blocks: usize,
    impl_methods: usize,
    use_trees: usize,
    path_refs: usize,
    method_call_sites: usize,
    method_calls: usize,
    macro_calls: usize,
    cfg_gates: usize,
    opaque_surfaces: usize,
    review_opaque_surfaces: usize,
    muted_opaque_surfaces: usize,
    muted_opaque_surfaces_by_reason: BTreeMap<AstOpaqueMuteReason, usize>,
    review_opaque_surface_sample_limit: usize,
    review_opaque_surface_examples: Vec<&'a AstOpaqueSurface>,
}

impl<'a> CompactAstSummary<'a> {
    fn from_ast(ast: &'a AstFacts) -> Self {
        let mut review_opaque_surfaces = 0;
        let mut muted_opaque_surfaces = 0;
        let mut muted_opaque_surfaces_by_reason = BTreeMap::new();
        let mut review_opaque_surface_examples = Vec::new();

        for surface in &ast.opaque_surfaces {
            match surface.visibility {
                AstOpaqueSurfaceVisibility::Review => {
                    review_opaque_surfaces += 1;
                    if review_opaque_surface_examples.len() < REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT {
                        review_opaque_surface_examples.push(surface);
                    }
                }
                AstOpaqueSurfaceVisibility::Muted { mute_reason } => {
                    muted_opaque_surfaces += 1;
                    *muted_opaque_surfaces_by_reason
                        .entry(mute_reason)
                        .or_insert(0) += 1;
                }
            }
        }

        Self {
            definitions: ast.definitions.len(),
            shape_hashes: ast.shape_hashes.len(),
            impl_blocks: ast.impls.len(),
            impl_methods: ast
                .impls
                .iter()
                .map(|impl_block| impl_block.methods.len())
                .sum(),
            use_trees: ast.use_trees.len(),
            path_refs: ast.path_refs.len(),
            method_call_sites: ast.method_call_counts.values().sum(),
            method_calls: ast.method_calls.len(),
            macro_calls: ast.macro_calls.len(),
            cfg_gates: ast.cfg_gates.len(),
            opaque_surfaces: ast.opaque_surfaces.len(),
            review_opaque_surfaces,
            muted_opaque_surfaces,
            muted_opaque_surfaces_by_reason,
            review_opaque_surface_sample_limit: REVIEW_OPAQUE_SURFACE_EXAMPLE_LIMIT,
            review_opaque_surface_examples,
        }
    }
}
