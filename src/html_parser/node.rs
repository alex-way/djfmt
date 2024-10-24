use super::element::Element;
use super::{comment::parse_comment, text::parse_text};
use crate::formatting::Formatable;
use winnow::combinator::peek;
use winnow::{
    error::{ErrMode, ErrorKind, ParserError},
    PResult, Parser,
};

#[derive(Debug, PartialEq)]
pub enum Node<'i> {
    Text(&'i str),
    Element(Element<'i>),
    Comment(&'i str),
}

impl<'i> Node<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        if let Ok(comment) = parse_comment.parse_next(input) {
            return Ok(Self::Comment(comment));
        }

        if let Ok(element) = Element::parse.parse_next(input) {
            return Ok(Self::Element(element));
        }

        if let Ok(text) = parse_text.parse_next(input) {
            if text.is_empty() {
                return Err(ErrMode::from_error_kind(input, ErrorKind::Verify));
            }
            return Ok(Self::Text(text));
        }

        Err(ErrMode::from_error_kind(input, ErrorKind::Verify))
    }
}

impl<'i> Formatable for Node<'i> {
    fn formatted(&self, indent_level: usize) -> String {
        let indent = "\t".repeat(indent_level);
        match self {
            Node::Text(text) => format!("{indent}{text}\n"),
            Node::Element(element) => element.formatted(indent_level),
            Node::Comment(comment) => format!("{indent}<!-- {comment} -->\n"),
        }
    }
}

pub fn parse_child_nodes<'i>(input: &mut &'i str) -> PResult<Vec<Node<'i>>> {
    let mut nodes = vec![];

    while !input.is_empty() {
        let initial_len = input.len();
        let peek_result = peek(Node::parse).parse_peek(input);

        if peek_result.is_err() {
            return Ok(nodes);
        }
        let node = Node::parse.parse_next(input);
        match node {
            Ok(node) => nodes.push(node),
            Err(_) => return Ok(nodes),
        }

        // Check if the parser consumed any input
        if input.len() == initial_len {
            return Ok(nodes);
        }
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use crate::html_parser::{attribute::Attributes, element::ElementVariant};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("<!-- -->", Node::Comment(""), "")]
    #[case("<!--     -->", Node::Comment(""), "")]
    #[case("<!---->", Node::Comment(""), "")]
    #[case("<!-- my comment -->", Node::Comment("my comment"), "")]
    #[case("<!-- my-comment -->", Node::Comment("my-comment"), "")]
    #[case("<!--my-comment-->", Node::Comment("my-comment"), "")]
    #[case("<!--     my-comment       -->", Node::Comment("my-comment"), "")]
    #[case("hello there", Node::Text("hello there"), "")]
    #[case("<img />", Node::Element(Element {
        id: None,
        name: "img",
        variant: ElementVariant::Void,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    }), "")]
    #[case("<div></div>", Node::Element(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    }), "")]
    #[case("<div><img /></div>", Node::Element(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![
            Node::Element(Element {
                id: None,
                name: "img",
                variant: ElementVariant::Void,
                attributes: Attributes::default(),
                classes: vec![],
                children: vec![],
            }),
        ],
    }), "")]
    fn test_node_parses_successfully(
        #[case] input: &str,
        #[case] expected: Node,
        #[case] remaining: &str,
    ) {
        let mut input = input;

        let actual = Node::parse.parse_next(&mut input).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(input, remaining);
    }

    #[rstest]
    #[case("<!---->test<!---->", vec![Node::Comment(""), Node::Text("test"),Node::Comment("")], "")]
    #[case("<div><!---->test<!----></div>", vec![
        Node::Element(Element {
            id: None,
            name: "div",
            variant: ElementVariant::Normal,
            attributes: Attributes::default(),
            classes: vec![],
            children: vec![Node::Comment(""), Node::Text("test"),Node::Comment("")],
        }),
    ], "")]
    #[case("<div/>", vec![Node::Element(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Void,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    })], "")]
    #[case("</div>", vec![], "</div>")]
    #[case("<div></div>", vec![Node::Element(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    })], "")]
    #[case("<div></div><div></div>", vec![
        Node::Element(Element {
            id: None,
            name: "div",
            variant: ElementVariant::Normal,
            attributes: Attributes::default(),
            classes: vec![],
            children: vec![],
        }),
        Node::Element(Element {
            id: None,
            name: "div",
            variant: ElementVariant::Normal,
            attributes: Attributes::default(),
            classes: vec![],
            children: vec![],
        }),
    ], "")]
    fn test_parse_child_nodes(
        #[case] input: &str,
        #[case] expected: Vec<Node>,
        #[case] remaining: &str,
    ) {
        let mut input = input;
        let actual = parse_child_nodes.parse_next(&mut input).unwrap();
        assert_eq!(input, remaining);
        assert_eq!(actual, expected);
    }
}
