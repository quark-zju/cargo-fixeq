//! Parse `cargo test` output.

use regex::Regex;
use std::sync::OnceLock;

/// Find `assert_eq!` failures from `cargo test` output.
pub(crate) fn find_assert_eq_failures(output: &str) -> Vec<AssertEqFailure> {
    let mut lhs: Option<String> = None;
    output
        .lines()
        .filter_map(move |line| {
            if let Some(captures) = assert_eq_failure_left_re().captures(line) {
                lhs = captures.get(1).map(|m| m.as_str().to_string());
            } else if let Some(captures) = assert_eq_failure_right_re().captures(line) {
                if let Some(actual) = &lhs {
                    let actual = actual.clone();
                    lhs = None;
                    let path = captures.get(1).map(|m| m.as_str().to_string());
                    let row = captures
                        .get(2)
                        .and_then(|s| s.as_str().parse::<usize>().ok());
                    if let (Some(path), Some(line)) = (path, row) {
                        return Some(AssertEqFailure { actual, line, path });
                    }
                }
            }
            None
        })
        .collect()
}

#[derive(Debug)]
pub(crate) struct AssertEqFailure {
    pub(crate) actual: String,
    pub(crate) line: usize,
    pub(crate) path: String,
}

fn assert_eq_failure_left_re() -> &'static Regex {
    static ASSERT_EQ_FAILURE_LEFT_RE: OnceLock<Regex> = OnceLock::new();
    ASSERT_EQ_FAILURE_LEFT_RE.get_or_init(|| Regex::new(r"^  left: `(.*)`,$").unwrap())
}

fn assert_eq_failure_right_re() -> &'static Regex {
    static ASSERT_EQ_FAILURE_RIGHT_RE: OnceLock<Regex> = OnceLock::new();
    ASSERT_EQ_FAILURE_RIGHT_RE.get_or_init(|| Regex::new(r"^ right: .*, (.*):(\d+):\d+$").unwrap())
}
