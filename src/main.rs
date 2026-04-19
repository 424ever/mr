use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use mr::{Manual, config::Settings, info, pager::WriteTarget};

#[derive(Parser)]
struct Cli {
    /// info file to display
    file: PathBuf,
    /// do not use a pager
    #[arg(long)]
    no_pager: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config: Settings = confy::load("mr", None)?;

    let mut output = match (cli.no_pager, config.ui.pager) {
        (true, _) => WriteTarget::new_unpaged(),
        (false, c) if c.is_empty() => WriteTarget::new_unpaged(),
        (false, c) => WriteTarget::new_paged(c)?,
    };

    let manual = info::read_nonsplit_manual(&cli.file)
        .context(format!("couldn't read {}", cli.file.to_str().unwrap()))?;

    manual.render(&mut output)?;

    output.wait()?;

    Ok(())
}
