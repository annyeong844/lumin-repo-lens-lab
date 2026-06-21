use crate::support::edge::{assert_first_dynamic_source, assert_first_source};
use crate::support::scan::scan_ok;

pub fn assert_magic_comment_literal_dynamic_import_is_not_nonliteral_risk() {
    let edges = scan_ok(
        "packages/nuxt/src/app/composables/manifest.ts",
        "_manifest = import(/* webpackIgnore: true */ /* @vite-ignore */ '#app-manifest')\n",
        1,
    );

    assert_first_dynamic_source(&edges, "#app-manifest");
}

pub fn assert_literal_dynamic_import_is_scanned() {
    let edges = scan_ok(
        "fixture.ts",
        "export async function lazy() { return import('./lazy'); }\n",
        1,
    );

    assert_first_dynamic_source(&edges, "./lazy");
}

pub fn assert_unrelated_interpolated_template_literals_are_accepted() {
    let edges = scan_ok(
        "fixture.ts",
        "const msg = `hello ${name}`;\nimport real from './real';\n",
        1,
    );

    assert_first_source(&edges, "./real");
}
