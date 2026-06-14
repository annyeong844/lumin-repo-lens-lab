#![allow(dead_code)]

#[path = "../src/protocol.rs"]
mod protocol;
#[path = "../src/scanner.rs"]
mod scanner;

use scanner::scan_file_text;

#[test]
fn scans_static_imports_and_reexports_on_happy_path() {
    let source = [
        "import { runtime } from './runtime';",
        "import type { T } from './types';",
        "export { helper } from './helper';",
    ]
    .join("\n");
    let result = scan_file_text("fixture.ts", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 3);
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./runtime" && !edge.type_only && !edge.re_export));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./types" && edge.type_only));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./helper" && edge.re_export));
}

#[test]
fn scans_multiline_named_import_blocks() {
    let source = [
        "import {",
        "  type RuntimeHelp,",
        "  runtimeValue,",
        "} from './runtime';",
        "import {",
        "  mapEvent,",
        "} from '@geulbat/protocol/ids';",
    ]
    .join("\n");
    let result = scan_file_text("fixture.ts", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 2);
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./runtime" && edge.line == 1 && !edge.type_only));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "@geulbat/protocol/ids" && edge.line == 5));
}

#[test]
fn marks_multiline_type_only_export_blocks() {
    let source = [
        "export {",
        "  type HistoryItem,",
        "  type FunctionCall,",
        "} from './provider/wire/types.js';",
    ]
    .join("\n");
    let result = scan_file_text("fixture.ts", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    let edge = &result.edges[0];
    assert_eq!(edge.source, "./provider/wire/types.js");
    assert!(edge.type_only);
    assert!(edge.re_export);
    assert_eq!(edge.line, 1);
}

#[test]
fn scans_semicolonless_nuxt_barrel_reexports() {
    let source = [
        "import '../../dist/app/types/augments'",
        "",
        "export { createNuxtApp, useNuxtApp } from './nuxt'",
        "export type { NuxtApp, RuntimeNuxtHooks } from './nuxt'",
        "export { useAsyncData, useFetch } from './composables/index'",
        "export type { AsyncData, UseFetchOptions } from './composables/index'",
    ]
    .join("\n");
    let result = scan_file_text("packages/nuxt/src/app/index.ts", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 5);
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "../../dist/app/types/augments" && !edge.re_export));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./nuxt" && edge.re_export && !edge.type_only));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./nuxt" && edge.re_export && edge.type_only));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./composables/index" && edge.re_export && !edge.type_only));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "./composables/index" && edge.re_export && edge.type_only));
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
    let result = scan_file_text("packages/kit/src/internal/esm.ts", &source);
    assert!(!result.ok);
    assert_eq!(
        result.risk,
        vec![
            "non-literal-dynamic-import".to_string(),
            "unsupported-syntax".to_string(),
        ]
    );
    assert!(result.edges.is_empty());
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
    let result = scan_file_text("packages/nuxt/schema.d.ts", &source);
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["ts-ambient-module".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn ignores_magic_comment_literal_dynamic_import_as_nonliteral_risk() {
    let result = scan_file_text(
        "packages/nuxt/src/app/composables/manifest.ts",
        "_manifest = import(/* webpackIgnore: true */ /* @vite-ignore */ '#app-manifest')\n",
    );
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "#app-manifest");
    assert!(result.edges[0].dynamic);
}

