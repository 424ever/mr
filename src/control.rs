/// Parsers for C0 and C1 control codes.
/// See https://en.wikipedia.org/wiki/C0_and_C1_control_codes
use winnow::{
    Parser, Result,
    stream::{Compare, Stream, StreamIsPartial},
    token,
};

pub const NULL: char = '\x00';
pub const BACKSPACE: char = '\x08';
pub const DELETE: char = '\x7f';

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
