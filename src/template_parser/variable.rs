use super::filter::{parse_filter_chain, Filter};
use crate::formatting::Formatable;
use winnow::combinator::peek;
use winnow::token::take;
use winnow::{
    ascii::multispace0,
    combinator::{delimited, opt},
    token::take_while,
    PResult, Parser,
};

/// Parses a variable name, which can contain alphanumeric characters and underscores but must start with an
/// alphabetic character or underscore.
pub fn parse_variable<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let mut rest_chars = take_while(0.., |c: char| c.is_alphanumeric() || c == '_' || c == '.');

    peek(take(1usize))
        .verify(|c: &str| c.chars().all(|ch| ch.is_alphabetic() || ch == '_'))
        .parse_next(input)?;

    rest_chars.parse_next(input)
}

#[derive(Debug)]
pub struct VariableTag<'i> {
    pub tag_type: &'i str,
    pub filters: Vec<Filter<'i>>,
}

impl<'i> Formatable for VariableTag<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        let mut formatted = String::new();
        formatted.push_str("{{ ");
        formatted.push_str(self.tag_type);
        for filter in &self.filters {
            formatted.push_str(&filter.formatted(0));
        }
        formatted.push_str(" }}");
        formatted
    }
}

impl<'i> PartialEq for VariableTag<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type
    }
}

impl<'i> VariableTag<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let tag = delimited(
            ("{{", multispace0),
            (
                parse_variable,
                (
                    delimited(multispace0, opt('|'), multispace0),
                    parse_filter_chain,
                ),
            ),
            (multispace0, "}}"),
        )
        .parse_next(input)?;
        let tag = Self {
            tag_type: tag.0,
            filters: tag.1 .1,
        };
        Ok(tag)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::template_parser::{argument::TagArgumentValue, text::SingleLineTextString};

    use super::*;

    #[rstest]
    #[case::no_properties("{{ my_var }}", VariableTag {
        tag_type: "my_var",
        filters: vec![],
    })]
    #[case::single_property("{{my_var.property}}", VariableTag {
        tag_type: "my_var.property",
        filters: vec![],
    })]
    #[case::single_property_index("{{my_var.0}}", VariableTag {
        tag_type: "my_var.0",
        filters: vec![],
    })]
    #[case::nested_property("{{my_var.property.nested}}", VariableTag {
        tag_type: "my_var.property.nested",
        filters: vec![],
    })]
    #[case::nested_property_index("{{my_var.0.1}}", VariableTag {
        tag_type: "my_var.0.1",
        filters: vec![],
    })]
    #[case::single_filter("{{ my_var|my_filter }}", VariableTag {
        tag_type: "my_var",
        filters: vec![Filter {
            filter_type: "my_filter",
            argument: None,
        }],
    })]
    #[case::multiple_filters("{{ my_var|my_filter|my_filter2 }}", VariableTag {
        tag_type: "my_var",
        filters: vec![Filter {
            filter_type: "my_filter",
            argument: None,
        }, Filter {
            filter_type: "my_filter2",
            argument: None,
        }],
    })]
    #[case::multiple_filters_with_arguments("{{ my_var|my_filter:\"arg1\"|my_filter2:\"arg2\" }}", VariableTag {
        tag_type: "my_var",
        filters: vec![Filter {
            filter_type: "my_filter",
            argument: Some(
                TagArgumentValue::Text(SingleLineTextString {
                    value: "arg1",
                    startquote_char: '"',
                })
            ),
        }, Filter {
            filter_type: "my_filter2",
            argument: Some(
                TagArgumentValue::Text(SingleLineTextString {
                    value: "arg2",
                    startquote_char: '"',
                })
            ),
        }],
    })]
    #[case::multiple_filters_with_spaced_arguments("{{ my_var | my_filter : \"arg1\" | my_filter2:\"arg2\" }}", VariableTag {
        tag_type: "my_var",
        filters: vec![Filter {
            filter_type: "my_filter",
            argument: Some(
                TagArgumentValue::Text(SingleLineTextString {
                    value: "arg1",
                    startquote_char: '"',
                })
            ),
        }, Filter {
            filter_type: "my_filter2",
            argument: Some(
                TagArgumentValue::Text(SingleLineTextString {
                    value: "arg2",
                    startquote_char: '"',
                })
            ),
        }],
    })]
    fn test_parsing_variable_tag(#[case] input: &str, #[case] expected: VariableTag) {
        let actual = VariableTag::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case::no_properties("my_var")]
    #[case::no_properties("myvar")]
    #[case::no_properties("myvar0")]
    #[case::no_properties("_myvar0")]
    #[case::no_properties("_MY0VAR0")]
    #[case::no_properties("_MY0V_AR0__")]
    fn test_parse_variable(#[case] input: &str) {
        parse_variable.parse(input).unwrap();
    }

    #[rstest]
    #[case("")]
    #[case("0myvar")]
    #[case("my-var")]
    fn test_parse_variable_fails(#[case] input: &str) {
        let actual = parse_variable.parse(input);
        assert!(actual.is_err());
    }
}
