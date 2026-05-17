use std::io::Write;

use crate::{
    RenderOptions,
    info::{
        TableEntry,
        render::{ResolvedNodeLines, writeln_indented},
    },
};

impl TableEntry {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: RenderOptions,
        node_lines: &ResolvedNodeLines,
    ) -> anyhow::Result<()> {
        writeln_indented(into, &self.title, opt)?;
        self.description
            .iter()
            .try_for_each(|n| n.render(into, opt.indented(5), node_lines))?;
        if self.description.len() == 0 {
            writeln!(into)?;
        }
        Ok(())
    }
}
