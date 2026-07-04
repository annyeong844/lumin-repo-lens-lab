use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DISCIPLINE_REQUEST_SCHEMA_VERSION: &str = "lumin-discipline-producer-request.v1";

const DISCIPLINE_NOTE: &str =
    "Regex-based. Subject to false positives from comments/strings. See references/false-positive-patterns.md.";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineArtifact {
    pub meta: DisciplineMeta,
    pub scanned_files: usize,
    pub unreadable_files: usize,
    pub total_lines: usize,
    pub totals: BTreeMap<String, usize>,
    pub rates_per_file: BTreeMap<String, Option<f64>>,
    pub rates_per_k_loc: BTreeMap<String, Option<f64>>,
    pub top_offenders: BTreeMap<String, Vec<PatternOffender>>,
    pub overall_top_offenders: Vec<FileViolationIndex>,
}

#[derive(Debug, Serialize)]
pub struct DisciplineMeta {
    pub generated: String,
    pub root: String,
    pub tool: &'static str,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PatternOffender {
    pub file: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileViolationIndex {
    pub file: String,
    pub total: usize,
    pub breakdown: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisciplineLang {
    Ts,
    Py,
    Go,
}

#[derive(Debug, Clone, Copy)]
struct PatternSpec {
    name: &'static str,
    langs: &'static [DisciplineLang],
    counter: fn(&str) -> usize,
}

const TS: &[DisciplineLang] = &[DisciplineLang::Ts];
const PY: &[DisciplineLang] = &[DisciplineLang::Py];
const GO: &[DisciplineLang] = &[DisciplineLang::Go];
const ALL: &[DisciplineLang] = &[DisciplineLang::Ts, DisciplineLang::Py, DisciplineLang::Go];

const PATTERNS: &[PatternSpec] = &[
    PatternSpec {
        name: ":any",
        langs: TS,
        counter: count_colon_any,
    },
    PatternSpec {
        name: "as any",
        langs: TS,
        counter: |source| count_word_sequence(source, &["as", "any"], false),
    },
    PatternSpec {
        name: "as unknown as",
        langs: TS,
        counter: |source| count_word_sequence(source, &["as", "unknown", "as"], false),
    },
    PatternSpec {
        name: "@ts-ignore",
        langs: TS,
        counter: |source| count_substring(source, "@ts-ignore"),
    },
    PatternSpec {
        name: "@ts-expect-error",
        langs: TS,
        counter: |source| count_substring(source, "@ts-expect-error"),
    },
    PatternSpec {
        name: "@ts-nocheck",
        langs: TS,
        counter: |source| count_substring(source, "@ts-nocheck"),
    },
    PatternSpec {
        name: "eslint-disable",
        langs: TS,
        counter: |source| count_substring(source, "eslint-disable"),
    },
    PatternSpec {
        name: "Function constructor",
        langs: TS,
        counter: count_new_function,
    },
    PatternSpec {
        name: "# type: ignore",
        langs: PY,
        counter: |source| count_hash_marker(source, "type:", "ignore", true),
    },
    PatternSpec {
        name: "# pyright: ignore",
        langs: PY,
        counter: |source| count_hash_marker(source, "pyright:", "ignore", true),
    },
    PatternSpec {
        name: "# pylint: disable",
        langs: PY,
        counter: |source| count_hash_marker(source, "pylint:", "disable", true),
    },
    PatternSpec {
        name: "# noqa",
        langs: PY,
        counter: |source| count_hash_marker(source, "noqa", "", true),
    },
    PatternSpec {
        name: "eval(",
        langs: PY,
        counter: |source| count_word_followed_by_open_paren(source, "eval"),
    },
    PatternSpec {
        name: "exec(",
        langs: PY,
        counter: |source| count_word_followed_by_open_paren(source, "exec"),
    },
    PatternSpec {
        name: "interface{}",
        langs: GO,
        counter: count_interface_empty,
    },
    PatternSpec {
        name: "panic(",
        langs: GO,
        counter: |source| count_word_followed_by_open_paren(source, "panic"),
    },
    PatternSpec {
        name: "unsafe.",
        langs: GO,
        counter: count_unsafe_member,
    },
    PatternSpec {
        name: "//nolint",
        langs: GO,
        counter: |source| count_slash_marker(source, "nolint"),
    },
    PatternSpec {
        name: "TODO",
        langs: ALL,
        counter: |source| count_word(source, "TODO"),
    },
    PatternSpec {
        name: "FIXME",
        langs: ALL,
        counter: |source| count_word(source, "FIXME"),
    },
    PatternSpec {
        name: "HACK",
        langs: ALL,
        counter: |source| count_word(source, "HACK"),
    },
    PatternSpec {
        name: "XXX",
        langs: ALL,
        counter: |source| count_word(source, "XXX"),
    },
];

pub fn build_discipline_artifact(request: DisciplineRequest) -> Result<DisciplineArtifact> {
    if request.schema_version != DISCIPLINE_REQUEST_SCHEMA_VERSION {
        bail!(
            "discipline-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let root = PathBuf::from(&request.root);
    let mut totals = empty_pattern_map();
    let mut by_file_distribution = empty_distribution_map();
    let mut file_violation_index = Vec::new();
    let mut source_total_lines = 0usize;
    let mut unreadable_files = 0usize;

    for raw_file in &request.files {
        let file = normalize_request_file(raw_file)?;
        let abs_path = root.join(&file);
        let source_bytes = match fs::read(&abs_path) {
            Ok(source) => source,
            Err(_) => {
                unreadable_files += 1;
                continue;
            }
        };
        let source = String::from_utf8_lossy(&source_bytes);
        source_total_lines += source_line_count(&source);
        let lang = lang_of(&file);
        let mut counts = BTreeMap::new();
        for pattern in PATTERNS {
            if !pattern.langs.contains(&lang) {
                continue;
            }
            let count = (pattern.counter)(&source);
            counts.insert(pattern.name.to_string(), count);
            *totals.entry(pattern.name.to_string()).or_default() += count;
            if count > 0 {
                by_file_distribution
                    .entry(pattern.name.to_string())
                    .or_default()
                    .push(PatternOffender {
                        file: path_to_slash_string(&file),
                        count,
                    });
            }
        }
        let total = counts.values().sum();
        if total > 0 {
            file_violation_index.push(FileViolationIndex {
                file: path_to_slash_string(&file),
                total,
                breakdown: counts,
            });
        }
    }

    for offenders in by_file_distribution.values_mut() {
        offenders.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.file.cmp(&right.file))
        });
    }

    file_violation_index.sort_by(|left, right| {
        right
            .total
            .cmp(&left.total)
            .then_with(|| left.file.cmp(&right.file))
    });

    let top_offenders = by_file_distribution
        .into_iter()
        .map(|(name, offenders)| (name, offenders.into_iter().take(10).collect()))
        .collect();

    Ok(DisciplineArtifact {
        meta: DisciplineMeta {
            generated: request.generated,
            root: request.root,
            tool: "measure-discipline.mjs",
            note: DISCIPLINE_NOTE,
        },
        scanned_files: request.files.len(),
        unreadable_files,
        total_lines: source_total_lines,
        rates_per_file: totals
            .iter()
            .map(|(name, count)| {
                (
                    name.clone(),
                    ratio(*count as f64, request.files.len() as f64, 3),
                )
            })
            .collect(),
        rates_per_k_loc: totals
            .iter()
            .map(|(name, count)| {
                (
                    name.clone(),
                    ratio((*count as f64) * 1000.0, source_total_lines as f64, 2),
                )
            })
            .collect(),
        totals,
        top_offenders,
        overall_top_offenders: file_violation_index.into_iter().take(20).collect(),
    })
}

fn empty_pattern_map() -> BTreeMap<String, usize> {
    PATTERNS
        .iter()
        .map(|pattern| (pattern.name.to_string(), 0))
        .collect()
}

fn empty_distribution_map() -> BTreeMap<String, Vec<PatternOffender>> {
    PATTERNS
        .iter()
        .map(|pattern| (pattern.name.to_string(), Vec::new()))
        .collect()
}

fn normalize_request_file(value: &str) -> Result<PathBuf> {
    let normalized = value.replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || looks_like_windows_absolute_path(&normalized)
    {
        bail!("discipline-artifact: invalid file path '{value}'");
    }
    let mut out = PathBuf::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => bail!("discipline-artifact: traversal is not accepted in '{value}'"),
            segment => out.push(segment),
        }
    }
    if out.as_os_str().is_empty() {
        bail!("discipline-artifact: invalid file path '{value}'");
    }
    Ok(out)
}

