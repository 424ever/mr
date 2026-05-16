use std::{
    collections::HashMap,
    io::{self, Write},
};

use glyphs::style;

use crate::{
    RenderOptions,
    info::{Id, Node, TextBlockContent, render::writeln_indented},
};

impl Node {
    pub(super) fn render<W: Write>(
        &self,
        mut into: W,
        opt: RenderOptions,
        node_lines: &HashMap<Id, usize>,
    ) -> io::Result<()> {
        let opt = opt.indented(7);
        self.general_text.iter().try_for_each(|b| match &b.content {
            TextBlockContent::Paragraph(paragraph) => paragraph.render(&mut into, opt),
            TextBlockContent::Menu(menu) => menu.render(&mut into, opt, node_lines),
            TextBlockContent::Printindex(printindex) => {
                printindex.render(&mut into, opt, node_lines)
            }
            TextBlockContent::Heading(heading) => heading.render(&mut into),
            TextBlockContent::Verbatim(verbatim) => {
                verbatim.lines.iter().try_for_each(|l| {
                    writeln_indented(
                        &mut into,
                        style(l).fg(glyphs::Color::Green),
                        opt.indented(5),
                    )
                })?;
                writeln!(into)
            }
            TextBlockContent::BunchOfUnknownLines(lines) => {
                lines.iter().try_for_each(|l| {
                    writeln_indented(
                        &mut into,
                        style(l).fg(glyphs::Color::BrightRed),
                        opt.indented(5),
                    )
                })?;
                writeln!(into)?;
                Ok(())
            }
        })
    }
}
