use winnow::combinator::delimited;
use winnow::error::ParserError;
use winnow::token::take_while;
use winnow::{
    combinator::{alt, rest},
    error::{ErrMode, ErrorKind},
    stream::AsChar,
    token::take_until,
    PResult, Parser,
};

use crate::formatting::Formatable;

/// Parses all non-template syntax text
pub fn parse_text<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let valid_token_starts = ("{%", "{{", "{#");
    alt((take_until(0.., valid_token_starts), rest)).parse_next(input)
}

#[derive(Debug, PartialEq)]
pub struct SingleLineTextString<'i> {
    pub value: &'i str,
    pub startquote_char: char,
}

impl<'i> SingleLineTextString<'i> {
    /// Parses a single line text string e.g. `'my_text'` or `"my_text"`
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let starts_with_single_quote = input.starts_with('\'');
        let starts_with_double_quote = input.starts_with('"');

        let startquote_char = match (starts_with_single_quote, starts_with_double_quote) {
            (true, false) => '\'',
            (false, true) => '"',
            _ => return Err(ErrMode::from_error_kind(input, ErrorKind::Verify)),
        };

        let value = delimited(
            startquote_char,
            take_while(0.., move |c: char| {
                c.is_ascii() && c != startquote_char && !c.is_newline()
            }),
            startquote_char,
        )
        .parse_next(input)?;

        let tag = Self {
            value,
            startquote_char,
        };
        Ok(tag)
    }
}

impl<'i> Formatable for SingleLineTextString<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        format!(
            "{}{}{}",
            self.startquote_char, self.value, self.startquote_char
        )
    }
}

impl Formatable for &str {
    fn formatted(&self, _indent_level: usize) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::take_all_text("\n", "\n", "")]
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

    #[rstest]
    #[case("\"my_text\"", SingleLineTextString{ value: "my_text", startquote_char: '\"' }, "")]
    #[case("'my_text'", SingleLineTextString{ value: "my_text", startquote_char: '\'' }, "")]
    #[case("\"my_text's\"", SingleLineTextString{ value: "my_text's", startquote_char: '\"' }, "")]
    #[case("\"my_text's\"thing", SingleLineTextString{ value: "my_text's", startquote_char: '\"' }, "thing")]
    fn test_parsing_single_line_text_string(
        #[case] input: &str,
        #[case] expected_extracted: SingleLineTextString,
        #[case] expected_remaining: &str,
    ) {
        let mut input = input;

        let actual = SingleLineTextString::parse.parse_next(&mut input).unwrap();

        assert_eq!(actual, expected_extracted);
        assert_eq!(input, expected_remaining);
    }
}
