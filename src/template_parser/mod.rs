use comment::Comment;
use tag::Tag;
use variable::VariableTag;
use winnow::{PResult, Parser};

mod comment;
mod filter;
mod tag;
mod text;
mod variable;

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Text(&'a str),
    Variable(VariableTag<'a>),
    Tag(Tag<'a>),
    Comment(Comment<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Template<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'i> Template<'i> {
    pub fn parse(_input: &mut &'i str) -> PResult<Self> {
        let text = text::parse_text.parse_next(_input)?;
        let nodes = vec![Node::Text(text)];
        Ok(Self { nodes })
    }
}
