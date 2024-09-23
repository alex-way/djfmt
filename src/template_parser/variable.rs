use super::filter::{parse_filter_chain, Filter};
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
    tag_type: &'i str,
    filters: Vec<Filter<'i>>,
}

impl<'i> PartialEq for VariableTag<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type
    }
}

impl<'i> VariableTag<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let tag = delimited(
            delimited(multispace0, "{{", multispace0),
            (
                parse_variable,
                (
                    delimited(multispace0, opt('|'), multispace0),
                    parse_filter_chain,
                ),
            ),
            delimited(multispace0, "}}", multispace0),
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
            argument: Some("arg1"),
        }, Filter {
            filter_type: "my_filter2",
            argument: Some("arg2"),
        }],
    })]
    #[case::multiple_filters_with_spaced_arguments("{{ my_var | my_filter : \"arg1\" | my_filter2:\"arg2\" }}", VariableTag {
        tag_type: "my_var",
        filters: vec![Filter {
            filter_type: "my_filter",
            argument: Some("arg1"),
        }, Filter {
            filter_type: "my_filter2",
            argument: Some("arg2"),
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
