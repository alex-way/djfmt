use winnow::{token::take_while, PResult, Parser};

/// Parse the text between tags
pub fn parse_text<'i>(input: &mut &'i str) -> PResult<&'i str> {
    take_while(0.., |c: char| c != '<').parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("hello there", "hello there")]
    fn test_text_parses_successfully(#[case] input: &str, #[case] expected: &str) {
        let actual = parse_text.parse(input).unwrap();
        assert_eq!(actual, expected);
    }
}
