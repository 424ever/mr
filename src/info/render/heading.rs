use std::io::Write;

use glyphs::{Color, style};

use crate::info::Heading;

impl Heading {
    pub(super) fn render(&self, into: &mut dyn Write) -> anyhow::Result<()> {
        writeln!(into, "{}", style(&self.text).fg(Color::Red).bold())?;
        Ok(())
    }
}
