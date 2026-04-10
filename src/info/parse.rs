use super::*;
use crate::{
    control::{DELETE, delete, form_feed, line_feed, unit_separator},
    parser_util::StrContextExt,
    parser_util::take_until_and_consume,
};
use winnow::{
    LocatingSlice, Parser, Result,
    ascii::{self, line_ending, space1, till_line_ending},
    combinator::{
        alt, delimited, dispatch, eof, fail, opt, peek, repeat, repeat_till, seq, terminated,
    },
    error::StrContext,
    stream::Location,
    token::{any, none_of, take_until},
};

type Stream<'i> = LocatingSlice<&'i str>;

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn separator<'a>(
    input: &mut Stream<'a>,
) -> Result<(Option<&'a str>, &'a str, Option<&'a str>, &'a str)> {
    (opt(form_feed), unit_separator, opt(form_feed), line_feed).parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub(super) fn nonsplit_info_file(input: &mut Stream) -> Result<NonsplitInfoFile> {
    seq! {NonsplitInfoFile {
        preamble: preamble,
        nodes: repeat(0.., node),
        tag_table: opt(tag_table),
        local_variables: opt(local_variables),
    }}
    .context("non-split info file".label())
    .parse_next(input)
}

#[expect(dead_code)]
// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn split_manual_main_file(input: &mut Stream) -> Result<SplitInfoMainFile> {
    seq! {SplitInfoMainFile{
        preamble:preamble,
        indirect_table:indirect_table,
        tag_table:tag_table,
        local_variables:local_variables,
    }}
    .context("split info main file".label())
    .parse_next(input)
}

#[expect(dead_code)]
// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn split_info_subfile(input: &mut Stream) -> Result<SplitInfoSubfile> {
    seq! {SplitInfoSubfile{
        preamble:preamble,
        nodes:repeat(0..,node),
    }}
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Preamble.html
fn preamble(input: &mut Stream) -> Result<Preamble> {
    // TODO: Don't bother parsing directory entries for now
    let content = repeat_till(1.., any, peek(separator)).parse_next(input)?.0;
    Ok(Preamble { content })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn node(input: &mut LocatingSlice<&str>) -> Result<Node> {
    _ = separator
        .context("separator".expected())
        .parse_next(input)?;
    _ = ("File:", space1)
        .context("node file text".expected())
        .parse_next(input)?;
    let file = take_until_and_consume(1.., ",")
        .context("node file name".expected())
        .map(|s: &str| s.to_string())
        .parse_next(input)?;
    _ = space1.context("whitespace".expected()).parse_next(input)?;
    _ = ("Node:", space1)
        .context("node name text".expected())
        .parse_next(input)?;
    let node = id.context("node name id".expected()).parse_next(input)?;
    _ = (
        ",".context("comma".expected()),
        space1.context("whitespace".expected()),
    )
        .parse_next(input)?;
    let next = opt(delimited(
        (
            "Next:".context("node next text".expected()),
            space1.context("whitespace".expected()),
        ),
        id.context("node next".expected()),
        (",", space1),
    ))
    .parse_next(input)?;
    let prev = opt(delimited(
        ("Prev:".context("node prev text".expected()), space1),
        id.context("node prev".expected()),
        (",", space1),
    ))
    .parse_next(input)?;
    let _ = ("Up:".context("node up text".expected()), space1).parse_next(input)?;
    let up = id.context("node up".expected()).parse_next(input)?;
    let _ = "\n".parse_next(input)?;

    let text_offset = input.current_token_start();

    let general_text = repeat_till(
        0..,
        any,
        alt((eof.map(|_| ()), peek(separator).map(|_| ()))),
    )
    .context("general text".label())
    .map(|(s, _)| s)
    .parse_next(input)?;

    Ok(Node {
        file,
        node,
        next,
        prev,
        up,
        general_text,
        text_offset,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn id(input: &mut Stream) -> Result<Id> {
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

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag_table(input: &mut Stream) -> Result<TagTable> {
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

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag_table_entry(input: &mut Stream) -> Result<TagTableEntry> {
    dispatch! {take_until_and_consume(3..=4, ": ");
        "Node" => tag.map(TagTableEntry::Node),
        "Ref" => tag.map(TagTableEntry::Ref),
        _ => fail,
    }
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag(input: &mut Stream) -> Result<Tag> {
    seq! {Tag{
        nodeid: repeat_till(1.., any, delete).map(|(s,_): (String,_)| s).context(StrContext::Expected("nodeid".into())),
        bytepos: ascii::dec_uint.context(StrContext::Expected("bytepos".into())),
    }}
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Local-Variables.html
fn local_variables(input: &mut Stream) -> Result<LocalVariables> {
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

fn indirect_table(_input: &mut Stream) -> Result<IndirectTable> {
    todo!()
}

#[expect(dead_code)]
fn indirect_entry(_input: &mut Stream) -> Result<IndirectEntry> {
    todo!()
}

#[cfg(test)]
mod tests {
    use winnow::LocatingSlice;

    use super::node;
    use crate::info::{Id, Node};

    #[test]
    fn node_name_with_special_chars() {
        let mut input = LocatingSlice::new(concat!(
            "\x1f\n",
            "File: file.info,  Node: node: 1,  Next: node (2),  Prev: (other)node 0,  Up: (dir)\n",
            "\n",
            "\x1f\n",
        ));
        let node = node(&mut input);
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
                general_text: "\n".to_string(),
                text_offset: 85,
            })
        );
    }
}
