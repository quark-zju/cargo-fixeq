use anyhow::Result;
use std::fs;

/// Fix `code` that matches `func`. Return the fixed code.
/// For testing purpose only.
fn fix_code(code: &str) -> Result<String> {
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("Cargo.toml"),
        r#"
[package]
name = "foo"
version = "0.1.0"
[lib]
path = "example.rs""#,
    )?;

    let lib_path = dir.path().join("example.rs");
    fs::write(&lib_path, code)?;

    let target_dir = dir.path().join("target");
    crate::main_with_args(vec!["--lib"], Some(dir.path()), Some(&target_dir))?;

    let new_content = fs::read_to_string(&lib_path)?;
    Ok(new_content)
}

#[test]
fn test_fixeq_multi_tests() {
    assert_eq!(
        fix_code(
            r#"
    #[test]
    fn plus1() { assert_eq!(1 + 2, 0); }
    #[test]
    fn plus2() {
        assert_eq!(3 + 4, 0);
        assert_eq!(3 + 4 + 5, 0);
    }
"#
        )
        .unwrap(),
        r#"
    #[test]
    fn plus1() { assert_eq!(1 + 2, 3); }
    #[test]
    fn plus2() {
        assert_eq!(3 + 4, 7);
        assert_eq!(3 + 4 + 5, 12);
    }
"#
    );
}

#[test]
fn test_fixeq_long_message() {
    assert_eq!(
        fix_code(
            r#"
    #[test]
    fn long_message() {
        assert_eq!(
            (1..8)
                .map(|i| i.to_string().repeat(20))
                .collect::<Vec<_>>()
                .join("\n"),
            "x"
        );
    }
"#
        )
        .unwrap(),
        r##"
    #[test]
    fn long_message() {
        assert_eq!(
            (1..8)
                .map(|i| i.to_string().repeat(20))
                .collect::<Vec<_>>()
                .join("\n"),
            r#"11111111111111111111
22222222222222222222
33333333333333333333
44444444444444444444
55555555555555555555
66666666666666666666
77777777777777777777"#
        );
    }
"##
    );
}

#[test]
fn test_fixeq_with_message() {
    assert_eq!(
        fix_code(
            r#"
    #[test]
    fn assert_with_msg() { assert_eq!(format!("foo {}", 1), "bar", "fmt {}", 2); }
"#
        )
        .unwrap(),
        r#"
    #[test]
    fn assert_with_msg() { assert_eq!(format!("foo {}", 1), "foo 1", "fmt {}", 2); }
"#
    );
}

#[test]
fn test_fixeq_same_line() {
    assert_eq!(
        fix_code("#[test] fn f() { assert_eq!(1, 2); assert_eq!('3', '4'); }\n").unwrap(),
        "#[test] fn f() { assert_eq!(1, 1); assert_eq!('3', '3'); }\n",
    );
}

#[test]
fn test_fixeq_macro_rhs() {
    assert_eq!(
        fix_code("#[test] fn f() { assert_eq!((0..3).collect::<Vec<u8>>(), vec![0]); }\n").unwrap(),
        "#[test] fn f() { assert_eq!((0..3).collect::<Vec<u8>>(), [0, 1, 2]); }\n",
    );
}

#[test]
fn test_fixeq_macro_rhs_with_message() {
    assert_eq!(
        fix_code("#[test] fn f() { assert_eq!((0..3).collect::<Vec<u8>>(), vec![0], \"m\"); }\n").unwrap(),
        "#[test] fn f() { assert_eq!((0..3).collect::<Vec<u8>>(), [0, 1, 2], \"m\"); }\n",
    );
}
