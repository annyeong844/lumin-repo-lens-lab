use oxc_ast::ast::{
    BindingPattern, Expression, ImportOrExportKind, MemberExpression, MethodDefinitionKind,
    ModuleExportName, PropertyKey, SimpleAssignmentTarget, TSAccessibility,
    VariableDeclarationKind,
};
use oxc_span::Span;

pub(super) fn expression_identifier_name(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.to_string()),
        Expression::ParenthesizedExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::ChainExpression(_) => None,
        Expression::TSAsExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            expression_identifier_name(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        _ => None,
    }
}

pub(super) fn expression_member_object_identifier_name(
    expression: &Expression<'_>,
) -> Option<String> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::ChainExpression(expression) => expression
            .expression
            .as_member_expression()
            .and_then(member_object_identifier_name),
        Expression::TSAsExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        expression => expression
            .as_member_expression()
            .and_then(member_object_identifier_name),
    }
}

pub(super) fn member_object_identifier_name(member: &MemberExpression<'_>) -> Option<String> {
    match member {
        MemberExpression::StaticMemberExpression(member) => {
            expression_identifier_name(&member.object)
        }
        MemberExpression::ComputedMemberExpression(member) => {
            expression_identifier_name(&member.object)
        }
        MemberExpression::PrivateFieldExpression(member) => {
            expression_identifier_name(&member.object)
        }
    }
}

pub(super) fn static_member_property_name(member: &MemberExpression<'_>) -> Option<String> {
    match member {
        MemberExpression::StaticMemberExpression(member) => Some(member.property.name.to_string()),
        MemberExpression::ComputedMemberExpression(member) => match &member.expression {
            Expression::StringLiteral(literal) => Some(literal.value.to_string()),
            _ => None,
        },
        MemberExpression::PrivateFieldExpression(_) => None,
    }
}

pub(super) fn assignment_target_identifier_name(
    target: &oxc_ast::ast::AssignmentTarget<'_>,
) -> Option<String> {
    target
        .as_simple_assignment_target()
        .and_then(simple_assignment_target_identifier_name)
}

pub(super) fn assignment_target_member_object_identifier_name(
    target: &oxc_ast::ast::AssignmentTarget<'_>,
) -> Option<String> {
    target
        .as_simple_assignment_target()
        .and_then(simple_assignment_target_member_object_identifier_name)
}

pub(super) fn simple_assignment_target_identifier_name(
    target: &SimpleAssignmentTarget<'_>,
) -> Option<String> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some(identifier.name.to_string())
        }
        SimpleAssignmentTarget::TSAsExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSNonNullExpression(expression) => {
            expression_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSTypeAssertion(expression) => {
            expression_identifier_name(&expression.expression)
        }
        _ => None,
    }
}

pub(super) fn simple_assignment_target_member_object_identifier_name(
    target: &SimpleAssignmentTarget<'_>,
) -> Option<String> {
    match target {
        SimpleAssignmentTarget::TSAsExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSSatisfiesExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSNonNullExpression(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        SimpleAssignmentTarget::TSTypeAssertion(expression) => {
            expression_member_object_identifier_name(&expression.expression)
        }
        target => target
            .as_member_expression()
            .and_then(member_object_identifier_name),
    }
}

pub(super) fn binding_identifier_name(pattern: &BindingPattern<'_>) -> Option<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}

pub(super) fn binding_identifier_name_ref<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

pub(super) fn module_export_identifier_name(name: &ModuleExportName<'_>) -> Option<String> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.to_string()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.to_string()),
        ModuleExportName::StringLiteral(_) => None,
    }
}

pub(super) fn property_key_name(key: &PropertyKey<'_>, computed: bool) -> Option<String> {
    match key {
        PropertyKey::PrivateIdentifier(identifier) => Some(format!("#{}", identifier.name)),
        PropertyKey::StaticIdentifier(identifier) if !computed => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) if !computed => Some(literal.value.to_string()),
        PropertyKey::Identifier(identifier) if !computed => Some(identifier.name.to_string()),
        _ => None,
    }
}

pub(super) fn visibility_text(
    accessibility: Option<TSAccessibility>,
    key: &PropertyKey<'_>,
) -> String {
    if matches!(key, PropertyKey::PrivateIdentifier(_)) {
        return "private".to_string();
    }
    match accessibility {
        Some(TSAccessibility::Private) => "private",
        Some(TSAccessibility::Protected) => "protected",
        Some(TSAccessibility::Public) | None => "public",
    }
    .to_string()
}

pub(super) fn variable_kind_text(kind: VariableDeclarationKind) -> &'static str {
    match kind {
        VariableDeclarationKind::Var => "var",
        VariableDeclarationKind::Let => "let",
        VariableDeclarationKind::Const => "const",
        VariableDeclarationKind::Using => "using",
        VariableDeclarationKind::AwaitUsing => "await using",
    }
}

pub(super) fn method_kind_text(kind: MethodDefinitionKind) -> &'static str {
    match kind {
        MethodDefinitionKind::Constructor => "constructor",
        MethodDefinitionKind::Method => "method",
        MethodDefinitionKind::Get => "get",
        MethodDefinitionKind::Set => "set",
    }
}

pub(super) fn is_type_only(kind: ImportOrExportKind) -> bool {
    matches!(kind, ImportOrExportKind::Type)
}

pub(super) fn ts_module_name(module: &oxc_ast::ast::TSModuleDeclaration<'_>) -> Option<String> {
    match &module.id {
        oxc_ast::ast::TSModuleDeclarationName::Identifier(identifier) => {
            Some(identifier.name.to_string())
        }
        oxc_ast::ast::TSModuleDeclarationName::StringLiteral(literal) => {
            Some(literal.value.to_string())
        }
    }
}

pub(super) fn definition_id(file: &str, node_kind: &str, span: Span) -> String {
    format!(
        "{}#{}:{}-{}",
        file.replace('\\', "/"),
        node_kind,
        span.start,
        span.end
    )
}
