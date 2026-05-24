pub mod parse;
mod render;
pub mod search;

use std::{collections::HashMap, io};

use winnow::Parser as _;

use crate::{Manual, RenderOptions};

type ResolvedNodeLines = HashMap<String, usize>;

#[derive(Debug)]
pub struct NonsplitManual {
    file: NonsplitInfoFile,
}

impl Manual for NonsplitManual {
    fn render<W>(&self, into: W, opt: crate::RenderOptions) -> anyhow::Result<()>
    where
        W: io::Write,
    {
        let res = self.resolve_lines(opt)?;
        self.file.render_nodes(into, &opt, &res, |_, _| {})
    }

    fn title(&self) -> &str {
        self.file
            .nodes
            .first()
            .as_ref()
            .map(|n| n.file.as_str())
            .unwrap_or("")
    }
}

impl NonsplitManual {
    fn resolve_lines(&self, opt: RenderOptions) -> anyhow::Result<ResolvedNodeLines> {
        self.file.resolve_node_begin_lines(&opt)
    }

    pub fn start_line_for(&self, opt: RenderOptions, node: &str) -> anyhow::Result<Option<usize>> {
        Ok(self.resolve_lines(opt)?.get(node).copied())
    }

    pub fn nodes(&self) -> &Vec<Node> {
        &self.file.nodes
    }
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn read_nonsplit_manual(content: &str) -> anyhow::Result<NonsplitManual> {
    parse::nonsplit_info_file
        .parse(&content)
        .map(|f| Ok(NonsplitManual { file: f }))
        .map_err(|e| anyhow::format_err!("{e}"))?
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonsplitInfoFile {
    preamble: Preamble,
    nodes: Vec<Node>,
    tag_table: Option<TagTable>,
    local_variables: Option<LocalVariables>,
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
    up: Option<Id>,
    descr: Option<String>,
    general_text: Vec<TextBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBlock {
    content: TextBlockContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextBlockContent {
    Paragraph(Paragraph),
    Menu(Menu),
    Printindex(Printindex),
    Heading(Heading),
    Verbatim(Verbatim),
    TableEntry(TableEntry),
    BunchOfUnknownLines(Vec<String>),
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
    node: Id,
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    level: HeadingLevel,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadingLevel {
    /// title/chapter
    Major,
    Section,
    SubSection,
    SubSubSection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verbatim {
    lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableEntry {
    title: String,
    description: Vec<TextBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