fn looks_like_windows_absolute_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

fn path_to_slash_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn lang_of(path: &Path) -> DisciplineLang {
    let text = path_to_slash_string(path);
    if text.ends_with(".py") {
        DisciplineLang::Py
    } else if text.ends_with(".go") {
        DisciplineLang::Go
    } else {
        DisciplineLang::Ts
    }
}

fn source_line_count(source: &str) -> usize {
    source
        .as_bytes()
        .iter()
        .filter(|byte| **byte == b'\n')
        .count()
        + 1
}

fn ratio(numerator: f64, denominator: f64, digits: i32) -> Option<f64> {
    if denominator == 0.0 {
        return None;
    }
    let factor = 10_f64.powi(digits);
    Some((numerator / denominator * factor).round() / factor)
}

fn count_substring(source: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    let mut count = 0;
    let mut offset = 0;
    while let Some(position) = source[offset..].find(needle) {
        count += 1;
        offset += position + needle.len();
    }
    count
}

fn count_colon_any(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b':' {
            i += 1;
            continue;
        }
        let mut cursor = i + 1;
        cursor = skip_ascii_whitespace(bytes, cursor);
        if bytes[cursor..].starts_with(b"any") && is_boundary(bytes, cursor + 3) {
            count += 1;
            i = cursor + 3;
        } else {
            i += 1;
        }
    }
    count
}

