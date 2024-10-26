use winnow::{
    ascii::multispace0,
    combinator::delimited,
    error::ParserError,
    prelude::*,
    stream::{AsChar, Stream, StreamIsPartial},
};

#[allow(dead_code)]
pub fn space_around<I, O, E>(parser: impl Parser<I, O, E>) -> impl Parser<I, O, E>
where
    I: StreamIsPartial + Stream,
    I::Token: AsChar + Clone,
    E: ParserError<I>,
{
    delimited(multispace0, parser, multispace0)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use winnow::{error::ErrorKind, token::literal};

    use super::*;

    #[rstest]
    fn test_space_around() {
        let input = "  comment  ";
        let actual = space_around::<_, _, ErrorKind>(literal("comment"))
            .parse(input)
            .unwrap();

        assert_eq!(actual, "comment")
    }
}
