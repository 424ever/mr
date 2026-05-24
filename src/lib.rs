use std::io::Write;

use terminal_size::{Height, Width};

pub mod config;
mod control;
pub mod info;
pub mod pager;
mod parser_util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderOptions {
    indent: usize,
    max_width: usize,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOptions {
    pub fn new() -> Self {
        Self {
            indent: 0,
            max_width: 79,
        }
    }

    pub fn new_for_terminal(dim: (Width, Height)) -> Self {
        Self {
            indent: 0,
            max_width: (dim.0.0.saturating_sub(1)).into(),
        }
    }

    pub fn indented(&self, indent: usize) -> Self {
        Self {
            indent: self.indent + indent,
            max_width: self.max_width - indent,
        }
    }

    pub fn indent(&self) -> usize {
        self.indent
    }

    pub fn max_width(&self) -> usize {
        self.max_width
    }
}

pub trait Manual {
    fn render<W>(&self, into: W, opt: RenderOptions) -> anyhow::Result<()>
    where
        W: Write;

    fn title(&self) -> &str;
}
