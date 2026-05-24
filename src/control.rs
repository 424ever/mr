/// Parsers for C0 and C1 control codes.
/// See https://en.wikipedia.org/wiki/C0_and_C1_control_codes
use winnow::{
    Parser, Result,
    stream::{Compare, Stream, StreamIsPartial},
    token,
};

pub const BACKSPACE: char = '\x08';
pub const DELETE: char = '\x7f';
pub const FORM_FEED: char = '\x0c';
pub const NULL: char = '\x00';
pub const UNIT_SEPARATOR: char = '\x1f';
pub const LINE_FEED: char = '\x0a';

/// ^L
pub fn form_feed<S: Stream + StreamIsPartial + Compare<char>>(input: &mut S) -> Result<S::Slice> {
    token::literal(FORM_FEED).parse_next(input)
}

/// ^_
pub fn unit_separator<S: Stream + StreamIsPartial + Compare<char>>(
    input: &mut S,
) -> Result<S::Slice> {
    token::literal(UNIT_SEPARATOR).parse_next(input)
}

/// ^J
pub fn line_feed<S: Stream + StreamIsPartial + Compare<char>>(input: &mut S) -> Result<S::Slice> {
    token::literal(LINE_FEED).parse_next(input)
}

/// ^@
pub fn null<S: Stream + StreamIsPartial + Compare<char>>(input: &mut S) -> Result<S::Slice> {
    token::literal(NULL).parse_next(input)
}

/// ^H
pub fn backspace<S: Stream + StreamIsPartial + Compare<char>>(input: &mut S) -> Result<S::Slice> {
    token::literal(BACKSPACE).parse_next(input)
}

/// ^?
pub fn delete<S: Stream + Compare<char> + StreamIsPartial>(
    input: &mut S,
) -> Result<<S as Stream>::Slice> {
    token::literal(DELETE).parse_next(input)
}