#[test]
fn reports_dynamic_import_options_for_member_import_calls_like_js_oracle() {
    let result = scan_file_text(
        "packages/vite/src/css.ts",
        "const pluginFn = await jiti.import(pluginName, { parentURL, try: true, default: true })\n",
    );
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["dynamic-import-options".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn reports_export_assignment_for_export_property_equality_like_js_oracle() {
    let result = scan_file_text(
        "packages/nuxt/src/components/templates.ts",
        "const exp = c.export === 'default' ? 'c.default || c' : `c['${c.export}']`\n",
    );
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["ts-export-assignment".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn accepts_simple_interpolated_template_mapping_lines() {
    let source = [
        "import { normalize } from './paths';",
        "const lines = details.map((line) => `  ${line}`).join('\\n');",
        "export const value = normalize(lines);",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "./paths");
}

#[test]
fn reports_require_context_before_general_require() {
    let result = scan_file_text(
        "fixture.ts",
        "const ctx = require.context('./pages', true, /\\.tsx$/);\n",
    );
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["require-context".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn reports_ts_import_equals_and_export_assignment_risks() {
    let result = scan_file_text(
        "fixture.ts",
        "import foo = require('./cjs');\nexport = foo;\n",
    );
    assert!(!result.ok);
    assert_eq!(
        result.risk,
        vec![
            "require-call".to_string(),
            "ts-export-assignment".to_string(),
            "ts-import-equals".to_string(),
        ]
    );
    assert!(result.edges.is_empty());
}

#[test]
fn reports_decorator_or_reflect_metadata_risk() {
    let result = scan_file_text("fixture.ts", "Reflect.metadata('role', 'service')(Service);\n");
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["decorator-or-reflect".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn reports_unsupported_syntax_for_ts_generic_type_annotation() {
    let source = "import { value } from './value';\ntype Loader = Promise<Result>;\n";
    let result = scan_file_text("fixture.ts", source);
    assert!(!result.ok);
    assert_eq!(result.risk, vec!["unsupported-syntax".to_string()]);
    assert!(result.edges.is_empty());
}

#[test]
fn ignores_angle_brackets_inside_comments_when_scanning_imports() {
    let source = [
        "import { readFileSync } from 'node:fs';",
        "import path from 'node:path';",
        "",
        "// Helper inventory is a Map<identity, def> for lookup.",
        "// Parse failures may include tags like `<script>` in messages.",
        "export function readArtifact() {",
        "  return readFileSync(path.join('out', 'symbols.json'), 'utf8');",
        "}",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 2);
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "node:fs" && edge.line == 1));
    assert!(result
        .edges
        .iter()
        .any(|edge| edge.source == "node:path" && edge.line == 2));
}

#[test]
fn ignores_angle_brackets_inside_strings_when_scanning_imports() {
    let source = [
        "import { existsSync } from 'node:fs';",
        "const message = 'failed to parse <path>: <message>';",
        "export const ok = existsSync(message);",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "node:fs");
}

#[test]
fn ignores_import_export_syntax_inside_strings() {
    let source = [
        "import { spawnSync } from 'node:child_process';",
        "const extractor = `",
        "from pathlib import Path",
        "export { TEXT_ZERO_REF_COUNT } from 'text-zero-ident-ref-count';",
        "import(module_name)",
        "`;",
        "export const run = () => spawnSync('python', ['-c', extractor]);",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "node:child_process");
}

#[test]
fn ignores_decorator_markers_inside_strings() {
    let source = [
        "import { normalize } from './paths';",
        "if (line.startsWith('@')) continue;",
        "export const value = normalize(line);",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "./paths");
}

#[test]
fn ignores_angle_brackets_inside_regex_literals() {
    let source = [
        "import { parseOxcOrThrow } from './parse-oxc.mjs';",
        "const scriptRe = /<script\\b[^>]*>/gi;",
        "export const parse = (source) => parseOxcOrThrow(source, scriptRe);",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "./parse-oxc.mjs");
}

#[test]
fn scans_literal_dynamic_import() {
    let result = scan_file_text(
        "fixture.ts",
        "export async function lazy() { return import('./lazy'); }\n",
    );
    assert!(result.ok);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "./lazy");
    assert!(result.edges[0].dynamic);
}

#[test]
fn reports_nonliteral_dynamic_import() {
    let result = scan_file_text("fixture.ts", "export function load(name) { return import(name); }\n");
    assert!(!result.ok);
    assert_eq!(
        result.risk,
        vec!["non-literal-dynamic-import".to_string()]
    );
}

#[test]
fn reports_template_dynamic_import() {
    let result = scan_file_text(
        "fixture.ts",
        "export function load(name) { return import(`./${name}.ts`); }\n",
    );
    assert!(!result.ok);
    assert_eq!(
        result.risk,
        vec!["template-dynamic-import".to_string()]
    );
}

#[test]
fn reports_multiline_template_dynamic_import() {
    let source = [
        "async function load(pathToFileURL, dir) {",
        "  const mod = await import(",
        "    `${pathToFileURL(dir).href}/_lib/alias-map.mjs?v=case`",
        "  );",
        "  return mod;",
        "}",
    ]
    .join("\n");
    let result = scan_file_text("fixture.mjs", &source);
    assert!(!result.ok);
    assert_eq!(
        result.risk,
        vec!["template-dynamic-import".to_string()]
    );
    assert!(result.edges.is_empty());
}

#[test]
fn accepts_unrelated_interpolated_template_literals() {
    let result = scan_file_text(
        "fixture.ts",
        "const msg = `hello ${name}`;\nimport real from './real';\n",
    );
    assert!(result.ok);
    assert_eq!(result.risk.len(), 0);
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].source, "./real");
}
