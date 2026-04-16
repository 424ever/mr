#![warn(rust_2018_idioms)]
use std::io::Write;

mod control;
pub mod info;
mod parser_util;

pub trait Manual {
    fn render<W>(&self, into: &mut W) -> anyhow::Result<()>
    where
        W: Write;

    fn title(&self) -> &str;
}
