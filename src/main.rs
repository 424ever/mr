use mr::info::parse_nonsplit_manual;
use std::fs;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// info file to display
    file: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let content = fs::read_to_string(cli.file)?;
    let manual = parse_nonsplit_manual(&content)?;

    for _node in manual.nodes() {}

    Ok(())
}
