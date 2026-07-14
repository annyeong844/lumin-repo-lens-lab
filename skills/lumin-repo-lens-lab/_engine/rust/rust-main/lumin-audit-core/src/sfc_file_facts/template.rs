use std::collections::BTreeSet;

use super::blocks::{attribute_value, line_of, quoted_attribute_value, SfcLanguage, TemplateBlock};
use super::conventions::{
    action_binding, astro_client_directive, store_binding, svelte_action_directive,
    svelte_store_subscription,
};
use super::protocol::{SfcFileConvention, SfcTemplateComponentRef};
use super::script::{ComponentBinding, ComponentBindings};

pub(super) struct TemplateFacts {
    pub(super) component_refs: Vec<SfcTemplateComponentRef>,
    pub(super) conventions: Vec<SfcFileConvention>,
}

pub(super) fn extract_template_facts(
    source: &str,
    file_path: &str,
    language: SfcLanguage,
    blocks: &[TemplateBlock<'_>],
    bindings: &ComponentBindings,
) -> TemplateFacts {
    let mut component_refs = Vec::new();
    let mut conventions = Vec::new();
    let mut store_subscription_keys = BTreeSet::new();
    for block in blocks {
        parse_template_block(
            source,
            file_path,
            language,
            block,
            bindings,
            &mut component_refs,
            &mut conventions,
        );
        if language == SfcLanguage::Svelte {
            collect_svelte_template_store_subscriptions(
                source,
                file_path,
                block,
                bindings,
                &mut store_subscription_keys,
                &mut conventions,
            );
        }
    }
    TemplateFacts {
        component_refs,
        conventions,
    }
}

fn parse_template_block(
    file_source: &str,
    file_path: &str,
    language: SfcLanguage,
    block: &TemplateBlock<'_>,
    bindings: &ComponentBindings,
    component_refs: &mut Vec<SfcTemplateComponentRef>,
    conventions: &mut Vec<SfcFileConvention>,
) {
    let source = block.content;
    let mut cursor = 0;
    while cursor < source.len() {
        let Some(relative_open) = source[cursor..].find('<') else {
            break;
        };
        let open = cursor + relative_open;
        let absolute_open = block.start_offset + open;
        if let Some(range) = block
            .excluded_ranges
            .iter()
            .find(|range| range.contains(&absolute_open))
        {
            cursor = range.end.saturating_sub(block.start_offset);
            continue;
        }
        if source[open..].starts_with("<!--") {
            cursor = source[open + 4..]
                .find("-->")
                .map_or(source.len(), |offset| open + offset + 7);
            continue;
        }
        let mut name_start = open + 1;
        while source[name_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            name_start += source[name_start..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
        }
        let Some(first) = source[name_start..].chars().next() else {
            break;
        };
        if !first.is_ascii_alphabetic() {
            cursor = name_start + first.len_utf8();
            continue;
        }
        let mut name_end = name_start + first.len_utf8();
        while let Some(character) = source[name_end..].chars().next() {
            if !is_tag_name_char(character) {
                break;
            }
            name_end += character.len_utf8();
        }
        let Some(relative_end) = source[name_end..].find('>') else {
            break;
        };
        let tag_end = name_end + relative_end;
        let tag_name = &source[name_start..name_end];
        let attrs = &source[name_end..tag_end];
        let line = line_of(file_source, absolute_open);

        collect_tag_conventions(
            file_path,
            language,
            tag_name,
            attrs,
            line,
            &block.kind,
            bindings,
            conventions,
        );

        if let Some(dynamic_name) = dynamic_binding_name(tag_name, attrs) {
            if let Some(binding) = bindings
                .imports
                .get(&dynamic_name)
                .or_else(|| bindings.exposed_names.get(&dynamic_name))
            {
                component_refs.push(template_record(
                    file_path,
                    tag_name,
                    &dynamic_name,
                    binding,
                    line,
                    &block.kind,
                    language,
                    "dynamic-component",
                    "muted",
                    Some("sfc-template-dynamic-component"),
                    None,
                ));
            }
            cursor = tag_end + 1;
            continue;
        }

        if tag_name.contains('.') {
            let mut parts = tag_name.split('.');
            let namespace_name = parts.next().unwrap_or_default();
            let member_name = parts.next().unwrap_or_default();
            if !member_name.is_empty() {
                if let Some(binding) = bindings.namespace_imports.get(namespace_name) {
                    component_refs.push(template_record(
                        file_path,
                        tag_name,
                        tag_name,
                        binding,
                        line,
                        &block.kind,
                        language,
                        "namespace-component-tag",
                        "muted",
                        Some("sfc-template-namespace-component"),
                        Some(member_name.to_string()),
                    ));
                }
            }
            cursor = tag_end + 1;
            continue;
        }

        for candidate in template_tag_candidates(tag_name) {
            let Some(binding) = bindings.exposed_names.get(&candidate) else {
                continue;
            };
            component_refs.push(template_record(
                file_path,
                tag_name,
                &candidate,
                binding,
                line,
                &block.kind,
                language,
                "component-tag",
                "binding",
                None,
                None,
            ));
            break;
        }
        cursor = tag_end + 1;
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "tag convention projection uses the checked SFC wire context"
)]
fn collect_tag_conventions(
    file_path: &str,
    language: SfcLanguage,
    tag_name: &str,
    attrs: &str,
    line: usize,
    block_kind: &str,
    bindings: &ComponentBindings,
    conventions: &mut Vec<SfcFileConvention>,
) {
    if language == SfcLanguage::Astro && !tag_name.contains('.') {
        if let Some(directive_name) = first_prefixed_attribute(attrs, "client:", false) {
            for candidate in template_tag_candidates(tag_name) {
                let Some(binding) = bindings.exposed_names.get(&candidate) else {
                    continue;
                };
                conventions.push(astro_client_directive(
                    file_path,
                    tag_name,
                    &candidate,
                    &directive_name,
                    binding,
                    line,
                    block_kind,
                ));
                break;
            }
        }
    }

    if language == SfcLanguage::Svelte {
        for directive_name in prefixed_attributes(attrs, "use:", true) {
            let action_name = directive_name.trim_start_matches("use:");
            let Some(binding) = action_binding(action_name, bindings) else {
                continue;
            };
            conventions.push(svelte_action_directive(
                file_path,
                tag_name,
                &directive_name,
                action_name,
                binding,
                line,
                block_kind,
            ));
        }
    }
}

fn first_prefixed_attribute(attrs: &str, prefix: &str, allow_dollar: bool) -> Option<String> {
    prefixed_attributes(attrs, prefix, allow_dollar)
        .into_iter()
        .next()
}

fn prefixed_attributes(attrs: &str, prefix: &str, allow_dollar: bool) -> Vec<String> {
    let bytes = attrs.as_bytes();
    let prefix_bytes = prefix.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index + prefix_bytes.len() < bytes.len() {
        if index > 0 && !bytes[index - 1].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if !bytes[index..].starts_with(prefix_bytes) {
            index += 1;
            continue;
        }
        let mut end = index + prefix_bytes.len();
        let Some(first) = bytes.get(end).copied() else {
            break;
        };
        if !is_directive_identifier_start(first, allow_dollar) {
            index += 1;
            continue;
        }
        end += 1;
        while bytes
            .get(end)
            .is_some_and(|byte| is_directive_identifier_continue(*byte, allow_dollar))
        {
            end += 1;
        }
        if bytes
            .get(end)
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(*byte, b'=' | b'/'))
        {
            out.push(attrs[index..end].to_string());
        }
        index = end;
    }
    out
}

fn is_directive_identifier_start(byte: u8, allow_dollar: bool) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || (allow_dollar && byte == b'$')
}