fn count_word_sequence(source: &str, words: &[&str], trailing_paren: bool) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        let Some(mut cursor) = match_word_at(bytes, i, words[0]) else {
            i += 1;
            continue;
        };
        let mut ok = true;
        for word in &words[1..] {
            let after_ws = skip_one_or_more_ascii_whitespace(bytes, cursor);
            let Some(after_word) =
                after_ws.and_then(|position| match_word_at(bytes, position, word))
            else {
                ok = false;
                break;
            };
            cursor = after_word;
        }
        if ok && trailing_paren {
            cursor = skip_ascii_whitespace(bytes, cursor);
            ok = bytes.get(cursor) == Some(&b'(');
            if ok {
                cursor += 1;
            }
        }
        if ok {
            count += 1;
            i = cursor;
        } else {
            i += 1;
        }
    }
    count
}

fn count_new_function(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        let Some(after_new) = match_word_at(bytes, i, "new") else {
            i += 1;
            continue;
        };
        let Some(after_ws) = skip_one_or_more_ascii_whitespace(bytes, after_new) else {
            i += 1;
            continue;
        };
        if bytes[after_ws..].starts_with(b"Function")
            && is_boundary(bytes, after_ws)
            && is_boundary(bytes, after_ws + "Function".len())
        {
            let cursor = skip_ascii_whitespace(bytes, after_ws + "Function".len());
            if bytes.get(cursor) == Some(&b'(') {
                count += 1;
                i = cursor + 1;
                continue;
            }
        }
        i += 1;
    }
    count
}

fn count_hash_marker(source: &str, first: &str, second: &str, require_boundary: bool) -> usize {
    let bytes = source.as_bytes();
    let first_bytes = first.as_bytes();
    let second_bytes = second.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        let mut cursor = skip_ascii_whitespace(bytes, i + 1);
        if !bytes[cursor..].starts_with(first_bytes) {
            i += 1;
            continue;
        }
        cursor += first_bytes.len();
        if !second.is_empty() {
            cursor = skip_ascii_whitespace(bytes, cursor);
            if !bytes[cursor..].starts_with(second_bytes) {
                i += 1;
                continue;
            }
            cursor += second_bytes.len();
        }
        if require_boundary && !is_boundary(bytes, cursor) {
            i += 1;
            continue;
        }
        count += 1;
        i = cursor;
    }
    count
}

fn count_word_followed_by_open_paren(source: &str, word: &str) -> usize {
    count_word_sequence(source, &[word], true)
}

fn count_interface_empty(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        let Some(mut cursor) = match_word_at(bytes, i, "interface") else {
            i += 1;
            continue;
        };
        cursor = skip_ascii_whitespace(bytes, cursor);
        if bytes.get(cursor) != Some(&b'{') {
            i += 1;
            continue;
        }
        cursor = skip_ascii_whitespace(bytes, cursor + 1);
        if bytes.get(cursor) != Some(&b'}') {
            i += 1;
            continue;
        }
        count += 1;
        i = cursor + 1;
    }
    count
}

fn count_unsafe_member(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        let Some(after_unsafe) = match_word_at(bytes, i, "unsafe") else {
            i += 1;
            continue;
        };
        let member_start = after_unsafe + 1;
        if bytes.get(after_unsafe) == Some(&b'.')
            && bytes
                .get(member_start)
                .is_some_and(|byte| is_word_byte(*byte))
        {
            count += 1;
            let mut cursor = member_start + 1;
            while bytes.get(cursor).is_some_and(|byte| is_word_byte(*byte)) {
                cursor += 1;
            }
            i = cursor;
        } else {
            i += 1;
        }
    }
    count
}

fn count_slash_marker(source: &str, marker: &str) -> usize {
    let bytes = source.as_bytes();
    let marker = marker.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] != b'/' || bytes[i + 1] != b'/' {
            i += 1;
            continue;
        }
        let cursor = skip_ascii_whitespace(bytes, i + 2);
        if bytes[cursor..].starts_with(marker) {
            count += 1;
            i = cursor + marker.len();
        } else {
            i += 1;
        }
    }
    count
}

