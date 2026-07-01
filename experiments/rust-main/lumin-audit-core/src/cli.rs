use anyhow::{bail, Result};

pub fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("artifact-registry") | Some("rust-analysis-summary") => {
            bail!("lumin-audit-core command is not implemented yet")
        }
        _ => bail!(
            "usage: lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran]\n       lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>"
        ),
    }
}
