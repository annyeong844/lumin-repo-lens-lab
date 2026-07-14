use super::{
    binding_identifier_name_ref, is_type_only, line_for_span, static_member_property_name,
    VueGlobalComponentRegistration,
};
use oxc_ast::ast::{
    Argument, CallExpression, Expression, FormalParameters, Function, FunctionBody, FunctionType,
    ImportDeclarationSpecifier, MemberExpression, ModuleExportName, ObjectProperty, Program,
    PropertyKey, Statement, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::{BTreeMap, BTreeSet};

const VUE_APP_FACTORY_NAMES: &[&str] = &["createApp", "createSSRApp"];
const VUE_APP_RETURNING_METHODS: &[&str] = &["component", "directive", "mixin", "provide", "use"];

#[derive(Debug)]
struct ImportBinding {
    binding_name: String,
    binding_source: String,
    binding_kind: String,
    imported_name: String,
}

pub(super) fn collect_vue_global_component_registrations(
    program: &Program<'_>,
    file_path: &str,
    line_starts: &[usize],
) -> Vec<VueGlobalComponentRegistration> {
    let imports = collect_import_bindings(program);
    let mut receiver_visitor = VueReceiverVisitor::default();
    receiver_visitor.visit_program(program);
    if receiver_visitor.receivers.is_empty() {
        return Vec::new();
    }

    let mut registration_visitor = VueRegistrationVisitor {
        file_path,
        line_starts,
        imports: &imports,
        receivers: &receiver_visitor.receivers,
        records: Vec::new(),
    };
    registration_visitor.visit_program(program);
    mark_duplicate_registrations(&mut registration_visitor.records);
    registration_visitor.records
}

fn collect_import_bindings(program: &Program<'_>) -> BTreeMap<String, ImportBinding> {
    let mut bindings = BTreeMap::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if is_type_only(import.import_kind) {
            continue;
        }
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        for specifier in specifiers {
            let (local_name, binding_kind, imported_name) = match specifier {
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => (
                    specifier.local.name.to_string(),
                    "default",
                    "default".to_string(),
                ),
                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                    if !is_type_only(specifier.import_kind) =>
                {
                    let imported_name = import_name(&specifier.imported);
                    (specifier.local.name.to_string(), "named", imported_name)
                }
                _ => continue,
            };
            bindings.insert(
                local_name.clone(),
                ImportBinding {
                    binding_name: local_name,
                    binding_source: import.source.value.to_string(),
                    binding_kind: binding_kind.to_string(),
                    imported_name,
                },
            );
        }
    }
    bindings
}

#[derive(Default)]
struct VueReceiverVisitor {
    receivers: BTreeSet<String>,
}

impl<'a> Visit<'a> for VueReceiverVisitor {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if declarator
            .init
            .as_ref()
            .is_some_and(is_vue_app_returning_expression)
        {
            if let Some(name) = binding_identifier_name_ref(&declarator.id) {
                self.receivers.insert(name.to_string());
            }
        }
        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if function.r#type == FunctionType::FunctionDeclaration
            && function.id.as_ref().is_some_and(|id| id.name == "install")
        {
            if let Some(name) = first_parameter_name(&function.params) {
                self.receivers.insert(name.to_string());
            }
        }
        walk::walk_function(self, function, flags);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if object_property_name(&property.key).as_deref() == Some("install") {
            let params = match &property.value {
                Expression::FunctionExpression(function) => Some(function.params.as_ref()),
                Expression::ArrowFunctionExpression(function) => Some(function.params.as_ref()),
                _ => None,
            };
            if let Some(name) = params.and_then(first_parameter_name) {
                self.receivers.insert(name.to_string());
            }
        }
        walk::walk_object_property(self, property);
    }
}

fn import_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(identifier) => identifier.name.to_string(),
        ModuleExportName::IdentifierReference(identifier) => identifier.name.to_string(),
        ModuleExportName::StringLiteral(literal) => literal.value.to_string(),
    }
}

fn object_property_name(key: &PropertyKey<'_>) -> Option<String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.to_string()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.to_string()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.to_string()),
        _ => None,
    }
}

fn first_parameter_name<'a>(params: &'a FormalParameters<'a>) -> Option<&'a str> {
    params
        .items
        .first()
        .and_then(|parameter| binding_identifier_name_ref(&parameter.pattern))
}

fn is_vue_app_returning_expression(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression else {
        return false;
    };
    if is_vue_app_factory_call(call) {
        return true;
    }
    let Some(member) = call.callee.as_member_expression() else {
        return false;
    };
    if !static_member_property_name(member)
        .as_deref()
        .is_some_and(|name| VUE_APP_RETURNING_METHODS.contains(&name))
    {
        return false;
    }
    is_vue_app_returning_expression(member_object_expression(member))
}

