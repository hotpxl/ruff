//! Generate CLI help.
#![allow(clippy::print_stdout)]

use std::path::PathBuf;
use std::{fs, str};

use anyhow::{bail, Context, Result};
use clap::CommandFactory;
use pretty_assertions::StrComparison;

use ruff_cli::args;

use crate::generate_all::{Mode, REGENERATE_ALL_COMMAND};
use crate::ROOT_DIR;

const COMMAND_HELP_BEGIN_PRAGMA: &str = "<!-- Begin auto-generated command help. -->\n";
const COMMAND_HELP_END_PRAGMA: &str = "<!-- End auto-generated command help. -->";

const SUBCOMMAND_HELP_BEGIN_PRAGMA: &str = "<!-- Begin auto-generated subcommand help. -->\n";
const SUBCOMMAND_HELP_END_PRAGMA: &str = "<!-- End auto-generated subcommand help. -->";

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, default_value_t, value_enum)]
    pub(crate) mode: Mode,
}

fn trim_lines(s: &str) -> String {
    s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n")
}

/// Takes the existing file contents, inserts the section, returns the transformed content
fn replace_docs_section(
    existing: &str,
    section: &str,
    begin_pragma: &str,
    end_pragma: &str,
) -> Result<String> {
    // Extract the prefix.
    let index = existing
        .find(begin_pragma)
        .with_context(|| "Unable to find begin pragma")?;
    let prefix = &existing[..index + begin_pragma.len()];

    // Extract the suffix.
    let index = existing
        .find(end_pragma)
        .with_context(|| "Unable to find end pragma")?;
    let suffix = &existing[index..];

    Ok(format!("{prefix}\n{section}{suffix}"))
}

pub(super) fn main(args: &Args) -> Result<()> {
    // Generate `ruff help`.
    let command_help = trim_lines(&help_text());

    // Generate `ruff help check`.
    let subcommand_help = trim_lines(&check_help_text());

    if args.mode.is_dry_run() {
        print!("{command_help}");
        print!("{subcommand_help}");
        return Ok(());
    }

    // Read the existing file.
    let filename = "docs/configuration.md";
    let file = PathBuf::from(ROOT_DIR).join(filename);
    let existing = fs::read_to_string(&file)?;

    let new = replace_docs_section(
        &existing,
        &format!("```text\n{command_help}\n```\n\n"),
        COMMAND_HELP_BEGIN_PRAGMA,
        COMMAND_HELP_END_PRAGMA,
    )?;
    let new = replace_docs_section(
        &new,
        &format!("```text\n{subcommand_help}\n```\n\n"),
        SUBCOMMAND_HELP_BEGIN_PRAGMA,
        SUBCOMMAND_HELP_END_PRAGMA,
    )?;

    match args.mode {
        Mode::Check => {
            if existing == new {
                println!("up-to-date: {filename}");
            } else {
                let comparison = StrComparison::new(&existing, &new);
                bail!("{filename} changed, please run `{REGENERATE_ALL_COMMAND}`:\n{comparison}");
            }
        }
        _ => {
            fs::write(file, &new)?;
        }
    }

    Ok(())
}

/// Returns the output of `ruff help`.
fn help_text() -> String {
    args::Args::command().render_help().to_string()
}

/// Returns the output of `ruff help check`.
fn check_help_text() -> String {
    let mut cmd = args::Args::command();

    // The build call is necessary for the help output to contain `Usage: ruff
    // check` instead of `Usage: check` see https://github.com/clap-rs/clap/issues/4685
    cmd.build();

    cmd.find_subcommand_mut("check")
        .expect("`check` subcommand not found")
        .render_help()
        .to_string()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::generate_all::Mode;

    use super::{main, Args};

    #[test]
    fn test_generate_json_schema() -> Result<()> {
        main(&Args { mode: Mode::Check })
    }
}
