use toml::Value as TomlValue;

pub(super) fn manifest_package_name(
    manifest_key: &str,
    value: &TomlValue,
    workspace_dependencies: Option<&toml::map::Map<String, TomlValue>>,
) -> Option<String> {
    if let Some(package) = value
        .as_table()
        .and_then(|table| table.get("package"))
        .and_then(TomlValue::as_str)
    {
        return Some(package.to_string());
    }
    if value
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(TomlValue::as_bool)
        == Some(true)
    {
        return workspace_dependencies
            .and_then(|dependencies| dependencies.get(manifest_key))
            .and_then(|workspace_value| {
                workspace_value
                    .as_table()
                    .and_then(|table| table.get("package"))
                    .and_then(TomlValue::as_str)
            })
            .map(str::to_string);
    }
    None
}

pub(super) fn manifest_dependency_value(value: &TomlValue) -> String {
    match value {
        TomlValue::String(version) => version.clone(),
        TomlValue::Table(table) => {
            if table.get("workspace").and_then(TomlValue::as_bool) == Some(true) {
                "workspace = true".to_string()
            } else if let Some(version) = table.get("version").and_then(TomlValue::as_str) {
                version.to_string()
            } else if let Some(path) = table.get("path").and_then(TomlValue::as_str) {
                format!("path = {path}")
            } else {
                "inline table".to_string()
            }
        }
        _ => "nonstandard value".to_string(),
    }
}
