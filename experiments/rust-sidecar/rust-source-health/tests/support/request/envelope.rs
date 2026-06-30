use serde_json::{json, Value};

pub fn request(files: Vec<Value>) -> Value {
    json!({
        "schemaVersion": 1,
        "root": "C:/repo",
        "files": files,
        "pathPolicy": {
            "include": ["**/*.rs"],
            "exclude": ["**/target/**", "**/vendor/**"]
        },
        "parser": {
            "editionPolicy": "fixed",
            "edition": "2021",
            "editionSource": "m6-policy-default"
        },
        "runtime": {
            "threadCount": 2,
            "workerStackBytes": 4194304
        }
    })
}
