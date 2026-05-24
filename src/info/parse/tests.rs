use winnow::{Parser as _, combinator::repeat};

#[cfg(test)]
use pretty_assertions::assert_eq;

use super::*;

fn make_stream(text: &str) -> super::Stream<'_> {
    text
}

#[test]
fn node_with_menu() {
    let input = make_stream(concat!(
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
        repeat(1.., text_block(0),).parse(input),
        Ok(vec![
            TextBlock {
                content: TextBlockContent::Heading(Heading {
                    level: HeadingLevel::Major,
                    text: "Heading".to_string()
                })
            },
            TextBlock {
                content: TextBlockContent::Paragraph(Paragraph {
                    lines: vec!["Text".into()]
                })
            },
            TextBlock {
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
    let input = make_stream(concat!(
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
            up: Some(Id {
                infofile: Some("dir".to_string()),
                nodename: None
            }),
            descr: None,
            general_text: vec![],
        })
    );
}

#[test]
fn test_menu() {
    let input = make_stream(concat!(
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

#[test]
fn test_foreign_menu_entries() {
    let input = make_stream("* Label: (manual)Node.");
    assert_eq!(
        menu_entry_with_label.parse(input),
        Ok(MenuEntry {
            label: Some("Label".into()),
            description: vec![],
            id: Id {
                infofile: Some("manual".into()),
                nodename: Some("Node".into())
            },
            trailing_newlines: 0
        })
    );
}

#[test]
fn test_paragraph() {
    let input = make_stream("Line 1\n");
    assert_eq!(
        paragraph(0).parse(input),
        Ok(Paragraph {
            lines: vec!["Line 1".into()]
        })
    );

    let input = make_stream(concat!("Line 1\n", "Line 2\n"));
    assert_eq!(
        paragraph(0).parse(input),
        Ok(Paragraph {
            lines: vec!["Line 1".into(), "Line 2".into(),]
        })
    );

    let input = make_stream(concat!("   Line 1\n", "Line 2\n"));
    assert_eq!(
        paragraph(0).parse(input),
        Ok(Paragraph {
            lines: vec!["Line 1".into(), "Line 2".into(),]
        })
    );

    let input = make_stream(concat!("  Line 1\n", "  Line 2\n"));
    assert!(paragraph(0).parse(input).is_err());
    assert_eq!(
        paragraph(2).parse(input),
        Ok(Paragraph {
            lines: vec!["Line 1".into(), "Line 2".into(),]
        })
    );

    let input = make_stream(concat!("     Line 1\n", "  Line 2\n"));
    assert!(paragraph(0).parse(input).is_err());
    assert_eq!(
        paragraph(2).parse(input),
        Ok(Paragraph {
            lines: vec!["Line 1".into(), "Line 2".into(),]
        })
    );
}

#[test]
fn test_table_entry() {
    let input = make_stream(concat!(
        "title\n",
        "     descr 1 line 1\n",
        "     descr 1 line 2\n",
        "\n",
        "     descr 2 line 1\n",
        "     descr 2 line 2\n",
        "\n"
    ));
    assert_eq!(
        table_entry(0).parse(input),
        Ok(TableEntry {
            title: "title".to_string(),
            description: vec![
                TextBlock {
                    content: TextBlockContent::Paragraph(Paragraph {
                        lines: vec!["descr 1 line 1".to_string(), "descr 1 line 2".to_string()]
                    })
                },
                TextBlock {
                    content: TextBlockContent::Paragraph(Paragraph {
                        lines: vec!["descr 2 line 1".to_string(), "descr 2 line 2".to_string()]
                    })
                }
            ]
        })
    )
}

#[test]
fn test_dir_node() {
    let input = make_stream(concat!(
        "\x1f\n",
        "File: dir,\tNode: Top\tThis is the top of the INTO tree\n",
        "\n",
        "* Menu:\n",
        "\n",
        "Heading 1\n",
        "* Item 1: (manual). Foobar1\n",
        "* Item 2: (manual). Foobar2\n",
        "\n",
        "Heading 2\n",
        "* Item 3: (manual). Foobar3\n",
        "* Item 4: (manual)Item 4.\n",
    ));

    assert_eq!(
        node.parse(input),
        Ok(Node {
            file: "dir".into(),
            node: Id {
                infofile: None,
                nodename: Some("Top".into())
            },
            next: None,
            prev: None,
            up: None,
            descr: Some("This is the top of the INTO tree".into()),
            general_text: vec![TextBlock {
                content: TextBlockContent::Menu(Menu {
                    items: vec![
                        MenuItem::Comment(MenuComment {
                            lines: vec!["Heading 1".into()],
                            trailing_newlines: 0
                        }),
                        MenuItem::Entry(MenuEntry {
                            label: Some("Item 1".into()),
                            description: vec!["Foobar1".into()],
                            id: Id {
                                infofile: Some("manual".into()),
                                nodename: None
                            },
                            trailing_newlines: 0
                        }),
                        MenuItem::Entry(MenuEntry {
                            label: Some("Item 2".into()),
                            description: vec!["Foobar2".into()],
                            id: Id {
                                infofile: Some("manual".into()),
                                nodename: None
                            },
                            trailing_newlines: 1
                        }),
                        MenuItem::Comment(MenuComment {
                            lines: vec!["Heading 2".into()],
                            trailing_newlines: 0
                        }),
                        MenuItem::Entry(MenuEntry {
                            label: Some("Item 3".into()),
                            description: vec!["Foobar3".into()],
                            id: Id {
                                infofile: Some("manual".into()),
                                nodename: None
                            },
                            trailing_newlines: 0
                        }),
                        MenuItem::Entry(MenuEntry {
                            label: Some("Item 4".into()),
                            description: vec![],
                            id: Id {
                                infofile: Some("manual".into()),
                                nodename: Some("Item 4".into())
                            },
                            trailing_newlines: 1
                        }),
                    ]
                })
            }]
        })
    );
}
