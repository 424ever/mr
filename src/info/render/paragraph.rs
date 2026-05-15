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
        if self.print_raw() {
            self.lines
                .iter()
                .try_for_each(|l| writeln_indented(&mut into, l, opt))?;
        } else {
            self.render_reflow(&mut into, opt)?;
        }
        writeln!(&mut into)?;
        Ok(())
    }

    fn render_reflow(&self, mut into: impl Write, opt: RenderOptions) -> io::Result<()> {
        flowing_lines(lines_into_words(self.lines.iter()), opt.max_width(), true)
            .try_for_each(|line| writeln_indented(&mut into, line, opt))
    }

    fn print_raw(&self) -> bool {
        self.lines.iter().any(|l| l.starts_with("    "))
    }
}
