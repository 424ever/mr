use std::{
    io::{self, stdout},
    process::{Child, Command, Stdio},
};

use anyhow::Context;

pub enum WriteTarget {
    Stdout,
    Pager(Child),
}

impl WriteTarget {
    pub fn new_unpaged() -> Self {
        Self::Stdout
    }

    pub fn new_paged(cmd: Vec<String>) -> anyhow::Result<Self> {
        Command::new(&cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::piped())
            .spawn()
            .map(Self::Pager)
            .context("failed to start pager")
    }

    pub fn wait(self) -> anyhow::Result<()> {
        match self {
            WriteTarget::Stdout => Ok(()),
            WriteTarget::Pager(mut child) => {
                child.wait().map(|_| ()).context("waiting on pager failed")
            }
        }
    }
}

impl io::Write for WriteTarget {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            WriteTarget::Stdout => stdout().write(buf),
            WriteTarget::Pager(child) => child
                .stdin
                .as_mut()
                .expect("can write to pager's stdin")
                .write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            WriteTarget::Stdout => stdout().flush(),
            WriteTarget::Pager(child) => child
                .stdin
                .as_mut()
                .expect("can write to pager's stdin")
                .flush(),
        }
    }
}
