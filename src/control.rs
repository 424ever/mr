/// Parsers for C0 and C1 control codes.
/// See https://en.wikipedia.org/wiki/C0_and_C1_control_codes
use winnow::{
    Bytes, Parser, Result,
    stream::{Compare, Stream, StreamIsPartial},
    token,
};

pub const DELETE: char = '\x7f';

/// ^L
pub fn form_feed<S: Stream + Compare<char> + StreamIsPartial>(input: &mut S) -> Result<char> {
    '\x0c'.parse_next(input)
}

/// ^_
pub fn unit_separator<S: Stream + Compare<char> + StreamIsPartial>(input: &mut S) -> Result<char> {
    '\x1f'.parse_next(input)
}

/// ^J
pub fn line_feed<S: Stream + Compare<char> + StreamIsPartial>(input: &mut S) -> Result<char> {
    '\x0a'.parse_next(input)
}

/// ^?
pub fn delete<S: Stream + Compare<char> + StreamIsPartial>(
    input: &mut S,
) -> Result<<S as Stream>::Slice> {
    token::literal(DELETE).parse_next(input)
}
