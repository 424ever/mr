#![warn(rust_2018_idioms)]
use std::{io::Cursor, thread};

use clap::Parser;
use mr::{Manual, info};

#[derive(Parser)]
struct Cli {
    /// info file to display
    file: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let manual = info::parse_nonsplit_manual(&cli.file)?;

    show_manual(&manual)?;

    Ok(())
}

fn show_manual<M: Manual>(man: &M) -> anyhow::Result<()> {
    let mut buf: Vec<u8> = vec![];

    let mut pager = streampager::Pager::new_using_system_terminal()?;
    pager.set_scroll_past_eof(false);
    pager.set_read_ahead_lines(usize::MAX / 2);

    man.render(&mut buf)?;
    pager.add_stream(Cursor::new(buf), man.title())?;

    let pager_thread = thread::spawn(move || -> anyhow::Result<()> {
        pager.run()?;
        Ok(())
    });

    pager_thread.join().unwrap()?;

    Ok(())
}
