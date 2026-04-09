use crate::{
    StrContextExt,
    control::{DELETE, delete, form_feed, line_feed, unit_separator},
    take_until_and_consume,
};
use winnow::{
    Bytes, ModalResult, Parser, Result,
    ascii::{self, line_ending, space1, till_line_ending},
    combinator::{
        alt, delimited, dispatch, eof, fail, opt, peek, preceded, repeat, repeat_till, seq,
        terminated,
    },
    error::{ContextError, ParseError, StrContext},
    stream::{Compare, Stream, StreamIsPartial},
    token::{any, none_of, one_of, take_till, take_until},
};

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn separator(input: &mut &str) -> Result<(Option<char>, char, Option<char>, char)> {
    (opt(form_feed), unit_separator, opt(form_feed), line_feed).parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Manual {
    Nonsplit(NonsplitInfoFile),
    Split(SplitInfoMainFile, Vec<SplitInfoSubfile>),
}

impl Manual {
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        match self {
            Manual::Nonsplit(nonsplit) => nonsplit.nodes.iter(),
            Manual::Split(split_info_main_file, split_info_subfiles) => todo!(),
        }
    }
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn parse_nonsplit_manual(input: &str) -> anyhow::Result<Manual> {
    Ok(Manual::Nonsplit(
        nonsplit_info_file
            .parse(input)
            .map_err(|e| anyhow::format_err!("{e}"))?,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonsplitInfoFile {
    preamble: Preamble,
    nodes: Vec<Node>,
    tag_table: Option<TagTable>,
    local_variables: Option<LocalVariables>,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn nonsplit_info_file(input: &mut &str) -> Result<NonsplitInfoFile> {
    seq! {NonsplitInfoFile {
        preamble: preamble,
        nodes: repeat(0.., node),
        tag_table: opt(tag_table),
        local_variables: opt(local_variables),
    }}
    .context("non-split info file".label())
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitInfoMainFile {
    preamble: Preamble,
    indirect_table: IndirectTable,
    tag_table: TagTable,
    local_variables: LocalVariables,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn split_manual_main_file(input: &mut &str) -> Result<SplitInfoMainFile> {
    seq! {SplitInfoMainFile{
        preamble:preamble,
        indirect_table:indirect_table,
        tag_table:tag_table,
        local_variables:local_variables,
    }}
    .context("split info main file".label())
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitInfoSubfile {
    preamble: Preamble,
    nodes: Vec<Node>,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn split_info_subfile(input: &mut &str) -> Result<SplitInfoSubfile> {
    seq! {SplitInfoSubfile{
        preamble:preamble,
        nodes:repeat(0..,node),
    }}
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preamble {
    content: String,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Preamble.html
fn preamble(input: &mut &str) -> Result<Preamble> {
    // TODO: Don't bother parsing directory entries for now
    let content = repeat_till(1.., any, peek(separator)).parse_next(input)?.0;
    Ok(Preamble { content })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    file: String,
    node: Id,
    next: Option<Id>,
    prev: Option<Id>,
    up: Id,
    general_text: String,
}

impl Node {
    pub fn text(&self) -> &str {
        &self.general_text
    }
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn node(input: &mut &str) -> Result<Node> {
    seq! {Node {
        _:separator.context("separator".expected()),
        _:("File:",space1).context("node file text".expected()),
        file:take_until_and_consume(1.., ",").context("node file name".expected()).map(|s:&str|s.to_string()),
        _:space1.context("whitespace".expected()),
        _:("Node:",space1).context("node name text".expected()),
        node:id.context("node name id".expected()),
        _:(",".context("comma".expected()),space1.context("whitespace".expected())),
        next:opt(delimited(("Next:".context("node next text".expected()),space1.context("whitespace".expected())), id.context("node next".expected()), (",",space1))),
        prev:opt(delimited(("Prev:".context("node prev text".expected()),space1), id.context("node prev".expected()), (",",space1))),
        _:("Up:".context("node up text".expected()),space1),
        up:id.context("node up".expected()),
        _:"\n",
        general_text:repeat_till(0..,any,alt((eof.map(|_|()),peek(separator).map(|_|())))).context("general text".label()).map(|(s,_)|s),
    }}
    .context("node".label())
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    infofile: Option<String>,
    nodename: Option<String>,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn id(input: &mut &str) -> Result<Id> {
    seq! {Id{
        infofile: opt(delimited('(', take_until(1.., ')').map(|s:&str| s.to_string()), ')')).context("infofile".expected()),
        nodename: opt(alt((
            repeat(1..,none_of([DELETE, ',', '\n'])),
            delimited(delete, take_until(1.., DELETE).map(|s:&str| s.to_string()), delete),
        ))).context("nodename".expected()),
    }}
    .context("node id".label())
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagTable {
    indirect: bool,
    entries: Vec<TagTableEntry>,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag_table(input: &mut &str) -> Result<TagTable> {
    let _ = separator
        .context("separator".expected())
        .parse_next(input)?;
    let _ = "Tag Table:\n"
        .context("tag table header".expected())
        .parse_next(input)?;

    let table = seq! {TagTable{
        indirect: opt("(Indirect)\n").map(|o| o.is_some()),
        entries: repeat(0.., terminated(tag_table_entry, line_ending)),
    }}
    .context("tag table".label())
    .parse_next(input);

    let _ = separator
        .context("separator".expected())
        .parse_next(input)?;
    let _ = "End Tag Table\n\n"
        .context("tag table end".expected())
        .parse_next(input)?;

    table
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagTableEntry {
    Node(Tag),
    Ref(Tag),
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag_table_entry(input: &mut &str) -> Result<TagTableEntry> {
    dispatch! {take_until_and_consume(3..=4, ": ");
        "Node" => tag.map(TagTableEntry::Node),
        "Ref" => tag.map(TagTableEntry::Ref),
        _ => fail,
    }
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    nodeid: String,
    bytepos: u64,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag(input: &mut &str) -> Result<Tag> {
    seq! {Tag{
        nodeid: repeat_till(1.., any, delete).map(|(s,_): (String,_)| s).context(StrContext::Expected("nodeid".into())),
        bytepos: ascii::dec_uint.context(StrContext::Expected("bytepos".into())),
    }}
    .parse_next(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVariables {
    coding: Option<String>,
    language: Option<String>,
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Local-Variables.html
fn local_variables(input: &mut &str) -> Result<LocalVariables> {
    let _ = separator.parse_next(input)?;
    let _ = "Local Variables:\n".parse_next(input)?;

    let vars = seq! {LocalVariables{
        coding: opt(delimited("coding: ",till_line_ending.map(|s: &str| s.to_string()), line_ending)),
        language: opt(delimited("Info-documentlanguage: ", till_line_ending.map(|s: &str| s.to_string()), line_ending)),
    }}
    .parse_next(input);

    let _ = "End:\n".parse_next(input)?;
    vars
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndirectTable {
    entries: Vec<IndirectEntry>,
}

fn indirect_table(input: &mut &str) -> Result<IndirectTable> {
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndirectEntry {
    filename: String,
    bytepos: u64,
}

fn indirect_entry(input: &mut &str) -> Result<IndirectEntry> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::info::{Id, Node, node};

    #[test]
    fn node_name_with_special_chars() {
        let mut input = concat!(
            "\n",
            "File: file.info,  Node: node: 1,  Next: node (2),  Prev: (other)node 0,  Up: (dir)\n",
            "\n",
            "\n",
        );
        let node = node(&mut input);
        assert!(node.is_ok());
        assert_eq!(
            node,
            Ok(Node {
                file: "file.info".to_string(),
                node: Id {
                    infofile: None,
                    nodename: Some("node: 1".to_string())
                },
                next: Some(Id {
                    infofile: None,
                    nodename: Some("node (2)".to_string())
                }),
                prev: Some(Id {
                    infofile: Some("other".to_string()),
                    nodename: Some("node 0".to_string())
                }),
                up: Id {
                    infofile: Some("dir".to_string()),
                    nodename: None
                },
                general_text: "\n".to_string()
            })
        );
    }
}
