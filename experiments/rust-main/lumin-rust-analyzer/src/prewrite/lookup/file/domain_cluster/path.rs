pub(super) fn posix_basename(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(_, basename)| basename)
        .unwrap_or(path)
}

pub(super) fn posix_dirname(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(dir, _)| if dir.is_empty() { "." } else { dir })
        .unwrap_or(".")
}