fn is_directive_identifier_continue(byte: u8, allow_dollar: bool) -> bool {
    byte.is_ascii_alphanumeric()
        || byte == b'_'
        || (!allow_dollar && byte == b'-')
        || (allow_dollar && byte == b'$')
}

fn collect_svelte_template_store_subscriptions(
    file_source: &str,
    file_path: &str,
    block: &TemplateBlock<'_>,
    bindings: &ComponentBindings,
    seen: &mut BTreeSet<String>,
    conventions: &mut Vec<SfcFileConvention>,
) {
    let source = block.content;
    let mut cursor = 0;
    while cursor < source.len() {
        let absolute_cursor = block.start_offset + cursor;
        if let Some(range) = block
            .excluded_ranges
            .iter()
            .find(|range| range.contains(&absolute_cursor))
        {
            cursor = range.end.saturating_sub(block.start_offset);
            continue;
        }
        if source[cursor..].starts_with("<!--") {
            cursor = source[cursor + 4..]
                .find("-->")
                .map_or(source.len(), |offset| cursor + offset + 7);
            continue;
        }
        let Some(relative_open) = source[cursor..].find('{') else {
            break;
        };
        let open = cursor + relative_open;
        let absolute_open = block.start_offset + open;
        if block
            .excluded_ranges
            .iter()
            .any(|range| range.contains(&absolute_open))
        {
            cursor = open + 1;
            continue;
        }
        let Some(relative_close) = source[open + 1..].find('}') else {
            break;
        };
        let close = open + 1 + relative_close;
        let expression = &source[open + 1..close];
        let mut expression_cursor = 0;
        while let Some(relative_dollar) = expression[expression_cursor..].find('$') {
            let dollar = expression_cursor + relative_dollar;
            let name_start = dollar + 1;
            let Some(first) = expression.as_bytes().get(name_start).copied() else {
                break;
            };
            if first == b'$' || !is_store_identifier_start(first) {
                expression_cursor = name_start;
                continue;
            }
            let mut name_end = name_start + 1;
            while expression
                .as_bytes()
                .get(name_end)
                .is_some_and(|byte| is_store_identifier_continue(*byte))
            {
                name_end += 1;
            }
            let store_name = &expression[name_start..name_end];
            let Some(binding) = store_binding(store_name, bindings) else {
                expression_cursor = name_end;
                continue;
            };
            let line = line_of(file_source, block.start_offset + open + 1 + dollar);
            let key = format!("{file_path}|${store_name}|{line}|{}", block.kind);
            if seen.insert(key) {
                conventions.push(svelte_store_subscription(
                    file_path,
                    store_name,
                    binding,
                    line,
                    &block.kind,
                ));
            }
            expression_cursor = name_end;
        }
        cursor = close + 1;
    }
}