fn is_vue_app_factory_call(call: &CallExpression<'_>) -> bool {
    if matches!(&call.callee, Expression::Identifier(identifier) if VUE_APP_FACTORY_NAMES.contains(&identifier.name.as_str()))
    {
        return true;
    }
    call.callee
        .as_member_expression()
        .and_then(static_member_property_name)
        .as_deref()
        .is_some_and(|name| VUE_APP_FACTORY_NAMES.contains(&name))
}

fn member_object_expression<'a>(member: &'a MemberExpression<'a>) -> &'a Expression<'a> {
    match member {
        MemberExpression::StaticMemberExpression(member) => &member.object,
        MemberExpression::ComputedMemberExpression(member) => &member.object,
        MemberExpression::PrivateFieldExpression(member) => &member.object,
    }
}

struct VueRegistrationVisitor<'b> {
    file_path: &'b str,
    line_starts: &'b [usize],
    imports: &'b BTreeMap<String, ImportBinding>,
    receivers: &'b BTreeSet<String>,
    records: Vec<VueGlobalComponentRegistration>,
}

impl<'a> Visit<'a> for VueRegistrationVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.collect_call(call);
        walk::walk_call_expression(self, call);
    }
}

impl VueRegistrationVisitor<'_> {
    fn collect_call(&mut self, call: &CallExpression<'_>) {
        let Some(member) = call.callee.as_member_expression() else {
            return;
        };
        if static_member_property_name(member).as_deref() != Some("component") {
            return;
        }
        let Expression::Identifier(receiver) = member_object_expression(member) else {
            return;
        };
        if !self.receivers.contains(receiver.name.as_str()) {
            return;
        }

        let component_name = literal_argument_string(call.arguments.first());
        let binding_name = argument_identifier_name(call.arguments.get(1));
        let binding = binding_name.and_then(|name| self.imports.get(name));
        let async_factory = define_async_component_factory(call.arguments.get(1));
        let line = line_for_span(self.line_starts, call.span);
        let api = format!("{}.component", receiver.name);

        if component_name.is_none() {
            let Some(binding) = binding else {
                return;
            };
            self.records.push(registration_record(
                self.file_path,
                api,
                None,
                Some(binding),
                None,
                None,
                line,
                "muted",
                Some("sfc-global-component-name-dynamic"),
            ));
            return;
        }

        if let Some((factory_kind, from_spec)) = async_factory {
            let reason = if from_spec.is_some() {
                "sfc-global-component-async-factory"
            } else {
                "sfc-global-component-async-factory-nonliteral"
            };
            self.records.push(registration_record(
                self.file_path,
                api,
                component_name,
                None,
                from_spec,
                Some(factory_kind),
                line,
                "muted",
                Some(reason),
            ));
            return;
        }

        let Some(binding) = binding else {
            self.records.push(registration_record(
                self.file_path,
                api,
                component_name,
                None,
                None,
                None,
                line,
                "muted",
                Some("sfc-global-component-value-unsupported"),
            ));
            return;
        };
        self.records.push(registration_record(
            self.file_path,
            api,
            component_name,
            Some(binding),
            None,
            None,
            line,
            "registration-syntax",
            None,
        ));
    }
}

fn literal_argument_string(argument: Option<&Argument<'_>>) -> Option<String> {
    match argument {
        Some(Argument::StringLiteral(literal)) => Some(literal.value.to_string()),
        _ => None,
    }
}

