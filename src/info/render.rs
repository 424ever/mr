use std::io::Write;

use yansi::Paint as _;

use super::{Heading, Menu, MenuItem, NonsplitInfoFile, TextBlockContent};
use crate::{Manual, info::Printindex};

impl Manual for NonsplitInfoFile {
    fn render<W>(&self, into: &mut W) -> anyhow::Result<()>
    where
        W: Write,
    {
        self.nodes
            .iter()
            .flat_map(|n| &n.general_text)
            .map(|b| {
                match &b.content {
                    TextBlockContent::Paragraph(paragraph) => {
                        write!(into, "{}\n\n", paragraph.lines.join("\n"))?;
                    }
                    TextBlockContent::Menu(menu) => menu.render(into)?,
                    TextBlockContent::Printindex(printindex) => printindex.render(into)?,
                    TextBlockContent::Heading(heading) => heading.render(into)?,
                };
                Ok(())
            })
            .collect::<anyhow::Result<()>>()?;

        Ok(())
    }

    fn title(&self) -> &str {
        self.nodes
            .iter()
            .next()
            .as_ref()
            .map(|n| n.file.as_str())
            .unwrap_or("")
    }
}

impl Heading {
    fn render<W: Write>(&self, into: &mut W) -> anyhow::Result<()> {
        write!(into, "{}\n\n", self.text.yellow().bold())?;
        Ok(())
    }
}

impl Printindex {
    fn render<W: Write>(&self, into: &mut W) -> anyhow::Result<()> {
        let longest_text = self.entries.iter().map(|e| e.text.len()).max().unwrap_or(0);

        write!(into, "{}", "* Index:\n".bold())?;
        self.entries
            .iter()
            .map(|e| {
                let pad = longest_text - e.text.len();
                write!(
                    into,
                    "  {}: {}{} (line {})\n",
                    e.text,
                    " ".repeat(pad),
                    e.node_spec.underline(),
                    e.line
                )?;
                Ok(())
            })
            .collect::<anyhow::Result<()>>()?;
        write!(into, "\n")?;

        Ok(())
    }
}

impl Menu {
    fn render<W: Write>(&self, into: &mut W) -> anyhow::Result<()> {
        let longest_entry_nodename = self
            .items
            .iter()
            .filter_map(|i| match i {
                MenuItem::Entry(entry) => {
                    Some(entry.id.nodename.as_ref().map(|n| n.len()).unwrap_or(0))
                }
                MenuItem::Comment(_comment) => None,
            })
            .max()
            .unwrap_or(0);

        write!(into, "{}", "* Menu:\n".bold())?;
        self.items
            .iter()
            .map(|i| {
                match i {
                    MenuItem::Entry(entry) => {
                        // TODO: labels
                        let pad = longest_entry_nodename
                            - entry.id.nodename.as_ref().map(|n| n.len()).unwrap_or(0);
                        write!(
                            into,
                            "  {}{}\t{}{}",
                            entry.id.nodename.clone().unwrap_or("".into()).underline(),
                            " ".repeat(pad),
                            entry.description.join(" ").italic(),
                            "\n".repeat(entry.trailing_newlines + 1)
                        )?;
                    }
                    MenuItem::Comment(comment) => {
                        write!(
                            into,
                            "  {}{}",
                            &comment.lines.join(" "),
                            "\n".repeat(comment.trailing_newlines + 1)
                        )?;
                    }
                };
                Ok(())
            })
            .collect::<anyhow::Result<()>>()?;

        Ok(())
    }
}
