use std::io::{self, Write};

pub mod config;
mod control;
pub mod info;
pub mod pager;
mod parser_util;

pub struct RenderOptions {
    pub max_width: usize,
}

pub trait Manual {
    fn render<W>(&self, into: W, opt: RenderOptions) -> io::Result<()>
    where
        W: Write;

    fn title(&self) -> &str;
}
