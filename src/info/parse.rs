use super::*;
use crate::{
    control::{DELETE, backspace, delete, null},
    parser_util::{StrContextExt, count, take_until_and_consume},
};
use winnow::{
    LocatingSlice, Parser, Result,
    ascii::{
        self, line_ending, multispace0, multispace1, newline, space0, space1, till_line_ending,
    },
    combinator::{
        alt, delimited, dispatch, fail, not, opt, peek, preceded, repeat, repeat_till, seq,
        terminated,
    },
    error::{ContextError, StrContext},
    stream::{Location, Offset},
    token::{any, literal, take_till, take_until},
};

type Stream<'i> = LocatingSlice<&'i str>;

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
// TODO: support optional form feeds
const SEPARATOR: &str = "\x1f\x0a";

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn nonsplit_info_file(input: &mut Stream<'_>) -> Result<NonsplitInfoFile> {
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
fn split_manual_main_file(input: &mut Stream<'_>) -> Result<SplitInfoMainFile> {
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
fn split_info_subfile(input: &mut Stream<'_>) -> Result<SplitInfoSubfile> {
    seq! {SplitInfoSubfile{
        preamble:preamble,
        nodes:repeat(0..,node),
    }}
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Preamble.html
fn preamble(input: &mut Stream<'_>) -> Result<Preamble> {
    // TODO: Don't bother parsing directory entries for now
    let content = take_until(1.., SEPARATOR).parse_next(input)?.to_string();
    Ok(Preamble { content })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn node(input: &mut LocatingSlice<&str>) -> Result<Node> {
    let invalid_id_chars = &[','];

    _ = SEPARATOR
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
    let node = id(invalid_id_chars)
        .context("node name id".expected())
        .parse_next(input)?;
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
        id(invalid_id_chars).context("node next".expected()),
        (",", space1),
    ))
    .parse_next(input)?;
    let prev = opt(delimited(
        ("Prev:".context("node prev text".expected()), space1),
        id(invalid_id_chars).context("node prev".expected()),
        (",", space1),
    ))
    .parse_next(input)?;
    let _ = ("Up:".context("node up text".expected()), space1).parse_next(input)?;
    let up = id(invalid_id_chars)
        .context("node up".expected())
        .parse_next(input)?;
    let _ = "\n\n".parse_next(input)?;

    let start_offset = input.current_token_start();

    let general_text: Vec<_> = repeat(0.., text_block)
        .context("general text".label())
        .parse_next(input)?;

    Ok(Node {
        file,
        node,
        next,
        prev,
        up,
        general_text,
        start_offset,
    })
}

fn text_block(input: &mut Stream<'_>) -> Result<TextBlock> {
    not(peek(SEPARATOR)).parse_next(input)?;

    let start_offset = input.current_token_start();

    let content = alt((
        printindex.map(TextBlockContent::Printindex),
        menu.map(TextBlockContent::Menu),
        paragraph.map(TextBlockContent::Paragraph),
    ))
    .parse_next(input)?;
    let _: Vec<_> = repeat(0.., newline)
        .context("newlines between text blocks".label())
        .parse_next(input)?;

    let end_offset = input.previous_token_end();

    Ok(TextBlock {
        start_offset,
        end_offset,
        content,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Menu.html
fn menu(input: &mut Stream<'_>) -> Result<Menu> {
    ("* Menu:", repeat::<_, _, (), _, _>(1.., newline))
        .void()
        .parse_next(input)?;
    _ = multispace0(input)?;
    let items = repeat(0.., menu_item.context("menu item".label())).parse_next(input)?;

    Ok(Menu { items })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Menu.html
fn menu_item(input: &mut Stream<'_>) -> Result<MenuItem> {
    not(peek(SEPARATOR)).parse_next(input)?;
    alt((
        menu_entry_without_label
            .context("item without label".label())
            .map(MenuItem::Entry),
        menu_entry_with_label
            .context("item with label".label())
            .map(MenuItem::Entry),
        menu_comment
            .context("comment".label())
            .map(MenuItem::Comment),
    ))
    .context("menu item".label())
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Menu.html
fn menu_entry_with_label(input: &mut Stream<'_>) -> Result<MenuEntry> {
    _ = "* ".parse_next(input)?;
    let label = alt((
        preceded(DELETE, take_until_and_consume(0.., DELETE)),
        take_until_and_consume(1.., ":"),
    ))
    .parse_next(input)?
    .to_string();
    let id = id(&['.', ',']).parse_next(input)?;
    _ = alt(('.', ',')).parse_next(input)?;
    let description = preceded(
        space1,
        repeat(
            0..,
            preceded(
                (not(alt((newline, '*')).void()), space0),
                take_until_and_consume(1.., '\n').map(|l: &str| l.trim().to_string()),
            ),
        ),
    )
    .parse_next(input)?;
    let trailing_newlines = count(0.., newline).parse_next(input)?;

    Ok(MenuEntry {
        label: Some(label),
        description,
        id,
        trailing_newlines,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Menu.html
fn menu_entry_without_label(input: &mut Stream<'_>) -> Result<MenuEntry> {
    _ = "* ".parse_next(input)?;
    let id = id(&[':']).context("id".expected()).parse_next(input)?;
    _ = "::".parse_next(input)?;
    let description = alt((
        newline.map(|_| vec![]),
        preceded(
            space1,
            repeat(
                0..,
                preceded(
                    (not(alt((newline, '*')).void()), space0),
                    take_until_and_consume(1.., '\n').map(|l: &str| l.trim().to_string()),
                ),
            ),
        ),
    ))
    .parse_next(input)?;
    let trailing_newlines = count(0.., newline).parse_next(input)?;

    Ok(MenuEntry {
        label: None,
        description,
        id,
        trailing_newlines,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Menu.html
fn menu_comment(input: &mut Stream<'_>) -> Result<MenuComment> {
    let lines = repeat(
        1..,
        (not(alt((newline, '*'))), take_until(0.., '\n'), newline)
            .take()
            .context("line".expected())
            .map(|l: &str| l.trim().to_string()),
    )
    .context("paragraph".label())
    .parse_next(input)?;

    let trailing_newlines = count(0.., newline).parse_next(input)?;

    Ok(MenuComment {
        lines,
        trailing_newlines,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Printindex.html
fn printindex(input: &mut Stream<'_>) -> Result<Printindex> {
    (
        null,
        backspace,
        literal("[index"),
        null,
        backspace,
        literal("]\n* Menu:\n\n"),
    )
        .parse_next(input)?;

    Ok(Printindex {
        entries: repeat(0.., terminated(index_entry, newline)).parse_next(input)?,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Printindex.html
fn index_entry(input: &mut Stream<'_>) -> Result<IndexEntry> {
    use winnow::stream::Stream as _;
    fn text_and_spec(line: &str) -> Result<(String, String)> {
        let (text, node_spec) = line.rsplit_once(':').ok_or(ContextError::new())?;
        Ok((text.trim().to_string(), node_spec.trim().to_string()))
    }

    _ = "* ".parse_next(input)?;

    let first_newline = input
        .offset_for(|c| c == '\n')
        .unwrap_or(input.eof_offset());
    let first_line = input.peek_slice(first_newline).trim_end();

    let (text, node_spec) = match first_line.chars().last() {
        // TODO: handle DELETE characters
        Some('.') => {
            // The first line only contains <entry text> and <node spec>
            //
            // Start at the next line for <line spec>
            text_and_spec(input.next_slice(first_newline - 1))?
        }
        Some(')') => {
            // The entire entry is on one line
            //
            // Start in this line after the '.' for <line_spec>
            let (_until_end_of_spec, rest) =
                first_line.rsplit_once('.').ok_or(ContextError::new())?;

            text_and_spec(input.next_slice(rest.offset_from(input.as_ref()) - 1))?
        }
        Some(_) => return Err(ContextError::new()),
        None => return Err(ContextError::new()),
    };

    let line_spec = preceded(
        ('.', multispace1),
        delimited('(', take_until(1.., ')'), ')'),
    )
    .parse_next(input)?
    .to_string();

    Ok(IndexEntry {
        text,
        node_spec,
        line_spec,
    })
}

fn paragraph(input: &mut Stream<'_>) -> Result<Paragraph> {
    terminated(
        repeat(
            1..,
            (not(newline), take_until(0.., '\n'), newline)
                .take()
                .context("line".expected())
                .map(|l: &str| l.trim_end().to_string()),
        )
        .map(|lines: Vec<_>| Paragraph { lines })
        .context("paragraph".label()),
        newline,
    )
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn id(invalid_id_chars: &[char]) -> impl Parser<Stream<'_>, Id, ContextError> {
    seq! {Id{
        infofile: opt(delimited('(', take_until(1.., ')').map(|s:&str| s.to_string()), ')')).context("infofile".expected()),
        nodename: opt(node_spec(invalid_id_chars)).context("node spec".expected()),
    }}
    .context("node id".label())
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn node_spec(terminating_chars: &[char]) -> impl Parser<Stream<'_>, String, ContextError> {
    alt((
        take_till(1.., move |t| {
            t == DELETE || t == '\n' || terminating_chars.contains(&t)
        })
        .output_into(),
        delimited(
            delete,
            take_until(1.., DELETE).map(|s: &str| s.to_string()),
            delete,
        ),
    ))
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag_table(input: &mut Stream<'_>) -> Result<TagTable> {
    let _ = SEPARATOR
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

    let _ = SEPARATOR
        .context("separator".expected())
        .parse_next(input)?;
    let _ = "End Tag Table\n\n"
        .context("tag table end".expected())
        .parse_next(input)?;

    table
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag_table_entry(input: &mut Stream<'_>) -> Result<TagTableEntry> {
    dispatch! {take_until_and_consume(3..=4, ": ");
        "Node" => tag.map(TagTableEntry::Node),
        "Ref" => tag.map(TagTableEntry::Ref),
        _ => fail,
    }
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Tag-Table.html
fn tag(input: &mut Stream<'_>) -> Result<Tag> {
    seq! {Tag{
        nodeid: repeat_till(1.., any, delete).map(|(s,_): (String,_)| s).context(StrContext::Expected("nodeid".into())),
        bytepos: ascii::dec_uint.context(StrContext::Expected("bytepos".into())),
    }}
    .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Local-Variables.html
fn local_variables(input: &mut Stream<'_>) -> Result<LocalVariables> {
    let _ = SEPARATOR
        .context("separator".expected())
        .parse_next(input)?;
    let _ = "Local Variables:\n".parse_next(input)?;

    let vars = seq! {LocalVariables{
        coding: opt(delimited("coding: ",till_line_ending.map(|s: &str| s.to_string()), line_ending)),
        language: opt(delimited("Info-documentlanguage: ", till_line_ending.map(|s: &str| s.to_string()), line_ending)),
    }}
    .parse_next(input);

    let _ = "End:\n".parse_next(input)?;
    vars
}

fn indirect_table(_input: &mut Stream<'_>) -> Result<IndirectTable> {
    todo!()
}

#[expect(dead_code)]
fn indirect_entry(_input: &mut Stream<'_>) -> Result<IndirectEntry> {
    todo!()
}

#[cfg(test)]
mod tests {
    use winnow::{LocatingSlice, Parser as _, combinator::repeat};

    use super::node;
    use crate::info::{
        Id, Menu, MenuComment, MenuEntry, MenuItem, Node, Paragraph, TextBlock, TextBlockContent,
        parse::{menu, text_block},
    };

    #[test]
    fn node_with_menu() {
        let input = LocatingSlice::new(concat!(
            "Heading\n",
            "*******\n",
            "\n",
            "Text\n",
            "\n",
            "* Menu:\n",
            "\n",
            "* Item 1:: Description\n",
            "* Label:Item 2. Description\n",
        ));
        assert_eq!(
            repeat(1.., text_block).parse(input),
            Ok(vec![
                TextBlock {
                    start_offset: 0,
                    end_offset: 17,
                    content: TextBlockContent::Paragraph(Paragraph {
                        lines: vec!["Heading".into(), "*******".into()]
                    })
                },
                TextBlock {
                    start_offset: 17,
                    end_offset: 23,
                    content: TextBlockContent::Paragraph(Paragraph {
                        lines: vec!["Text".into()]
                    })
                },
                TextBlock {
                    start_offset: 23,
                    end_offset: 83,
                    content: TextBlockContent::Menu(Menu {
                        items: vec![
                            MenuItem::Entry(MenuEntry {
                                label: None,
                                description: vec!["Description".into()],
                                id: Id {
                                    infofile: None,
                                    nodename: Some("Item 1".into())
                                },
                                trailing_newlines: 0
                            }),
                            MenuItem::Entry(MenuEntry {
                                label: Some("Label".into()),
                                description: vec!["Description".into()],
                                id: Id {
                                    infofile: None,
                                    nodename: Some("Item 2".into())
                                },
                                trailing_newlines: 0
                            })
                        ]
                    })
                }
            ])
        );
    }

    #[test]
    fn node_name_with_special_chars() {
        let input = LocatingSlice::new(concat!(
            "\x1f\n",
            "File: file.info,  Node: node: 1,  Next: node (2),  Prev: (other)node 0,  Up: (dir)\n",
            "\n",
        ));
        let node = node.parse(input);
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
                general_text: vec![],
                start_offset: 86,
            })
        );
    }

    #[test]
    fn test_menu() {
        let input = LocatingSlice::new(concat!(
            "* Menu:\n",
            "A comment paragraph\n",
            "\n",
            "* Item 1:: Description I\n",
            "Description II\n",
            "\n",
            "\n",
            "Another comment paragraph\n",
            "with 2 lines\n",
            "* Item 2:: Description I\n",
            " Description II\n",
            "* Label for item 3:Item 3. Description\n",
            "* Item 4::\n",
            "\n",
        ));

        assert_eq!(
            menu.parse(input),
            Ok(Menu {
                items: vec![
                    MenuItem::Comment(MenuComment {
                        lines: vec!["A comment paragraph".into()],
                        trailing_newlines: 1
                    }),
                    MenuItem::Entry(MenuEntry {
                        label: None,
                        description: vec!["Description I".into(), "Description II".into()],
                        id: Id {
                            infofile: None,
                            nodename: Some("Item 1".into())
                        },
                        trailing_newlines: 2
                    }),
                    MenuItem::Comment(MenuComment {
                        lines: vec!["Another comment paragraph".into(), "with 2 lines".into()],
                        trailing_newlines: 0
                    }),
                    MenuItem::Entry(MenuEntry {
                        label: None,
                        description: vec!["Description I".into(), "Description II".into()],
                        id: Id {
                            infofile: None,
                            nodename: Some("Item 2".into())
                        },
                        trailing_newlines: 0
                    }),
                    MenuItem::Entry(MenuEntry {
                        label: Some("Label for item 3".into()),
                        description: vec!["Description".into()],
                        id: Id {
                            infofile: None,
                            nodename: Some("Item 3".into())
                        },
                        trailing_newlines: 0
                    }),
                    MenuItem::Entry(MenuEntry {
                        label: None,
                        description: vec![],
                        id: Id {
                            infofile: None,
                            nodename: Some("Item 4".into())
                        },
                        trailing_newlines: 1
                    }),
                ]
            })
        );
    }
}
