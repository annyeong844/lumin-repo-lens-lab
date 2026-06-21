use serde_json::Value;

pub(super) fn assert_syntax_summary(artifact: &Value) {
    assert_eq!(artifact["summary"]["syntaxParseErrorFiles"], 0);
    assert_eq!(artifact["summary"]["syntaxParseErrors"], 0);
    assert_eq!(artifact["summary"]["syntaxReviewSignals"], 1);
    assert_eq!(artifact["summary"]["syntaxMutedSignals"], 2);
    assert!(
        artifact["summary"]["syntaxDefinitions"]
            .as_u64()
            .unwrap_or(0)
            >= 3
    );
    assert_eq!(artifact["summary"]["syntaxPathRefs"], 1);
    assert_eq!(artifact["summary"]["syntaxMethodCallSites"], 4);
    assert_eq!(artifact["summary"]["syntaxMethodCalls"], 3);
    assert_eq!(artifact["summary"]["syntaxMacroCalls"], 2);
    assert_eq!(artifact["summary"]["syntaxCfgGates"], 1);
    assert_eq!(artifact["summary"]["syntaxOpaqueSurfaces"], 3);
    assert_eq!(artifact["summary"]["syntaxReviewOpaqueSurfaces"], 2);
    assert_eq!(artifact["summary"]["syntaxMutedOpaqueSurfaces"], 1);
    assert_eq!(
        artifact["summary"]["syntaxReviewSignalExamples"][0]["file"],
        "src/lib.rs"
    );
    assert_eq!(
        artifact["summary"]["syntaxReviewSignalExamples"][0]["kind"],
        "unwrap-call"
    );
    assert_eq!(
        artifact["summary"]["syntaxReviewOpaqueSurfaceExamples"][0]["file"],
        "src/lib.rs"
    );
    assert_eq!(
        artifact["summary"]["syntaxReviewOpaqueSurfaceExamples"][0]["reason"],
        "cfg-condition-not-evaluated"
    );
    assert_eq!(
        artifact["summary"]["syntaxReviewOpaqueSurfaceExamples"][1]["detail"],
        "custom_macro"
    );
}
