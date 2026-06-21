use crate::support::risk::assert_risk;

pub fn assert_nonliteral_dynamic_import_is_reported() {
    assert_risk(
        "fixture.ts",
        "export function load(name) { return import(name); }\n",
        &["non-literal-dynamic-import"],
    );
}

pub fn assert_template_dynamic_import_is_reported() {
    assert_risk(
        "fixture.ts",
        "export function load(name) { return import(`./${name}.ts`); }\n",
        &["template-dynamic-import"],
    );
}

pub fn assert_multiline_template_dynamic_import_is_reported() {
    let source = [
        "async function load(pathToFileURL, dir) {",
        "  const mod = await import(",
        "    `${pathToFileURL(dir).href}/_lib/alias-map.mjs?v=case`",
        "  );",
        "  return mod;",
        "}",
    ]
    .join("\n");

    assert_risk("fixture.mjs", &source, &["template-dynamic-import"]);
}
