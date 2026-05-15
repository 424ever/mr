use std::{
    io::{self},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;
use mr::{Manual, RenderOptions, config::Settings, info, pager::WriteTarget};
use terminal_size::terminal_size;

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
    let config: Settings = Settings::load()?;

    let mut output = match (cli.no_pager, config.ui.pager) {
        (true, _) => WriteTarget::new_unpaged(),
        (false, c) if c.is_empty() => WriteTarget::new_unpaged(),
        (false, c) => WriteTarget::new_paged(c)?,
    };

    let manual = info::read_nonsplit_manual(&cli.file)
        .context(format!("parsing {} failed", cli.file.to_str().unwrap()))?;

    let opt = if let Some(dim) = terminal_size() {
        RenderOptions::new_for_terminal(dim)
    } else {
        RenderOptions::new()
    };

    match manual.render(&mut output, opt) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e),
    }?;

    output.wait()?;

    Ok(())
}
