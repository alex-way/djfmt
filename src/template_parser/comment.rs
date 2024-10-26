use winnow::{
    ascii::multispace0,
    combinator::{alt, delimited, peek},
    error::{ErrorKind, ParserError},
    prelude::*,
    stream::{AsChar, Compare, Offset, Stream, StreamIsPartial},
    token::{literal, take_until},
};

use crate::formatting::Formatable;

/// A comment tag. Can either be in the single line form (`{# comment #}`) or the multi-line form
/// (`{% comment %}
/// {% endcomment %}`).
#[derive(Debug, PartialEq)]
pub struct Comment<'i>(pub &'i str);

impl Formatable for Comment<'_> {
    fn formatted(&self, _indent_level: usize) -> String {
        self.0.to_string()
    }
}

impl<'i> Comment<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        alt((parse_single_line_comment, parse_multi_line_comment)).parse_next(input)
    }
}

pub fn parse_single_line_comment<'i>(input: &mut &'i str) -> PResult<Comment<'i>> {
    let contents = delimited("{#", take_until(0.., "#}"), "#}").parse_next(input)?;

    Ok(Comment(contents.trim()))
}

fn escape_tag<I, O, E>(parser: impl Parser<I, O, E>) -> impl Parser<I, O, E>
where
    I: StreamIsPartial + Stream + Compare<&'static str>,
    I::Token: AsChar + Clone,
    E: ParserError<I>,
{
    delimited(
        (literal("{%"), multispace0),
        parser,
        (multispace0, literal("%}")),
    )
}

pub fn parse_multi_line_comment<'i>(input: &mut &'i str) -> PResult<Comment<'i>> {
    escape_tag("comment").parse_next(input)?;

    let start = input.checkpoint();
    let mut end = input.checkpoint();

    while !input.is_empty() {
        let _chunk = take_until(0.., "{%").parse_next(input)?;
        end = input.checkpoint();

        let end_comment: PResult<_, ErrorKind> = peek(escape_tag("endcomment")).parse_next(input);
        if end_comment.is_ok() {
            break;
        }

        "{%".parse_next(input)?;
        end = input.checkpoint();
    }

    input.reset(&start);

    let diff = end.offset_from(&start);

    let comment = input.next_slice(diff);

    let _end_comment = escape_tag("endcomment").parse_next(input)?;

    Ok(Comment(comment))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use winnow::error::ErrorKind;

    use super::*;

    #[rstest]
    #[case::empty_comment("{% comment %}", "")]
    #[case::empty_comment("{%comment%}", "")]
    fn test_escape_tag(#[case] input: &str, #[case] expected: &str) {
        let mut input = input;

        escape_tag::<_, _, ErrorKind>("comment")
            .parse_next(&mut input)
            .unwrap();

        assert_eq!(input, expected)
    }

    #[rstest]
    #[case::single_line_no_spaces("{#comment#}", Comment("comment"))]
    #[case::single_line_spaces("{# comment #}", Comment("comment"))]
    #[case::single_line_excessive_spaces("{#  comment  #}", Comment("comment"))]
    #[case::single_line_excessive_spaces_and_hashes("{#  #comment#  #}", Comment("#comment#"))]
    fn test_single_line_comment_parsing(#[case] input: &str, #[case] expected: Comment) {
        let actual = parse_single_line_comment.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case::multi_line_no_spaces("{%comment%}{%endcomment%}", Comment(""))]
    #[case::multi_line_spaces("{% comment %}{% endcomment %}", Comment(""))]
    #[case::multi_line_spaces_with_content(
        "{% comment %}this is a comment{% endcomment %}",
        Comment("this is a comment")
    )]
    #[case::multi_line_spaces_with_multi_line_content(
        "{% comment %}this is a comment\n{% endcomment %}",
        Comment("this is a comment\n")
    )]
    #[case::multi_line_spaces_with_multi_line_content_and_variable(
        "{% comment %}this is a comment\n{{variable}}{% endcomment %}",
        Comment("this is a comment\n{{variable}}")
    )]
    #[case::multi_line_spaces_with_multi_line_content_and_variable_and_block(
        "{% comment %}this is a comment\n{{variable}}{% block %}{% endcomment %}",
        Comment("this is a comment\n{{variable}}{% block %}")
    )]
    fn test_multi_line_comment_parsing(#[case] input: &str, #[case] expected: Comment) {
        let actual = parse_multi_line_comment.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
