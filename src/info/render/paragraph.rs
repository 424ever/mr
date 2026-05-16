use std::io::{self, Write};

use crate::{
    RenderOptions,
    info::{
        Paragraph,
        render::{flowing_lines, lines_into_words, writeln_indented},
    },
};

impl Paragraph {
    pub(super) fn render<W: Write>(&self, mut into: W, opt: RenderOptions) -> io::Result<()> {
        flowing_lines(lines_into_words(self.lines.iter()), opt.max_width(), true)
            .try_for_each(|line| writeln_indented(&mut into, line, opt))?;
        writeln!(&mut into)?;
        Ok(())
    }
}
