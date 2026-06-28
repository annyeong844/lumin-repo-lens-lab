use anyhow::Result;
use serde_json::Value;

mod broad_scope;
mod cfg_opacity;
mod coverage;
mod derive_macro;
mod files;
mod mode;
mod muted_syntax;
mod plan;
mod selection;
mod style_signal;

pub fn assert_targeted_package_scope(artifact: &Value) -> Result<()> {
    plan::assert_targeted_plan(artifact)?;
    files::assert_targeted_files(artifact)?;
    Ok(())
}

pub fn assert_cfg_opacity_runs_oracle(artifact: &Value) -> Result<()> {
    cfg_opacity::assert_runs_oracle(artifact)
}

pub fn assert_muted_syntax_skip(artifact: &Value) -> Result<()> {
    muted_syntax::assert_skip(artifact)
}

pub fn assert_broad_scope_uncapped_run(artifact: &Value) -> Result<()> {
    broad_scope::assert_uncapped_run(artifact)
}

pub fn assert_style_signal_skip(artifact: &Value) -> Result<()> {
    style_signal::assert_skip(artifact)
}

pub fn assert_review_derive_macro_run(artifact: &Value) -> Result<()> {
    derive_macro::assert_review_derive_runs_oracle(artifact)
}
