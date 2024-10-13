use crate::formatting::Formatable;

use super::filter::parse_filter_chain;
use super::variable::parse_variable;
use winnow::{
    ascii::multispace0,
    combinator::{delimited, opt},
    error::ParserError,
    PResult, Parser,
};

/// A tag in a template. Can either be a simple tag (`{% my_tag %}`) or a tag with arguments
#[derive(Debug)]
pub struct Tag<'i> {
    tag_type: &'i str,
    #[allow(dead_code)]
    arguments: Vec<&'i str>,
}

impl<'i> PartialEq for Tag<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type
    }
}

impl<'i> Tag<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let tag = generic_tag((
            parse_variable,
            delimited(multispace0, opt('|'), multispace0),
            parse_filter_chain,
        ))
        .parse_next(input)?;

        let tag = Self {
            tag_type: tag.0,
            arguments: vec![],
            // arguments: tag.2,
        };
        Ok(tag)
    }
}

impl<'i> Formatable for Tag<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        format!("{:?}", self)
    }
}

#[allow(dead_code)]
/// A simple parser for an individual tag: `{% my_tag %}`
pub fn parse_individual_tag<'i>(input: &mut &'i str) -> PResult<Tag<'i>> {
    let tag = generic_tag(parse_variable).parse_next(input)?;

    let tag = Tag {
        tag_type: tag,
        arguments: vec![],
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

    let tag = Tag {
        tag_type: tag.0,
        arguments: vec![],
        // arguments: tag.2,
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

    use super::*;

    #[rstest]
    #[case::no_argument("{%my_tag%}", Tag {
        tag_type: "my_tag", arguments: vec![]
    })]
    #[case::no_argument_with_spaces("{% my_tag %}", Tag {
        tag_type: "my_tag", arguments: vec![]
    })]
    // #[case::single_argument_with_spaces("{% my_tag \"my_arg\" %}", Tag {
    //     tag_type: "my_tag", arguments: vec!["my_arg"]
    // })]
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
    //     tag_type: "my_tag", arguments: vec!["my_arg"]
    // })]
    fn test_parse_individual_tag(#[case] input: &str, #[case] expected: Tag) {
        let actual = parse_individual_tag.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
