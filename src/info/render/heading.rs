use std::io::{self, Write};

use glyphs::{Color, style};

use crate::info::Heading;

impl Heading {
    pub(super) fn render<W: Write>(&self, mut into: W) -> io::Result<()> {
        writeln!(into, "{}", style(&self.text).fg(Color::Red).bold())
    }
}
