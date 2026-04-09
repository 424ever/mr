use winnow::{LocatingSlice, Parser};

mod parse;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manual(NonsplitInfoFile);

impl Manual {
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.0.nodes.iter()
    }
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn parse_nonsplit_manual(input: &str) -> anyhow::Result<Manual> {
    parse::nonsplit_info_file
        .parse(LocatingSlice::new(input))
        .map(Manual)
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
    general_text: String,
    /// Offset (in bytes) at which `general_text` starts within the file
    text_offset: usize,
}

impl Node {
    pub fn text(&self) -> &str {
        &self.general_text
    }
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
