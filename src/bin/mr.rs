use std::fs;

use clap::Parser;
use mr::parse_nonsplit_manual;

#[derive(Parser)]
struct Cli {
    /// info file to display
    file: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let content = fs::read_to_string(cli.file)?;
    let manual = parse_nonsplit_manual(&content)?;

    for node in manual.nodes() {
        println!("{}", node.text());
    }

    Ok(())
}
