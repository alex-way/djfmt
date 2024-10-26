use super::{
    attribute::Attributes,
    node::{parse_child_nodes, Node},
    tag::Tag,
};
use crate::{formatting::Formatable, html_parser::tag::ClosingTag};
use winnow::{
    error::{ErrMode, ErrorKind, ParserError},
    PResult, Parser,
};

#[allow(dead_code)]
const VOID_ELEMENT_NAMES: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Debug, PartialEq)]
pub enum ElementVariant {
    Normal,
    Void,
}

#[derive(Debug, PartialEq)]
pub struct Element<'i> {
    pub id: Option<&'i str>,
    pub name: &'i str,
    pub variant: ElementVariant,
    pub attributes: Attributes<'i>,
    pub classes: Vec<&'i str>,
    pub children: Vec<Node<'i>>,
}

impl<'i> Element<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let mut opening_tag = Tag::parse.parse_next(input)?;

        let id = opening_tag.attributes.pop("id");
        let classes_attr = opening_tag.attributes.pop("class");

        let mut classes = vec![];
        if let Some(classes_attr) = classes_attr {
            classes.extend(classes_attr.split(' '));
        }

        if opening_tag.variant == ElementVariant::Void {
            return Ok(Self {
                id,
                name: opening_tag.name,
                variant: ElementVariant::Void,
                attributes: opening_tag.attributes,
                classes,
                children: vec![],
            });
        }

        let closing_tag_peek = ClosingTag::parse.parse_peek(input);

        if closing_tag_peek.is_ok() {
            let closing_tag = ClosingTag::parse.parse_next(input)?;

            if opening_tag.name != closing_tag.name {
                return Err(ErrMode::from_error_kind(input, ErrorKind::Verify));
            } else if opening_tag.name == closing_tag.name {
                return Ok(Self {
                    id,
                    name: opening_tag.name,
                    variant: ElementVariant::Normal,
                    attributes: opening_tag.attributes,
                    classes,
                    children: vec![],
                });
            }
        }

        let children = parse_child_nodes.parse_next(input)?;

        let closing_tag = ClosingTag::parse.parse_next(input)?;

        if opening_tag.name != closing_tag.name {
            return Err(ErrMode::from_error_kind(input, ErrorKind::Verify));
        }

        Ok(Self {
            id,
            name: opening_tag.name,
            variant: ElementVariant::Normal,
            attributes: opening_tag.attributes,
            classes,
            children,
        })
    }
}

impl<'i> Formatable for Element<'i> {
    fn formatted(&self, indent_level: usize) -> String {
        let mut html = String::new();

        // Create the indent string for the current level
        let mut indent = "\t".repeat(indent_level);

        // Add the opening tag with the current indentation
        html.push_str(&format!("{}<{}", indent, self.name));

        // Add the id attribute if it exists
        if let Some(id) = self.id {
            html.push_str(&format!(" id=\"{}\"", id));
        }

        // Add the classes if they exist
        if !self.classes.is_empty() {
            html.push_str(" class=\"");
            html.push_str(&self.classes.join(" "));
            html.push('"');
        }

        // Add the attributes if they exist
        for (key, val) in self.attributes.iter() {
            html.push(' ');
            html.push_str(key);
            if let Some(val) = val {
                html.push_str("=\"");
                html.push_str(val);
                html.push('"');
            }
        }

        // Add the closing tag
        match self.variant {
            ElementVariant::Normal => {
                html.push('>');
            }
            ElementVariant::Void => {
                html.push_str(" />\n");
                return html;
            }
        }

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
        html.push_str(&format!("{}</{}>\n", indent, self.name));

        html
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("<div></div>", Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    })]
    #[case("<div id=\"my-id\"></div>", Element {
        id: Some("my-id"),
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    })]
    fn test_element_parses_successfully(#[case] input: &str, #[case] expected: Element) {
        let actual = Element::parse.parse(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    }, "<div></div>\n")]
    #[case(Element {
        id: Some("my-id"),
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    }, "<div id=\"my-id\"></div>\n")]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec!["my-class"],
        children: vec![],
    }, "<div class=\"my-class\"></div>\n")]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec!["my-class", "my-other-class"],
        children: vec![],
    }, "<div class=\"my-class my-other-class\"></div>\n")]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![Node::Text("hello there")],
    }, "<div>\n\thello there\n</div>\n")]
    #[case(Element {
        id: Some("my-id"),
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec!["my-class"],
        children: vec![Node::Element(Element {
            id: None,
            name: "div",
            variant: ElementVariant::Normal,
            attributes: Attributes::default(),
            classes: vec![],
            children: vec![],
        })],
    }, "<div id=\"my-id\" class=\"my-class\">\n\t<div></div>\n</div>\n")]
    fn test_element_format(#[case] input: Element, #[case] expected: &str) {
        let actual = input.formatted(0);
        assert_eq!(actual, expected);
    }

    #[rstest]
    fn test_element_format_kitchen_sink() {
        let mut attributes = Attributes::default();
        attributes.insert("width", Some("40"));

        let element = Element {
            id: Some("my-id"),
            name: "div",
            variant: ElementVariant::Normal,
            attributes,
            classes: vec!["my-class"],
            children: vec![
                Node::Element(Element {
                    id: None,
                    name: "div",
                    variant: ElementVariant::Normal,
                    attributes: Attributes::default(),
                    classes: vec![],
                    children: vec![],
                }),
                Node::Comment("my comment"),
            ],
        };
        let expected =
            "<div id=\"my-id\" class=\"my-class\" width=\"40\">\n\t<div></div>\n\t<!-- my comment -->\n</div>\n";
        let actual = element.formatted(0);
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case("<div></div>", Ok(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    }), "")]
    #[case("</div>", Err(ErrMode::from_error_kind(&"", ErrorKind::Verify)), "</div>")]
    fn test_element_doesnt_consume_input_after_closing_tag(
        #[case] input: &str,
        #[case] expected_extracted: PResult<Element>,
        #[case] expected_remaining: &str,
    ) {
        let mut input = input;

        let actual = Element::parse.parse_next(&mut input);

        assert_eq!(actual, expected_extracted);
        assert_eq!(input, expected_remaining);
    }
}
