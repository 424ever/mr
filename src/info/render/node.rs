use std::io::Write;

use glyphs::style;

use crate::{
    RenderOptions,
    info::{
        Node, TextBlock, TextBlockContent,
        render::{ResolvedNodeLines, writeln_indented},
    },
};

impl Node {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: &RenderOptions,
        node_lines: &ResolvedNodeLines,
    ) -> anyhow::Result<()> {
        let opt = opt.indented(7);
        self.general_text
            .iter()
            .try_for_each(|b| b.render(into, opt, node_lines))
    }
}

impl TextBlock {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: RenderOptions,
        node_lines: &ResolvedNodeLines,
    ) -> anyhow::Result<()> {
        match &self.content {
            TextBlockContent::Paragraph(paragraph) => paragraph.render(into, opt),
            TextBlockContent::Menu(menu) => menu.render(into, opt, node_lines),
            TextBlockContent::Printindex(printindex) => printindex.render(into, opt, node_lines),
            TextBlockContent::Heading(heading) => heading.render(into),
            TextBlockContent::Verbatim(verbatim) => verbatim.render(into, opt),
            TextBlockContent::TableEntry(entry) => entry.render(into, opt, node_lines),
            TextBlockContent::BunchOfUnknownLines(lines) => {
                lines.iter().try_for_each(|l| {
                    writeln_indented(into, style(l).fg(glyphs::Color::BrightRed), opt.indented(3))
                })?;
                writeln!(into)?;
                Ok(())
            }
        }
    }
}
