use crate::formatting::Formatable;
use tag::{ClosingTag, Tag};
use winnow::{combinator::repeat, PResult, Parser};

mod attribute;
mod comment;
mod element;
pub mod node;
mod tag;
mod text;

#[derive(Debug)]
pub struct Element<'i> {
    opening_tag: Tag<'i>,
    closing_tag: ClosingTag<'i>,
    children: Vec<Element<'i>>,
}

impl<'i> PartialEq for Element<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.opening_tag == other.opening_tag
            && self.closing_tag == other.closing_tag
            && self.children == other.children
    }
}

impl<'i> Element<'i> {
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
}

impl<'i> Formatable for Element<'i> {
    fn formatted(&self, indent_level: usize) -> String {
        let mut html = String::new();

        // Create the indent string for the current level
        let mut indent = "\t".repeat(indent_level);

        // Add the opening tag with the current indentation
        html.push_str(&format!("{}{}", indent, self.opening_tag.formatted(0)));

        if !self.children.is_empty() {
            html.push('\n');
        }

        // Add each child, increasing the indentation for each child
        for child in &self.children {
            html.push_str(&child.formatted(indent_level + 1)); // Recursively increase the indentation
        }

        if self.children.is_empty() {
            indent = "".to_string();
        }

        // Add the closing tag with the current indentation
        html.push_str(&format!("{}{}", indent, self.closing_tag.formatted(0)));
        html.push('\n');

        html
    }
}

#[cfg(test)]
mod tests {
    use attribute::Attributes;
    use element::ElementVariant;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_simple_element() {
        let input = r#"<div></div>"#;
        let expected = Element {
            opening_tag: Tag {
                name: "div",
                variant: ElementVariant::Normal,
                attributes: Attributes::default(),
            },
            closing_tag: ClosingTag { name: "div" },
            children: vec![],
        };
        let actual = Element::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn test_nested_element() {
        let input = r#"<div><div height="30"></div></div>"#;
        let expected = Element {
            opening_tag: Tag {
                name: "div",
                variant: ElementVariant::Normal,
                attributes: Attributes::default(),
            },
            closing_tag: ClosingTag { name: "div" },
            children: vec![Element {
                opening_tag: Tag {
                    name: "div",
                    variant: ElementVariant::Normal,
                    attributes: Attributes {
                        kvs: [("height", Some("30"))].into_iter().collect(),
                    },
                },
                closing_tag: ClosingTag { name: "div" },
                children: vec![],
            }],
        };
        let actual = Element::parse.parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[rstest]
    fn test_nested_element_format() {
        let input = Element {
            opening_tag: Tag {
                name: "div",
                variant: ElementVariant::Normal,
                attributes: Attributes::default(),
            },
            closing_tag: ClosingTag { name: "div" },
            children: vec![
                Element {
                    opening_tag: Tag {
                        name: "div",
                        variant: ElementVariant::Normal,
                        attributes: Attributes {
                            kvs: [("height", Some("30"))].into_iter().collect(),
                        },
                    },
                    closing_tag: ClosingTag { name: "div" },
                    children: vec![],
                },
                Element {
                    opening_tag: Tag {
                        name: "div",
                        variant: ElementVariant::Normal,
                        attributes: Attributes {
                            kvs: [("height", Some("30"))].into_iter().collect(),
                        },
                    },
                    closing_tag: ClosingTag { name: "div" },
                    children: vec![],
                },
            ],
        };

        let expected = "<div>\n\t<div height=\"30\"></div>\n\t<div height=\"30\"></div>\n</div>\n";

        let actual = input.formatted(0);
        assert_eq!(expected, actual);
    }
}
