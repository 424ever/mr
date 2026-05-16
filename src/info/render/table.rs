use std::{collections::HashMap, io::Write};

use crate::{
    RenderOptions,
    info::{Id, TableEntry, render::writeln_indented},
};

impl TableEntry {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: RenderOptions,
        node_lines: &HashMap<Id, usize>,
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
