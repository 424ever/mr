use super::*;
use crate::{
    control::{DELETE, backspace, delete, form_feed, line_feed, null, unit_separator},
    parser_util::{StrContextExt, count, take_until_and_consume},
};
use unicode_segmentation::UnicodeSegmentation;
use winnow::{
    Parser,
    ascii::{
        dec_uint, line_ending, multispace0, multispace1, newline, space0, space1, till_line_ending,
    },
    combinator::{
        alt, cut_err, delimited, dispatch, eof, fail, not, opt, peek, preceded, repeat,
        repeat_till, seq, terminated,
    },
    error::{ContextError, ErrMode, StrContext},
    stream::{AsChar, Offset as _, Range},
    token::{any, literal, one_of, take_till, take_until},
};

type Stream<'i> = &'i str;
type Result<T> = winnow::ModalResult<T>;

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
fn separator(input: &mut Stream<'_>) -> Result<()> {
    (opt(form_feed), unit_separator, opt(form_feed), line_feed)
        .void()
        .map_err(|_| ErrMode::Backtrack(ContextError::new()))
        .parse_next(input)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Whole-Manual.html
pub fn nonsplit_info_file(input: &mut Stream<'_>) -> Result<NonsplitInfoFile> {
    let preamble = preamble.parse_next(input)?;
    let nodes = repeat(1.., node.context("node".label())).parse_next(input)?;
    let tag_table = opt(tag_table).parse_next(input)?;
    let local_variables = opt(local_variables).parse_next(input)?;

    Ok(NonsplitInfoFile {
        preamble,
        nodes,
        tag_table,
        local_variables,
    })
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
    let content = repeat_till(1.., any, peek(separator)).parse_next(input)?.0;
    Ok(Preamble { content })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn node(input: &mut Stream<'_>) -> Result<Node> {
    let invalid_id_chars = &[',', '\t'];

    _ = separator
        .context("node start".label())
        .context("separator".expected())
        .parse_next(input)?;

    let file = delimited(
        ("File:", space1),
        take_until_and_consume(1.., ",").map(|s: &str| s.to_string()),
        space1,
    )
    .parse_next(input)?;

    let node = preceded(("Node:", space1), id(invalid_id_chars))
        .context("node identifier".label())
        .parse_next(input)?;

    let next = opt(preceded(
        (",", space1, "Next:", space1),
        id(invalid_id_chars),
    ))
    .context("next node".label())
    .parse_next(input)?;

    let prev = opt(preceded(
        (",", space1, "Prev:", space1),
        id(invalid_id_chars),
    ))
    .context("prev node".label())
    .parse_next(input)?;

    let up = opt(preceded((",", space1, "Up:", space1), id(invalid_id_chars)))
        .context("up node".label())
        .parse_next(input)?;

    let descr = terminated(opt(preceded("\t", take_until(1.., "\n"))), "\n\n")
        .parse_next(input)?
        .map(str::to_owned);

    let general_text = repeat_till(
        0..,
        cut_err(text_block(0)),
        alt((separator.void(), eof.void())),
    )
    .context("node body".label())
    .parse_next(input)?
    .0;

    Ok(Node {
        file,
        node,
        next,
        prev,
        up,
        descr,
        general_text,
    })
}

fn text_block<'a>(min_indent: usize) -> impl Parser<Stream<'a>, TextBlock, ErrMode<ContextError>> {
    move |input: &mut Stream<'a>| {
        let content = if min_indent == 0 {
            alt((
                top_level_text_block_content,
                nestable_text_block_content(min_indent),
            ))
            .parse_next(input)
        } else {
            nestable_text_block_content(min_indent).parse_next(input)
        }?;

        repeat::<_, _, (), _, _>(0.., line_ending).parse_next(input)?;

        Ok(TextBlock { content })
    }
}

fn top_level_text_block_content(input: &mut Stream<'_>) -> Result<TextBlockContent> {
    alt((
        printindex.map(TextBlockContent::Printindex),
        menu.map(TextBlockContent::Menu),
        heading.map(TextBlockContent::Heading),
    ))
    .parse_next(input)
}

fn nestable_text_block_content<'a>(
    min_indent: usize,
) -> impl Parser<Stream<'a>, TextBlockContent, ErrMode<ContextError>> {
    alt((
        table_entry(min_indent).map(TextBlockContent::TableEntry),
        verbatim(min_indent).map(TextBlockContent::Verbatim),
        paragraph(min_indent).map(TextBlockContent::Paragraph),
        repeat(
            1..,
            indented_line(min_indent)
                .verify(|l: &str| !l.is_empty())
                .map(|l| l.to_string()),
        )
        .map(TextBlockContent::BunchOfUnknownLines),
    ))
}

