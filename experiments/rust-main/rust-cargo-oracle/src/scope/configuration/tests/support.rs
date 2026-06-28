use crate::environment::CompilationEnvironment;
use crate::protocol::RustcCommandSource;
use crate::toolchain::Toolchain;

pub(super) fn fallback_toolchain() -> Toolchain {
    Toolchain {
        cargo_version: None,
        rustc_version_verbose: None,
        rustc_bin: "rustc".to_string(),
        rustc_source: RustcCommandSource::DefaultRustc,
        host: Some("host-target".to_string()),
    }
}

pub(super) fn empty_environment() -> CompilationEnvironment {
    CompilationEnvironment::from_vars(Vec::<(String, String)>::new())
}
