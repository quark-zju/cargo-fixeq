//! Parse Rust source code.

use proc_macro2::{LineColumn, TokenTree};
use syn::{spanned::Spanned, visit::Visit, ExprMacro, Ident};

/// Find locations of `assert_eq!`s from source code.
pub(crate) fn find_assert_eqs(code: &str) -> Vec<AssertEqLocation> {
    let mut visitor = AssertEqVisitor::default();
    if let Ok(syntax_tree) = syn::parse_file(&code) {
        visitor.visit_file(&syntax_tree);
    }
    visitor.out
}

#[derive(Clone)]
pub(crate) struct Location {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

#[derive(Debug, Clone)]
pub(crate) struct AssertEqLocation {
    pub(crate) assert: Location,
    pub(crate) rhs: Location,
}

#[derive(Default)]
struct AssertEqVisitor {
    out: Vec<AssertEqLocation>,
}

impl<'ast> Visit<'ast> for AssertEqVisitor {
    fn visit_expr_macro(&mut self, i: &'ast ExprMacro) {
        let path = &i.mac.path;
        if path.is_ident(&Ident::new("assert_eq", path.span())) {
            let mut start = None;
            let mut end = None;
            let mut seen_comma = 0;
            for token in i.mac.tokens.clone() {
                if let TokenTree::Punct(ref p) = token {
                    if p.as_char() == ',' {
                        seen_comma += 1;
                        continue;
                    }
                }
                // assert_eq!(actual , expected , message, ...)
                //                   ^ ^      ^ ^
                //                   | start  | seen_comma=2
                //        seen_comma=1        end
                if seen_comma == 1 {
                    if start.is_none() && seen_comma == 1 {
                        start = Some(token.span().start());
                    }
                    if end.is_none() {
                        end = Some(token.span().end());
                    }
                }
            }
            if let (Some(start), Some(end)) = (start, end) {
                let rhs = Location { start, end };
                let assert = Location {
                    start: i.span().start(),
                    end: i.span().end(),
                };
                self.out.push(AssertEqLocation { assert, rhs });
            }
        }
    }
}

impl Location {
    pub(crate) fn overlaps_line_col(&self, line: usize, col: usize) -> bool {
        self.start.line <= line
            && self.end.line >= line
            && (self.start.line > line || self.start.column <= col)
            && (self.end.line < line || self.end.column >= col)
    }
}

use std::fmt;
impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{}-{},{}",
            self.start.line, self.start.column, self.end.line, self.end.column
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_assert_eqs() {
        assert_eq!(
            format!(
                "{:#?}",
                find_assert_eqs(
                    r#"
fn eq<T: Eq>(a: T, b: T) -> bool {
    a == b
}

fn main() {
    // single line
    assert_eq!(true, true);

    // multi-line
    assert_eq!(
        eq(1, 2),
        eq(
            eq(1, 2),
            eq(2, 2),
        ),
    );
}"#
                )
            ),
            r#"[
    AssertEqLocation {
        assert: 8,4-8,26,
        rhs: 8,21-8,25,
    },
    AssertEqLocation {
        assert: 11,4-17,5,
        rhs: 13,8-13,10,
    },
]"#
        );
    }
}
