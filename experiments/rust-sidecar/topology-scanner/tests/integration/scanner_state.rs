use crate::support::risk::assert_risk;
use crate::support::topology_contract::{template_mapping, template_state};

#[test]
fn reports_scanner_state_ambiguous_for_nested_conditional_template_warning() {
    let source = [
        "import { logger } from '../../utils.ts';",
        "logger.warn(`[nuxt:compiler] Duplicate ${name !== oldName ? ` defined as \\`${name}\\`` : ''} with ${source ? `the same source \\`${source}\\`` : 'no source'} found.`);",
    ]
    .join("\n");

    assert_risk(
        "packages/nuxt/src/compiler/plugins/keyed-functions.ts",
        &source,
        &["scanner-state-ambiguous"],
    );
}

#[test]
fn reports_scanner_state_ambiguous_before_decorator_plugin_edges() {
    let source = [
        "import type { Plugin } from 'vite';",
        "import { ensureDependencyInstalled, logger } from '@nuxt/kit';",
        "import type { Nuxt } from '@nuxt/schema';",
        "let transformSync: typeof import('@babel/core').transformSync;",
        "logger.warn(`Install ${result.map(d => `\\`${d}\\``).join(' and ')} to enable decorator support.`);",
    ]
    .join("\n");

    assert_risk(
        "packages/vite/src/plugins/decorators.ts",
        &source,
        &["scanner-state-ambiguous"],
    );
}

#[test]
fn reports_scanner_state_ambiguous_for_nested_template_warning() {
    let source = [
        "import { logger } from '@nuxt/kit';",
        "logger.warn(`Install ${result.map(d => `\\`${d}\\``).join(' and ')} to enable decorator support.`);",
    ]
    .join("\n");

    assert_risk(
        "packages/vite/src/plugins/decorators.ts",
        &source,
        &["scanner-state-ambiguous"],
    );
}

#[test]
fn accepts_simple_interpolated_template_mapping_lines() {
    template_mapping::assert_simple_interpolated_template_mapping_lines_are_accepted();
}

#[test]
fn accepts_nested_template_interpolation_without_escaped_backticks_like_js_oracle() {
    template_state::assert_nested_template_interpolation_without_escaped_backticks_matches_js_oracle(
    );
}

#[test]
fn does_not_add_scanner_state_for_single_conditional_escaped_backtick_template() {
    template_state::assert_single_conditional_escaped_backtick_template_does_not_leak_state();
}
