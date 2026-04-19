use std::io::{self, Write};

use anyhow::Context as _;
use yansi::Paint as _;

use super::{Heading, Menu, MenuItem, NonsplitInfoFile, TextBlockContent};
use crate::{
    Manual, RenderOptions,
    info::{Paragraph, Printindex},
};

impl Manual for NonsplitInfoFile {
    fn render<W>(&self, mut into: W, opt: RenderOptions) -> anyhow::Result<()>
    where
        W: Write,
    {
        self.nodes
            .iter()
            .flat_map(|n| &n.general_text)
            .try_for_each(|b| match &b.content {
                TextBlockContent::Paragraph(paragraph) => paragraph.render(&mut into, &opt),
                TextBlockContent::Menu(menu) => menu.render(&mut into),
                TextBlockContent::Printindex(printindex) => printindex.render(&mut into),
                TextBlockContent::Heading(heading) => heading.render(&mut into),
            })
            .context("write")
    }

    fn title(&self) -> &str {
        self.nodes
            .first()
            .as_ref()
            .map(|n| n.file.as_str())
            .unwrap_or("")
    }
}

impl Paragraph {
    fn render<W: Write>(&self, mut into: W, opt: &RenderOptions) -> io::Result<()> {
        if self.print_raw() {
            self.lines
                .iter()
                .try_for_each(|l| writeln!(&mut into, "       {}", l))?;
        } else {
            let allowed = opt.max_width.saturating_sub(7);
            let mut curr_width = 0;

            write!(&mut into, "       ")?;
            for word in self.lines.iter().flat_map(|l| l.split_whitespace()) {
                let len = word.chars().count();

                if (curr_width + len) > allowed {
                    if curr_width > 0 {
                        write!(&mut into, "\n       {} ", word)?;
                        curr_width = len + 1;
                    } else {
                        write!(&mut into, "{}\n       ", word)?;
                        curr_width = 0;
                    }
                } else {
                    write!(&mut into, "{} ", word)?;
                    curr_width += len + 1;
                }
            }
        }
        writeln!(&mut into, "\n")
    }

    fn print_raw(&self) -> bool {
        self.lines.iter().any(|l| l.starts_with("    "))
    }
}

impl Heading {
    fn render<W: Write>(&self, mut into: W) -> io::Result<()> {
        let pad = match self.level {
            crate::info::HeadingLevel::Major => "",
            crate::info::HeadingLevel::Section => "",
            crate::info::HeadingLevel::SubSection => "   ",
            crate::info::HeadingLevel::SubSubSection => "    ",
        };

        writeln!(into, "{}{}", pad, self.text.red().bold())
    }
}

impl Printindex {
    fn render<W: Write>(&self, mut into: W) -> io::Result<()> {
        let longest_text = self.entries.iter().map(|e| e.text.len()).max().unwrap_or(0);

        write!(into, "       {}", "* Index:\n".bold())?;
        self.entries.iter().try_for_each(|e| {
            let pad = longest_text - e.text.len();
            writeln!(
                into,
                "         {}: {}{} (line {})",
                e.text,
                " ".repeat(pad),
                e.node_spec.underline(),
                e.line
            )
        })?;
        writeln!(into)?;

        Ok(())
    }
}

impl Menu {
    fn render<W: Write>(&self, mut into: W) -> io::Result<()> {
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

        write!(into, "       {}", "* Menu:\n".bold())?;
        self.items.iter().try_for_each(|i| {
            match i {
                MenuItem::Entry(entry) => {
                    // TODO: labels
                    let pad = longest_entry_nodename
                        - entry.id.nodename.as_ref().map(|n| n.len()).unwrap_or(0);
                    write!(
                        into,
                        "         {}{}\t{}{}",
                        entry.id.nodename.clone().unwrap_or("".into()).underline(),
                        " ".repeat(pad),
                        entry.description.join(" ").italic(),
                        "\n".repeat(entry.trailing_newlines + 1)
                    )
                }
                MenuItem::Comment(comment) => {
                    write!(
                        into,
                        "         {}{}",
                        &comment.lines.join(" "),
                        "\n".repeat(comment.trailing_newlines + 1)
                    )
                }
            }
        })
    }
}
