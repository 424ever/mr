use std::{
    io::{self, Write},
    iter::Peekable,
};

use itertools::Itertools as _;
use unicode_segmentation::UnicodeSegmentation as _;
use yansi::Paint as _;

use super::{Heading, Menu, MenuItem, NonsplitInfoFile, TextBlockContent};
use crate::{
    Manual, RenderOptions,
    info::{Paragraph, Printindex},
};

impl Manual for NonsplitInfoFile {
    fn render<W>(&self, mut into: W, opt: RenderOptions) -> io::Result<()>
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
            self.render_reflow(&mut into, opt)?;
        }
        writeln!(&mut into)?;
        Ok(())
    }

    fn render_reflow(&self, mut into: impl Write, opt: &RenderOptions) -> io::Result<()> {
        let allowed = opt.max_width.saturating_sub(7);
        let mut words = self
            .lines
            .iter()
            .flat_map(|l| l.split_whitespace())
            .peekable();

        while let Some(words_for_line) = Self::words_for_next_line(&mut words, allowed) {
            let line = if words.peek().is_some() {
                Self::line_padded_to(words_for_line, allowed)
            } else {
                // only single spaces for the last line in the paragraph
                words_for_line.join(" ")
            };

            writeln!(&mut into, "       {}", line)?;
        }

        Ok(())
    }

    fn words_for_next_line<S, I>(words: &mut Peekable<I>, allowed: usize) -> Option<Vec<S>>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        words.peek()?;

        let mut len_sum = 0;
        Some(
            words
                .peeking_take_while(move |w| {
                    let wl = w.as_ref().graphemes(true).count();
                    if len_sum == 0 {
                        len_sum = wl;
                        true
                    } else {
                        let len_with_word = len_sum + wl + 1;
                        if len_with_word <= allowed {
                            len_sum = len_with_word;
                            true
                        } else {
                            false
                        }
                    }
                })
                .collect(),
        )
    }

    fn line_padded_to(words: Vec<&str>, pad_to: usize) -> String {
        assert!(!words.is_empty());

        if words.len() == 1 {
            return words[0].to_string();
        }

        let total_chars: usize = words.iter().map(|w| w.graphemes(true).count()).sum();
        let gaps = words.len() - 1;
        assert!(total_chars + gaps <= pad_to);
        let total_spaces = pad_to - total_chars;

        let spaces_in_every_gap = " ".repeat(total_spaces / gaps);
        let mut remaining_gaps_with_extra_space = total_spaces % gaps;

        let mut res = String::with_capacity(pad_to);
        res.push_str(words[0]);
        for w in &words[1..] {
            if remaining_gaps_with_extra_space > 0 {
                remaining_gaps_with_extra_space -= 1;
                res.push(' ');
            }
            res.push_str(&spaces_in_every_gap);
            res.push_str(w)
        }

        res
    }

    fn print_raw(&self) -> bool {
        self.lines.iter().any(|l| l.starts_with("    "))
    }
}

impl Heading {
    fn render<W: Write>(&self, mut into: W) -> io::Result<()> {
        writeln!(into, "{}", self.text.red().bold())
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
                        entry.description.join(" "),
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
