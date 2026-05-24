use std::io::Write;

use glyphs::{Color, style, visible_len};

use crate::{
    RenderOptions,
    info::{
        Menu, MenuEntry, MenuItem,
        render::{
            ResolvedNodeLines, flowing_lines, lines_into_words, node_location, write_indented,
            writeln_indented,
        },
    },
};

impl Menu {
    pub(super) fn render(
        &self,
        into: &mut dyn Write,
        opt: RenderOptions,
        node_lines: &ResolvedNodeLines,
    ) -> anyhow::Result<()> {
        let longest_entry_nodename = self
            .items
            .iter()
            .filter_map(|i| match i {
                MenuItem::Entry(entry) => Some(visible_len(&render_entry_id(&entry, node_lines))),
                MenuItem::Comment(_comment) => None,
            })
            .max()
            .unwrap_or(0);

        writeln_indented(into, style("* Menu:").bold(), opt)?;

        let id_opt = opt.indented(2);
        let descr_opt = id_opt.indented(longest_entry_nodename + 2);

        self.items.iter().try_for_each(|i| {
            match i {
                MenuItem::Entry(entry) => {
                    // TODO: labels
                    let id = render_entry_id(&entry, node_lines);
                    let pad = longest_entry_nodename - visible_len(&id);
                    write_indented(
                        &mut *into,
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
                        writeln!(into, "{}", first)?;
                    } else {
                        writeln!(into)?;
                    }
                    // Further lines
                    descr.try_for_each(|l| writeln_indented(into, l, descr_opt))?;
                    write!(into, "{}", "\n".repeat(entry.trailing_newlines))
                }
                MenuItem::Comment(comment) => {
                    flowing_lines(
                        lines_into_words(comment.lines.iter()),
                        opt.max_width(),
                        false,
                    )
                    .try_for_each(|l| writeln_indented(into, l, id_opt))?;
                    write!(into, "{}", "\n".repeat(comment.trailing_newlines))
                }
            }
        })?;
        writeln!(into)?;
        Ok(())
    }
}

fn render_entry_id(entry: &MenuEntry, map: &ResolvedNodeLines) -> String {
    let label = if let Some(ref label) = entry.label {
        Some(label)
    } else if let Some(ref nodename) = entry.id.nodename {
        Some(nodename)
    } else {
        None
    };

    if let Some(label) = label {
        format!(
            "{} ({})",
            label,
            style(node_location(map, &entry.id)).bold().fg(Color::Blue)
        )
    } else {
        format!(
            "({})",
            style(node_location(map, &entry.id)).bold().fg(Color::Blue)
        )
    }
}
