use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{BufRead, BufReader};
use std::ops::Range;
use std::path::{Component, Path};
use std::process::{Command, Stdio};
use std::time::Instant;

const MAX_GIT_PATH_ARG_CHARS: usize = 12 * 1024;
const MAX_BLAME_WORKERS: usize = 4;
const BLAME_WORKER_STACK_BYTES: usize = 1024 * 1024;
const COMMIT_MARKER: &[u8] = b"LUMIN-STALENESS-FILE-COMMIT:";

pub(super) struct FileHistoryResult {
    pub(super) file_touch_times: HashMap<String, Option<i64>>,
    pub(super) line_blame_times: HashMap<String, HashMap<i64, i64>>,
    pub(super) file_touch_files: i64,
    pub(super) file_touch_git_calls: i64,
    pub(super) file_touch_wall_ms: i64,
    pub(super) line_blame_files: i64,
    pub(super) line_blame_git_calls: i64,
    pub(super) line_blame_unavailable_files: i64,
    pub(super) line_blame_cache_hits: i64,
    pub(super) line_blame_cache_misses: i64,
    pub(super) line_blame_worker_count: i64,
    pub(super) line_blame_wall_ms: i64,
}

pub(super) fn collect_file_history(
    root: &Path,
    candidate_lines_by_file: &BTreeMap<String, Vec<i64>>,
) -> Result<FileHistoryResult> {
    let files = candidate_lines_by_file.keys().cloned().collect::<Vec<_>>();
    let file_touch_started = Instant::now();
    let (file_touch_times, file_touch_git_calls) = collect_file_touches(root, &files)?;
    let file_touch_wall_ms = file_touch_started.elapsed().as_millis() as i64;
    let tracked = candidate_lines_by_file
        .iter()
        .filter(|(file, _)| {
            file_touch_times.get(*file).copied().flatten().is_some()
                && is_current_in_root_file(root, file)
        })
        .collect::<Vec<_>>();
    let line_blame_unavailable_files = candidate_lines_by_file.len() - tracked.len();
    let worker_count = tracked
        .len()
        .min(MAX_BLAME_WORKERS)
        .min(std::thread::available_parallelism().map_or(1, usize::from));
    let line_blame_started = Instant::now();
    let line_blame_times = if worker_count == 0 {
        HashMap::new()
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(worker_count)
            .stack_size(BLAME_WORKER_STACK_BYTES)
            .thread_name(|index| format!("lumin-staleness-blame-{index}"))
            .build()
            .context("staleness-artifact: failed to build local Git blame pool")?;
        let results = pool.install(|| {
            tracked
                .par_iter()
                .map(|(file, lines)| blame_file(root, file, lines))
                .collect::<Vec<_>>()
        });
        let mut times = HashMap::with_capacity(results.len());
        for result in results {
            let (file, line_times) = result?;
            times.insert(file, line_times);
        }
        times
    };
    let line_blame_wall_ms = line_blame_started.elapsed().as_millis() as i64;
    let line_blame_cache_misses =
        tracked.iter().filter(|(_, lines)| lines.len() > 1).count() as i64;
    let line_blame_cache_hits = tracked
        .iter()
        .map(|(_, lines)| lines.len().saturating_sub(1) as i64)
        .sum();

    Ok(FileHistoryResult {
        file_touch_times,
        line_blame_times,
        file_touch_files: files.len() as i64,
        file_touch_git_calls,
        file_touch_wall_ms,
        line_blame_files: tracked.len() as i64,
        line_blame_git_calls: tracked.len() as i64,
        line_blame_unavailable_files: line_blame_unavailable_files as i64,
        line_blame_cache_hits,
        line_blame_cache_misses,
        line_blame_worker_count: worker_count as i64,
        line_blame_wall_ms,
    })
}

fn is_current_in_root_file(root: &Path, file: &str) -> bool {
    let path = Path::new(file);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return false;
    }
    root.join(path).is_file()
}

fn collect_file_touches(
    root: &Path,
    files: &[String],
) -> Result<(HashMap<String, Option<i64>>, i64)> {
    let ranges = path_chunk_ranges(files);
    let mut times = HashMap::with_capacity(files.len());
    for file in files {
        times.insert(file.clone(), None);
    }
    for range in &ranges {
        collect_file_touch_chunk(root, &files[range.clone()], &mut times)?;
    }
    Ok((times, ranges.len() as i64))
}

