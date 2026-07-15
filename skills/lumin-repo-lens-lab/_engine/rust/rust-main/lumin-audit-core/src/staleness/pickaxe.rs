use aho_corasick::AhoCorasick;
use anyhow::{bail, Context, Result};
use std::collections::{BTreeSet, HashMap};
use std::io::{BufRead, BufReader};
use std::ops::Range;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

pub(super) const PICKAXE_MODE: &str = "batched-text-diff-count-v1";
const MIN_PICKAXE_SYMBOL_LEN: usize = 4;
const MAX_PICKAXE_REGEX_CHARS: usize = 12 * 1024;
const COMMIT_MARKER: &[u8] = b"LUMIN-STALENESS-COMMIT:";

#[derive(Debug, Clone)]
pub(super) struct Mention {
    pub(super) status: &'static str,
    pub(super) ts: Option<i64>,
}

pub(super) struct PickaxeResult {
    pub(super) mentions: HashMap<String, Mention>,
    pub(super) git_calls: i64,
    pub(super) eligible_symbols: i64,
    pub(super) patch_lines: i64,
    pub(super) wall_ms: i64,
}

pub(super) fn collect_symbol_mentions(
    root: &Path,
    since: &str,
    skip_pickaxe: bool,
    symbols: impl Iterator<Item = String>,
) -> Result<PickaxeResult> {
    let started = Instant::now();
    let symbols = symbols
        .filter(|symbol| is_pickaxe_eligible(symbol))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let eligible_symbols = symbols.len() as i64;
    if skip_pickaxe || symbols.is_empty() {
        return Ok(PickaxeResult {
            mentions: HashMap::new(),
            git_calls: 0,
            eligible_symbols,
            patch_lines: 0,
            wall_ms: started.elapsed().as_millis() as i64,
        });
    }

    let ranges = symbol_chunk_ranges(&symbols);
    let mut warm_timestamps = vec![None; symbols.len()];
    let mut patch_lines = 0;
    for range in &ranges {
        patch_lines += collect_chunk(
            root,
            since,
            &symbols[range.clone()],
            &mut warm_timestamps[range.clone()],
        )?;
    }

    let mentions = symbols
        .into_iter()
        .zip(warm_timestamps)
        .map(|(symbol, ts)| {
            let mention = match ts {
                Some(ts) => Mention {
                    status: "warm",
                    ts: Some(ts),
                },
                None => Mention {
                    status: "cold",
                    ts: None,
                },
            };
            (symbol, mention)
        })
        .collect();

    Ok(PickaxeResult {
        mentions,
        git_calls: ranges.len() as i64,
        eligible_symbols,
        patch_lines,
        wall_ms: started.elapsed().as_millis() as i64,
    })
}

