use super::element::Element;
use super::{comment::parse_comment, text::parse_text};
use crate::formatting::Formatable;
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

#[cfg(test)]
mod tests {
    use crate::html_parser::{attribute::Attributes, element::ElementVariant};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("<!-- -->", Node::Comment(""))]
    #[case("<!--     -->", Node::Comment(""))]
    #[case("<!---->", Node::Comment(""))]
    #[case("<!-- my comment -->", Node::Comment("my comment"))]
    #[case("<!-- my-comment -->", Node::Comment("my-comment"))]
    #[case("<!--my-comment-->", Node::Comment("my-comment"))]
    #[case("<!--     my-comment       -->", Node::Comment("my-comment"))]
    #[case("hello there", Node::Text("hello there"))]
    #[case("<div></div>", Node::Element(Element {
        id: None,
        name: "div",
        variant: ElementVariant::Normal,
        attributes: Attributes::default(),
        classes: vec![],
        children: vec![],
    }))]
    fn test_node_parses_successfully(#[case] input: &str, #[case] expected: Node) {
        let actual = Node::parse.parse(input).unwrap();
        assert_eq!(actual, expected);
    }
}
