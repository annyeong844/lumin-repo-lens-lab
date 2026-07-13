use serde_json::{Map, Value};

use super::support::{
    array_field, insert_string, insert_value, make_result, number_field, string_array,
    string_field, SarifState,
};

pub(super) fn collect_secondary_results(
    state: &mut SarifState,
    root: &str,
    topology: Option<&Value>,
    discipline: Option<&Value>,
    barrels: Option<&Value>,
) {
    collect_topology_results(state, root, topology);
    collect_discipline_results(state, root, discipline);
    collect_barrel_results(state, root, barrels);
}

fn collect_topology_results(state: &mut SarifState, root: &str, topology: Option<&Value>) {
    let Some(topology) = topology else {
        return;
    };
    state.artifacts_used.push("topology.json");
    for scc in array_field(topology, "sccs") {
        let members = string_array(scc.get("members"));
        let mut preview = members
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(" → ");
        if members.len() > 3 {
            preview.push_str(" → …");
        }
        for member in &members {
            let mut properties = Map::new();
            insert_value(
                &mut properties,
                "sccSize",
                scc.get("size").cloned().unwrap_or_else(|| Value::from(0)),
            );
            insert_value(&mut properties, "sccMembers", Value::from(members.clone()));
            state.results.push(make_result(
                "GA002",
                "warning",
                format!(
                    "File participates in SCC of size {}. Cycle preview: {preview}",
                    scc.get("size").and_then(Value::as_i64).unwrap_or(0)
                ),
                member,
                Some(1),
                properties,
                root,
            ));
        }
    }

    for largest in array_field(topology, "largestFiles") {
        let loc = number_field(largest, "loc").unwrap_or(0);
        if loc < 1000 {
            continue;
        }
        let mut properties = Map::new();
        insert_value(&mut properties, "loc", Value::from(loc));
        state.results.push(make_result(
            "GA004",
            "note",
            format!("File has {loc} LOC (threshold: 1000). Consider splitting."),
            &string_field(largest, "file").unwrap_or_default(),
            Some(1),
            properties,
            root,
        ));
    }

    for hotspot in array_field(topology, "crossSubmoduleTop")
        .into_iter()
        .take(5)
    {
        let count = number_field(hotspot, "count").unwrap_or(0);
        if count < 20 {
            continue;
        }
        let edge = string_field(hotspot, "edge").unwrap_or_default();
        let mut properties = Map::new();
        insert_string(&mut properties, "edge", edge.clone());
        insert_value(&mut properties, "importCount", Value::from(count));
        state.results.push(make_result(
            "GA005",
            "note",
            format!("Cross-submodule hotspot: {edge} ({count} imports)."),
            ".",
            Some(1),
            properties,
            root,
        ));
    }
}

fn collect_discipline_results(state: &mut SarifState, root: &str, discipline: Option<&Value>) {
    let Some(discipline) = discipline else {
        return;
    };
    let offenders = array_field(discipline, "overallTopOffenders");
    if offenders.is_empty() {
        return;
    }
    state.artifacts_used.push("discipline.json");
    for offender in offenders {
        let file = string_field(offender, "file").unwrap_or_default();
        let Some(breakdown) = offender.get("breakdown").and_then(Value::as_object) else {
            continue;
        };
        for (pattern, count) in breakdown {
            let count = count.as_i64().unwrap_or(0);
            if count == 0 {
                continue;
            }
            let mut properties = Map::new();
            insert_string(&mut properties, "pattern", pattern.clone());
            insert_value(&mut properties, "count", Value::from(count));
            state.results.push(make_result(
                "GA003",
                "note",
                format!("Discipline: {count}× `{pattern}` in this file."),
                &file,
                Some(1),
                properties,
                root,
            ));
        }
    }
}

fn collect_barrel_results(state: &mut SarifState, root: &str, barrels: Option<&Value>) {
    let Some(barrels) = barrels else {
        return;
    };
    let Some(by_package) = barrels.get("byPackage").and_then(Value::as_object) else {
        return;
    };
    state.artifacts_used.push("barrels.json");
    for (package, info) in by_package {
        for import in array_field(info, "sampleRootImporters") {
            if import
                .get("eslintDisable")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                continue;
            }
            let mut properties = Map::new();
            insert_string(&mut properties, "package", package.clone());
            insert_value(
                &mut properties,
                "symbols",
                import
                    .get("symbols")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(Vec::new())),
            );
            insert_value(
                &mut properties,
                "reExport",
                Value::from(
                    import
                        .get("reExport")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                ),
            );
            state.results.push(make_result(
                "GA006",
                "warning",
                format!("Root-level barrel import of `{package}`. Prefer subpath export."),
                &string_field(import, "file").unwrap_or_default(),
                number_field(import, "line"),
                properties,
                root,
            ));
        }
    }
}
