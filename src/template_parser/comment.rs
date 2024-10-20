use winnow::{
    combinator::{alt, delimited},
    token::take_until,
    PResult, Parser,
};

/// A comment tag. Can either be in the single line form (`{# comment #}`) or the multi-line form
/// (`{% comment %}
/// {% endcomment %}`).
#[derive(Debug)]
pub struct Comment<'i> {
    pub contents: &'i str,
}

impl<'i> PartialEq for Comment<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.contents == other.contents
    }
}

impl<'i> Comment<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        alt((parse_single_line_comment, parse_multi_line_comment)).parse_next(input)
    }
}

pub fn parse_single_line_comment<'i>(input: &mut &'i str) -> PResult<Comment<'i>> {
    let contents = delimited("{#", take_until(0.., "#}"), "#}").parse_next(input)?;

    let tag = Comment {
        contents: contents.trim(),
    };
    Ok(tag)
}

pub fn parse_multi_line_comment<'i>(input: &mut &'i str) -> PResult<Comment<'i>> {
    let contents = delimited(
        alt(("{%comment%}", "{% comment %}")),
        take_until(0.., "{%"),
        alt(("{%endcomment%}", "{% endcomment %}")),
    )
    .parse_next(input)?;

    let tag = Comment {
        contents: contents.trim(),
    };
    Ok(tag)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::single_line_no_spaces("{#comment#}", Comment {
        contents: "comment"
    })]
    #[case::single_line_spaces("{# comment #}", Comment {
        contents: "comment"
    })]
    #[case::single_line_excessive_spaces("{#  comment  #}", Comment {
        contents: "comment"
    })]
    #[case::single_line_excessive_spaces_and_hashes("{#  #comment#  #}", Comment {
        contents: "#comment#"
    })]
    fn test_single_line_comment_parsing(#[case] input: &str, #[case] expected: Comment) {
        let actual = parse_single_line_comment.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case::multi_line_no_spaces("{%comment%}{%endcomment%}", Comment {
        contents: ""
    })]
    #[case::multi_line_spaces("{% comment %}{% endcomment %}", Comment {
        contents: ""
    })]
    #[case::multi_line_spaces_with_content("{% comment %}this is a comment{% endcomment %}", Comment {
        contents: "this is a comment"
    })]
    #[case::multi_line_spaces_with_multi_line_content("{% comment %}this is a comment\n{% endcomment %}", Comment {
        contents: "this is a comment"
    })]
    #[case::multi_line_spaces_with_multi_line_content_and_variable("{% comment %}this is a comment\n{{variable}}{% endcomment %}", Comment {
        contents: "this is a comment\n{{variable}}"
    })]
    // #[case::multi_line_spaces_with_multi_line_content_and_variable_and_block("{% comment %}this is a comment\n{{variable}}{% block %}{% endcomment %}", Comment {
    //     contents: "this is a comment\n{{variable}}{% block %}"
    // })]
    fn test_multi_line_comment_parsing(#[case] input: &str, #[case] expected: Comment) {
        let actual = parse_multi_line_comment.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
