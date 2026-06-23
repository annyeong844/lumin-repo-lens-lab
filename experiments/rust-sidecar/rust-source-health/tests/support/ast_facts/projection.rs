use serde_json::Value;

use crate::artifact::file_health;

pub fn assert_core_ast_fact_projection(artifact: &Value, path: &str) {
    let ast = &file_health(artifact, path)["ast"];
    assert_eq!(ast["definitions"][0]["kind"], "function");
    assert_eq!(ast["definitions"][0]["name"], "build");
    assert_eq!(ast["definitions"][0]["visibility"], "public");
    assert_eq!(ast["definitions"][1]["kind"], "struct");
    assert_eq!(ast["definitions"][1]["visibility"], "crate");
    assert_eq!(ast["shapeHashes"][0]["kind"], "shape-hash");
    assert_eq!(ast["shapeHashes"][0]["name"], "Maybe");
    assert_eq!(ast["shapeHashes"][0]["visibility"], "crate");
    assert_eq!(ast["shapeHashes"][0]["shapeKind"], "record-struct");
    assert_eq!(
        ast["shapeHashes"][0]["normalizedVersion"],
        "rust-shape-hash.normalized.v1"
    );
    assert_eq!(ast["shapeHashes"][0]["confidence"], "high");
    assert!(ast["shapeHashes"][0]["hash"]
        .as_str()
        .is_some_and(|hash| hash.starts_with("sha256:")));
    assert_eq!(ast["shapeHashes"][0]["fields"][0]["name"], "id");
    assert_eq!(ast["shapeHashes"][0]["fields"][0]["type"], "usize");
    assert_eq!(ast["shapeHashes"][0]["fields"][0]["visibility"], "public");
    assert_eq!(ast["shapeHashes"][0]["fields"][1]["name"], "label");
    assert_eq!(ast["shapeHashes"][0]["fields"][1]["type"], "usize");
    assert_eq!(ast["shapeHashes"][0]["fields"][1]["visibility"], "private");
    assert_eq!(ast["impls"][0]["target"], "Maybe");
    assert!(ast["impls"][0].get("trait").is_none());
    assert_eq!(ast["impls"][0]["methods"][0]["name"], "normalize");
    assert_eq!(ast["impls"][0]["methods"][0]["visibility"], "public");
    assert_eq!(ast["impls"][0]["methods"][0]["hasReceiver"], true);
    assert_eq!(ast["impls"][0]["methods"][1]["name"], "make");
    assert_eq!(ast["impls"][0]["methods"][1]["visibility"], "crate");
    assert_eq!(ast["impls"][0]["methods"][1]["hasReceiver"], false);
    assert_eq!(ast["useTrees"][0]["visibility"], "public");
    assert!(ast["useTrees"][0].get("name").is_none());
    assert_eq!(ast["useTrees"][1]["path"], "model::Thing");
    assert_eq!(ast["useTrees"][1]["name"], "Thing");
    assert_eq!(ast["useTrees"][1]["alias"], "Alias");
    assert_eq!(ast["useTrees"][2]["glob"], true);
    assert!(ast["useTrees"][2].get("alias").is_none());
    assert_eq!(ast["pathRefs"][0]["path"], "crate::factory::make");
    assert_eq!(ast["pathRefs"][0]["name"], "make");
    assert_eq!(ast["methodCallCounts"]["normalize"], 1);
    assert!(ast["methodCalls"].as_array().is_some_and(Vec::is_empty));
    assert_eq!(ast["macroCalls"][0]["path"], "custom_macro");
}
