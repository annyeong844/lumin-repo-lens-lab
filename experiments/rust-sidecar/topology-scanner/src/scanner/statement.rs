use crate::protocol::{ModuleEdge, RiskKind};

use super::dynamic::scan_dynamic_imports;
use super::edges::{export_specifiers_type_only, push_edge, quoted_after};
use super::risk::{contains_export_assignment_risk, has_unsupported_angle_syntax};

pub(super) fn starts_multiline_import_export_block(trimmed: &str) -> bool {
    trimmed.starts_with("import {")
        || trimmed.starts_with("import type {")
        || trimmed.starts_with("export {")
        || trimmed.starts_with("export type {")
}

pub(super) fn scan_line(
    line: &str,
    risk_line: &str,
    line_no: usize,
    edges: &mut Vec<ModuleEdge>,
    risk: &mut Vec<RiskKind>,
) {
    let trimmed = line.trim_start();
    let risk_trimmed = risk_line.trim_start();
    let starting_risk_len = risk.len();

    if risk_trimmed.starts_with('@') || risk_trimmed.contains("Reflect.metadata(") {
        risk.push(RiskKind::DecoratorOrReflect);
    }
    if risk_trimmed.contains("require.context(") {
        risk.push(RiskKind::RequireContext);
    } else if risk_trimmed.contains("require(") {
        risk.push(RiskKind::RequireCall);
    }
    if risk_trimmed.starts_with("import ") && risk_trimmed.contains(" = require(") {
        risk.push(RiskKind::TsImportEquals);
    }
    if contains_export_assignment_risk(risk_line) {
        risk.push(RiskKind::TsExportAssignment);
    }
    if risk_trimmed.contains("import.meta.glob(") {
        risk.push(RiskKind::ImportMetaGlob);
    }
    if risk_trimmed.starts_with("declare module ") {
        risk.push(RiskKind::TsAmbientModule);
    }
    if has_unsupported_angle_syntax(risk_line) {
        risk.push(RiskKind::UnsupportedSyntax);
    }

    if risk_line.contains("import") {
        scan_dynamic_imports(line, line_no, edges, risk);
    }
    if risk.len() > starting_risk_len {
        return;
    }

    if risk_trimmed.starts_with("import ") {
        let type_only = trimmed.starts_with("import type ");
        if let Some(source) = quoted_after(trimmed, " from ") {
            push_edge(edges, source, line_no, type_only, false);
        } else if let Some(source) = quoted_after(trimmed, "import ") {
            push_edge(edges, source, line_no, false, false);
        }
    }

    if risk_trimmed.starts_with("export ") && trimmed.contains(" from ") {
        let type_only = trimmed.starts_with("export type ") || export_specifiers_type_only(trimmed);
        if let Some(source) = quoted_after(trimmed, " from ") {
            push_edge(edges, source, line_no, type_only, true);
        }
    }
}
