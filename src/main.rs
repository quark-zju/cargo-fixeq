//! Run `cargo test` and fix `assert_eq!`s.

mod fix;
mod parse_code;
mod parse_out;

use std::{
    env,
    io::{self, Write},
    process::{self, Command},
    str,
};
use anyhow::{Context};

fn main() -> anyhow::Result<()> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let exitcode = loop {
        eprint!("Running tests...");
        let output = Command::new(&cargo)
            .arg("test")
            .args(env::args_os().skip(1).skip_while(|s| s == "fixeq"))
            .output().context("running tests")?;

        let forward_output = || -> anyhow::Result<()> {
            eprintln!("Last 'cargo test' output:");
            io::stderr().flush().context("flushing stderr")?;
            io::stdout().write_all(&output.stdout).context("forwarding test stdout")?;
            io::stderr().write_all(&output.stderr).context("forwarding test stderr")?;
            Ok(())
        };

        if output.status.success() {
            eprintln!(" succeeded.");
            forward_output().context("reporting success")?;
            break 0;
        }

        let out = str::from_utf8(&output.stdout).unwrap_or("");
        let failures = parse_out::find_assert_eq_failures(out);
        let count = fix::fix(failures).context("fixing failures")?;

        if count == 0 {
            eprintln!(" failed.");
            forward_output().context("reporting failure")?;
            break output.status.code().unwrap_or(0);
        } else {
            eprintln!(" fixed {} assert_eq!s.", count);
        }
    };
    process::exit(exitcode);
}
