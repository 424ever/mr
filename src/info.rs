pub mod parse;
mod render;

use std::{fs, path::Path};

use winnow::{LocatingSlice, Parser};

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn read_nonsplit_manual<P: AsRef<Path>>(path: P) -> anyhow::Result<NonsplitInfoFile> {
    let content = fs::read_to_string(path)?;
    parse::nonsplit_info_file
        .parse(LocatingSlice::new(&content))
        .map_err(|e| anyhow::format_err!("{e}"))
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
    Heading(Heading),
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
pub struct Heading {
    text: String,
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
