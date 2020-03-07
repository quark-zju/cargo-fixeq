use anyhow::Result;
use std::fs;

/// Fix `code` that matches `func`. Return the fixed code.
/// For testing purpose only.
fn fix_code(code: &str) -> Result<String> {
    let orig_cwd = std::env::current_dir()?;
    let orig_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();

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

    std::env::set_current_dir(dir.path())?;
    std::env::set_var("CARGO_MANIFEST_DIR", dir.path());

    crate::main_with_args(vec!["--lib"])?;

    // Restore environment.
    std::env::set_current_dir(orig_cwd)?;
    std::env::set_var("CARGO_MANIFEST_DIR", orig_manifest_dir);

    let new_content = fs::read_to_string(&lib_path)?;
    Ok(new_content)
}

#[test]
fn test_fixeq() {
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

    // FIXME: assert_eq! message is dropped
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
    fn assert_with_msg() { assert_eq!(format!("foo {}", 1), "foo 1"); }
"#
    );
}
