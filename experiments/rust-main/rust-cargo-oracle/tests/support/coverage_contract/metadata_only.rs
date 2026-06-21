use anyhow::Result;
use lumin_rust_cargo_oracle::CargoCheckMode;

use crate::support::{coverage, findings::empty, real_cargo_env::RealCargoEnv};

pub fn assert_metadata_only_without_cargo_findings() -> Result<()> {
    let env = RealCargoEnv::type_error()?;
    let artifact = env.run_with_mode(CargoCheckMode::MetadataOnly)?;

    empty::assert_no_findings(&artifact)?;
    assert_eq!(artifact["meta"]["input"]["cargoCheckMode"], "metadata-only");

    let stream = coverage::coverage(&artifact, "cov.cargo-check.cargo-event-stream")?;
    assert_eq!(stream["status"], "unavailable");
    assert_eq!(stream["streamParseStatus"], "not-run");
    assert!(stream["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("metadata-only"));

    let absence = coverage::coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(absence["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("metadata-only"));
    assert_eq!(
        artifact["summary"]["semanticClean"]["status"],
        "unavailable"
    );
    Ok(())
}
