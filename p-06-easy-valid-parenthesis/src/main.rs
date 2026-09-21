fn is_correct_parenthesis(str: &str) -> bool {
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
            assert_eq!(is_correct_parenthesis(input), output, "{input} -> {output}")
        }
    }
}
