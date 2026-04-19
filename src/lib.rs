use std::io::Write;

pub mod config;
mod control;
pub mod info;
pub mod pager;
mod parser_util;

pub trait Manual {
    fn render<W>(&self, into: W) -> anyhow::Result<()>
    where
        W: Write;

    fn title(&self) -> &str;
}
