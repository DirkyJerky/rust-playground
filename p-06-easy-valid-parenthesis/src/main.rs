/// Types of valid parentheses
#[derive(PartialEq, Eq, Debug)]
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

/// Alternate solution, works by using a self-referential `nom` parser to verify correctness
fn is_correct_parentheses_v2(str: &str) -> bool {
    unimplemented!()
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
        ];

        for (input, output) in cases {
            assert_eq!(is_correct_parentheses(input), output, "{input} -> {output}")
        }
    }
}
