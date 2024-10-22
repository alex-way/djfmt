use crate::formatting::Formatable;

use super::variable::parse_variable;
use super::{argument::TagArgument, filter::parse_filter_chain};
use winnow::combinator::separated;
use winnow::{
    ascii::multispace0,
    combinator::{delimited, opt},
    error::ParserError,
    PResult, Parser,
};

/// A tag in a template. Can either be a simple tag (`{% my_tag %}`) or a tag with arguments
#[derive(Debug)]
pub struct Tag<'i> {
    pub tag_type: &'i str,
    pub arguments: Vec<TagArgument<'i>>,
}

impl<'i> PartialEq for Tag<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type
    }
}

impl<'i> Tag<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let (tag_type, arguments) = generic_tag((
            parse_variable,
            separated(
                0..,
                delimited(multispace0, TagArgument::parse, multispace0),
                ' ',
            ),
        ))
        .parse_next(input)?;

        let tag = Self {
            tag_type,
            arguments,
        };
        Ok(tag)
    }
}

impl<'i> Formatable for Tag<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        let mut formatted = String::new();
        formatted.push_str("{% ");
        formatted.push_str(self.tag_type);
        for argument in &self.arguments {
            formatted.push(' ');
            formatted.push_str(&argument.formatted(0));
        }
        formatted.push_str(" %}");
        formatted
    }
}

#[allow(dead_code)]
/// A simple parser for an individual tag: `{% my_tag %}`
pub fn parse_individual_tag<'i>(input: &mut &'i str) -> PResult<Tag<'i>> {
    let tag = generic_tag(parse_variable).parse_next(input)?;

    let arguments = separated(
        0..,
        delimited(multispace0, TagArgument::parse, multispace0),
        ' ',
    )
    .parse_next(input)?;

    let tag = Tag {
        tag_type: tag,
        arguments,
    };
    Ok(tag)
}

#[allow(dead_code)]
pub fn parse_specific_tag<'i>(input: &mut &'i str, tag_name: &str) -> PResult<Tag<'i>> {
    let tag = generic_tag((
        tag_name,
        delimited(multispace0, opt('|'), multispace0),
        parse_filter_chain,
    ))
    .parse_next(input)?;

    let arguments = separated(
        0..,
        delimited(multispace0, TagArgument::parse, multispace0),
        ' ',
    )
    .parse_next(input)?;

    let tag = Tag {
        tag_type: tag.0,
        arguments,
    };
    Ok(tag)
}

pub fn generic_tag<'i, O, E>(parser: impl Parser<&'i str, O, E>) -> impl Parser<&'i str, O, E>
where
    E: ParserError<&'i str>,
{
    delimited(("{%", multispace0), parser, (multispace0, "%}"))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::template_parser::{argument::TagArgumentValue, text::SingleLineTextString};

    use super::*;

    #[rstest]
    #[case::no_argument("{%my_tag%}", Tag {
        tag_type: "my_tag", arguments: vec![]
    })]
    #[case::no_argument_with_spaces("{% my_tag %}", Tag {
        tag_type: "my_tag", arguments: vec![]
    })]
    #[case::single_argument_with_spaces("{% my_tag \"my_arg\" %}", Tag {
        tag_type: "my_tag", arguments: vec![TagArgument {
            value: TagArgumentValue::Text(SingleLineTextString {
                value: "my_arg",
                startquote_char: '"',
            }),
            filters: vec![],
        }]
    })]
    fn test_tag_pair_parsing(#[case] input: &str, #[case] expected: Tag) {
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case::no_argument("{%my_tag%}", Tag {
        tag_type: "my_tag", arguments: vec![]
    })]
    #[case::no_argument_with_spaces("{% my_tag %}", Tag {
        tag_type: "my_tag", arguments: vec![]
    })]
    // #[case::single_argument_with_spaces("{% my_tag \"my_arg\" %}", Tag {
    //     tag_type: "my_tag", arguments: vec![TagArgument {
    //         value: TagArgumentValue::Text(SingleLineTextString {
    //             value: "my_arg",
    //             startquote_char: '"',
    //         }),
    //         filters: vec![],
    //     }]
    // })]
    fn test_parse_individual_tag(#[case] input: &str, #[case] expected: Tag) {
        let actual = parse_individual_tag.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
