use super::{node::Node, tag::Tag};
use crate::formatting::Formatable;
use std::collections::HashMap;
use winnow::{PResult, Parser};

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
    pub attributes: HashMap<&'i str, Option<&'i str>>,
    pub classes: Vec<&'i str>,
    pub children: Vec<Node<'i>>,
}

impl<'i> Element<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let opening_tag = Tag::parse.parse_next(input)?;
        let mut children = vec![];

        while !input.is_empty() {
            let node = Node::parse.parse_next(input);

            match node {
                Ok(node) => children.push(node),
                _ => break,
            }
        }

        Ok(Self {
            id: None,
            name: opening_tag.name,
            variant: ElementVariant::Normal,
            attributes: opening_tag.attributes.kvs,
            classes: vec![],
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
        for (key, val) in &self.attributes {
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
                html.push_str(" />");
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
        html.push_str(&format!("{}</{}>", indent, self.name));
        html.push('\n');

        html
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: HashMap::new(),
        classes: vec![],
        children: vec![],
    }, "<div></div>\n")]
    #[case(Element {
        id: Some("my-id"),
        name: "div",
        variant: ElementVariant::Normal,
        attributes: HashMap::new(),
        classes: vec![],
        children: vec![],
    }, "<div id=\"my-id\"></div>\n")]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: HashMap::new(),
        classes: vec!["my-class"],
        children: vec![],
    }, "<div class=\"my-class\"></div>\n")]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: HashMap::new(),
        classes: vec!["my-class", "my-other-class"],
        children: vec![],
    }, "<div class=\"my-class my-other-class\"></div>\n")]
    #[case(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: HashMap::new(),
        classes: vec![],
        children: vec![Node::Text("hello there")],
    }, "<div>\n\thello there\n</div>\n")]
    #[case(Element {
        id: Some("my-id"),
        name: "div",
        variant: ElementVariant::Normal,
        attributes: HashMap::new(),
        classes: vec!["my-class"],
        children: vec![Node::Element(Element {
            id: None,
            name: "div",
            variant: ElementVariant::Normal,
            attributes: HashMap::new(),
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
        let mut attributes: HashMap<&str, Option<&str>> = HashMap::new();
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
                    attributes: HashMap::new(),
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
}
