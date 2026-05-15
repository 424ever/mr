use std::{
    collections::HashMap,
    io::{self, Write},
};

use glyphs::{style, visible_len};

use crate::{
    RenderOptions,
    info::{
        Id, Menu, MenuItem,
        render::{flowing_lines, lines_into_words, render_id, write_indented, writeln_indented},
    },
};

impl Menu {
    pub(super) fn render<W: Write>(
        &self,
        mut into: W,
        opt: RenderOptions,
        node_lines: &HashMap<Id, usize>,
    ) -> io::Result<()> {
        let longest_entry_nodename = self
            .items
            .iter()
            .filter_map(|i| match i {
                MenuItem::Entry(entry) => Some(visible_len(&render_id(&entry.id, node_lines))),
                MenuItem::Comment(_comment) => None,
            })
            .max()
            .unwrap_or(0);

        writeln_indented(&mut into, style("* Menu:").bold(), opt)?;

        let id_opt = opt.indented(2);
        let descr_opt = id_opt.indented(longest_entry_nodename + 2);

        self.items.iter().try_for_each(|i| {
            match i {
                MenuItem::Entry(entry) => {
                    // TODO: labels
                    let id = render_id(&entry.id, node_lines);
                    let pad = longest_entry_nodename - visible_len(&id);
                    write_indented(
                        &mut into,
                        format_args!("{}{}  ", id, " ".repeat(pad)),
                        id_opt,
                    )?;
                    let mut descr = flowing_lines(
                        lines_into_words(entry.description.iter()),
                        descr_opt.max_width(),
                        false,
                    );
                    // First line
                    let first = descr.next();
                    if let Some(first) = first {
                        writeln!(&mut into, "{}", first)?;
                    }
                    // Further lines
                    descr.try_for_each(|l| writeln_indented(&mut into, l, descr_opt))?;
                    write!(&mut into, "{}", "\n".repeat(entry.trailing_newlines))
                }
                MenuItem::Comment(comment) => {
                    flowing_lines(
                        lines_into_words(comment.lines.iter()),
                        opt.max_width(),
                        false,
                    )
                    .try_for_each(|l| writeln_indented(&mut into, l, id_opt))?;
                    write!(into, "{}", "\n".repeat(comment.trailing_newlines))
                }
            }
        })?;
        writeln!(&mut into, "")
    }
}
