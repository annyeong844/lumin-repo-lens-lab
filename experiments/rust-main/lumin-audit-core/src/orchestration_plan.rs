use anyhow::{bail, Result};
use serde::Serialize;

pub const ORCHESTRATION_PLAN_SCHEMA_VERSION: &str = "lumin-audit-orchestration-plan.v1";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AuditProfile {
    #[default]
    Quick,
    Full,
    Ci,
}

impl AuditProfile {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "quick" => Ok(Self::Quick),
            "full" => Ok(Self::Full),
            "ci" => Ok(Self::Ci),
            profile => bail!("unsupported audit profile: {profile}. Use quick|full|ci."),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Quick => "quick",
            Self::Full => "full",
            Self::Ci => "ci",
        }
    }

    fn includes_full_steps(self) -> bool {
        !matches!(self, Self::Quick)
    }
}

impl Serialize for AuditProfile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OrchestrationPlanOptions {
    pub profile: AuditProfile,
    pub sarif: bool,
    pub pre_write: bool,
    pub post_write: bool,
    pub canon_draft: bool,
    pub check_canon: bool,
    pub rust_analyzer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationPlan {
    pub schema_version: &'static str,
    pub plan_owner: &'static str,
    pub execution_owner: &'static str,
    pub profile: AuditProfile,
    pub emit_sarif: bool,
    pub base_pipeline: BasePipelinePlan,
    pub lifecycle: LifecyclePlan,
    pub summary: OrchestrationPlanSummary,
    pub steps: Vec<OrchestrationStep>,
    pub skipped: Vec<PlannedSkip>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationPlanSummary {
    pub planned_step_count: usize,
    pub required_step_count: usize,
    pub optional_step_count: usize,
    pub skipped_step_count: usize,
    pub rust_owned_step_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BasePipelinePlan {
    pub requested: bool,
    pub status: BasePipelineStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasePipelineStatus {
    Planned,
    Skipped,
}

impl Serialize for BasePipelineStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::Planned => "planned",
            Self::Skipped => "skipped",
        };
        serializer.serialize_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecyclePlan {
    pub pre_write: LifecycleModePlan,
    pub post_write: LifecycleModePlan,
    pub canon_draft: LifecycleModePlan,
    pub check_canon: LifecycleModePlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleModePlan {
    pub requested: bool,
    pub mode: LifecycleMode,
    pub execution_owner: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_input: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleMode {
    MutuallyExclusive,
    Orthogonal,
}

impl Serialize for LifecycleMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::MutuallyExclusive => "mutually-exclusive",
            Self::Orthogonal => "orthogonal",
        };
        serializer.serialize_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationStep {
    pub order: usize,
    pub step: &'static str,
    pub script: &'static str,
    pub phase: &'static str,
    pub required: bool,
    pub producer_owner: ProducerOwner,
    pub execution_owner: &'static str,
    pub mode: StepMode,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requires_artifacts: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precondition: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason_when_unmet: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerOwner {
    JsMjs,
    Rust,
}

impl Serialize for ProducerOwner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::JsMjs => "js-mjs",
            Self::Rust => "rust",
        };
        serializer.serialize_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    Always,
    FullOrCi,
    CiOrSarif,
    ExplicitOptIn,
    Precondition,
    FullOrCiPrecondition,
}

impl Serialize for StepMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::Always => "always",
            Self::FullOrCi => "full-or-ci",
            Self::CiOrSarif => "ci-or-sarif",
            Self::ExplicitOptIn => "explicit-opt-in",
            Self::Precondition => "precondition",
            Self::FullOrCiPrecondition => "full-or-ci-precondition",
        };
        serializer.serialize_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedSkip {
    pub step: &'static str,
    pub reason: &'static str,
}

pub fn build_orchestration_plan(options: OrchestrationPlanOptions) -> OrchestrationPlan {
    let emit_sarif = options.sarif || matches!(options.profile, AuditProfile::Ci);
    let base_pipeline = base_pipeline_plan(options, emit_sarif);
    let mut steps = Vec::new();
    let mut skipped = Vec::new();

    if matches!(base_pipeline.status, BasePipelineStatus::Skipped) {
        if let Some(reason) = base_pipeline.reason {
            skipped.push(PlannedSkip {
                step: "base-audit-profile",
                reason,
            });
        }
    } else {
        push_base_pipeline_steps(&mut steps, &mut skipped, options, emit_sarif);
    }

    let summary = OrchestrationPlanSummary {
        planned_step_count: steps.len(),
        required_step_count: steps.iter().filter(|step| step.required).count(),
        optional_step_count: steps.iter().filter(|step| !step.required).count(),
        skipped_step_count: skipped.len(),
        rust_owned_step_count: steps
            .iter()
            .filter(|step| step.producer_owner == ProducerOwner::Rust)
            .count(),
    };

    OrchestrationPlan {
        schema_version: ORCHESTRATION_PLAN_SCHEMA_VERSION,
        plan_owner: "lumin-audit-core",
        execution_owner: "lumin-audit-core",
        profile: options.profile,
        emit_sarif,
        base_pipeline,
        lifecycle: lifecycle_plan(options),
        summary,
        steps,
        skipped,
    }
}

fn base_pipeline_plan(options: OrchestrationPlanOptions, emit_sarif: bool) -> BasePipelinePlan {
    if options.pre_write && options.post_write {
        return BasePipelinePlan {
            requested: false,
            status: BasePipelineStatus::Skipped,
            reason: Some("--pre-write and --post-write are mutually exclusive"),
        };
    }

    if options.pre_write
        && !options.post_write
        && !options.canon_draft
        && !options.check_canon
        && !emit_sarif
    {
        return BasePipelinePlan {
            requested: false,
            status: BasePipelineStatus::Skipped,
            reason: Some(
                "pre-write-only mode uses intent-shaped cold-cache instead of full quick audit",
            ),
        };
    }

    BasePipelinePlan {
        requested: true,
        status: BasePipelineStatus::Planned,
        reason: None,
    }
}

fn lifecycle_plan(options: OrchestrationPlanOptions) -> LifecyclePlan {
    LifecyclePlan {
        pre_write: LifecycleModePlan {
            requested: options.pre_write,
            mode: LifecycleMode::MutuallyExclusive,
            execution_owner: "audit-repo.mjs",
            required_input: Some("--intent <file|->"),
            notes: Some(
                "engine selection remains JS-owned until Rust orchestrator execution moves",
            ),
        },
        post_write: LifecycleModePlan {
            requested: options.post_write,
            mode: LifecycleMode::MutuallyExclusive,
            execution_owner: "audit-repo.mjs",
            required_input: Some("--pre-write-advisory <file>"),
            notes: Some("post-write delta producer remains JS-owned"),
        },
        canon_draft: LifecycleModePlan {
            requested: options.canon_draft,
            mode: LifecycleMode::Orthogonal,
            execution_owner: "audit-repo.mjs",
            required_input: None,
            notes: Some("canon-draft child execution remains JS-owned"),
        },
        check_canon: LifecycleModePlan {
            requested: options.check_canon,
            mode: LifecycleMode::Orthogonal,
            execution_owner: "audit-repo.mjs",
            required_input: None,
            notes: Some("check-canon child execution remains JS-owned"),
        },
    }
}

fn push_base_pipeline_steps(
    steps: &mut Vec<OrchestrationStep>,
    skipped: &mut Vec<PlannedSkip>,
    options: OrchestrationPlanOptions,
    emit_sarif: bool,
) {
    push_step(
        steps,
        "triage-repo.mjs",
        "triage-repo.mjs",
        "triage",
        true,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );

    if options.rust_analyzer {
        push_step(
            steps,
            "lumin-rust-analyzer",
            "lumin-rust-analyzer",
            "rust-analysis",
            false,
            ProducerOwner::Rust,
            StepMode::ExplicitOptIn,
        );
    }

    push_step(
        steps,
        "build-framework-resource-surfaces.mjs",
        "build-framework-resource-surfaces.mjs",
        "framework-resource-surfaces",
        false,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );
    push_step(
        steps,
        "measure-topology.mjs",
        "measure-topology.mjs",
        "topology",
        false,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );
    push_step(
        steps,
        "measure-discipline.mjs",
        "measure-discipline.mjs",
        "discipline",
        false,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );

    if options.profile.includes_full_steps() {
        for (script, phase) in [
            ("build-call-graph.mjs", "call-graph"),
            ("check-barrel-discipline.mjs", "barrel-discipline"),
            ("build-shape-index.mjs", "shape-index"),
            ("build-function-clone-index.mjs", "function-clones"),
            ("build-block-clone-index.mjs", "block-clones"),
        ] {
            push_step(
                steps,
                script,
                script,
                phase,
                false,
                ProducerOwner::JsMjs,
                StepMode::FullOrCi,
            );
        }
    }

    push_step(
        steps,
        "build-symbol-graph.mjs",
        "build-symbol-graph.mjs",
        "symbol-graph",
        true,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );
    push_step(
        steps,
        "build-unused-deps.mjs",
        "build-unused-deps.mjs",
        "dependency-hygiene",
        false,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );
    push_precondition_step(
        steps,
        StepSpec {
            script: "build-resolver-diagnostics.mjs",
            phase: "resolver-diagnostics",
            required_artifacts: vec!["symbols.json"],
            precondition: "symbols.json exists",
            skip_reason_when_unmet:
                "symbols.json missing (symbol graph step failed or was skipped)",
            mode: StepMode::Precondition,
        },
    );
    push_precondition_step(
        steps,
        StepSpec {
            script: "build-entry-surface.mjs",
            phase: "entry-surface",
            required_artifacts: vec!["symbols.json"],
            precondition: "symbols.json exists",
            skip_reason_when_unmet:
                "symbols.json missing (symbol graph step failed or was skipped)",
            mode: StepMode::Precondition,
        },
    );
    push_precondition_step(
        steps,
        StepSpec {
            script: "build-module-reachability.mjs",
            phase: "module-reachability",
            required_artifacts: vec!["symbols.json", "entry-surface.json"],
            precondition: "symbols.json and entry-surface.json exist",
            skip_reason_when_unmet: "symbols.json or entry-surface.json missing",
            mode: StepMode::Precondition,
        },
    );
    push_step(
        steps,
        "classify-dead-exports.mjs",
        "classify-dead-exports.mjs",
        "dead-exports",
        false,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );
    push_precondition_step(
        steps,
        StepSpec {
            script: "export-action-safety.mjs",
            phase: "action-safety",
            required_artifacts: vec!["dead-classify.json"],
            precondition: "dead-classify.json exists",
            skip_reason_when_unmet:
                "dead-classify.json missing (classify step failed or was skipped)",
            mode: StepMode::Precondition,
        },
    );

    if options.profile.includes_full_steps() {
        push_precondition_step(
            steps,
            StepSpec {
                script: "merge-runtime-evidence.mjs",
                phase: "runtime-evidence",
                required_artifacts: vec![
                    "coverage/coverage-final.json",
                    ".nyc_output/coverage-final.json",
                ],
                precondition: "coverage-final.json exists in coverage/ or .nyc_output/",
                skip_reason_when_unmet: "no coverage-final.json in coverage/ or .nyc_output/",
                mode: StepMode::FullOrCiPrecondition,
            },
        );
        push_precondition_step(
            steps,
            StepSpec {
                script: "measure-staleness.mjs",
                phase: "staleness",
                required_artifacts: Vec::new(),
                precondition: "root is a git working tree",
                skip_reason_when_unmet: "not a git working tree",
                mode: StepMode::FullOrCiPrecondition,
            },
        );
    }

    push_precondition_step(
        steps,
        StepSpec {
            script: "rank-fixes.mjs",
            phase: "rank-fixes",
            required_artifacts: vec!["dead-classify.json"],
            precondition: "dead-classify.json exists",
            skip_reason_when_unmet:
                "dead-classify.json missing (classify step failed or was skipped)",
            mode: StepMode::Precondition,
        },
    );
    push_step(
        steps,
        "checklist-facts.mjs",
        "checklist-facts.mjs",
        "checklist-facts",
        false,
        ProducerOwner::JsMjs,
        StepMode::Always,
    );

    if emit_sarif {
        push_step(
            steps,
            "emit-sarif.mjs",
            "emit-sarif.mjs",
            "sarif",
            false,
            ProducerOwner::JsMjs,
            StepMode::CiOrSarif,
        );
    } else {
        skipped.push(PlannedSkip {
            step: "emit-sarif.mjs",
            reason: "not in --sarif mode",
        });
    }
}

struct StepSpec {
    script: &'static str,
    phase: &'static str,
    required_artifacts: Vec<&'static str>,
    precondition: &'static str,
    skip_reason_when_unmet: &'static str,
    mode: StepMode,
}

fn push_step(
    steps: &mut Vec<OrchestrationStep>,
    step: &'static str,
    script: &'static str,
    phase: &'static str,
    required: bool,
    producer_owner: ProducerOwner,
    mode: StepMode,
) {
    steps.push(OrchestrationStep {
        order: steps.len() + 1,
        step,
        script,
        phase,
        required,
        producer_owner,
        execution_owner: "lumin-audit-core",
        mode,
        requires_artifacts: Vec::new(),
        precondition: None,
        skip_reason_when_unmet: None,
    });
}

fn push_precondition_step(steps: &mut Vec<OrchestrationStep>, spec: StepSpec) {
    steps.push(OrchestrationStep {
        order: steps.len() + 1,
        step: spec.script,
        script: spec.script,
        phase: spec.phase,
        required: false,
        producer_owner: ProducerOwner::JsMjs,
        execution_owner: "lumin-audit-core",
        mode: spec.mode,
        requires_artifacts: spec.required_artifacts,
        precondition: Some(spec.precondition),
        skip_reason_when_unmet: Some(spec.skip_reason_when_unmet),
    });
}
