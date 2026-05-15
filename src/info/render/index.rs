use std::{
    collections::HashMap,
    io::{self, Write},
};

use glyphs::style;

use crate::{
    RenderOptions,
    info::{
        Id, Printindex,
        render::{render_id, writeln_indented},
    },
};

impl Printindex {
    pub(super) fn render<W: Write>(
        &self,
        mut into: W,
        opt: RenderOptions,
        node_lines: &HashMap<Id, usize>,
    ) -> io::Result<()> {
        writeln_indented(&mut into, style("* Index:\n").bold(), opt)?;
        let opt = opt.indented(2);
        self.entries.iter().try_for_each(|e| {
            writeln_indented(
                &mut into,
                format_args!(
                    "{}: {}",
                    style(&e.text).underline(),
                    render_id(
                        &Id {
                            infofile: None,
                            nodename: Some(e.node_spec.clone())
                        },
                        node_lines,
                    )
                ),
                opt,
            )
        })?;
        writeln!(into)?;

        Ok(())
    }
}
