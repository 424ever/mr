use std::fmt::Debug;
use winnow::{
    Parser,
    combinator::repeat,
    error::{ParserError, StrContext, StrContextValue},
    stream::{Compare, FindSlice, Range, Stream, StreamIsPartial},
    token::take_until,
};

pub fn take_until_and_consume<'a, Occurrences, Literal, Input, Error>(
    occurrences: Occurrences,
    literal: Literal,
) -> impl Parser<Input, <Input as Stream>::Slice, Error> + use<'a, Occurrences, Literal, Input, Error>
where
    Occurrences: Into<Range> + Clone + 'a,
    Input: StreamIsPartial + Stream + FindSlice<Literal> + Compare<Literal>,
    Literal: Clone + Debug + 'a,
    Error: ParserError<Input>,
{
    move |input: &mut Input| {
        let res = take_until(occurrences.clone(), literal.clone()).parse_next(input);
        winnow::token::literal(literal.clone()).parse_next(input)?;
        res
    }
}

pub fn count<Input, ParseNext, Output, Error>(
    occurrences: impl Into<Range>,
    parse_next: ParseNext,
) -> impl Parser<Input, usize, Error>
where
    Input: Stream,
    ParseNext: Parser<Input, Output, Error>,
    Error: ParserError<Input>,
{
    repeat(occurrences, parse_next)
}

pub trait StrContextExt {
    fn label(&self) -> StrContext;
    fn expected(&self) -> StrContext;
}

impl StrContextExt for &'static str {
    fn label(&self) -> StrContext {
        StrContext::Label(self)
    }

    fn expected(&self) -> StrContext {
        StrContext::Expected(StrContextValue::Description(self))
    }
}
