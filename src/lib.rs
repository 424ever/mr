#![warn(rust_2018_idioms)]
use std::io::Write;

mod control;
pub mod info;
mod parser_util;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Manual {
    title: String,
    text: String,
}

impl Manual {
    pub fn new(title: String, text: String) -> Self {
        Self { title, text }
    }

    pub fn render<W: Write>(&self, into: &mut W) -> anyhow::Result<()> {
        into.write_all(self.text.as_bytes())?;
        Ok(())
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}
