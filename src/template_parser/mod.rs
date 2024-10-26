use crate::formatting::Formatable;
use comment::Comment;
use tag::Tag;
use variable::VariableTag;
use winnow::{PResult, Parser};

mod argument;
mod comment;
mod filter;
mod tag;
mod text;
mod utils;
mod variable;

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Variable(VariableTag<'a>),
    Tag(Tag<'a>),
    Comment(&'a str),
    Text(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct Template<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'i> Template<'i> {
    pub fn parse(input: &mut &'i str) -> PResult<Self> {
        let mut nodes = vec![];
        while !input.is_empty() {
            // Peek at the input to ensure the parser can successfully parse the next node
            if Comment::parse.parse_peek(input).is_ok() {
                if let Ok(variable) = Comment::parse.parse_next(input) {
                    nodes.push(Node::Comment(variable.0));
                    continue;
                }
            }

            if variable::VariableTag::parse.parse_peek(input).is_ok() {
                if let Ok(variable) = variable::VariableTag::parse.parse_next(input) {
                    nodes.push(Node::Variable(variable));
                    continue;
                }
            }

            if tag::Tag::parse.parse_peek(input).is_ok() {
                if let Ok(tag) = tag::Tag::parse.parse_next(input) {
                    nodes.push(Node::Tag(tag));
                    continue;
                }
            }

            if text::parse_text.parse_peek(input).is_ok() {
                if let Ok(text) = text::parse_text.parse_next(input) {
                    if !text.is_empty() {
                        nodes.push(Node::Text(text));
                        continue;
                    }
                }
            }

            break;
        }

        Ok(Self { nodes })
    }
}

impl<'i> Formatable for Template<'i> {
    fn formatted(&self, indent_level: usize) -> String {
        let mut result = String::new();
        for node in &self.nodes {
            match node {
                Node::Variable(var) => result.push_str(&var.formatted(indent_level)),
                Node::Tag(tag) => result.push_str(&tag.formatted(indent_level)),
                Node::Comment(comment) => result.push_str(&comment.formatted(indent_level)),
                Node::Text(text) => result.push_str(text),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::empty_template("", Template { nodes: vec![] })]
    #[case("text", Template { nodes: vec![
        Node::Text("text"),
    ] })]
    #[case("{{text}}", Template { nodes: vec![
        Node::Variable(VariableTag { tag_type: "text", filters: vec![] }),
    ] })]
    #[case("hello{{text}}world", Template { nodes: vec![
        Node::Text("hello"),
        Node::Variable(VariableTag { tag_type: "text", filters: vec![] }),
        Node::Text("world"),
    ] })]
    #[case("hello{{text}}there{% thing %}world", Template { nodes: vec![
        Node::Text("hello"),
        Node::Variable(VariableTag { tag_type: "text", filters: vec![] }),
        Node::Text("there"),
        Node::Tag(Tag { tag_type: "thing", arguments: vec![] }),
        Node::Text("world"),
    ] })]
    #[case("hello{{text}}there{# comment #}again{% thing %}world", Template { nodes: vec![
        Node::Text("hello"),
        Node::Variable(VariableTag { tag_type: "text", filters: vec![] }),
        Node::Text("there"),
        Node::Comment("comment"),
        Node::Text("again"),
        Node::Tag(Tag { tag_type: "thing", arguments: vec![] }),
        Node::Text("world"),
    ] })]
    fn test_parsing_template(#[case] input: &str, #[case] expected: Template) {
        let actual = Template::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }

    #[rstest]
    fn test_formatting_multiple_times_doesnt_change_output() {
        let expected = "<div></div>";
        let parsed = Template::parse.parse(expected).unwrap();

        let first_format = parsed.formatted(0);
        assert_eq!(expected, first_format);

        let second_parse = Template::parse.parse(first_format.as_str()).unwrap();

        let second_format = second_parse.formatted(0);
        assert_eq!(expected, second_format);
    }
}
