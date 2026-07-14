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
