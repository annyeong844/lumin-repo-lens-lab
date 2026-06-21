use crate::support::risk::assert_risk;

#[test]
fn reports_decorator_or_reflect_metadata_risk() {
    assert_risk(
        "fixture.ts",
        "Reflect.metadata('role', 'service')(Service);\n",
        &["decorator-or-reflect"],
    );
}

#[test]
fn reports_ts_ambient_module_in_declaration_file() {
    let source = [
        "import type {",
        "  NuxtHooks as _NuxtHooks,",
        "} from '@nuxt/schema'",
        "",
        "declare module 'nuxt/schema' {",
        "  interface NuxtHooks extends _NuxtHooks {}",
        "}",
    ]
    .join("\n");

    assert_risk("packages/nuxt/schema.d.ts", &source, &["ts-ambient-module"]);
}

#[test]
fn reports_export_property_equality_like_js_oracle() {
    assert_risk(
        "packages/nuxt/src/components/templates.ts",
        "const exp = c.export === 'default' ? 'c.default || c' : `c['${c.export}']`\n",
        &["ts-export-assignment"],
    );
}

#[test]
fn reports_ts_import_equals_and_export_assignment_risks() {
    assert_risk(
        "fixture.ts",
        "import foo = require('./cjs');\nexport = foo;\n",
        &["require-call", "ts-export-assignment", "ts-import-equals"],
    );
}

#[test]
fn reports_unsupported_syntax_for_ts_generic_type_annotation() {
    let source = "import { value } from './value';\ntype Loader = Promise<Result>;\n";
    assert_risk("fixture.ts", source, &["unsupported-syntax"]);
}
