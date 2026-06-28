use toml::Value as TomlValue;

const DEPENDENCY_SECTIONS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

pub(super) struct DependencyTable<'a> {
    pub(super) section: String,
    pub(super) entries: &'a toml::map::Map<String, TomlValue>,
}

pub(super) fn dependency_tables(value: &TomlValue) -> Vec<DependencyTable<'_>> {
    let mut tables = Vec::new();
    for section in DEPENDENCY_SECTIONS {
        if let Some(entries) = value.get(*section).and_then(TomlValue::as_table) {
            tables.push(DependencyTable {
                section: (*section).to_string(),
                entries,
            });
        }
    }
    if let Some(targets) = value.get("target").and_then(TomlValue::as_table) {
        for (target, target_value) in targets {
            for section in DEPENDENCY_SECTIONS {
                if let Some(entries) = target_value.get(*section).and_then(TomlValue::as_table) {
                    tables.push(DependencyTable {
                        section: format!("target.{target}.{section}"),
                        entries,
                    });
                }
            }
        }
    }
    tables
}