fn heading(input: &mut Stream<'_>) -> Result<Heading> {
    let text = terminated(till_line_ending, line_ending)
        .parse_next(input)?
        .to_string();
    let ul = peek(one_of(['*', '=', '-', '.'])).parse_next(input)?;
    repeat::<_, _, (), _, _>(text.graphemes(true).count(), literal(ul)).parse_next(input)?;
    line_ending.parse_next(input)?;

    Ok(Heading {
        level: match ul {
            '*' => HeadingLevel::Major,
            '=' => HeadingLevel::Section,
            '-' => HeadingLevel::SubSection,
            '.' => HeadingLevel::SubSubSection,
            _ => unreachable!(),
        },
        text,
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

    _ = space0.parse_next(input)?;

    let id = id(&['.', ',']).parse_next(input)?;
    _ = alt(('.', ',')).parse_next(input)?;

    let description = opt(preceded(
        space1,
        repeat(
            0..,
            preceded(
                (not(alt((newline, '*')).void()), space0),
                take_until_and_consume(1.., '\n').map(|l: &str| l.trim().to_string()),
            ),
        ),
    ))
    .parse_next(input)?
    .unwrap_or_default();
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
        .map_err(ErrMode::Backtrack)
        .parse_next(input)?;

    Ok(Printindex {
        entries: repeat(0.., terminated(index_entry, newline)).parse_next(input)?,
    })
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Printindex.html
fn index_entry(input: &mut Stream<'_>) -> Result<IndexEntry> {
    use winnow::stream::Stream as _;
    fn text_and_spec(line: &str) -> Result<(String, String)> {
        let (text, node_spec) = line
            .rsplit_once(':')
            .ok_or(ErrMode::Backtrack(ContextError::new()))?;
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
            let (_until_end_of_spec, rest) = first_line
                .rsplit_once('.')
                .ok_or(ErrMode::Backtrack(ContextError::new()))?;

            text_and_spec(input.next_slice(rest.offset_from(input) - 1))?
        }
        Some(_) => return Err(ErrMode::Backtrack(ContextError::new())),
        None => return Err(ErrMode::Backtrack(ContextError::new())),
    };

    let line = preceded(
        ('.', multispace1),
        delimited('(', preceded(("line", multispace1), dec_uint), ')'),
    )
    .parse_next(input)?;

    Ok(IndexEntry {
        text,
        node: Id {
            infofile: None,
            nodename: Some(node_spec),
        },
        line,
    })
}

fn paragraph<'a>(min_indent: usize) -> impl Parser<Stream<'a>, Paragraph, ErrMode<ContextError>> {
    fn valid_line(l: &str) -> bool {
        !l.is_empty() && !l.chars().next().unwrap().is_space()
    }

    (
        opt(indented_line((min_indent + 1)..=(min_indent + 3)).verify(valid_line)),
        repeat(
            0..,
            indented_line(min_indent)
                .verify(valid_line)
                .map(|l: &str| l.trim_end().to_string()),
        ),
    )
        .verify_map(|(more_indented_first, mut lines): (_, Vec<_>)| {
            if let Some(first) = more_indented_first {
                lines.insert(0, first.into());
            }
            if !lines.is_empty() {
                Some(Paragraph { lines })
            } else {
                None
            }
        })
}

fn table_entry<'a>(
    min_indent: usize,
) -> impl Parser<Stream<'a>, TableEntry, ErrMode<ContextError>> {
    (
        indented_line(min_indent),
        repeat(1.., text_block(min_indent + 5)),
    )
        .map(|(title, description)| TableEntry {
            title: title.to_string(),
            description,
        })
}

fn verbatim<'a>(min_indent: usize) -> impl Parser<Stream<'a>, Verbatim, ErrMode<ContextError>> {
    repeat(
        1..,
        indented_line(min_indent + 5).map(|l: &str| l.trim_end().to_string()),
    )
    .map(|lines: Vec<_>| Verbatim { lines })
}

fn indented_line<'a>(
    min_indent: impl Into<Range>,
) -> impl Parser<Stream<'a>, &'a str, ErrMode<ContextError>> {
    delimited(
        repeat::<_, _, (), _, _>(min_indent, ' ').take(),
        take_until(0.., '\n'),
        newline,
    )
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Regular-Nodes.html
fn id(invalid_id_chars: &[char]) -> impl Parser<Stream<'_>, Id, ErrMode<ContextError>> {
    seq! {Id{
        infofile: opt(delimited('(', take_until(1.., ')').map(|s:&str| s.to_string()), ')')).context("infofile".expected()),
        nodename: opt(node_spec(invalid_id_chars)).context("node spec".expected()),
    }}
    .context("node id".label())
    .map_err(ErrMode::Backtrack)
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
        bytepos: dec_uint.context(StrContext::Expected("bytepos".into())),
    }}
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

// https://www.gnu.org/software/texinfo/manual/texinfo/html_node/Info-Format-Local-Variables.html
fn local_variables(input: &mut Stream<'_>) -> Result<LocalVariables> {
    let _ = separator
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
mod tests;
