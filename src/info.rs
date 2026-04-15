pub mod parse;

use std::fs;

use winnow::{LocatingSlice, Parser};
use yansi::Paint;

use crate::Manual;

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn parse_nonsplit_manual(path: &str) -> anyhow::Result<Manual> {
    let content = fs::read_to_string(path)?;
    parse::nonsplit_info_file
        .parse(LocatingSlice::new(&content))
        .map(|n| n.into_manual(path.to_string()))
        .map_err(|e| anyhow::format_err!("{e}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonsplitInfoFile {
    preamble: Preamble,
    nodes: Vec<Node>,
    tag_table: Option<TagTable>,
    local_variables: Option<LocalVariables>,
}

impl NonsplitInfoFile {
    fn into_manual(self, title: String) -> Manual {
        Manual::new(
            title,
            self.nodes
                .iter()
                .flat_map(|n| &n.general_text)
                .map(|b| match &b.content {
                    TextBlockContent::Paragraph(paragraph) => {
                        let mut p = paragraph.lines.join("\n");
                        p.push('\n');
                        p.push('\n');
                        p
                    }
                    TextBlockContent::Menu(menu) => render_menu(menu),
                    TextBlockContent::Printindex(printindex) => render_index(printindex),
                })
                .collect::<Vec<_>>()
                .join(""),
        )
    }
}

fn render_index(index: &Printindex) -> String {
    let longest_text = index
        .entries
        .iter()
        .map(|e| e.text.len())
        .max()
        .unwrap_or(0);

    let mut s = index
        .entries
        .iter()
        .map(|e| {
            let pad = longest_text - e.text.len();
            format!(
                "  {}: {}{} (line {})\n",
                e.text,
                " ".repeat(pad),
                e.node_spec.underline(),
                e.line
            )
        })
        .fold("* Index:\n".to_string(), |mut s, it| {
            s.push_str(&it);
            s
        });
    s.push('\n');
    s
}

fn render_menu(menu: &Menu) -> String {
    let longest_entry_nodename = menu
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

    menu.items
        .iter()
        .map(|i| match i {
            MenuItem::Entry(entry) => {
                // TODO: labels
                let pad = longest_entry_nodename
                    - entry.id.nodename.as_ref().map(|n| n.len()).unwrap_or(0);
                format!(
                    "  {}{}\t{}{}",
                    entry.id.nodename.clone().unwrap_or("".into()).underline(),
                    " ".repeat(pad),
                    entry.description.join(" ").italic(),
                    "\n".repeat(entry.trailing_newlines + 1)
                )
            }
            MenuItem::Comment(comment) => format!(
                "  {}{}",
                &comment.lines.join(" "),
                "\n".repeat(comment.trailing_newlines + 1)
            ),
        })
        .fold("* Menu:\n".to_string(), |mut s, it| {
            s.push_str(&it);
            s
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitInfoMainFile {
    preamble: Preamble,
    indirect_table: IndirectTable,
    tag_table: TagTable,
    local_variables: LocalVariables,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitInfoSubfile {
    preamble: Preamble,
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preamble {
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    file: String,
    node: Id,
    next: Option<Id>,
    prev: Option<Id>,
    up: Id,
    general_text: Vec<TextBlock>,
    /// Offset (in bytes) at which `general_text` starts within the file
    start_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    /// Offset (in bytes) from the start of the file where `content` starts
    start_offset: usize,
    /// Offset (in bytes) from the start of the file where `content` ends
    end_offset: usize,
    content: TextBlockContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextBlockContent {
    Paragraph(Paragraph),
    Menu(Menu),
    Printindex(Printindex),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paragraph {
    lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    Entry(MenuEntry),
    Comment(MenuComment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuEntry {
    label: Option<String>,
    description: Vec<String>,
    id: Id,
    trailing_newlines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuComment {
    lines: Vec<String>,
    trailing_newlines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printindex {
    entries: Vec<IndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    text: String,
    node_spec: String,
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    infofile: Option<String>,
    nodename: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagTable {
    indirect: bool,
    entries: Vec<TagTableEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagTableEntry {
    Node(Tag),
    Ref(Tag),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    nodeid: String,
    bytepos: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVariables {
    coding: Option<String>,
    language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndirectTable {
    entries: Vec<IndirectEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndirectEntry {
    filename: String,
    bytepos: u64,
}
