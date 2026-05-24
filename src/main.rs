use std::{
    fs,
    io::{self, ErrorKind, Write as _},
};

use anyhow::Context;
use clap::Parser;
use mr::{Manual, RenderOptions, config::Settings, info, pager::WriteTarget};
use terminal_size::terminal_size;

#[derive(Parser)]
struct Cli {
    /// manual to display
    manual: String,
    /// do not use a pager
    #[arg(long)]
    no_pager: bool,
    /// print the abstract syntax tree instead of manual
    #[arg(long)]
    print_ast: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config: Settings = Settings::load()?;

    let mut output = match (cli.no_pager, config.ui.pager) {
        (true, _) => WriteTarget::new_unpaged(),
        (false, c) if c.is_empty() => WriteTarget::new_unpaged(),
        (false, c) => WriteTarget::new_paged(c)?,
    };

    let content = fs::read_to_string(&cli.manual)?;
    let manual =
        info::read_nonsplit_manual(&content).context(format!("parsing {} failed", cli.manual))?;

    let opt = if let Some(dim) = terminal_size() {
        RenderOptions::new_for_terminal(dim)
    } else {
        RenderOptions::new()
    };

    if cli.print_ast {
        ignore_broken_pipe(
            write!(output, "{:#?}", manual).context("writing ast to output failed"),
        )?;
    } else {
        ignore_broken_pipe(manual.render(&mut output, opt))?;
    }

    output.wait()?;

    Ok(())
}

fn ignore_broken_pipe(r: anyhow::Result<()>) -> anyhow::Result<()> {
    if let Err(ref e) = r {
        if let Some(ioe) = e.downcast_ref::<io::Error>()
            && ioe.kind() == ErrorKind::BrokenPipe
        {
            return Ok(());
        }
    }
    r
}
