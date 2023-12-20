//! Parse `cargo test` output (Rust >= 1.73 format).

/// Find `assert_eq!` failures from `cargo test` output.
pub(crate) fn find_assert_eq_failures(output: &str) -> Vec<AssertEqFailure> {
    // Example:
    // thread 'main' panicked at $DIR/main.rs:6:5:
    // assertion `left == right` failed: my custom message value
    //   left: 1
    //  right: 2
    // See https://github.com/rust-lang/rust/pull/111071
    // https://github.com/rust-lang/rust/commit/950e3d9989c6ebbf7b43961e4268bd3e403a84bb
    let lines: Vec<&str> = output.lines().collect();
    (0..lines.len())
        .filter_map(|i| maybe_extract_failure(&lines, i))
        .collect()
}

fn maybe_extract_failure(lines: &[&str], i: usize) -> Option<AssertEqFailure> {
    let path_line_col = lines
        .get(i)?
        .split_once(" panicked at ")?
        .1
        .strip_suffix(":")?;
    let _ = lines
        .get(i + 1)?
        .strip_prefix("assertion `left == right` failed")?;
    let actual = lines.get(i + 2)?.strip_prefix("  left: ")?;
    let _ = lines.get(i + 3)?.strip_prefix(" right: ")?;
    let (path_line, col) = path_line_col.rsplit_once(':')?;
    let (path, line) = path_line.rsplit_once(':')?;
    let line: usize = line.parse().ok()?;
    let col: usize = col.parse().ok()?;
    Some(AssertEqFailure {
        actual: actual.to_string(),
        path: path.to_string(),
        line,
        col,
    })
}

#[derive(Debug)]
pub(crate) struct AssertEqFailure {
    pub(crate) actual: String,
    pub(crate) path: String,
    pub(crate) line: usize,
    pub(crate) col: usize,
}
