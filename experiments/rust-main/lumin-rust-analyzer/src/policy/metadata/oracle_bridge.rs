use serde::Serialize;

pub(super) fn oracle_bridge_policy() -> OracleBridgePolicyMetadata {
    OracleBridgePolicyMetadata {
        js_ts_precedent: OracleBridgePrecedent {
            parser: "_lib/parse-oxc.mjs",
            oracle: "_lib/tsconfig-paths.mjs",
            provenance: "_lib/finding-provenance.mjs",
        },
        rust_parser_lane: RustParserLane::RaApSyntaxViaRustSourceHealth,
        rust_oracle_lane: RustOracleLane::CargoRustcViaRustCargoOracle,
        file_provenance: true,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgePolicyMetadata {
    js_ts_precedent: OracleBridgePrecedent,
    rust_parser_lane: RustParserLane,
    rust_oracle_lane: RustOracleLane,
    file_provenance: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum RustParserLane {
    #[serde(rename = "ra_ap_syntax via rust-source-health")]
    RaApSyntaxViaRustSourceHealth,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum RustOracleLane {
    #[serde(rename = "Cargo/rustc via rust-cargo-oracle")]
    CargoRustcViaRustCargoOracle,
}

#[derive(Debug, Serialize)]
struct OracleBridgePrecedent {
    parser: &'static str,
    oracle: &'static str,
    provenance: &'static str,
}