fn is_store_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_store_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

#[allow(
    clippy::too_many_arguments,
    reason = "wire-row fields stay explicit at the SFC projection boundary"
)]
fn template_record(
    file_path: &str,
    tag_name: &str,
    normalized_tag_name: &str,
    binding: &ComponentBinding,
    line: usize,
    block_kind: &str,
    language: SfcLanguage,
    template_kind: &'static str,
    status: &'static str,
    reason: Option<&'static str>,
    member_name: Option<String>,
) -> SfcTemplateComponentRef {
    SfcTemplateComponentRef {
        consumer_file: file_path.to_string(),
        tag_name: tag_name.to_string(),
        normalized_tag_name: normalized_tag_name.to_string(),
        binding_name: binding.binding_name.clone(),
        binding_source: binding.binding_source.clone(),
        from_spec: binding.binding_source.clone(),
        binding_kind: binding.binding_kind.clone(),
        imported_name: binding.imported_name.clone(),
        source: "sfc-template-component-ref",
        language: language.as_str(),
        template_kind,
        confidence: if status == "muted" {
            "muted-review"
        } else {
            "binding-review"
        },
        eligible_for_fan_in: false,
        eligible_for_safe_fix: false,
        status,
        reason,
        line,
        sfc_block_kind: block_kind.to_string(),
        member_name,
    }
}

fn dynamic_binding_name(tag_name: &str, attrs: &str) -> Option<String> {
    if tag_name.eq_ignore_ascii_case("component") {
        return quoted_attribute_value(attrs, ":is")
            .or_else(|| quoted_attribute_value(attrs, "v-bind:is"))
            .filter(|value| is_identifier(value));
    }
    if tag_name.eq_ignore_ascii_case("svelte:component") {
        let value = attribute_value(attrs, "this")?;
        let value = value.trim();
        let value = value.strip_prefix('{')?.strip_suffix('}')?.trim();
        return is_identifier(value).then(|| value.to_string());
    }
    None
}

fn template_tag_candidates(tag_name: &str) -> Vec<String> {
    if is_pascal_tag(tag_name) {
        return vec![tag_name.to_string()];
    }
    pascal_from_kebab(tag_name).map_or_else(Vec::new, |pascal| vec![pascal, tag_name.to_string()])
}

fn pascal_from_kebab(value: &str) -> Option<String> {
    let mut parts = value.split('-');
    let first = parts.next()?;
    if first.is_empty()
        || !first.starts_with(|character: char| character.is_ascii_lowercase())
        || !first
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return None;
    }
    let rest: Vec<&str> = parts.collect();
    if rest.is_empty()
        || rest.iter().any(|part| {
            part.is_empty()
                || !part
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
    {
        return None;
    }
    let mut out = capitalize(first);
    for part in rest {
        out.push_str(&capitalize(part));
    }
    Some(out)
}

fn capitalize(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    let mut out = first.to_ascii_uppercase().to_string();
    out.extend(characters);
    out
}

fn is_pascal_tag(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '_' | '$'))
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '$'))
}

fn is_tag_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | ':' | '-')
}
