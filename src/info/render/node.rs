use std::io::Write;

use glyphs::{Color, style};

use crate::{
    RenderOptions,
    info::{
        Id, Node, TextBlock, TextBlockContent,
        render::{ResolvedNodeLines, color_escape, color_reset, node_location, writeln_indented},
    },
};

impl Node {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: &RenderOptions,
        node_lines: &ResolvedNodeLines,
    ) -> anyhow::Result<()> {
        // node line
        writeln!(
            into,
            "{}-- Node: {}{}{}{}{}",
            color_escape(Color::BrightBlack),
            self.node.nodename.as_deref().unwrap_or("?unknown?"),
            render_referenced_node(self.next.as_ref(), "Next", node_lines),
            render_referenced_node(self.prev.as_ref(), "Prev", node_lines),
            render_referenced_node(self.up.as_ref(), "Up", node_lines),
            color_reset()
        )?;
        // content
        let opt = opt.indented(7);
        self.general_text
            .iter()
            .try_for_each(|b| b.render(into, opt, node_lines))
    }
}

fn render_referenced_node(
    id: Option<&Id>,
    ref_name: &str,
    node_lines: &ResolvedNodeLines,
) -> String {
    if let Some(id) = id {
        if let Some(ref nodename) = id.nodename {
            format!(
                ", {}: {} ({})",
                ref_name,
                nodename,
                node_location(node_lines, id)
            )
        } else {
            format!(", {}: ({})", ref_name, node_location(node_lines, id))
        }
    } else {
        "".into()
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
