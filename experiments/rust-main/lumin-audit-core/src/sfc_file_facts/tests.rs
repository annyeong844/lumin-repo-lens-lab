use anyhow::{bail, Context, Result};

use super::{build_sfc_file_facts_response, SfcFileFactsRequest};

fn build(request: serde_json::Value) -> Result<serde_json::Value> {
    let request = serde_json::from_value::<SfcFileFactsRequest>(request)?;
    Ok(serde_json::to_value(build_sfc_file_facts_response(
        request,
    )?)?)
}

#[test]
fn extracts_checked_vue_svelte_and_astro_file_facts() -> Result<()> {
    let vue_source = [
        "<template>",
        "  <Card />",
        "  <user-list />",
        "  <component :is=\"DynamicCard\" />",
        "  < UI.Card.Detail />",
        "  <!-- <CommentedCard /> -->",
        "</template>",
        "<script setup lang=\"ts\">",
        "import DefaultCard, { UsedByVue as Card, type Props } from '../src/card';",
        "import UserList from '../src/user-list';",
        "import DynamicCard from '../src/dynamic-card';",
        "import * as UI from '../src/ui';",
        "import 'external-sfc-package';",
        "</script>",
        "<script src=\"./external.ts\"></script>",
        "<style>",
        ".hero { background-image: url(\"./logo.svg\"); }",
        ".escaped { background: url(./my\\ icon.svg); }",
        "@import \"./theme.css\";",
        "/* url(\"./commented.svg\") */",
        "</style>",
    ]
    .join("\n");
    let svelte_source = [
        "<script context=\"module\" lang=\"ts\">",
        "import { UsedBySvelte } from '../src/svelte-use';",
        "</script>",
        "<script src=\"../src/svelte-src.ts\"></script>",
        "<UsedBySvelte />",
        "<svelte:component this={ UsedBySvelte } />",
        "<style>.icon { background: url('../assets/icon.svg'); }</style>",
    ]
    .join("\n");
    let astro_source = [
        "---",
        "import { UsedByAstro } from '../src/astro-use';",
        "---",
        "<UsedByAstro client:load />",
        "<style>@import url(\"./astro.css\");</style>",
    ]
    .join("\n");
    let response = build(serde_json::json!({
        "schemaVersion": "lumin-sfc-file-facts-request.v1",
        "files": [
            {
                "filePath": "components/App.vue",
                "source": vue_source
            },
            {
                "filePath": "components/Page.svelte",
                "source": svelte_source
            },
            {
                "filePath": "pages/Home.astro",
                "source": astro_source
            }
        ]
    }))?;

    assert_eq!(
        response["schemaVersion"],
        "lumin-sfc-file-facts-response.v1"
    );
    let files = response["files"].as_array().context("files array")?;
    let vue = &files[0];
    assert!(vue["scriptImportConsumers"]
        .as_array()
        .context("Vue imports")?
        .iter()
        .any(|row| row["name"] == "UsedByVue" && row["localName"] == "Card"));
    assert!(vue["scriptImportConsumers"]
        .as_array()
        .context("Vue imports")?
        .iter()
        .any(|row| row["name"] == "Props" && row["typeOnly"] == true));
    assert_eq!(vue["scriptSources"][0]["fromSpec"], "./external.ts");
    assert_eq!(
        vue["styleAssetReferences"].as_array().map(Vec::len),
        Some(3)
    );
    assert!(vue["templateComponentRefs"]
        .as_array()
        .context("Vue template refs")?
        .iter()
        .any(|row| row["tagName"] == "component"
            && row["reason"] == "sfc-template-dynamic-component"));
    assert!(vue["templateComponentRefs"]
        .as_array()
        .context("Vue template refs")?
        .iter()
        .any(|row| row["tagName"] == "UI.Card.Detail"
            && row["memberName"] == "Card"
            && row["reason"] == "sfc-template-namespace-component"));

    let svelte = &files[1];
    assert_eq!(svelte["scriptSources"][0]["line"], 4);
    assert!(svelte["templateComponentRefs"]
        .as_array()
        .context("Svelte template refs")?
        .iter()
        .any(|row| row["tagName"] == "UsedBySvelte" && row["line"] == 5));
    assert!(svelte["templateComponentRefs"]
        .as_array()
        .context("Svelte template refs")?
        .iter()
        .any(|row| row["tagName"] == "svelte:component" && row["line"] == 6));

    let astro = &files[2];
    assert_eq!(astro["scriptImportConsumers"][0]["name"], "UsedByAstro");
    assert_eq!(astro["templateComponentRefs"][0]["tagName"], "UsedByAstro");
    assert_eq!(astro["styleAssetReferences"][0]["fromSpec"], "./astro.css");
    Ok(())
}

