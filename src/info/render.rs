mod heading;
mod index;
mod menu;
mod node;
mod paragraph;
mod table;
mod verbatim;

use std::{
    collections::HashMap,
    fmt::Display,
    io::{self, Write},
    iter::Peekable,
};

use glyphs::{Color, style};
use itertools::Itertools as _;
use unicode_segmentation::UnicodeSegmentation as _;

use crate::{
    Manual, RenderOptions,
    info::{Id, NonsplitInfoFile},
};

const MAX_REF_DIGITS: usize = 5;

type ResolvedNodeLines = HashMap<String, usize>;

impl Manual for NonsplitInfoFile {
    fn render<W>(&self, into: W, opt: RenderOptions) -> anyhow::Result<()>
    where
        W: Write,
    {
        let node_lines = self.resolve_node_begin_lines(opt)?;

        self.render_nodes(into, opt, &node_lines, |_, _| {})?;

        Ok(())
    }

    fn title(&self) -> &str {
        self.nodes
            .first()
            .as_ref()
            .map(|n| n.file.as_str())
            .unwrap_or("")
    }
}

impl NonsplitInfoFile {
    fn render_nodes<F: FnMut(&Id, &W), W: Write>(
        &self,
        mut into: W,
        opt: RenderOptions,
        node_lines: &ResolvedNodeLines,
        mut before_render: F,
    ) -> anyhow::Result<()> {
        self.nodes.iter().try_for_each(|n| {
            before_render(&n.node, &into);
            n.render(&mut into, opt, node_lines)
        })?;
        Ok(())
    }

    fn resolve_node_begin_lines(&self, opt: RenderOptions) -> anyhow::Result<ResolvedNodeLines> {
        let w = CountNewlines::new();
        let mut map = HashMap::new();
        let fake = HashMap::new();

        self.render_nodes(w, opt, &fake, |id, w| {
            if let Some(ref name) = id.nodename {
                map.insert(name.clone(), w.count() + 1);
            }
        })?;

        Ok(map)
    }
}

fn node_ref(map: &ResolvedNodeLines, id: &Id) -> String {
    match (&id.infofile, &id.nodename) {
        (None, Some(name)) => match map.get(name) {
            Some(line) => format!("{}G", line),
            None => "?".repeat(MAX_REF_DIGITS + 1),
        },
        (None, None) => format!("unknown"),
        (Some(file), _) => {
            format!("in {}", file)
        }
    }
}

fn render_id(node: &Id, node_lines: &ResolvedNodeLines) -> String {
    format!(
        "{} ({})",
        node.nodename.clone().unwrap_or("".into()),
        style(node_ref(node_lines, node)).bold().fg(Color::Blue),
    )
}

struct FlowingLines<S, I>
where
    I: Iterator<Item = S>,
{
    words: Peekable<I>,
    max_width: usize,
    use_full_width: bool,
}

impl<S, I> Iterator for FlowingLines<S, I>
where
    S: AsRef<str> + Display,
    I: Iterator<Item = S>,
{
    fn next(&mut self) -> Option<Self::Item> {
        let words_for_line = self.words_for_next_line()?;
        if self.words.peek().is_some() && self.use_full_width {
            Some(self.line_padded(words_for_line))
        } else {
            // only single spaces for the last line
            Some(words_for_line.into_iter().join(" "))
        }
    }

    type Item = String;
}

impl<S, I> FlowingLines<S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    fn words_for_next_line(&mut self) -> Option<Vec<S>> {
        self.words.peek()?;

        let mut len_sum = 0;
        Some(
            self.words
                .peeking_take_while(|w| {
                    let wl = w.as_ref().graphemes(true).count();
                    if len_sum == 0 {
                        len_sum = wl;
                        true
                    } else {
                        let len_with_word = len_sum + wl + 1;
                        if len_with_word <= self.max_width {
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

    fn line_padded(&self, words: Vec<S>) -> String {
        assert!(!words.is_empty());

        if words.len() == 1 {
            return words[0].as_ref().to_string();
        }

        let total_chars: usize = words
            .iter()
            .map(|w| w.as_ref().graphemes(true).count())
            .sum();
        let gaps = words.len() - 1;
        assert!(total_chars + gaps <= self.max_width);
        let total_spaces = self.max_width - total_chars;

        let spaces_in_every_gap = " ".repeat(total_spaces / gaps);
        let mut remaining_gaps_with_extra_space = total_spaces % gaps;

        let mut res = String::with_capacity(self.max_width);
        res.push_str(words[0].as_ref());
        for w in &words[1..] {
            if remaining_gaps_with_extra_space > 0 {
                remaining_gaps_with_extra_space -= 1;
                res.push(' ');
            }
            res.push_str(&spaces_in_every_gap);
            res.push_str(w.as_ref())
        }

        res
    }
}

fn flowing_lines<S: AsRef<str> + Display, I: Iterator<Item = S>>(
    words: I,
    max_width: usize,
    use_full_width: bool,
) -> impl Iterator<Item = String> {
    FlowingLines {
        words: words.peekable(),
        max_width,
        use_full_width,
    }
}

fn lines_into_words<'a, I: Iterator<Item = &'a String>>(lines: I) -> impl Iterator<Item = &'a str> {
    lines.flat_map(|l| l.split_whitespace())
}

fn write_indented<W: Write, D: Display>(mut into: W, d: D, opt: RenderOptions) -> io::Result<()> {
    write!(into, "{}{}", " ".repeat(opt.indent()), d)
}

fn writeln_indented<D: Display>(into: &mut dyn Write, d: D, opt: RenderOptions) -> io::Result<()> {
    writeln!(into, "{}{}", " ".repeat(opt.indent()), d)
}

struct CountNewlines {
    count: usize,
}

impl CountNewlines {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn count(&self) -> usize {
        self.count
    }
}

impl Write for CountNewlines {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.count += buf.iter().filter(|&c| *c == b'\n').count();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
