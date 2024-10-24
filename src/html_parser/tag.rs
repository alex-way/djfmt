use crate::formatting::Formatable;
use crate::html_parser::attribute::Attributes;
use std::hash::BuildHasher;
use winnow::{
    ascii::{alpha1, multispace0},
    combinator::{delimited, opt},
    PResult, Parser,
};

use super::element::ElementVariant;

pub fn parse_tag_name<'i>(input: &mut &'i str) -> PResult<&'i str> {
    alpha1.parse_next(input)
}

/// An HTML open tag, like `<a href="google.com">`.
#[derive(Debug)]
pub struct Tag<'i, S> {
    /// Like 'div'
    pub name: &'i str,
    pub attributes: Attributes<'i, S>,
    pub variant: ElementVariant,
}

impl<'i, S> PartialEq for Tag<'i, S>
where
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.attributes == other.attributes
            && self.variant == other.variant
    }
}

impl<'i, S> Tag<'i, S>
where
    S: BuildHasher + Default,
{
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let (name, attributes, variant) = delimited(
            ('<', multispace0),
            (
                parse_tag_name,
                delimited(multispace0, Attributes::parse, multispace0),
                opt('/'),
            ),
            (multispace0, '>'),
        )
        .parse_next(input)?;

        Ok(Self {
            name,
            attributes,
            variant: match variant {
                Some(_) => ElementVariant::Void,
                None => ElementVariant::Normal,
            },
        })
    }
}

impl<'i, S> Formatable for Tag<'i, S> {
    fn formatted(&self, _indent_level: usize) -> String {
        let mut html = String::new();
        html.push('<');
        html.push_str(self.name);
        for (key, val) in &self.attributes.kvs {
            html.push(' ');
            html.push_str(key);
            if let Some(val) = val {
                html.push_str("=\"");
                html.push_str(val);
                html.push('"');
            }
        }
        html.push('>');
        html
    }
}

/// An HTML closing tag, like `</a>`.
#[derive(Debug, PartialEq)]
pub struct ClosingTag<'i> {
    /// Like 'div'
    pub name: &'i str,
}

impl<'i> ClosingTag<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let tag =
            delimited(("</", multispace0), parse_tag_name, (multispace0, ">")).parse_next(input)?;

        Ok(Self { name: tag })
    }
}

impl<'i> Formatable for ClosingTag<'i> {
    fn formatted(&self, _indent_level: usize) -> String {
        format!("</{}>", self.name)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use std::collections::hash_map::RandomState;

    use super::*;

    #[rstest]
    fn test_link_tag() {
        let input = r#"<a href="https://google.com">"#;
        let expected = Tag {
            name: "a",
            variant: ElementVariant::Normal,
            attributes: Attributes {
                kvs: [("href", Some("https://google.com"))].into_iter().collect(),
            },
        };
        let actual = Tag::<RandomState>::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn test_tag() {
        let input = r#"<div width="40" height="30">"#;
        let expected = Tag::<RandomState> {
            name: "div",
            variant: ElementVariant::Normal,
            attributes: Attributes {
                kvs: [("width", Some("40")), ("height", Some("30"))]
                    .into_iter()
                    .collect(),
            },
        };
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn test_self_closing_tag() {
        let input = r#"<div width="40" height="30"/>"#;
        let expected = Tag::<RandomState> {
            name: "div",
            variant: ElementVariant::Void,
            attributes: Attributes {
                kvs: [("width", Some("40")), ("height", Some("30"))]
                    .into_iter()
                    .collect(),
            },
        };
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn test_self_closing_tag_with_spaces() {
        let input = r#"<div width="40" height="30" />"#;
        let expected = Tag::<RandomState> {
            name: "div",
            variant: ElementVariant::Void,
            attributes: Attributes {
                kvs: [("width", Some("40")), ("height", Some("30"))]
                    .into_iter()
                    .collect(),
            },
        };
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("</div>", ClosingTag { name: "div" })]
    #[case("</ div>", ClosingTag { name: "div" })]
    #[case("</div >", ClosingTag { name: "div" })]
    #[case("</ div >", ClosingTag { name: "div" })]
    fn test_closing_tag(#[case] input: &str, #[case] expected: ClosingTag) {
        let actual = ClosingTag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
