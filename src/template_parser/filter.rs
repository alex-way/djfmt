use crate::formatting::Formatable;

use super::{argument::TagArgumentValue, variable::parse_variable};
use winnow::{
    ascii::multispace0,
    combinator::{delimited, opt, separated},
    error::{ErrMode, ErrorKind, ParserError},
    PResult, Parser,
};

pub fn parse_filter_chain<'i>(input: &mut &'i str) -> PResult<Vec<Filter<'i>>> {
    let _ = opt(delimited(multispace0, '|', multispace0)).parse_next(input)?;

    let mut parse_filter = separated(0.., Filter::parse, (multispace0, '|', multispace0));
    parse_filter.parse_next(input)
}

#[derive(Debug)]
pub struct Filter<'i> {
    pub filter_type: &'i str,
    pub argument: Option<TagArgumentValue<'i>>,
}

impl<'i> PartialEq for Filter<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.filter_type == other.filter_type && self.argument == other.argument
    }
}

impl<'i> Filter<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let filter_type = parse_variable.parse_next(input)?;

        let semicolon = opt(delimited(multispace0, ':', multispace0)).parse_next(input)?;

        let argument = opt(TagArgumentValue::parse).parse_next(input)?;

        if semicolon.is_some() && argument.is_none() {
            return Err(ErrMode::from_error_kind(input, ErrorKind::Verify));
        }

        let filter = Self {
            filter_type,
            argument,
        };

        Ok(filter)
    }
}

impl<'i> Formatable for Filter<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        let return_string = format!("|{}", self.filter_type).to_string();
        match &self.argument {
            Some(argument) => match argument {
                TagArgumentValue::Text(text) => return_string + &format!(":\"{}\"", text.value),
                TagArgumentValue::Variable(variable) => return_string + &format!(":{}", variable),
            },
            None => return_string,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::template_parser::text::SingleLineTextString;

    use super::*;

    #[rstest]
    #[case::no_filters("", vec![])]
    #[case::single_simple_filter("my_filter", vec![Filter {
        filter_type: "my_filter",
        argument: None,
    }])]
    #[case::single_simple_filter("|my_filter", vec![Filter {
        filter_type: "my_filter",
        argument: None,
    }])]
    #[case::multiple_simple_filters("my_filter|my_filter2", vec![Filter {
        filter_type: "my_filter",
        argument: None,
    }, Filter {
        filter_type: "my_filter2",
        argument: None,
    }])]
    #[case::multiple_simple_filters_with_spaces("my_filter | my_filter2", vec![Filter {
        filter_type: "my_filter",
        argument: None,
    }, Filter {
        filter_type: "my_filter2",
        argument: None,
    }])]
    fn test_parsing_filter_chain(#[case] input: &str, #[case] expected: Vec<Filter>) {
        let actual = parse_filter_chain.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case::no_argument("my_filter", Filter {
        filter_type: "my_filter",
        argument: None,
    })]
    #[case::single_argument("my_filter:\"my_arg\"", Filter {
        filter_type: "my_filter",
        argument: Some(
            TagArgumentValue::Text(SingleLineTextString {
                value: "my_arg",
                startquote_char: '"',
            })
        ),
    })]
    #[case::single_argument("my_filter:'my_arg'", Filter {
        filter_type: "my_filter",
        argument: Some(
            TagArgumentValue::Text(SingleLineTextString {
                value: "my_arg",
                startquote_char: '\'',
            })
        ),
    })]
    fn test_filter_parsing(#[case] input: &str, #[case] expected: Filter) {
        let actual = Filter::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("my_filter:")]
    #[case("my_filter:\"")]
    #[case("my_filter:'")]
    #[case("my_filter'")]
    #[case("my_filter\"")]
    #[case("my_filter:\"my_arg")]
    #[case("my_filter:'my_arg")]
    #[case("my_filter:'my_arg\"")]
    fn test_filter_parsing_unsuccessful(#[case] input: &str) {
        let actual = Filter::parse.parse(input);
        assert!(actual.is_err())
    }

    #[rstest]
    #[case::no_argument(Filter {
        filter_type: "my_filter",
        argument: None,
    },"|my_filter")]
    #[case::single_argument(Filter {
        filter_type: "my_filter",
        argument: Some(
            TagArgumentValue::Text(SingleLineTextString {
                value: "my_arg",
                startquote_char: '"',
            })
        ),
    }, "|my_filter:\"my_arg\"")]
    fn test_formatting_filter(#[case] input: Filter, #[case] expected: String) {
        let actual = input.formatted(0);
        assert_eq!(actual, expected)
    }
}
