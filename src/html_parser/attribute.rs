use std::{collections::HashMap, hash::BuildHasher};
use winnow::{
    ascii::{multispace0, multispace1},
    combinator::{alt, delimited, opt, separated, separated_pair},
    token::take_while,
    PResult, Parser,
};

const INVALID_ATTRIBUTE_CHARS: &[char] = &['>', '<', '/', '=', '"', '\'', ' ', '\t', '\n', '\r'];
const INVALID_UNQUOTED_ATTRIBUTE_CHARS: &[char] = INVALID_ATTRIBUTE_CHARS;
const INVALID_DOUBLE_QUOTED_ATTRIBUTE_CHARS: &[char] = &['"'];
const INVALID_SINGLE_QUOTED_ATTRIBUTE_CHARS: &[char] = &['\''];

/// Parse the key of a HTML attribute
fn parse_key<'i>(input: &mut &'i str) -> PResult<&'i str> {
    take_while(1.., |c: char| !INVALID_ATTRIBUTE_CHARS.contains(&c)).parse_next(input)
}

/// Parses an HTML attribute value that is not quoted.
fn parse_unquoted_val<'i>(input: &mut &'i str) -> PResult<&'i str> {
    take_while(1.., |c: char| {
        !INVALID_UNQUOTED_ATTRIBUTE_CHARS.contains(&c)
    })
    .parse_next(input)
}

/// Parse the value of an HTML attribute
fn parse_double_quoted_val<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let inner = take_while(0.., |c: char| {
        !INVALID_DOUBLE_QUOTED_ATTRIBUTE_CHARS.contains(&c)
    });
    delimited('"', inner, '"').parse_next(input)
}

fn parse_single_quoted_val<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let inner = take_while(0.., |c: char| {
        !INVALID_SINGLE_QUOTED_ATTRIBUTE_CHARS.contains(&c)
    });
    delimited('\'', inner, '\'').parse_next(input)
}

/// Parses an HTML attribute.
/// Looks something like `key="val"`.
fn parse_attribute<'i>(input: &mut &'i str) -> PResult<(&'i str, Option<&'i str>)> {
    separated_pair(
        parse_key,
        opt(delimited(multispace0, '=', multispace0)),
        opt(alt((
            parse_double_quoted_val,
            parse_single_quoted_val,
            parse_unquoted_val,
        ))),
    )
    .parse_next(input)
}

/// HTML attributes
#[derive(Debug)]
pub struct Attributes<'i, S> {
    pub kvs: HashMap<&'i str, Option<&'i str>, S>,
}

impl<'i, S> Default for Attributes<'i, S>
where
    S: BuildHasher + Default,
{
    fn default() -> Self {
        let kvs: HashMap<&'i str, Option<&'i str>, S> = HashMap::default();
        Attributes { kvs }
    }
}

impl<'i, S> PartialEq for Attributes<'i, S>
where
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.kvs == other.kvs
    }
}

impl<'i, S> Attributes<'i, S>
where
    S: BuildHasher + Default,
{
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let kvs = separated(0.., parse_attribute, multispace1).parse_next(input)?;
        Ok(Self { kvs })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use std::collections::hash_map::RandomState;

    use super::*;

    #[rstest]
    #[case("width", "width")]
    #[case("my-class", "my-class")]
    fn test_key(#[case] input: &str, #[case] expected: &str) {
        let actual = parse_key.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("''", "")]
    #[case("'40'", "40")]
    #[case("'hello world'", "hello world")]
    fn test_parsing_single_quoted_val(#[case] input: &str, #[case] expected: &str) {
        let actual = parse_single_quoted_val.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("\"\"", "")]
    #[case("\"40\"", "40")]
    #[case("\"hello world\"", "hello world")]
    fn test_parsing_double_quoted_val(#[case] input: &str, #[case] expected: &str) {
        let actual = parse_double_quoted_val.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    #[case("width", Attributes {
        kvs: [("width", None)].into_iter().collect(),
    })]
    #[case("-1width", Attributes {
        kvs: [("-1width", None)].into_iter().collect(),
    })]
    #[case("1width", Attributes {
        kvs: [("1width", None)].into_iter().collect(),
    })]
    #[case("1-width", Attributes {
        kvs: [("1-width", None)].into_iter().collect(),
    })]
    #[case("width=\"40\"", Attributes {
        kvs: [("width", Some("40"))].into_iter().collect(),
    })]
    #[case("width   =    \"40\"", Attributes {
        kvs: [("width", Some("40"))].into_iter().collect(),
    })]
    #[case("value=yes", Attributes {
        kvs: [("value", Some("yes"))].into_iter().collect(),
    })]
    #[case("width=\"40\"", Attributes {
        kvs: [("width", Some("40"))].into_iter().collect(),
    })]
    #[case("width=\"40\" height=\"30\"", Attributes {
        kvs: [("width", Some("40")), ("height", Some("30"))]
            .into_iter()
            .collect(),
    })]
    #[case("width=\"40\" height=\"30\" class=\"my-class\"", Attributes {
        kvs: [("width", Some("40")), ("height", Some("30")), ("class", Some("my-class"))]
            .into_iter()
            .collect(),
    })]
    #[case("key  =    value key-here1  =    value123   width=\"40\" length='40' height=\"30\" class=\"my-class\"", Attributes {
        kvs: [
            ("key", Some("value")),
            ("key-here1", Some("value123")),
            ("width", Some("40")),
            ("length", Some("40")),
            ("height", Some("30")),
            ("class", Some("my-class")),
        ]
            .into_iter()
            .collect(),
    })]
    fn test_attributes(#[case] input: &str, #[case] expected: Attributes<RandomState>) {
        let actual = Attributes::<RandomState>::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
