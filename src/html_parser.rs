use std::{collections::HashMap, hash::BuildHasher};

use winnow::{
    ascii::{alpha1, multispace0, multispace1},
    combinator::{delimited, opt, repeat, separated, separated_pair},
    token::{literal, take_while},
    PResult, Parser,
};

/// Parse the key of a HTML attribute
fn parse_key<'i>(input: &mut &'i str) -> PResult<&'i str> {
    alpha1.parse_next(input)
}

/// Parse the value of an HTML attribute
fn parse_val<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let inner = take_while(1.., |c: char| {
        c.is_alphanumeric()
            || c == '.'
            || c == '/'
            || c == ':'
            || c == '-'
            || c == '_'
            || c == '{'
            || c == '}'
            || c == '%'
            || c == ' '
    });
    delimited('"', inner, '"').parse_next(input)
}

/// Parses an HTML attribute.
/// Looks something like `key="val"`.
fn parse_attribute<'i>(input: &mut &'i str) -> PResult<(&'i str, &'i str)> {
    separated_pair(
        parse_key,
        delimited(multispace0, '=', multispace0),
        parse_val,
    )
    .parse_next(input)
}

/// HTML attributes
#[derive(Debug)]
pub struct Attributes<'i, S> {
    kvs: HashMap<&'i str, &'i str, S>,
}

impl<'i, S> Default for Attributes<'i, S>
where
    S: BuildHasher + Default,
{
    fn default() -> Self {
        let kvs: HashMap<&'i str, &'i str, S> = HashMap::default();
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
    fn parse(input: &mut &'i str) -> PResult<Self> {
        let kvs = separated(0.., parse_attribute, multispace1).parse_next(input)?;
        Ok(Self { kvs })
    }
}

/// An HTML open tag, like `<a href="google.com">`.
#[derive(Debug)]
pub struct Tag<'i, S> {
    /// Like 'div'
    tag_type: &'i str,
    attributes: Attributes<'i, S>,
}

impl<'i, S> PartialEq for Tag<'i, S>
where
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type && self.attributes == other.attributes
    }
}

impl<'i, S> Tag<'i, S>
where
    S: BuildHasher + Default,
{
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let parse_parts = (alpha1, multispace0, Attributes::parse);
        let parse_tag = parse_parts.map(|(tag_type, _space_char, attributes)| Self {
            tag_type,
            attributes,
        });
        let closing_tag = (multispace0, opt(literal("/")), '>', multispace0);
        let tag = delimited('<', parse_tag, closing_tag).parse_next(input)?;
        Ok(tag)
    }
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push('<');
        html.push_str(self.tag_type);
        for (key, val) in &self.attributes.kvs {
            html.push(' ');
            html.push_str(key);
            html.push_str("=\"");
            html.push_str(val);
            html.push('"');
        }
        html.push('>');
        html
    }
}

/// An HTML closing tag, like `</a>`.
#[derive(Debug)]
pub struct ClosingTag<'i> {
    /// Like 'div'
    tag_type: &'i str,
}

impl<'i> PartialEq for ClosingTag<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.tag_type == other.tag_type
    }
}

impl<'i> ClosingTag<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let parse_parts = (alpha1, multispace0);
        let parse_tag = parse_parts.map(|(tag_type, _space_char)| Self { tag_type });
        let tag = delimited("</", parse_tag, (multispace0, ">", multispace0)).parse_next(input)?;
        Ok(tag)
    }

    pub fn to_html(&self) -> String {
        format!("</{}>", self.tag_type)
    }
}

/// An HTML closing tag, like `</a>`.
#[derive(Debug)]
pub struct Element<'i, S> {
    opening_tag: Tag<'i, S>,
    closing_tag: ClosingTag<'i>,
    children: Vec<Element<'i, S>>,
}

impl<'i, S> PartialEq for Element<'i, S>
where
    S: BuildHasher + Default,
{
    fn eq(&self, other: &Self) -> bool {
        self.opening_tag == other.opening_tag
            && self.closing_tag == other.closing_tag
            && self.children == other.children
    }
}

