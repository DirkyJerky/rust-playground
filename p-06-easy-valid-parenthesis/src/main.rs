use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{all_consuming, complete},
    multi::many0,
};

/// Types of valid parentheses
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Parenthesis {
    /// '(' or ')'
    Round,
    /// '[' or ']'
    Box,
    /// '{' or '}'
    Brace,
}

/// Test whether a string encodes a valid set of properly closed (and possibly nested) parentheses.
/// This function works by building up a stack, adding entries with the opening of every new parenthesis,
/// and the popping and verifying entries from the stack upon encountering closing parenthesis.
fn is_correct_parentheses(str: &str) -> bool {
    let mut state: Vec<Parenthesis> = Vec::new();

    for char in str.chars() {
        let paren_type: Parenthesis = match char {
            '(' | ')' => Parenthesis::Round,
            '[' | ']' => Parenthesis::Box,
            '{' | '}' => Parenthesis::Brace,
            _ => panic!("unexpected char: {char}"),
        };

        match char {
            '(' | '[' | '{' => state.push(paren_type),
            ')' | ']' | '}' => {
                if !state.pop().map(|v| v == paren_type).unwrap_or(false) {
                    return false;
                }
            }
            _ => panic!("unexpected char: {char}"),
        }
    }

    state.len() == 0
}

#[derive(Debug)]
struct ParsedParentheses {
    paren_type: Parenthesis,
    children: Vec<ParsedParentheses>,
}

fn single_open_parenthesis_parser(input: &str) -> IResult<&str, Parenthesis> {
    one_of("([{")
        .map(|res| match res {
            '(' => Parenthesis::Round,
            '[' => Parenthesis::Box,
            '{' => Parenthesis::Brace,
            _ => panic!("unexpected char: {res}"),
        })
        .parse(input)
}

fn single_closed_parenthesis_parser(paren: Parenthesis) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| {
        match paren {
            Parenthesis::Round => tag(")"),
            Parenthesis::Box => tag("]"),
            Parenthesis::Brace => tag("}"),
        }
        .parse(input)
    }
}

/// Self-referential `nom` parser for multiple possibly-nested parenthesis
fn parentheses_parser(input: &str) -> IResult<&str, Vec<ParsedParentheses>> {
    complete(many0(single_open_parenthesis_parser.flat_map(
        |opening_paren| {
            parentheses_parser
                .and(single_closed_parenthesis_parser(opening_paren))
                .map(move |(children, _closing_paren)| ParsedParentheses {
                    paren_type: opening_paren,
                    children,
                })
        },
    )))
    .parse(input)
}

/// Alternate solution, works by using a self-referential `nom` parser to verify correctness
fn is_correct_parentheses_v2(str: &str) -> bool {
    let res = all_consuming(parentheses_parser).parse(str);
    dbg!(&res);
    res.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cases() {
        let cases: [(&str, bool); _] = [
            ("()", true),
            ("()[]{}", true),
            ("(]", false),
            ("([])", true),
            ("([)]", false),
            ("()([](){{}{[]([[]{()}{}()])}})", true),
        ];

        for (input, output) in cases {
            assert_eq!(is_correct_parentheses(input), output, "{input} -> {output}");
            assert_eq!(
                is_correct_parentheses_v2(input),
                output,
                "{input} -> {output}"
            )
        }
    }
}