#[test]
fn supports_vue_options_component_bindings() -> Result<()> {
    let source = [
        "<template><LocalCard /></template>",
        "<script lang=\"ts\">",
        "import LocalCard from './LocalCard.vue';",
        "export default { components: { LocalCard } };",
        "</script>",
    ]
    .join("\n");
    let response = build(serde_json::json!({
        "schemaVersion": "lumin-sfc-file-facts-request.v1",
        "files": [{
            "filePath": "src/App.vue",
            "source": source
        }]
    }))?;
    assert_eq!(
        response["files"][0]["templateComponentRefs"][0]["bindingSource"],
        "./LocalCard.vue"
    );
    Ok(())
}

#[test]
fn extracts_checked_per_file_framework_conventions() -> Result<()> {
    let vue_macro = [
        "<template><!-- defineOptions({ components: { CommentOnly } }) --></template>",
        "<script setup lang=\"ts\">",
        "import MacroCard from './MacroCard.vue';",
        "import MacroAlias from './macro-alias';",
        "const dynamicName = 'DynamicCard';",
        "defineOptions({ components: { MacroCard, 'macro-alias': MacroAlias, Missing, [dynamicName]: MacroCard } });",
        "</script>",
    ]
    .join("\n");
    let vue_options = [
        "<template />",
        "<script lang=\"ts\">",
        "import OptionsCard from './OptionsCard.vue';",
        "import OptionsAlias from './options-alias';",
        "const dynamicName = 'DynamicCard';",
        "export default { components: { OptionsCard, 'options-alias': OptionsAlias, Missing, [dynamicName]: OptionsCard } };",
        "</script>",
    ]
    .join("\n");
    let astro = [
        "---",
        "import { UsedByAstro } from './astro-use';",
        "---",
        "<UsedByAstro client:load />",
        "<MissingAstro client:load />",
        "<div client:load />",
        "<!-- <UsedByAstro client:visible /> -->",
    ]
    .join("\n");
    let svelte = [
        "<script context=\"module\" lang=\"ts\">",
        "import { enhance } from './actions';",
        "</script>",
        "<script lang=\"ts\">",
        "import { writable } from 'svelte/store';",
        "import { importedCount } from './stores';",
        "function localAction(node) { return { destroy() {} }; }",
        "const localConstAction = (node) => ({ destroy() {} });",
        "const localCount = writable(0);",
        "const notAction = 1;",
        "const notStore = 1;",
        "$: doubled = $localCount * 2;",
        "function shadowed($importedCount) { return $importedCount; }",
        "</script>",
        "<form use:enhance></form>",
        "<div use:localAction></div>",
        "<section use:localConstAction></section>",
        "<button use:notAction></button>",
        "<p>{$importedCount}</p>",
        "<p>{$localCount}</p>",
        "<p>$plainText</p>",
        "<p>{$missingStore}</p>",
        "<p>{$notStore}</p>",
        "<!-- <div use:commentAction></div><p>{$commentStore}</p> -->",
    ]
    .join("\n");
    let response = build(serde_json::json!({
        "schemaVersion": "lumin-sfc-file-facts-request.v1",
        "files": [
            {"filePath": "pages/Macro.vue", "source": vue_macro},
            {"filePath": "pages/Options.vue", "source": vue_options},
            {"filePath": "pages/Home.astro", "source": astro},
            {"filePath": "components/Page.svelte", "source": svelte}
        ]
    }))?;

    let files = response["files"].as_array().context("files array")?;
    let macro_rows = files[0]["frameworkConventionComponents"]
        .as_array()
        .context("Vue macro rows")?;
    assert_eq!(macro_rows.len(), 2);
    assert!(macro_rows.iter().any(|row| {
        row["conventionKind"] == "macro-registration"
            && row["componentName"] == "MacroCard"
            && row["bindingSource"] == "./MacroCard.vue"
            && row["status"] == "muted"
            && row["eligibleForFanIn"] == false
            && row["eligibleForSafeFix"] == false
    }));
    assert!(macro_rows.iter().any(|row| {
        row["componentName"] == "macro-alias"
            && row["normalizedTagNames"] == serde_json::json!(["macro-alias", "MacroAlias"])
    }));

    let options_rows = files[1]["frameworkConventionComponents"]
        .as_array()
        .context("Vue options rows")?;
    assert_eq!(options_rows.len(), 2);
    assert!(options_rows.iter().any(|row| {
        row["conventionKind"] == "options-registration"
            && row["optionName"] == "components"
            && row["componentName"] == "OptionsCard"
    }));

    let astro_rows = files[2]["frameworkConventionComponents"]
        .as_array()
        .context("Astro rows")?;
    assert_eq!(astro_rows.len(), 1);
    assert_eq!(astro_rows[0]["directiveName"], "client:load");
    assert_eq!(astro_rows[0]["bindingName"], "UsedByAstro");

    let svelte_rows = files[3]["frameworkConventionComponents"]
        .as_array()
        .context("Svelte rows")?;
    assert_eq!(
        svelte_rows
            .iter()
            .filter(|row| row["conventionKind"] == "action-directive")
            .count(),
        3
    );
    let store_names = svelte_rows
        .iter()
        .filter(|row| row["conventionKind"] == "store-auto-subscription")
        .filter_map(|row| row["storeName"].as_str())
        .collect::<Vec<_>>();
    assert!(store_names.contains(&"importedCount"));
    assert!(store_names.contains(&"localCount"));
    assert!(!store_names.iter().any(|name| matches!(
        *name,
        "missingStore" | "notStore" | "commentStore" | "plainText"
    )));
    assert_eq!(
        svelte_rows
            .iter()
            .filter(|row| row["storeName"] == "importedCount")
            .count(),
        1,
        "the shadowed script reference must not duplicate the template evidence"
    );
    Ok(())
}

#[test]
fn rejects_duplicate_files_and_script_parse_failures() -> Result<()> {
    let duplicate = serde_json::from_value::<SfcFileFactsRequest>(serde_json::json!({
        "schemaVersion": "lumin-sfc-file-facts-request.v1",
        "files": [
            {"filePath": "src/App.vue", "source": "<template />"},
            {"filePath": "src/App.vue", "source": "<template />"}
        ]
    }))?;
    let duplicate_error = match build_sfc_file_facts_response(duplicate) {
        Ok(_) => bail!("duplicate request unexpectedly succeeded"),
        Err(error) => error,
    };
    assert!(duplicate_error
        .to_string()
        .contains("duplicate files[].filePath"));

    let malformed = serde_json::from_value::<SfcFileFactsRequest>(serde_json::json!({
        "schemaVersion": "lumin-sfc-file-facts-request.v1",
        "files": [{"filePath": "src/App.vue", "source": "<script>const =</script>"}]
    }))?;
    let malformed_error = match build_sfc_file_facts_response(malformed) {
        Ok(_) => bail!("malformed SFC unexpectedly succeeded"),
        Err(error) => error,
    };
    assert!(malformed_error
        .to_string()
        .contains("failed to parse vue block"));
    Ok(())
}