fn count_word(source: &str, word: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if let Some(after) = match_word_at(bytes, i, word) {
            count += 1;
            i = after;
        } else {
            i += 1;
        }
    }
    count
}

fn match_word_at(bytes: &[u8], position: usize, word: &str) -> Option<usize> {
    let word = word.as_bytes();
    if !bytes[position..].starts_with(word) {
        return None;
    }
    let end = position + word.len();
    if is_boundary(bytes, position) && is_boundary(bytes, end) {
        Some(end)
    } else {
        None
    }
}

fn skip_ascii_whitespace(bytes: &[u8], mut position: usize) -> usize {
    while bytes.get(position).is_some_and(u8::is_ascii_whitespace) {
        position += 1;
    }
    position
}

fn skip_one_or_more_ascii_whitespace(bytes: &[u8], position: usize) -> Option<usize> {
    if !bytes.get(position).is_some_and(u8::is_ascii_whitespace) {
        return None;
    }
    Some(skip_ascii_whitespace(bytes, position))
}

fn is_boundary(bytes: &[u8], position: usize) -> bool {
    let before = position
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_some_and(|byte| is_word_byte(*byte));
    let after = bytes.get(position).is_some_and(|byte| is_word_byte(*byte));
    before != after
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use std::fs;
    use tempfile::tempdir;

    fn request(root: &Path, files: Vec<&str>) -> DisciplineRequest {
        DisciplineRequest {
            schema_version: DISCIPLINE_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-04T00:00:00.000Z".to_string(),
            root: root.to_string_lossy().to_string(),
            files: files.into_iter().map(ToOwned::to_owned).collect(),
        }
    }

    #[test]
    fn counts_language_specific_patterns_and_rates() -> Result<()> {
        let temp = tempdir()?;
        fs::create_dir_all(temp.path().join("src"))?;
        fs::write(
            temp.path().join("src/app.ts"),
            "const a: any = x as any;\n// TODO\nnew Function('x');\n",
        )?;
        fs::write(
            temp.path().join("src/tool.py"),
            "# type: ignore\nvalue = eval('1')\n# noqa\n",
        )?;
        fs::write(
            temp.path().join("src/main.go"),
            "var x interface{}\nfunc main(){ panic(\"x\"); unsafe.Pointer(nil) } //nolint\n",
        )?;

        let artifact = build_discipline_artifact(request(
            temp.path(),
            vec!["src/app.ts", "src/tool.py", "src/main.go"],
        ))?;

        assert_eq!(artifact.scanned_files, 3);
        assert_eq!(artifact.unreadable_files, 0);
        assert_eq!(artifact.totals[":any"], 1);
        assert_eq!(artifact.totals["as any"], 1);
        assert_eq!(artifact.totals["Function constructor"], 1);
        assert_eq!(artifact.totals["# type: ignore"], 1);
        assert_eq!(artifact.totals["# noqa"], 1);
        assert_eq!(artifact.totals["eval("], 1);
        assert_eq!(artifact.totals["interface{}"], 1);
        assert_eq!(artifact.totals["panic("], 1);
        assert_eq!(artifact.totals["unsafe."], 1);
        assert_eq!(artifact.totals["//nolint"], 1);
        assert_eq!(artifact.totals["TODO"], 1);
        assert_eq!(artifact.top_offenders["TODO"][0].file, "src/app.ts");
        Ok(())
    }

    #[test]
    fn unreadable_or_missing_files_are_counted_without_silent_success() -> Result<()> {
        let temp = tempdir()?;
        let artifact = build_discipline_artifact(request(temp.path(), vec!["missing.ts"]))?;

        assert_eq!(artifact.scanned_files, 1);
        assert_eq!(artifact.unreadable_files, 1);
        assert_eq!(artifact.total_lines, 0);
        assert_eq!(artifact.rates_per_k_loc[":any"], None);
        Ok(())
    }

    #[test]
    fn invalid_utf8_matches_node_lossy_utf8_reading() -> Result<()> {
        let temp = tempdir()?;
        fs::write(
            temp.path().join("invalid.ts"),
            [0xff, b':', b'a', b'n', b'y'],
        )?;

        let artifact = build_discipline_artifact(request(temp.path(), vec!["invalid.ts"]))?;

        assert_eq!(artifact.unreadable_files, 0);
        assert_eq!(artifact.totals[":any"], 1);
        Ok(())
    }

    #[test]
    fn rejects_paths_outside_the_requested_root() -> Result<()> {
        let temp = tempdir()?;
        let err = build_discipline_artifact(request(temp.path(), vec!["../outside.ts"]))
            .err()
            .context("traversal path should fail")?;
        assert!(err.to_string().contains("traversal"));
        Ok(())
    }
}
