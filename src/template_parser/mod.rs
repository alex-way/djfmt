use comment::Comment;
use tag::Tag;
use variable::VariableTag;
use winnow::{PResult, Parser};

mod argument;
mod comment;
mod filter;
mod tag;
mod text;
mod variable;

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Variable(VariableTag<'a>),
    Tag(Tag<'a>),
    Comment(Comment<'a>),
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
            if let Ok(variable) = comment::Comment::parse.parse_next(input) {
                nodes.push(Node::Comment(variable));
                continue;
            }
            if let Ok(variable) = variable::VariableTag::parse.parse_next(input) {
                nodes.push(Node::Variable(variable));
                continue;
            }

            if let Ok(tag) = tag::Tag::parse.parse_next(input) {
                nodes.push(Node::Tag(tag));
                continue;
            }

            if let Ok(text) = text::parse_text.parse_next(input) {
                if !text.is_empty() {
                    nodes.push(Node::Text(text));
                    continue;
                }
            }

            break;
        }

        Ok(Self { nodes })
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
        Node::Comment(Comment { contents: "comment" }),
        Node::Text("again"),
        Node::Tag(Tag { tag_type: "thing", arguments: vec![] }),
        Node::Text("world"),
    ] })]
    fn test_parsing_template(#[case] input: &str, #[case] expected: Template) {
        let actual = Template::parse.parse(input).unwrap();
        assert_eq!(actual, expected)
    }
}