fn path_chunk_ranges(paths: &[String]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut argument_chars = 0;
    for (index, path) in paths.iter().enumerate() {
        let next_chars = path.len() + 1;
        if index > start && argument_chars + next_chars > MAX_GIT_PATH_ARG_CHARS {
            ranges.push(start..index);
            start = index;
            argument_chars = 0;
        }
        argument_chars += next_chars;
    }
    if start < paths.len() {
        ranges.push(start..paths.len());
    }
    ranges
}

fn collect_file_touch_chunk(
    root: &Path,
    files: &[String],
    times: &mut HashMap<String, Option<i64>>,
) -> Result<()> {
    let mut command = Command::new("git");
    command
        .args(["-c", "core.quotePath=false", "log"])
        .arg("--format=%x00LUMIN-STALENESS-FILE-COMMIT:%at%x00")
        .args([
            "--name-only",
            "-z",
            "--no-renames",
            "--no-show-signature",
            "--",
        ])
        .args(files)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = command
        .spawn()
        .context("staleness-artifact: failed to start batched file-touch Git log")?;
    let stdout = child
        .stdout
        .take()
        .context("staleness-artifact: batched file-touch stdout unavailable")?;
    let complete = parse_file_touch_stream(BufReader::new(stdout), files, times)
        .context("staleness-artifact: failed to read batched file-touch Git log")?;
    let status = if complete {
        match child
            .try_wait()
            .context("staleness-artifact: failed to poll completed file-touch Git log")?
        {
            Some(status) => status,
            None => {
                child
                    .kill()
                    .context("staleness-artifact: failed to stop completed file-touch Git log")?;
                child
                    .wait()
                    .context("staleness-artifact: failed to reap completed file-touch Git log")?
            }
        }
    } else {
        child
            .wait()
            .context("staleness-artifact: failed to wait for batched file-touch Git log")?
    };
    if !complete && !status.success() {
        bail!("staleness-artifact: batched file-touch Git log failed with status {status}");
    }
    Ok(())
}

fn parse_file_touch_stream(
    mut reader: impl BufRead,
    files: &[String],
    times: &mut HashMap<String, Option<i64>>,
) -> std::io::Result<bool> {
    let expected = files.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut remaining = expected.len();
    let mut current_ts = None;
    let mut token = Vec::new();
    while reader.read_until(0, &mut token)? > 0 {
        if token.last() == Some(&0) {
            token.pop();
        }
        if let Some(marker) = token.strip_prefix(COMMIT_MARKER) {
            current_ts = std::str::from_utf8(marker)
                .ok()
                .and_then(|text| text.parse().ok());
        } else if let Some(timestamp) = current_ts {
            let path = token.strip_prefix(b"\n").unwrap_or(&token);
            if let Ok(path) = std::str::from_utf8(path) {
                if expected.contains(path) && times.get(path).is_some_and(Option::is_none) {
                    times.insert(path.to_string(), Some(timestamp));
                    remaining -= 1;
                    if remaining == 0 {
                        return Ok(true);
                    }
                }
            }
        }
        token.clear();
    }
    Ok(remaining == 0)
}

fn blame_file(root: &Path, file: &str, lines: &[i64]) -> Result<(String, HashMap<i64, i64>)> {
    let mut command = Command::new("git");
    command.args(["blame", "--line-porcelain"]);
    if lines.len() <= 1 {
        let line = lines.first().copied().filter(|line| *line > 0).unwrap_or(1);
        command.args(["-L", &format!("{line},{line}")]);
    }
    let output = command
        .args(["--", file])
        .current_dir(root)
        .output()
        .with_context(|| format!("staleness-artifact: failed to start Git blame for {file}"))?;
    if !output.status.success() {
        bail!(
            "staleness-artifact: Git blame failed for {file}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok((
        file.to_string(),
        parse_line_blame_times(&String::from_utf8_lossy(&output.stdout)),
    ))
}

