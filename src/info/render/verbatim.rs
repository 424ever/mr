use std::io::Write;

use glyphs::style;

use crate::{
    RenderOptions,
    info::{Verbatim, render::writeln_indented},
};

impl Verbatim {
    pub(super) fn render(&self, into: &mut dyn Write, opt: RenderOptions) -> anyhow::Result<()> {
        self.lines.iter().try_for_each(|l| {
            writeln_indented(into, style(l).fg(glyphs::Color::Green), opt.indented(5))
        })?;
        writeln!(into)?;
        Ok(())
    }
}
