use crate::support::edge::assert_first_source;
use crate::support::scan::scan_ok;

pub fn assert_simple_interpolated_template_mapping_lines_are_accepted() {
    let source = [
        "import { normalize } from './paths';",
        "const lines = details.map((line) => `  ${line}`).join('\\n');",
        "export const value = normalize(lines);",
    ]
    .join("\n");
    let edges = scan_ok("fixture.mjs", &source, 1);

    assert_first_source(&edges, "./paths");
}