impl<'i, S> Element<'i, S>
where
    S: BuildHasher + Default,
{
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let element =
            (Tag::parse, repeat(0.., Element::parse), ClosingTag::parse).parse_next(input)?;
        let element = Self {
            opening_tag: element.0,
            closing_tag: element.2,
            children: element.1,
        };

        Ok(element)
    }

    pub fn to_html(&self, indent_level: usize) -> String {
        let mut html = String::new();

        // Create the indent string for the current level
        let mut indent = "\t".repeat(indent_level);

        // Add the opening tag with the current indentation
        html.push_str(&format!("{}{}", indent, self.opening_tag.to_html()));

        if !self.children.is_empty() {
            html.push('\n');
        }

        // Add each child, increasing the indentation for each child
        for child in &self.children {
            html.push_str(&child.to_html(indent_level + 1)); // Recursively increase the indentation
        }

        if self.children.is_empty() {
            indent = "".to_string();
        }

        // Add the closing tag with the current indentation
        html.push_str(&format!("{}{}", indent, self.closing_tag.to_html()));
        html.push('\n');

        html
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use std::collections::hash_map::RandomState;

    use super::*;

    #[test]
    fn test_key() {
        let input = "width";
        let actual = parse_key.parse(input).unwrap();
        let expected = "width";
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_val() {
        let input = r#""40""#;
        let actual = parse_val.parse(input).unwrap();
        let expected = "40";
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_attributes() {
        let input = r#"width="40" height = "30""#;
        let actual = Attributes::<RandomState>::parse.parse(input).unwrap();
        let expected = Attributes {
            kvs: [("width", "40"), ("height", "30")].into_iter().collect(),
        };
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_link_tag() {
        let input = r#"<a href="https://google.com">"#;
        let expected = Tag {
            tag_type: "a",
            attributes: Attributes {
                kvs: [("href", "https://google.com")].into_iter().collect(),
            },
        };
        let actual = Tag::<RandomState>::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_tag() {
        let input = r#"<div width="40" height="30">"#;
        let expected = Tag::<RandomState> {
            tag_type: "div",
            attributes: Attributes {
                kvs: [("width", "40"), ("height", "30")].into_iter().collect(),
            },
        };
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_self_closing_tag() {
        let input = r#"<div width="40" height="30"/>"#;
        let expected = Tag::<RandomState> {
            tag_type: "div",
            attributes: Attributes {
                kvs: [("width", "40"), ("height", "30")].into_iter().collect(),
            },
        };
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_self_closing_tag_with_spaces() {
        let input = r#"<div width="40" height="30" />"#;
        let expected = Tag::<RandomState> {
            tag_type: "div",
            attributes: Attributes {
                kvs: [("width", "40"), ("height", "30")].into_iter().collect(),
            },
        };
        let actual = Tag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_closing_tag() {
        let input = r#"</div>"#;
        let expected = ClosingTag { tag_type: "div" };
        let actual = ClosingTag::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_simple_element() {
        let input = r#"<div></div>"#;
        let expected = Element::<RandomState> {
            opening_tag: Tag {
                tag_type: "div",
                attributes: Attributes::default(),
            },
            closing_tag: ClosingTag { tag_type: "div" },
            children: vec![],
        };
        let actual = Element::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nested_element() {
        let input = r#"<div><div height="30"></div></div>"#;
        let expected = Element::<RandomState> {
            opening_tag: Tag {
                tag_type: "div",
                attributes: Attributes::default(),
            },
            closing_tag: ClosingTag { tag_type: "div" },
            children: vec![Element::<RandomState> {
                opening_tag: Tag {
                    tag_type: "div",
                    attributes: Attributes {
                        kvs: [("height", "30")].into_iter().collect(),
                    },
                },
                closing_tag: ClosingTag { tag_type: "div" },
                children: vec![],
            }],
        };
        let actual = Element::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_nested_element_format() {
        let input = Element::<RandomState> {
            opening_tag: Tag {
                tag_type: "div",
                attributes: Attributes::default(),
            },
            closing_tag: ClosingTag { tag_type: "div" },
            children: vec![
                Element::<RandomState> {
                    opening_tag: Tag {
                        tag_type: "div",
                        attributes: Attributes {
                            kvs: [("height", "30")].into_iter().collect(),
                        },
                    },
                    closing_tag: ClosingTag { tag_type: "div" },
                    children: vec![],
                },
                Element::<RandomState> {
                    opening_tag: Tag {
                        tag_type: "div",
                        attributes: Attributes {
                            kvs: [("height", "30")].into_iter().collect(),
                        },
                    },
                    closing_tag: ClosingTag { tag_type: "div" },
                    children: vec![],
                },
            ],
        };

        let expected = "<div>\n\t<div height=\"30\"></div>\n\t<div height=\"30\"></div>\n</div>\n";

        let actual = input.to_html(0);
        assert_eq!(expected, actual);
    }
}
