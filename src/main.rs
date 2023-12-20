//! Run `cargo test` and fix `assert_eq!`s.

mod fix;
mod parse_code;
mod parse_out;

#[cfg(test)]
mod tests;

use anyhow::Context;
use std::{
    env,
    ffi::OsStr,
    io::{self, Write},
    path::Path,
    process::{self, Command},
    str,
};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = env::args_os()
        .skip(1)
        .skip_while(|s| s == "fixeq")
        .collect();
    let exitcode = main_with_args(args, None, None)?;
    process::exit(exitcode);
}

pub(crate) fn main_with_args<S: AsRef<OsStr>>(
    args: impl IntoIterator<Item = S>,
    cwd: Option<&Path>,
    target_dir: Option<&Path>,
) -> anyhow::Result<i32> {
    let args: Vec<_> = args.into_iter().collect();
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let exitcode = loop {
        eprint!("Running tests...");
        let mut cargo_cmd = Command::new(&cargo);
        if let Some(cwd) = cwd {
            cargo_cmd.current_dir(cwd);
        }
        if let Some(target_dir) = target_dir {
            cargo_cmd.env("CARGO_TARGET_DIR", target_dir);
        }
        let output = cargo_cmd
            .arg("test")
            .args(&args)
            .output()
            .context("running tests")?;

        let forward_output = || -> anyhow::Result<()> {
            eprintln!("Last 'cargo test' output:");
            io::stderr().flush().context("flushing stderr")?;
            io::stdout()
                .write_all(&output.stdout)
                .context("forwarding test stdout")?;
            io::stderr()
                .write_all(&output.stderr)
                .context("forwarding test stderr")?;
            Ok(())
        };

        if output.status.success() {
            eprintln!(" succeeded.");
            forward_output().context("reporting success")?;
            break 0;
        }

        let out = str::from_utf8(&output.stdout).unwrap_or("");
        let failures = parse_out::find_assert_eq_failures(out);
        let count = fix::fix(failures, cwd).context("fixing failures")?;

        if count == 0 {
            eprintln!(" failed.");
            forward_output().context("reporting failure")?;
            break output.status.code().unwrap_or(0);
        } else {
            eprintln!(" fixed {} assert_eq!s.", count);
        }
    };
    Ok(exitcode)
}
