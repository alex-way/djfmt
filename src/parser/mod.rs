use winnow::ascii::{alpha1, multispace0};
use winnow::combinator::{delimited, peek, preceded};
use winnow::error::ParserError;
use winnow::token::{take_until0, take_while};
use winnow::{PResult, Parser};

/// Reference material: https://docs.djangoproject.com/en/5.0/ref/templates/language/

/// A combinator which takes an `inner` parser and produces a parser which also
/// consumes both leading and trailing whitespaces, returning the output of
/// `inner`.
pub(crate) fn trim<'a, F, O, E>(inner: F) -> impl Parser<&'a str, O, E>
where
    E: ParserError<&'a str>,
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn parse_django_variable<'s>(i: &mut &'s str) -> PResult<&'s str> {
    let tag_chars = preceded(
        peek(alpha1),
        take_while(1.., |c: char| c.is_alphanumeric() || c == '_' || c == '.'),
    );
    let tag = delimited("{{", trim(tag_chars), "}}").parse_next(i)?;
    Ok(tag)
}

pub fn parse_comment_contents<'s>(i: &mut &'s str) -> PResult<&'s str> {
    let mut comment = take_until0("#}").parse_next(i)?;
    // todo: use a parser combinator to do this, rather than .trim()
    comment = comment.trim();
    Ok(comment)
}

/// Parse a Django single line comment. For example: `{# my comment #}`
/// Must be on a single line.
pub fn parse_django_single_line_comment<'s>(i: &mut &'s str) -> PResult<&'s str> {
    let comment = delimited("{#", parse_comment_contents, "#}").parse_next(i)?;
    Ok(comment)
}

// /// Parse a Django filter. For example: `{% myvar|filter %}`
// pub fn parse_django_tag<'s>(i: &mut &'s str) -> PResult<&'s str> {
//     let tag_chars = preceded(
//         peek(alpha1),
//         take_while(1.., |c: char| c.is_alphanumeric() || c == '_' || c == '.'),
//     );
//     let tag =
//         delimited("{%", delimited(multispace0, tag_chars, multispace0), "%}").parse_next(i)?;
//     Ok(tag)
// }

// /// Parse a Django filter. For example: `{{ myvar|filter }}`
// pub fn parse_django_filter<'s>(i: &mut &'s str) -> PResult<&'s str> {
//     let tag_chars = preceded(
//         peek(alpha1),
//         take_while(1.., |c: char| c.is_alphanumeric() || c == '_' || c == '.'),
//     );
//     let tag =
//         delimited("{%", delimited(multispace0, tag_chars, multispace0), "%}").parse_next(i)?;
//     Ok(tag)
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        input,
        expected,
        case(r#"{{myvar}}"#, "myvar"),
        case(r#"{{ myvar }}"#, "myvar"),
        case(r#"{{ my_var }}"#, "my_var"),
        case(r#"{{ my_var.sub }}"#, "my_var.sub"),
        case(r#"{{ my_var.objects.all }}"#, "my_var.objects.all")
    )]
    fn test_valid_parse_django_variable(input: &str, expected: &str) {
        let output = parse_django_variable.parse(input).unwrap();
        assert_eq!(output, expected);
    }

    #[rstest(
        input,
        case(r#"{{_myvar}}"#),
        case(r#"{{1myvar}}"#),
        case(r#"{{my-var}}"#)
    )]
    fn test_invalid_parse_django_variable(input: &str) {
        let result = parse_django_variable.parse(input);
        assert!(result.is_err());
    }

    #[rstest(
        input,
        expected,
        case(r#"{#myvar#}"#, "myvar"),
        case(r#"{# myvar #}"#, "myvar"),
        case(r#"{# Hello #Gamers #}"#, "Hello #Gamers"),
        case(r#"{#    Hello #Gamers     #}"#, "Hello #Gamers"),
        case(r#"{# {% if foo %}bar{% else %} #}"#, r#"{% if foo %}bar{% else %}"#)
    )]
    fn test_valid_parse_django_single_line_comment(input: &str, expected: &str) {
        let output = parse_django_single_line_comment.parse(input).unwrap();
        assert_eq!(output, expected);
    }

    #[rstest(input, case(r#"{#myva#}r#}"#))]
    fn test_invalid_parse_django_single_line_comment(input: &str) {
        let result = parse_django_single_line_comment.parse(input);
        assert!(result.is_err());
    }
}
