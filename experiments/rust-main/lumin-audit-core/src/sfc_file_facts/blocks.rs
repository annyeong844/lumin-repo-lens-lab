use anyhow::{bail, Result};
use std::ops::Range;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SfcLanguage {
    Vue,
    Svelte,
    Astro,
}

impl SfcLanguage {
    pub(super) fn from_path(file_path: &str) -> Result<Self> {
        match Path::new(file_path)
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("vue") => Ok(Self::Vue),
            Some("svelte") => Ok(Self::Svelte),
            Some("astro") => Ok(Self::Astro),
            _ => bail!("sfc-file-facts-artifact: unsupported SFC file '{file_path}'"),
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Vue => "vue",
            Self::Svelte => "svelte",
            Self::Astro => "astro",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ScriptDialect {
    Ts,
    Tsx,
    Js,
    Jsx,
}

#[derive(Debug)]
pub(super) struct ScriptBlock<'a> {
    pub(super) content: &'a str,
    pub(super) start_offset: usize,
    pub(super) kind: String,
    pub(super) dialect: ScriptDialect,
}

#[derive(Debug)]
pub(super) struct StyleBlock<'a> {
    pub(super) content: &'a str,
    pub(super) start_offset: usize,
    pub(super) kind: String,
}

#[derive(Debug)]
pub(super) struct TemplateBlock<'a> {
    pub(super) content: &'a str,
    pub(super) start_offset: usize,
    pub(super) kind: String,
    pub(super) excluded_ranges: Vec<Range<usize>>,
}

#[derive(Debug)]
struct MarkupBlock<'a> {
    attrs: &'a str,
    content: &'a str,
    open_offset: usize,
    content_offset: usize,
    full_range: Range<usize>,
}

pub(super) fn line_of(source: &str, offset: usize) -> usize {
    source.as_bytes()[..offset.min(source.len())]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

pub(super) fn script_blocks<'a>(source: &'a str, language: SfcLanguage) -> Vec<ScriptBlock<'a>> {
    if language == SfcLanguage::Astro {
        return astro_frontmatter(source)
            .map(|(content, start_offset, _)| ScriptBlock {
                content,
                start_offset,
                kind: "astro-frontmatter".to_string(),
                dialect: ScriptDialect::Ts,
            })
            .into_iter()
            .collect();
    }

    markup_blocks(source, "script")
        .into_iter()
        .filter(|block| attribute_value(block.attrs, "src").is_none())
        .map(|block| ScriptBlock {
            content: block.content,
            start_offset: block.content_offset,
            kind: if language == SfcLanguage::Vue && contains_ascii_word(block.attrs, "setup") {
                "vue-script-setup".to_string()
            } else {
                format!("{}-script", language.as_str())
            },
            dialect: script_dialect(block.attrs),
        })
        .collect()
}

pub(super) fn script_source_blocks(
    source: &str,
    file_path: &str,
    language: SfcLanguage,
) -> Vec<super::protocol::SfcScriptSource> {
    if language == SfcLanguage::Astro {
        return Vec::new();
    }
    markup_blocks(source, "script")
        .into_iter()
        .filter_map(|block| {
            let from_spec = attribute_value(block.attrs, "src")?;
            is_relative_spec(&from_spec).then(|| super::protocol::SfcScriptSource {
                consumer_file: file_path.to_string(),
                from_spec,
                name: "*",
                kind: "sfc-script-src",
                type_only: false,
                line: line_of(source, block.open_offset),
                sfc_block_kind: format!("{}-script-src", language.as_str()),
                sfc_language: language.as_str(),
            })
        })
        .collect()
}

pub(super) fn style_blocks<'a>(source: &'a str, language: SfcLanguage) -> Vec<StyleBlock<'a>> {
    markup_blocks(source, "style")
        .into_iter()
        .map(|block| StyleBlock {
            content: block.content,
            start_offset: block.content_offset,
            kind: format!("{}-style", language.as_str()),
        })
        .collect()
}

pub(super) fn template_blocks<'a>(
    source: &'a str,
    language: SfcLanguage,
) -> Vec<TemplateBlock<'a>> {
    match language {
        SfcLanguage::Vue => markup_blocks(source, "template")
            .into_iter()
            .map(|block| TemplateBlock {
                content: block.content,
                start_offset: block.content_offset,
                kind: "vue-template".to_string(),
                excluded_ranges: Vec::new(),
            })
            .collect(),
        SfcLanguage::Svelte => {
            let excluded_ranges = markup_blocks(source, "script")
                .into_iter()
                .chain(markup_blocks(source, "style"))
                .map(|block| block.full_range)
                .collect();
            vec![TemplateBlock {
                content: source,
                start_offset: 0,
                kind: "svelte-template".to_string(),
                excluded_ranges,
            }]
        }
        SfcLanguage::Astro => {
            let start_offset =
                astro_frontmatter(source).map_or(0, |(_, _, template_start)| template_start);
            vec![TemplateBlock {
                content: &source[start_offset..],
                start_offset,
                kind: "astro-template".to_string(),
                excluded_ranges: Vec::new(),
            }]
        }
    }
}

