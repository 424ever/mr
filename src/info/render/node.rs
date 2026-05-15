use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{
    RenderOptions,
    info::{Id, Node, TextBlockContent},
};

impl Node {
    pub(super) fn render<W: Write>(
        &self,
        mut into: W,
        opt: RenderOptions,
        node_lines: &HashMap<Id, usize>,
    ) -> io::Result<()> {
        let opt = opt.indented(7);
        self.general_text.iter().try_for_each(|b| match &b.content {
            TextBlockContent::Paragraph(paragraph) => paragraph.render(&mut into, opt),
            TextBlockContent::Menu(menu) => menu.render(&mut into, opt, node_lines),
            TextBlockContent::Printindex(printindex) => {
                printindex.render(&mut into, opt, node_lines)
            }
            TextBlockContent::Heading(heading) => heading.render(&mut into),
        })
    }
}
