//! Apply fixes to source code files.

use crate::parse_code::{self, AssertEqLocation, Location};
use crate::parse_out::AssertEqFailure;
use anyhow::Context;
use std::{
    borrow::Cow,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

/// Attempt to fix failures. Return count of fixes applied.
pub(crate) fn fix(failures: Vec<AssertEqFailure>) -> anyhow::Result<usize> {
    let mut assert_eqs_by_path = HashMap::<PathBuf, Vec<AssertEqLocation>>::new();
    let mut content_by_path = HashMap::<PathBuf, String>::new();
    let mut fixes_by_path = HashMap::<PathBuf, Vec<Fix>>::new();

    let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let crate_root = Path::new(&crate_root);

    for failure in failures {
        let path = crate_root.join(&failure.path);
        if !content_by_path.contains_key(&path) {
            let content =
                fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
            content_by_path.insert(path.clone(), content);
        }
        let assert_eqs = assert_eqs_by_path.entry(path.clone()).or_insert_with(|| {
            let code = &content_by_path[&path];
            parse_code::find_assert_eqs(&code)
        });
        if let Some(assert_eq) = assert_eqs
            .iter()
            .find(|a| a.assert.overlaps_line(failure.line))
        {
            let fix = Fix {
                location: assert_eq.rhs.clone(),
                content: failure.actual,
            };
            fixes_by_path.entry(path.clone()).or_default().push(fix);
        }
    }

    let mut count = 0;
    for (path, fixes) in fixes_by_path.into_iter() {
        count += fixes.len();
        let content = &content_by_path[&path];
        let new_content = apply_fixes(content, fixes);
        fs::write(&path, new_content).with_context(|| format!("writing {:?}", path))?;
    }

    Ok(count)
}

/// Apply fixes to code.
pub(crate) fn apply_fixes(code: &str, mut fixes: Vec<Fix>) -> String {
    let mut lines = code.lines().map(|s| format!("{}\n", s)).collect::<Vec<_>>();
    fixes.sort_unstable_by(|lhs, rhs| rhs.location.start.line.cmp(&lhs.location.start.line));
    for fix in fixes {
        let loc = fix.location;
        // loc uses 1-based index.
        for i in loc.start.line.max(1)..=loc.end.line {
            if let Some(line) = lines.get(i - 1) {
                let mut new_line = String::new();
                if i == loc.start.line {
                    new_line += &line.chars().take(loc.start.column).collect::<String>();
                    new_line += &normalize_multi_line(&fix.content);
                }
                if i == loc.end.line {
                    new_line += &line.chars().skip(loc.end.column).collect::<String>();
                }
                lines[i - 1] = new_line;
            }
        }
    }
    lines.concat()
}

/// Replace code at `location` with `content`.
pub(crate) struct Fix {
    pub(crate) location: Location,
    pub(crate) content: String,
}

fn normalize_multi_line(s: &str) -> Cow<str> {
    if s.ends_with('\"') && s.contains(r"\n") && s.len() > 80 {
        let mut unescaped = String::with_capacity(s.len());
        let mut escaped = false;
        let mut giveup = false;
        for ch in s.chars() {
            match (ch, escaped) {
                ('\\', false) => {
                    escaped = true;
                }
                ('n', true) => {
                    unescaped.push('\n');
                    escaped = false;
                }
                ('r', true) => {
                    unescaped.push('\r');
                    escaped = false;
                }
                ('t', true) => {
                    unescaped.push('\t');
                    escaped = false;
                }
                ('\'', true) | ('\"', true) | ('\\', true) => {
                    unescaped.push(ch);
                    escaped = false;
                }
                (ch, false) => {
                    unescaped.push(ch);
                    escaped = false;
                }
                (_, true) => {
                    // Unsupported escape sequences, like \x, \u, \0.
                    giveup = true;
                    break;
                }
            }
        }
        if !giveup {
            let mut pound = "#".to_string();
            while unescaped.contains(&format!("{}\"", &pound)) {
                pound.push('#')
            }
            // Rewrite "a\nb" to
            // r#"a
            // b"#
            return Cow::Owned(format!("r{}{}{}", &pound, unescaped, &pound));
        }
    }
    Cow::Borrowed(s)
}
