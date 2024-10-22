use crate::formatting::Formatable;

use super::{
    filter::{parse_filter_chain, Filter},
    text::SingleLineTextString,
    variable::parse_variable,
};
use winnow::{PResult, Parser};

#[derive(Debug, PartialEq)]
pub enum TagArgumentValue<'i> {
    Text(SingleLineTextString<'i>),
    Variable(&'i str),
}

#[derive(Debug, PartialEq)]
pub struct TagArgument<'i> {
    pub value: TagArgumentValue<'i>,
    pub filters: Vec<Filter<'i>>,
}

impl<'i> TagArgument<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let starts_with_quote = input.starts_with('\'') || input.starts_with('"');

        let value = if starts_with_quote {
            let thing = SingleLineTextString::parse.parse_next(input)?;
            TagArgumentValue::Text(thing)
        } else {
            let variable = parse_variable.parse_next(input)?;
            TagArgumentValue::Variable(variable)
        };

        let filters = parse_filter_chain.parse_next(input)?;

        let argument = Self { value, filters };

        Ok(argument)
    }
}

impl<'i> Formatable for TagArgument<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("argument", TagArgument {
        value: TagArgumentValue::Variable("argument"),
        filters: vec![],
    })]
    #[case("'argument'", TagArgument {
        value: TagArgumentValue::Text(SingleLineTextString {
            value: "argument",
            startquote_char: '\'',
        }),
        filters: vec![],
    })]
    #[case("\"argument\"", TagArgument {
        value: TagArgumentValue::Text(SingleLineTextString {
            value: "argument",
            startquote_char: '"',
        }),
        filters: vec![],
    })]
    #[case("\"argument\"|my_filter", TagArgument {
        value: TagArgumentValue::Text(SingleLineTextString {
            value: "argument",
            startquote_char: '"',
        }),
        filters: vec![Filter {
            filter_type: "my_filter",
            argument: None,
        }],
    })]
    fn test_parsing_filter_chain(#[case] input: &str, #[case] expected: TagArgument) {
        let actual = TagArgument::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
