use anyhow::Result;
use lumin_rust_common::usage_error;

pub(super) fn validate_non_empty_strings(values: &[String], path: &str) -> Result<()> {
    for (index, value) in values.iter().enumerate() {
        require_non_empty(value, &format!("{path}[{index}]"))?;
    }
    Ok(())
}

pub(super) fn validate_optional_string(value: Option<&str>, path: &str) -> Result<()> {
    if value == Some("") {
        return Err(usage_error(format!(
            "{path} must be a non-empty string when present"
        )));
    }
    Ok(())
}

pub(super) fn require_non_empty(value: &str, path: &str) -> Result<()> {
    if value.is_empty() {
        return Err(usage_error(format!("{path} must be a non-empty string")));
    }
    Ok(())
}

pub(super) fn valid_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn is_unsafe_repo_relative_path(value: &str) -> bool {
    if value.is_empty() || value.contains('\\') || value.starts_with('/') {
        return true;
    }
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return true;
    }
    value.split('/').any(|part| part.is_empty() || part == "..")
}
