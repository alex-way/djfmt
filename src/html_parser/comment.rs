use winnow::{combinator::delimited, token::take_until, PResult, Parser};

pub fn parse_comment<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let mut comment = delimited("<!--", take_until(0.., "-->"), "-->");

    let content = comment.parse_next(input)?;
    Ok(content.trim())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("<!-- -->", "")]
    #[case("<!--     -->", "")]
    #[case("<!---->", "")]
    #[case("<!-- my comment -->", "my comment")]
    #[case("<!-- my-comment -->", "my-comment")]
    #[case("<!--my-comment-->", "my-comment")]
    #[case("<!--     my-comment       -->", "my-comment")]
    fn test_comment_parses_successfully(#[case] input: &str, #[case] expected: &str) {
        let actual = parse_comment.parse(input).unwrap();
        assert_eq!(actual, expected);
    }
}
