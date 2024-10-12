use winnow::{
    combinator::{alt, rest},
    token::take_until,
    PResult, Parser,
};

pub fn parse_text<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let valid_token_starts = ("{%", "{{", "{#");
    alt((take_until(0.., valid_token_starts), rest)).parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::take_all_text("my_text", "my_text", "")]
    #[case::dont_take_filter("{% my_filter %}", "", "{% my_filter %}")]
    #[case::take_text_up_until_filter("thing{% my_filter %}", "thing", "{% my_filter %}")]
    #[case::take_text_up_until_partial_filter("thing{%", "thing", "{%")]
    #[case::take_text_up_until_variable("thing{{", "thing", "{{")]
    #[case::take_text_up_until_single_line_comment("thing{#", "thing", "{#")]
    fn test_parsing_text(
        #[case] input: &str,
        #[case] expected_extracted: &str,
        #[case] expected_remaining: &str,
    ) {
        let mut input = input;

        let actual = parse_text.parse_next(&mut input).unwrap();

        assert_eq!(actual, expected_extracted);
        assert_eq!(input, expected_remaining);
    }
}