fn argument_identifier_name<'a>(argument: Option<&'a Argument<'a>>) -> Option<&'a str> {
    match argument {
        Some(Argument::Identifier(identifier)) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn define_async_component_factory(
    argument: Option<&Argument<'_>>,
) -> Option<(&'static str, Option<String>)> {
    let Expression::CallExpression(call) = argument?.as_expression()? else {
        return None;
    };
    if !matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "defineAsyncComponent")
    {
        return None;
    }
    Some((
        "defineAsyncComponent",
        async_loader_import_source(call.arguments.first()),
    ))
}

fn async_loader_import_source(argument: Option<&Argument<'_>>) -> Option<String> {
    match argument?.as_expression()? {
        Expression::ArrowFunctionExpression(function) => {
            function_body_import_source(&function.body, function.expression)
        }
        Expression::FunctionExpression(function) => function
            .body
            .as_ref()
            .and_then(|body| function_body_import_source(body, false)),
        _ => None,
    }
}

fn function_body_import_source(body: &FunctionBody<'_>, expression_body: bool) -> Option<String> {
    if expression_body {
        let Statement::ExpressionStatement(statement) = body.statements.first()? else {
            return None;
        };
        return import_expression_literal_source(&statement.expression);
    }
    body.statements.iter().find_map(|statement| {
        let Statement::ReturnStatement(statement) = statement else {
            return None;
        };
        statement
            .argument
            .as_ref()
            .and_then(import_expression_literal_source)
    })
}

fn import_expression_literal_source(expression: &Expression<'_>) -> Option<String> {
    let Expression::ImportExpression(import) = expression else {
        return None;
    };
    let Expression::StringLiteral(source) = &import.source else {
        return None;
    };
    Some(source.value.to_string())
}

#[allow(
    clippy::too_many_arguments,
    reason = "wire-row fields stay explicit at the Vue registration boundary"
)]
fn registration_record(
    file_path: &str,
    api: String,
    component_name: Option<String>,
    binding: Option<&ImportBinding>,
    from_spec: Option<String>,
    factory_kind: Option<&str>,
    line: usize,
    status: &str,
    reason: Option<&str>,
) -> VueGlobalComponentRegistration {
    let binding_source = binding.map(|binding| binding.binding_source.clone());
    VueGlobalComponentRegistration {
        registration_file: file_path.to_string(),
        framework: "vue".to_string(),
        api,
        normalized_tag_names: component_name
            .as_deref()
            .map(normalized_component_names)
            .unwrap_or_default(),
        component_name,
        binding_name: binding.map(|binding| binding.binding_name.clone()),
        binding_source: binding_source.clone(),
        from_spec: binding_source.or(from_spec),
        binding_kind: binding.map(|binding| binding.binding_kind.clone()),
        imported_name: binding.map(|binding| binding.imported_name.clone()),
        source: "sfc-global-component-registration".to_string(),
        status: status.to_string(),
        confidence: if status == "muted" {
            "muted-review"
        } else {
            "registration-review"
        }
        .to_string(),
        eligible_for_fan_in: false,
        eligible_for_safe_fix: false,
        reason: reason.map(str::to_string),
        factory_kind: factory_kind.map(str::to_string),
        ambiguity_key: None,
        line,
    }
}

fn normalized_component_names(component_name: &str) -> Vec<String> {
    let mut names = vec![component_name.to_string()];
    for candidate in [
        pascal_from_kebab(component_name),
        kebab_from_pascal(component_name),
    ]
    .into_iter()
    .flatten()
    {
        if !names.contains(&candidate) {
            names.push(candidate);
        }
    }
    names
}

fn pascal_from_kebab(value: &str) -> Option<String> {
    let mut parts = value.split('-');
    let first = parts.next()?;
    if !valid_kebab_part(first) || !first.as_bytes()[0].is_ascii_lowercase() {
        return None;
    }
    let rest = parts.collect::<Vec<_>>();
    if rest.is_empty() || rest.iter().any(|part| !valid_kebab_part(part)) {
        return None;
    }
    Some(
        std::iter::once(first)
            .chain(rest)
            .map(|part| {
                let mut chars = part.chars();
                let first = chars.next().unwrap_or_default().to_ascii_uppercase();
                format!("{first}{}", chars.as_str())
            })
            .collect(),
    )
}

fn valid_kebab_part(part: &str) -> bool {
    !part.is_empty()
        && part
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn kebab_from_pascal(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || !bytes[0].is_ascii_uppercase()
        || !bytes.iter().all(u8::is_ascii_alphanumeric)
    {
        return None;
    }
    let mut out = String::with_capacity(value.len() + 4);
    for (index, byte) in bytes.iter().copied().enumerate() {
        let previous = index.checked_sub(1).and_then(|index| bytes.get(index));
        let next = bytes.get(index + 1);
        if index > 0
            && byte.is_ascii_uppercase()
            && (previous.is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                || (previous.is_some_and(u8::is_ascii_uppercase)
                    && next.is_some_and(u8::is_ascii_lowercase)))
        {
            out.push('-');
        }
        out.push(char::from(byte.to_ascii_lowercase()));
    }
    Some(out)
}

fn mark_duplicate_registrations(records: &mut [VueGlobalComponentRegistration]) {
    let mut groups = BTreeMap::<String, Vec<usize>>::new();
    for (index, record) in records.iter().enumerate() {
        let Some(component_name) = &record.component_name else {
            continue;
        };
        if record.binding_name.is_none() && record.from_spec.is_none() {
            continue;
        }
        groups
            .entry(format!("{}:{component_name}", record.api))
            .or_default()
            .push(index);
    }
    for indexes in groups.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes {
            let record = &mut records[*index];
            record.status = "muted".to_string();
            record.confidence = "muted-review".to_string();
            record.reason = Some("sfc-global-component-duplicate-registration".to_string());
            record.ambiguity_key = record.component_name.clone();
        }
    }
}
