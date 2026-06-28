use crate::support::risk::assert_risk;
use crate::support::topology_contract::{dynamic_import_edges, dynamic_import_risks};

#[test]
fn reports_dynamic_import_options_for_member_import_calls_like_js_oracle() {
    assert_risk(
        "packages/vite/src/css.ts",
        "const pluginFn = await jiti.import(pluginName, { parentURL, try: true, default: true })\n",
        &["dynamic-import-options"],
    );
}

#[test]
fn reports_nonliteral_dynamic_import() {
    dynamic_import_risks::assert_nonliteral_dynamic_import_is_reported();
}

#[test]
fn reports_template_dynamic_import() {
    dynamic_import_risks::assert_template_dynamic_import_is_reported();
}

#[test]
fn reports_multiline_template_dynamic_import() {
    dynamic_import_risks::assert_multiline_template_dynamic_import_is_reported();
}

#[test]
fn reports_nuxt_generic_function_nonliteral_dynamic_import() {
    let source = [
        "export async function importModule<T = unknown> (id: string): Promise<T> {",
        "  const resolvedPath = resolveModule(id)",
        "  return await import(pathToFileURL(resolvedPath).href).then(r => r.default || r) as Promise<T>",
        "}",
    ]
    .join("\n");

    assert_risk(
        "packages/kit/src/internal/esm.ts",
        &source,
        &["non-literal-dynamic-import", "unsupported-syntax"],
    );
}

#[test]
fn ignores_magic_comment_literal_dynamic_import_as_nonliteral_risk() {
    dynamic_import_edges::assert_magic_comment_literal_dynamic_import_is_not_nonliteral_risk();
}

#[test]
fn scans_literal_dynamic_import() {
    dynamic_import_edges::assert_literal_dynamic_import_is_scanned();
}

#[test]
fn accepts_unrelated_interpolated_template_literals() {
    dynamic_import_edges::assert_unrelated_interpolated_template_literals_are_accepted();
}