pub(super) fn is_pickaxe_eligible(symbol: &str) -> bool {
    if symbol.len() < MIN_PICKAXE_SYMBOL_LEN {
        return false;
    }
    let mut chars = symbol.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_' || first == '$') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

fn symbol_chunk_ranges(symbols: &[String]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut regex_chars = 2;
    for (index, symbol) in symbols.iter().enumerate() {
        let separator = usize::from(index > start);
        let symbol_chars = escaped_regex_len(symbol);
        if index > start && regex_chars + separator + symbol_chars > MAX_PICKAXE_REGEX_CHARS {
            ranges.push(start..index);
            start = index;
            regex_chars = 2;
        }
        regex_chars += usize::from(index > start) + symbol_chars;
    }
    if start < symbols.len() {
        ranges.push(start..symbols.len());
    }
    ranges
}

fn escaped_regex_len(symbol: &str) -> usize {
    symbol.len() + symbol.bytes().filter(|byte| *byte == b'$').count() * 2
}

fn combined_regex(symbols: &[String]) -> String {
    let mut regex = String::from("(");
    for (index, symbol) in symbols.iter().enumerate() {
        if index > 0 {
            regex.push('|');
        }
        for ch in symbol.chars() {
            if ch == '$' {
                regex.push_str("[$]");
            } else {
                regex.push(ch);
            }
        }
    }
    regex.push(')');
    regex
}

fn collect_chunk(
    root: &Path,
    since: &str,
    symbols: &[String],
    warm_timestamps: &mut [Option<i64>],
) -> Result<i64> {
    let regex = combined_regex(symbols);
    let matcher = AhoCorasick::new(symbols.iter().map(String::as_bytes))
        .context("staleness-artifact: failed to build batched symbol matcher")?;
    let mut command = Command::new("git");
    command.arg("log");
    if !since.is_empty() {
        command.arg(format!("--since={since}"));
    }
    command
        .arg(format!("-G{regex}"))
        .args([
            "--format=LUMIN-STALENESS-COMMIT:%at",
            "--patch",
            "--unified=0",
            "--no-color",
            "--no-ext-diff",
            "--no-textconv",
            "--no-show-signature",
        ])
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = command
        .spawn()
        .context("staleness-artifact: failed to start batched Git pickaxe")?;
    let stdout = child
        .stdout
        .take()
        .context("staleness-artifact: batched Git pickaxe stdout unavailable")?;
    let parse_result = parse_patch_stream(BufReader::new(stdout), &matcher, warm_timestamps);
    let status = child
        .wait()
        .context("staleness-artifact: failed to wait for batched Git pickaxe")?;
    let patch_lines =
        parse_result.context("staleness-artifact: failed to read batched Git pickaxe")?;
    if !status.success() {
        bail!("staleness-artifact: batched Git pickaxe failed with status {status}");
    }
    Ok(patch_lines)
}

fn parse_patch_stream(
    mut reader: impl BufRead,
    matcher: &AhoCorasick,
    warm_timestamps: &mut [Option<i64>],
) -> std::io::Result<i64> {
    let mut line = Vec::new();
    let mut commit_ts = None;
    let mut deltas = vec![0_i64; warm_timestamps.len()];
    let mut match_ends = vec![0_usize; warm_timestamps.len()];
    let mut patch_lines = 0;
    while reader.read_until(b'\n', &mut line)? > 0 {
        trim_line_ending(&mut line);
        if let Some(ts) = parse_commit_marker(&line) {
            project_commit_mentions(commit_ts, &mut deltas, warm_timestamps);
            commit_ts = Some(ts);
            line.clear();
            continue;
        }
        let direction = if line.starts_with(b"+++ ") || line.starts_with(b"--- ") {
            0
        } else if line.first() == Some(&b'+') {
            1
        } else if line.first() == Some(&b'-') {
            -1
        } else {
            0
        };
        if direction != 0 && commit_ts.is_some() {
            patch_lines += 1;
            record_line_deltas(
                &line[1..],
                direction,
                matcher,
                warm_timestamps,
                &mut deltas,
                &mut match_ends,
            );
        }
        line.clear();
    }
    project_commit_mentions(commit_ts, &mut deltas, warm_timestamps);
    Ok(patch_lines)
}

fn trim_line_ending(line: &mut Vec<u8>) {
    while line
        .last()
        .is_some_and(|byte| matches!(byte, b'\r' | b'\n'))
    {
        line.pop();
    }
}

fn parse_commit_marker(line: &[u8]) -> Option<i64> {
    let timestamp = line.strip_prefix(COMMIT_MARKER)?;
    std::str::from_utf8(timestamp).ok()?.parse().ok()
}

fn record_line_deltas(
    line: &[u8],
    direction: i64,
    matcher: &AhoCorasick,
    warm_timestamps: &[Option<i64>],
    deltas: &mut [i64],
    match_ends: &mut [usize],
) {
    match_ends.fill(0);
    for found in matcher.find_overlapping_iter(line) {
        let index = found.pattern().as_usize();
        if warm_timestamps[index].is_some() || found.start() < match_ends[index] {
            continue;
        }
        deltas[index] += direction;
        match_ends[index] = found.end();
    }
}

fn project_commit_mentions(
    commit_ts: Option<i64>,
    deltas: &mut [i64],
    warm_timestamps: &mut [Option<i64>],
) {
    if let Some(commit_ts) = commit_ts {
        for (index, delta) in deltas.iter().enumerate() {
            if *delta != 0 && warm_timestamps[index].is_none() {
                warm_timestamps[index] = Some(commit_ts);
            }
        }
    }
    deltas.fill(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    use std::fs;

    #[test]
    fn chunks_cover_every_symbol_without_exceeding_transport_size() {
        let symbols = (0..2_000)
            .map(|index| format!("candidateSymbol{index:04}"))
            .collect::<Vec<_>>();
        let ranges = symbol_chunk_ranges(&symbols);
        assert!(ranges.len() > 1);
        assert_eq!(ranges.first().map(|range| range.start), Some(0));
        assert_eq!(ranges.last().map(|range| range.end), Some(symbols.len()));
        for pair in ranges.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
        for range in ranges {
            assert!(combined_regex(&symbols[range]).len() <= MAX_PICKAXE_REGEX_CHARS);
        }
    }

    #[test]
    fn counts_non_overlapping_occurrences_like_git_pickaxe() -> Result<()> {
        let matcher = AhoCorasick::new(["aaa", "name"])?;
        let warm = [None, None];
        let mut deltas = [0, 0];
        let mut match_ends = [0, 0];
        record_line_deltas(
            b"aaaa name name name",
            1,
            &matcher,
            &warm,
            &mut deltas,
            &mut match_ends,
        );
        assert_eq!(deltas, [1, 3]);
        Ok(())
    }

    #[test]
    fn batched_mentions_match_individual_git_pickaxe_results() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        git(root, &["init", "-q"], None)?;
        git(
            root,
            &["config", "user.email", "lumin@example.invalid"],
            None,
        )?;
        git(root, &["config", "user.name", "Lumin Test"], None)?;
        fs::write(
            root.join("source.ts"),
            concat!(
                "export const alphaName = 1;\n",
                "export const betaName = 1;\n",
                "export const stableName = 1;\n",
                "export const dollar$Name = 1;\n",
            ),
        )?;
        git(root, &["add", "."], None)?;
        git(
            root,
            &["commit", "-q", "-m", "initial"],
            Some("2024-01-01T00:00:00Z"),
        )?;

        fs::write(
            root.join("source.ts"),
            concat!(
                "export const alphaName = alphaName + 1;\n",
                "export const betaName = 1;\n",
                "export const stableName = 2;\n",
                "export const dollar$Name = dollar$Name + 1;\n",
            ),
        )?;
        git(root, &["add", "."], None)?;
        git(
            root,
            &["commit", "-q", "-m", "change counts"],
            Some("2025-01-01T00:00:00Z"),
        )?;

        let names = [
            "alphaName",
            "betaName",
            "stableName",
            "dollar$Name",
            "missingName",
        ];
        let result = collect_symbol_mentions(
            root,
            "5 years ago",
            false,
            names.into_iter().map(str::to_string),
        )?;
        assert_eq!(result.git_calls, 1);
        assert_eq!(result.eligible_symbols, names.len() as i64);
        for name in names {
            let expected = individual_pickaxe_timestamp(root, name)?;
            let actual = result.mentions.get(name).map(|mention| mention.ts);
            assert_eq!(actual, Some(expected), "symbol {name}");
        }
        Ok(())
    }

    fn individual_pickaxe_timestamp(root: &Path, symbol: &str) -> Result<Option<i64>> {
        let output = git(
            root,
            &[
                "log",
                "--since=5 years ago",
                &format!("-S{symbol}"),
                "--format=%at",
                "-1",
            ],
            None,
        )?;
        Ok(output.trim().parse().ok())
    }

    fn git(root: &Path, args: &[&str], date: Option<&str>) -> Result<String> {
        let mut command = Command::new("git");
        command.args(args).current_dir(root);
        if let Some(date) = date {
            command
                .env("GIT_AUTHOR_DATE", date)
                .env("GIT_COMMITTER_DATE", date);
        }
        let output = command.output()?;
        if !output.status.success() {
            bail!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
