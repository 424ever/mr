#![warn(rust_2018_idioms)]
use std::{
    env::VarError,
    fmt::Display,
    io::stdout,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use anyhow::{Context, anyhow};
use clap::{Parser, ValueEnum};
use mr::{Manual, info};

#[derive(Parser)]
struct Cli {
    /// info file to display
    file: PathBuf,
    /// what pager to use
    #[arg(long,default_value_t = Pager::Env)]
    pager: Pager,
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
enum Pager {
    /// use the program in the $PAGER environment variable, or `less` if it is unset or empty
    Env,
    /// don't use a pager
    None,
}

impl Display for Pager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pager::Env => write!(f, "env")?,
            Pager::None => write!(f, "none")?,
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let manual = info::read_nonsplit_manual(&cli.file)?;

    show_manual(&manual, cli.pager)?;

    Ok(())
}

fn show_manual<M: Manual>(man: &M, pager: Pager) -> anyhow::Result<()> {
    match pager {
        Pager::Env => {
            let mut sub = start_env_pager()?;
            man.render(sub.stdin.as_mut().expect("can write to pager's stin"))?;
            sub.wait()?;
        }
        Pager::None => man.render(&mut stdout())?,
    };

    Ok(())
}

fn start_env_pager() -> anyhow::Result<Child> {
    let prog = match std::env::var("PAGER") {
        Ok(v) if v.is_empty() => Ok("less".to_string()),
        Err(VarError::NotPresent) => Ok("less".to_string()),
        Err(VarError::NotUnicode(_)) => Err(anyhow!("$PAGER is not valid unicode")),
        Ok(v) => Ok(v),
    }?;

    Ok(Command::new("sh")
        .arg("-c")
        .arg(&prog)
        .stdin(Stdio::piped())
        .spawn()
        .context(format!("failed to start pager '{}'", prog))?)
}
