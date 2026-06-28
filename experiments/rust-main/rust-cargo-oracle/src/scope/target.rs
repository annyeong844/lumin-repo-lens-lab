use std::collections::{BTreeMap, BTreeSet};

use crate::cargo_json::CargoJsonMessages;
use crate::metadata::{CargoMetadata, CargoPackage};
use crate::protocol::{OracleScopeTarget, OracleScopeTargetSource};

pub(super) fn target_entries(
    metadata: Option<&CargoMetadata>,
    messages: CargoJsonMessages<'_>,
    selected: &[CargoPackage],
) -> Vec<OracleScopeTarget> {
    let mut entries = BTreeMap::<String, OracleScopeTarget>::new();
    let selected_ids = selected
        .iter()
        .map(|pkg| pkg.id.as_str())
        .collect::<BTreeSet<_>>();
    for message in messages.compiler_target_events() {
        let package_id = message.package_id();
        if !selected_ids.is_empty()
            && package_id.is_some_and(|package_id| !selected_ids.contains(package_id))
        {
            continue;
        }
        if let Some(target) = message.target() {
            let name = target.name();
            let package_id = package_id.unwrap_or("");
            entries.insert(
                format!("{package_id}:{name}"),
                OracleScopeTarget {
                    package_id: package_id.to_string(),
                    package_name: package_name_for_id(metadata, package_id),
                    target_name: name.to_string(),
                    target_kinds: target.kinds(),
                    source: OracleScopeTargetSource::CargoJsonMessage,
                },
            );
        }
    }
    if entries.is_empty() {
        for pkg in selected {
            for target in pkg
                .targets
                .iter()
                .filter(|target| target.is_default_checked())
            {
                entries.insert(
                    format!("{}:{}", pkg.id, target.name),
                    OracleScopeTarget {
                        package_id: pkg.id.clone(),
                        package_name: Some(pkg.name.clone()),
                        target_name: target.name.clone(),
                        target_kinds: target.kind.clone(),
                        source: OracleScopeTargetSource::CargoMetadataDefaultSelection,
                    },
                );
            }
        }
    }
    entries.into_values().collect()
}

fn package_name_for_id(metadata: Option<&CargoMetadata>, package_id: &str) -> Option<String> {
    metadata
        .and_then(|metadata| metadata.packages.iter().find(|pkg| pkg.id == package_id))
        .map(|pkg| pkg.name.clone())
}
