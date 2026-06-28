#![allow(dead_code)]

#[path = "registry/array.rs"]
mod array;
#[path = "registry/load.rs"]
mod load;
#[path = "registry/serialized.rs"]
mod serialized;
#[path = "registry/string_set.rs"]
mod string_set;

use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

pub fn cargo_check_oracle() -> Result<Value> {
    load::cargo_check_oracle()
}

pub fn serialized_string<T: serde::Serialize>(value: T) -> Result<String> {
    serialized::serialized_string(value)
}

pub fn strings(value: &Value) -> BTreeSet<String> {
    string_set::strings(value)
}

pub fn array_contains_string(value: &Value, expected: &str) -> bool {
    array::array_contains_string(value, expected)
}
