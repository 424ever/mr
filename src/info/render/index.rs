use std::io::Write;

use glyphs::{Color, style};

use crate::{
    RenderOptions,
    info::{
        Id, Printindex, ResolvedNodeLines,
        render::{node_location, writeln_indented},
    },
};

impl Printindex {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: RenderOptions,
        node_lines: &ResolvedNodeLines,
    ) -> anyhow::Result<()> {
        writeln_indented(into, style("* Index:\n").bold(), opt)?;
        let opt = opt.indented(2);
        self.entries.iter().try_for_each(|e| {
            writeln_indented(
                into,
                format_args!(
                    "{}: {}",
                    style(&e.text).underline(),
                    render_id(&e.node, node_lines,)
                ),
                opt,
            )
        })?;
        writeln!(into)?;

        Ok(())
    }
}

fn render_id(node: &Id, node_lines: &ResolvedNodeLines) -> String {
    if let Some(ref nodename) = node.nodename {
        format!("{} ({})", nodename, node_location(node_lines, node))
    } else {
        format!(
            "({})",
            style(node_location(node_lines, node))
                .bold()
                .fg(Color::Blue)
        )
    }
}
