use std::io::Write;

use crate::{
    RenderOptions,
    info::{
        Paragraph,
        render::{flowing_lines, lines_into_words, writeln_indented},
    },
};

impl Paragraph {
    pub(super) fn render(&self, into: &mut dyn Write, opt: RenderOptions) -> anyhow::Result<()> {
        flowing_lines(lines_into_words(self.lines.iter()), opt.max_width(), true)
            .try_for_each(|line| writeln_indented(into, line, opt))?;
        writeln!(into)?;
        Ok(())
    }
}