pub(super) fn attribute_value(attrs: &str, target: &str) -> Option<String> {
    find_attribute_value(attrs, target).map(|(value, _)| value)
}

pub(super) fn quoted_attribute_value(attrs: &str, target: &str) -> Option<String> {
    find_attribute_value(attrs, target).and_then(|(value, quoted)| quoted.then_some(value))
}

fn find_attribute_value(attrs: &str, target: &str) -> Option<(String, bool)> {
    let bytes = attrs.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        index = skip_space(bytes, index);
        let start = index;
        while index < bytes.len() && is_attribute_name_byte(bytes[index]) {
            index += 1;
        }
        if start == index {
            index += 1;
            continue;
        }
        let name = &attrs[start..index];
        index = skip_space(bytes, index);
        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }
        index = skip_space(bytes, index + 1);
        let Some((value, end, quoted)) = parse_attribute_value(attrs, index) else {
            index = (index + 1).min(bytes.len());
            continue;
        };
        index = end;
        if name.eq_ignore_ascii_case(target) {
            return Some((value, quoted));
        }
    }
    None
}

pub(super) fn is_relative_spec(value: &str) -> bool {
    value.starts_with("./") || value.starts_with("../")
}

fn markup_blocks<'a>(source: &'a str, tag: &str) -> Vec<MarkupBlock<'a>> {
    let lower = source.to_ascii_lowercase();
    let open_prefix = format!("<{tag}");
    let close_tag = format!("</{tag}>");
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(relative_open) = lower[cursor..].find(&open_prefix) {
        let open = cursor + relative_open;
        let after_name = open + open_prefix.len();
        if lower
            .as_bytes()
            .get(after_name)
            .is_some_and(|byte| is_tag_name_byte(*byte))
        {
            cursor = after_name;
            continue;
        }
        let Some(relative_open_end) = lower[after_name..].find('>') else {
            break;
        };
        let open_end = after_name + relative_open_end;
        let content_offset = open_end + 1;
        let Some(relative_close) = lower[content_offset..].find(&close_tag) else {
            break;
        };
        let content_end = content_offset + relative_close;
        let full_end = content_end + close_tag.len();
        out.push(MarkupBlock {
            attrs: &source[after_name..open_end],
            content: &source[content_offset..content_end],
            open_offset: open,
            content_offset,
            full_range: open..full_end,
        });
        cursor = full_end;
    }
    out
}

fn astro_frontmatter(source: &str) -> Option<(&str, usize, usize)> {
    let open_len = if source.starts_with("---\r\n") {
        5
    } else if source.starts_with("---\n") {
        4
    } else {
        return None;
    };
    let mut line_start = open_len;
    while line_start <= source.len() {
        let line_end = source[line_start..]
            .find('\n')
            .map_or(source.len(), |offset| line_start + offset);
        let line = source[line_start..line_end]
            .trim_end_matches('\r')
            .trim_end();
        if line == "---" {
            let template_start = if line_end < source.len() {
                line_end + 1
            } else {
                line_end
            };
            return Some((&source[open_len..line_start], open_len, template_start));
        }
        if line_end == source.len() {
            break;
        }
        line_start = line_end + 1;
    }
    None
}

fn script_dialect(attrs: &str) -> ScriptDialect {
    match attribute_value(attrs, "lang")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "tsx" => ScriptDialect::Tsx,
        "jsx" => ScriptDialect::Jsx,
        "js" | "javascript" => ScriptDialect::Js,
        _ => ScriptDialect::Ts,
    }
}

fn contains_ascii_word(value: &str, target: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.match_indices(target).any(|(index, matched)| {
        let before = lower.as_bytes().get(index.wrapping_sub(1)).copied();
        let after = lower.as_bytes().get(index + matched.len()).copied();
        before.is_none_or(|byte| !is_ascii_word_byte(byte))
            && after.is_none_or(|byte| !is_ascii_word_byte(byte))
    })
}

fn is_ascii_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn parse_attribute_value(attrs: &str, index: usize) -> Option<(String, usize, bool)> {
    let bytes = attrs.as_bytes();
    let quote = *bytes.get(index)?;
    if quote == b'\'' || quote == b'"' {
        let rest = &attrs[index + 1..];
        let end = rest.find(quote as char)?;
        return Some((rest[..end].to_string(), index + end + 2, true));
    }
    if quote == b'{' {
        let rest = &attrs[index + 1..];
        let end = rest.find('}')?;
        return Some((
            attrs[index..index + end + 2].to_string(),
            index + end + 2,
            false,
        ));
    }
    let mut end = index;
    while end < bytes.len()
        && !bytes[end].is_ascii_whitespace()
        && !matches!(bytes[end], b'"' | b'\'' | b'=' | b'<' | b'>' | b'`')
    {
        end += 1;
    }
    (end > index).then(|| (attrs[index..end].to_string(), end, false))
}

fn skip_space(bytes: &[u8], mut index: usize) -> usize {
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    index
}

fn is_attribute_name_byte(byte: u8) -> bool {
    !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'<' | b'>' | b'"' | b'\'' | b'`')
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}