fn parse_line_blame_times(out: &str) -> HashMap<i64, i64> {
    let mut times = HashMap::new();
    let mut current_final_line = None;
    let mut current_author_time = None;
    for line in out.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if (parts.len() == 3 || parts.len() == 4)
            && is_blame_hash(parts[0])
            && parts[1].parse::<i64>().is_ok()
            && parts[2].parse::<i64>().is_ok()
        {
            current_final_line = parts[2].parse().ok();
            current_author_time = None;
            continue;
        }
        if let Some(rest) = line.strip_prefix("author-time ") {
            current_author_time = rest.parse().ok();
            continue;
        }
        if line.starts_with('\t') {
            if let (Some(final_line), Some(author_time)) = (current_final_line, current_author_time)
            {
                times.insert(final_line, author_time);
            }
            current_final_line = None;
            current_author_time = None;
        }
    }
    times
}

fn is_blame_hash(text: &str) -> bool {
    let len = text.len();
    (7..=64).contains(&len) && text.chars().all(|ch| ch == '^' || ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    use std::fs;

    #[test]
    fn path_chunks_cover_every_file() {
        let files = (0..2_000)
            .map(|index| format!("packages/package-{index:04}/src/index.ts"))
            .collect::<Vec<_>>();
        let ranges = path_chunk_ranges(&files);
        assert!(ranges.len() > 1);
        assert_eq!(ranges.first().map(|range| range.start), Some(0));
        assert_eq!(ranges.last().map(|range| range.end), Some(files.len()));
        for pair in ranges.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
    }

    #[test]
    fn batched_file_touches_and_blame_match_individual_git_results() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        git(root, &["init", "-q"], None)?;
        git(
            root,
            &["config", "user.email", "lumin@example.invalid"],
            None,
        )?;
        git(root, &["config", "user.name", "Lumin Test"], None)?;
        fs::write(root.join("alpha.ts"), "export const alphaName = 1;\n")?;
        fs::write(root.join("beta.ts"), "export const betaName = 1;\n")?;
        fs::write(root.join("old-name.ts"), "export const renamedName = 1;\n")?;
        fs::write(root.join("removed.ts"), "export const removedName = 1;\n")?;
        git(root, &["add", "."], None)?;
        git(
            root,
            &["commit", "-q", "-m", "initial"],
            Some("2024-01-01T00:00:00Z"),
        )?;
        fs::write(root.join("alpha.ts"), "export const alphaName = 2;\n")?;
        git(root, &["mv", "old-name.ts", "renamed.ts"], None)?;
        fs::remove_file(root.join("removed.ts"))?;
        git(root, &["add", "."], None)?;
        git(
            root,
            &["commit", "-q", "-m", "update alpha"],
            Some("2025-01-01T00:00:00Z"),
        )?;

        let candidates = BTreeMap::from([
            ("alpha.ts".to_string(), vec![1]),
            ("beta.ts".to_string(), vec![1]),
            ("renamed.ts".to_string(), vec![1]),
            ("removed.ts".to_string(), vec![1]),
        ]);
        let result = collect_file_history(root, &candidates)?;
        assert_eq!(result.file_touch_files, 4);
        assert_eq!(result.file_touch_git_calls, 1);
        assert_eq!(result.line_blame_files, 3);
        assert_eq!(result.line_blame_git_calls, 3);
        assert_eq!(result.line_blame_unavailable_files, 1);
        assert!((1..=3).contains(&result.line_blame_worker_count));
        for file in candidates.keys() {
            assert_eq!(
                result.file_touch_times.get(file).copied().flatten(),
                individual_file_touch(root, file)?,
                "file touch {file}"
            );
            if file == "removed.ts" {
                assert!(!result.line_blame_times.contains_key(file));
                continue;
            }
            assert_eq!(
                result
                    .line_blame_times
                    .get(file)
                    .and_then(|times| times.get(&1))
                    .copied(),
                individual_line_touch(root, file, 1)?,
                "line blame {file}"
            );
        }
        Ok(())
    }

    fn individual_file_touch(root: &Path, file: &str) -> Result<Option<i64>> {
        let output = git(
            root,
            &["log", "-1", "--format=%at", "--follow", "--", file],
            None,
        )?;
        Ok(output.trim().parse().ok())
    }

    fn individual_line_touch(root: &Path, file: &str, line: i64) -> Result<Option<i64>> {
        let range = format!("{line},{line}");
        let output = git(
            root,
            &["blame", "--line-porcelain", "-L", &range, "--", file],
            None,
        )?;
        Ok(parse_line_blame_times(&output).get(&line).copied())
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
