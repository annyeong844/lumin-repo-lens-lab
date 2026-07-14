use super::blocks::{attribute_value, line_of, quoted_attribute_value, SfcLanguage, TemplateBlock};
use super::protocol::SfcTemplateComponentRef;
use super::script::{ComponentBinding, ComponentBindings};

pub(super) fn extract_template_component_refs(
    source: &str,
    file_path: &str,
    language: SfcLanguage,
    blocks: &[TemplateBlock<'_>],
    bindings: &ComponentBindings,
) -> Vec<SfcTemplateComponentRef> {
    let mut out = Vec::new();
    for block in blocks {
        parse_template_block(source, file_path, language, block, bindings, &mut out);
    }
    out
}

fn parse_template_block(
    file_source: &str,
    file_path: &str,
    language: SfcLanguage,
    block: &TemplateBlock<'_>,
    bindings: &ComponentBindings,
    out: &mut Vec<SfcTemplateComponentRef>,
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

        if let Some(dynamic_name) = dynamic_binding_name(tag_name, attrs) {
            if let Some(binding) = bindings
                .imports
                .get(&dynamic_name)
                .or_else(|| bindings.exposed_names.get(&dynamic_name))
            {
                out.push(template_record(
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
                    out.push(template_record(
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
            out.push(template_record(
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
