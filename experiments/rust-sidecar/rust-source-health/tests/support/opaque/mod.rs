#![allow(dead_code, unused_imports)]

mod review_and_muted;
pub mod summary;
pub mod surfaces;

use serde_json::Value;

pub fn assert_review_and_muted_surfaces(artifact: &Value, path: &str) {
    review_and_muted::assert_review_and_muted_surfaces(artifact, path);
}
