use serde_json::Value;

use crate::artifact::file_health;

pub fn assert_core_ast_fact_projection(artifact: &Value, path: &str) {
    let ast = &file_health(artifact, path)["ast"];
    assert_eq!(ast["definitions"][0]["kind"], "function");
    assert_eq!(ast["definitions"][0]["name"], "build");
    assert_eq!(ast["definitions"][0]["visibility"], "public");
    assert_eq!(ast["definitions"][1]["kind"], "struct");
    assert_eq!(ast["definitions"][1]["visibility"], "crate");
    assert_eq!(ast["useTrees"][0]["visibility"], "public");
    assert_eq!(ast["useTrees"][2]["glob"], true);
    assert_eq!(ast["pathRefs"][0]["path"], "crate::factory::make");
    assert_eq!(ast["pathRefs"][0]["name"], "make");
    assert_eq!(ast["methodCallCounts"]["normalize"], 1);
    assert!(ast["methodCalls"].as_array().is_some_and(Vec::is_empty));
    assert_eq!(ast["macroCalls"][0]["path"], "custom_macro");
}
