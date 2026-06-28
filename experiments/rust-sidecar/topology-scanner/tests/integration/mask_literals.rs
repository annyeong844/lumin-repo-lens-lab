use crate::support::scan::scan_ok;

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
    let edges = scan_ok("fixture.mjs", &source, 2);

    assert_eq!(edges[0].source, "node:fs");
    assert_eq!(edges[1].source, "node:path");
}

#[test]
fn ignores_angle_brackets_inside_regex_literals() {
    let source = [
        "import { parseOxcOrThrow } from './parse-oxc.mjs';",
        "const scriptRe = /<script\\b[^>]*>/gi;",
        "export const parse = (source) => parseOxcOrThrow(source, scriptRe);",
    ]
    .join("\n");
    let edges = scan_ok("fixture.mjs", &source, 1);

    assert_eq!(edges[0].source, "./parse-oxc.mjs");
}

#[test]
fn handles_box_drawing_section_comments_before_regex_literals() {
    let source = [
        "import { parseOxcOrThrow } from './parse-oxc.mjs';",
        "// ── Comment-based walker ───────────────────────────────────",
        "const tsIgnoreRe = /^\\s*@ts-ignore\\b/;",
        "export const parse = (source) => parseOxcOrThrow(source, tsIgnoreRe);",
    ]
    .join("\n");
    let edges = scan_ok("_lib/extract-ts-escapes.mjs", &source, 1);

    assert_eq!(edges[0].source, "./parse-oxc.mjs");
}

#[test]
fn handles_non_ascii_text_before_regex_literals() {
    let source = [
        "import { parseOxcOrThrow } from './parse-oxc.mjs';",
        "const message = '확인 불가';",
        "const scriptRe = /<script\\b[^>]*>/gi;",
        "export const parse = (source) => parseOxcOrThrow(source, scriptRe, message);",
    ]
    .join("\n");
    let edges = scan_ok("fixture.mjs", &source, 1);

    assert_eq!(edges[0].source, "./parse-oxc.mjs");
}

#[test]
fn ignores_angle_brackets_inside_strings_when_scanning_imports() {
    let source = [
        "import { existsSync } from 'node:fs';",
        "const message = 'failed to parse <path>: <message>';",
        "export const ok = existsSync(message);",
    ]
    .join("\n");
    let edges = scan_ok("fixture.mjs", &source, 1);

    assert_eq!(edges[0].source, "node:fs");
}

#[test]
fn ignores_decorator_markers_inside_strings() {
    let source = [
        "import { normalize } from './paths';",
        "if (line.startsWith('@')) continue;",
        "export const value = normalize(line);",
    ]
    .join("\n");
    let edges = scan_ok("fixture.mjs", &source, 1);

    assert_eq!(edges[0].source, "./paths");
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
    let edges = scan_ok("fixture.mjs", &source, 1);

    assert_eq!(edges[0].source, "node:child_process");
}
