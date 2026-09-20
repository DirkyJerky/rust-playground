use std::{ops::Add, str::FromStr};

/// Represents a single literal value in a regex pattern.
/// Either a single regular character, or a wildcard ('.')
#[derive(Clone, Debug, PartialEq, Eq)]
enum RegexLiteral {
    /// A specific character value
    Char(char),
    /// Any character value ('.')
    Wild,
}

impl RegexLiteral {
    /// True if the regex literal pattern matches the provided character
    fn matches(&self, chr: &char) -> bool {
        match self {
            RegexLiteral::Wild => true,
            RegexLiteral::Char(self_chr) => self_chr == chr,
        }
    }
}

/// Represents a range of how many instances a literal can be expected to appear in a pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RegexQualifier {
    /// The lower bound.  Required, min 0.
    lower: usize,
    /// The upper bound.  Optional, None represents infinity
    upper: Option<usize>,
}

impl RegexQualifier {
    /// Counts a literal once and exactly once
    const SINGLE: RegexQualifier = RegexQualifier {
        lower: 1,
        upper: Some(1),
    };
    /// Counts a literal exactly zero or one times.
    /// Used by the question mark ('?') operator.
    const OPTIONAL: RegexQualifier = RegexQualifier {
        lower: 0,
        upper: Some(1),
    };
    /// Counts a literal one or more times.
    /// Used by the plus ('+') operator.
    const ONE_OR_MORE: RegexQualifier = RegexQualifier {
        lower: 1,
        upper: None,
    };
    /// Counts a literal zero or more times.
    /// Used by the times ('*') operator.
    const ZERO_OR_MORE: RegexQualifier = RegexQualifier {
        lower: 0,
        upper: None,
    };

    /// Returns a copy of this qualifier that has been "reduced" by one.
    /// This is used to represent the consumption of a single character by this qualifier during processing.
    /// Returns `None` if this qualifier cannot "consume" any more characters, i.e. its `upper` limit is 0.
    fn deduct(&self) -> Option<Self> {
        if self.upper == Some(0) {
            None
        } else {
            Some(RegexQualifier {
                lower: self.lower.saturating_sub(1),
                upper: self.upper.map(|v| v - 1),
            })
        }
    }

    /// Reports whether this qualifier is "satified".
    /// This is true if this qualifier would pass regex processing if it didn't "consume" any more characters.
    fn is_satisfied(&self) -> bool {
        return self.lower == 0;
    }
}

impl Add for RegexQualifier {
    type Output = RegexQualifier;

    /// Add the two bounds seperately
    fn add(self, rhs: Self) -> Self::Output {
        RegexQualifier {
            lower: self.lower + rhs.lower,
            upper: match (self.upper, rhs.upper) {
                (None, None) => None,
                (Some(_), None) | (None, Some(_)) => None,
                (Some(v1), Some(v2)) => Some(v1 + v2),
            },
        }
    }
}

/// Represents a single literal paired with a numerical qualifier
#[derive(Clone, Debug, PartialEq, Eq)]
struct RegexToken {
    value: RegexLiteral,
    qualifier: RegexQualifier,
}

/// Represents every possible state during the char-by-char parsing of a regex pattern
enum RegexParsingStateMachine {
    /// Expecting a literal or start of escape sequence
    ExpectingLiteral,
    /// Expecting a literal, start of escape sequence, or a qualifier to attach to the previously parsed literal
    /// Holds on to the reference of the previous literal
    ExpectingLiteralOrQualifier { literal: RegexLiteral },
    /// Expecting any character to follow
    EscapeNext,
    /// Expecting an optional numerical value or comma during the first step of processing a '{n1,n2}' qualifier
    /// Holds on to the reference to the literal this qualifier attaches to, and the set of characters of the first number parsed so far
    BracketQualifierExpectingFirstDigitsOrComma {
        literal: RegexLiteral,
        first_num: String,
    },
    /// Expecting an optional numerical value or closing bracket during the second step of processing a '{n1,n2}' qualifier
    /// Holds on to the reference to the literal this qualifier attaches to, the optional first number, and the set of characters of the second number parsed so far
    BracketQualifierExpectingSecondDigitsOrClose {
        literal: RegexLiteral,
        first_num: Option<usize>,
        second_num: String,
    },
}

impl RegexParsingStateMachine {
    /// Where the state machine begins
    const START: RegexParsingStateMachine = RegexParsingStateMachine::ExpectingLiteral;
}

/// Possible syntax errors that can occur during Regex pattern parsing.
/// Each possible value includes the index in the pattern string where the error occured.
#[derive(Debug, PartialEq, Eq)]
pub enum RegexParseError {
    /// Found a special character (one of '?+*{,}') in a context it is not allowed.
    InvalidUseOfSpecialCharacter { pos: usize },
    /// Found an invalid number during the parsing of a '{n1,n2}' qualifier
    /// Typically occurs only if the value overflows `usize::MAX`
    InvalidNumberInQualifier { pos: usize, str: String },
    /// Found a regular character during the parsing of a '{n1,n2}' qualifier
    InvalidUseOfRegularCharacter { pos: usize },
    /// Found an incomplete escape sequence at the end of the pattern
    ExpectingEscapedCharacter { pos: usize },
}

impl RegexParsingStateMachine {
    /// Processes a single character, checking syntax and advancing the current state machine.
    ///
    /// Returns `Ok` if the syntax is correct, containing an optionally emitted token, and the next state of the state machine.
    /// Returns `Err` if a syntax error occurs.
    fn consume(
        self,
        next: char,
        pos: usize,
    ) -> Result<(Option<RegexToken>, RegexParsingStateMachine), RegexParseError> {
        match self {
            RegexParsingStateMachine::ExpectingLiteral => match next {
                '?' | '+' | '*' | '{' | ',' | '}' => {
                    Err(RegexParseError::InvalidUseOfSpecialCharacter { pos })
                }
                '\\' => Ok((None, RegexParsingStateMachine::EscapeNext)),
                '.' => Ok((
                    None,
                    RegexParsingStateMachine::ExpectingLiteralOrQualifier {
                        literal: RegexLiteral::Wild,
                    },
                )),
                ch => Ok((
                    None,
                    RegexParsingStateMachine::ExpectingLiteralOrQualifier {
                        literal: RegexLiteral::Char(ch),
                    },
                )),
            },
            RegexParsingStateMachine::ExpectingLiteralOrQualifier { literal } => match next {
                ',' | '}' => Err(RegexParseError::InvalidUseOfSpecialCharacter { pos }),
                '?' => Ok((
                    Some(RegexToken {
                        value: literal,
                        qualifier: RegexQualifier::OPTIONAL,
                    }),
                    RegexParsingStateMachine::ExpectingLiteral,
                )),
                '+' => Ok((
                    Some(RegexToken {
                        value: literal,
                        qualifier: RegexQualifier::ONE_OR_MORE,
                    }),
                    RegexParsingStateMachine::ExpectingLiteral,
                )),
                '*' => Ok((
                    Some(RegexToken {
                        value: literal,
                        qualifier: RegexQualifier::ZERO_OR_MORE,
                    }),
                    RegexParsingStateMachine::ExpectingLiteral,
                )),
                '{' => Ok((
                    None,
                    RegexParsingStateMachine::BracketQualifierExpectingFirstDigitsOrComma {
                        literal,
                        first_num: String::new(),
                    },
                )),
                '\\' => Ok((
                    Some(RegexToken {
                        value: literal,
                        qualifier: RegexQualifier::SINGLE,
                    }),
                    RegexParsingStateMachine::EscapeNext,
                )),
                '.' => Ok((
                    Some(RegexToken {
                        value: literal,
                        qualifier: RegexQualifier::SINGLE,
                    }),
                    RegexParsingStateMachine::ExpectingLiteralOrQualifier {
                        literal: RegexLiteral::Wild,
                    },
                )),
                ch => Ok((
                    Some(RegexToken {
                        value: literal,
                        qualifier: RegexQualifier::SINGLE,
                    }),
                    RegexParsingStateMachine::ExpectingLiteralOrQualifier {
                        literal: RegexLiteral::Char(ch),
                    },
                )),
            },
            RegexParsingStateMachine::EscapeNext => Ok((
                None,
                RegexParsingStateMachine::ExpectingLiteralOrQualifier {
                    literal: RegexLiteral::Char(next),
                },
            )),
            RegexParsingStateMachine::BracketQualifierExpectingFirstDigitsOrComma {
                literal,
                mut first_num,
            } => match next {
                '0'..'9' => {
                    first_num.push(next);
                    Ok((
                        None,
                        RegexParsingStateMachine::BracketQualifierExpectingFirstDigitsOrComma {
                            literal,
                            first_num,
                        },
                    ))
                }
                ',' => {
                    if first_num.len() == 0 {
                        Ok((
                            None,
                            RegexParsingStateMachine::BracketQualifierExpectingSecondDigitsOrClose {
                                literal,
                                first_num: None,
                                second_num: String::new(),
                            },
                        ))
                    } else {
                        if let Ok(num) = first_num.parse::<usize>() {
                            Ok((
                                None,
                                RegexParsingStateMachine::BracketQualifierExpectingSecondDigitsOrClose {
                                    literal,
                                    first_num: Some(num),
                                    second_num: String::new(),
                                },
                            ))
                        } else {
                            Err(RegexParseError::InvalidNumberInQualifier {
                                pos,
                                str: first_num,
                            })
                        }
                    }
                }
                '?' | '+' | '*' | '{' | '}' => {
                    Err(RegexParseError::InvalidUseOfSpecialCharacter { pos })
                }
                _ => Err(RegexParseError::InvalidUseOfRegularCharacter { pos }),
            },
            RegexParsingStateMachine::BracketQualifierExpectingSecondDigitsOrClose {
                literal,
                first_num,
                mut second_num,
            } => match next {
                '0'..'9' => {
                    second_num.push(next);
                    Ok((
                        None,
                        RegexParsingStateMachine::BracketQualifierExpectingSecondDigitsOrClose {
                            literal,
                            first_num,
                            second_num,
                        },
                    ))
                }
                '}' => {
                    if second_num.len() == 0 {
                        Ok((
                            Some(RegexToken {
                                value: literal,
                                qualifier: RegexQualifier {
                                    lower: first_num.unwrap_or_default(),
                                    upper: None,
                                },
                            }),
                            RegexParsingStateMachine::ExpectingLiteral,
                        ))
                    } else {
                        if let Ok(second_num) = second_num.parse::<usize>() {
                            Ok((
                                Some(RegexToken {
                                    value: literal,
                                    qualifier: RegexQualifier {
                                        lower: first_num.unwrap_or_default(),
                                        upper: Some(second_num),
                                    },
                                }),
                                RegexParsingStateMachine::ExpectingLiteral,
                            ))
                        } else {
                            Err(RegexParseError::InvalidNumberInQualifier {
                                pos,
                                str: second_num,
                            })
                        }
                    }
                }
                '?' | '+' | '*' | '{' | ',' => {
                    Err(RegexParseError::InvalidUseOfSpecialCharacter { pos })
                }
                _ => Err(RegexParseError::InvalidUseOfRegularCharacter { pos }),
            },
        }
    }

    /// Finishes the current state of processing, emitting any final tokens and checking for complete sytax.
    ///
    /// Returns `Ok` if the syntax is correct, and optionally returns a final token.
    /// Returns `Err` if a syntax error occurs.
    fn finish(self, pos: usize) -> Result<Option<RegexToken>, RegexParseError> {
        match self {
            RegexParsingStateMachine::ExpectingLiteral => Ok(None),
            RegexParsingStateMachine::ExpectingLiteralOrQualifier { literal } => {
                Ok(Some(RegexToken {
                    value: literal,
                    qualifier: RegexQualifier::SINGLE,
                }))
            }
            RegexParsingStateMachine::EscapeNext => {
                Err(RegexParseError::ExpectingEscapedCharacter { pos })
            }
            RegexParsingStateMachine::BracketQualifierExpectingFirstDigitsOrComma {
                literal: _,
                first_num,
            } => Err(RegexParseError::InvalidNumberInQualifier {
                pos,
                str: first_num,
            }),
            RegexParsingStateMachine::BracketQualifierExpectingSecondDigitsOrClose {
                literal: _,
                first_num: _,
                second_num,
            } => Err(RegexParseError::InvalidNumberInQualifier {
                pos,
                str: second_num,
            }),
        }
    }
}

/// A parsed Regex pattern.
#[derive(Clone, Debug)]
pub struct SimpleRegex {
    tokens: Vec<RegexToken>,
}

impl FromStr for SimpleRegex {
    type Err = RegexParseError;

    /// Parses a SimpleRegex from the given string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SimpleRegex::new(s)
    }
}

impl SimpleRegex {
    /// Constructs a `SimpleRegex` from the given string.
    ///
    /// Returns `Ok` if the regex is syntactically correct.
    /// Returns `Err` if not.
    pub fn new(str: &str) -> Result<Self, RegexParseError> {
        let mut tokens: Vec<RegexToken> = Vec::new();

        let mut state: RegexParsingStateMachine = RegexParsingStateMachine::START;

        for (idx, next) in str.chars().enumerate() {
            match state.consume(next, idx)? {
                (opt_token, new_state) => {
                    if let Some(val) = opt_token {
                        tokens.push(val);
                    }
                    state = new_state;
                }
            }

            continue;
        }

        match state.finish(str.len())? {
            Some(token) => tokens.push(token),
            None => {}
        }

        return Ok(SimpleRegex { tokens: tokens }.optimize());
    }

    /// Performs some internal optimization steps, by merging any neighboring tokens that match on the same literals
    fn optimize(self) -> Self {
        let mut tokens: Vec<RegexToken> = Vec::new();

        // TODO: Use an itertools adapter here

        let mut held_token: Option<RegexToken> = None;

        for token in self.tokens {
            match held_token {
                None => held_token = Some(token),
                Some(found_held_token) => {
                    if found_held_token.value == token.value {
                        held_token = Some(RegexToken {
                            value: token.value,
                            qualifier: token.qualifier + found_held_token.qualifier,
                        })
                    } else {
                        tokens.push(found_held_token);
                        held_token = Some(token);
                    }
                }
            }
        }

        if let Some(found_token) = held_token {
            tokens.push(found_token);
        }

        SimpleRegex { tokens }
    }

    /// Check if the provided string matches this regex pattern.
    pub fn check(&self, str: &str) -> bool {
        let chars_vec = str.chars().collect::<Vec<char>>();
        let chars: &[char] = chars_vec.as_slice();

        self.recurse_check(chars, 0, self.tokens.get(0))
    }

    /// Internal function to perform a greedy depth-first matching between the regex tokens and the provided string.
    fn recurse_check(
        &self,
        chars: &[char],
        reference_token_no: usize,
        token: Option<&RegexToken>,
    ) -> bool {
        let first_char = chars.first();

        // dbg!("recurse_check");
        // dbg!(chars);
        // dbg!(reference_token_no);
        // dbg!(token);

        match (first_char, token) {
            (None, None) => true,     // No more tokens and characters means we are done
            (Some(_), None) => false, // Characters left without tokens to match to them means this processing state is not done
            (None, Some(token)) => {
                token.qualifier.is_satisfied()
                    && self.recurse_check(
                        chars,
                        reference_token_no + 1,
                        self.tokens.get(reference_token_no + 1),
                    )
            } // No characters means we are good if this and all the following tokens are satisfied
            (Some(char), Some(token)) => {
                // Check if the current regex state token can handle the next character
                // If it can, then recurse with the current regex state having "consumed" the character
                if token.value.matches(&char) {
                    match token.qualifier.deduct() {
                        None => {}
                        Some(reduced_qualifier) => {
                            let new_token: RegexToken = RegexToken {
                                value: token.value.clone(),
                                qualifier: reduced_qualifier,
                            };
                            return self.recurse_check(
                                &chars[1..],
                                reference_token_no,
                                Some(&new_token),
                            );
                        }
                    }
                }

                // Otherwise, if the current token cannot match but otherwise is satisfied (such as with a non-matching ?), then attempt to continue with the next token
                if token.qualifier.is_satisfied() {
                    let next_token = self.tokens.get(reference_token_no + 1);
                    return self.recurse_check(chars, reference_token_no + 1, next_token);
                }

                // Otherwise, this current regex state is not satisfied
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_repr_1() {
        let r: SimpleRegex = ".a?b+c*d{10,}e{10,20}f{,20}g{,}"
            .parse()
            .expect("Should parse");
        let expected: SimpleRegex = SimpleRegex {
            tokens: vec![
                RegexToken {
                    value: RegexLiteral::Wild,
                    qualifier: RegexQualifier::SINGLE,
                },
                RegexToken {
                    value: RegexLiteral::Char('a'),
                    qualifier: RegexQualifier::OPTIONAL,
                },
                RegexToken {
                    value: RegexLiteral::Char('b'),
                    qualifier: RegexQualifier::ONE_OR_MORE,
                },
                RegexToken {
                    value: RegexLiteral::Char('c'),
                    qualifier: RegexQualifier::ZERO_OR_MORE,
                },
                RegexToken {
                    value: RegexLiteral::Char('d'),
                    qualifier: RegexQualifier {
                        lower: 10,
                        upper: None,
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('e'),
                    qualifier: RegexQualifier {
                        lower: 10,
                        upper: Some(20),
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('f'),
                    qualifier: RegexQualifier {
                        lower: 0,
                        upper: Some(20),
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('g'),
                    qualifier: RegexQualifier {
                        lower: 0,
                        upper: None,
                    },
                },
            ],
        };

        assert!(
            r.tokens
                .iter()
                .zip(expected.tokens.iter())
                .all(|(v1, v2)| v1 == v2)
        );
    }

    #[test]
    pub fn test_repr_2() {
        let r: SimpleRegex = "a+a*b?bccd{,}d+e?e?e?f{1,2}f{3,4}"
            .parse()
            .expect("Should parse");
        let expected: SimpleRegex = SimpleRegex {
            tokens: vec![
                RegexToken {
                    value: RegexLiteral::Char('a'),
                    qualifier: RegexQualifier::ONE_OR_MORE,
                },
                RegexToken {
                    value: RegexLiteral::Char('b'),
                    qualifier: RegexQualifier {
                        lower: 1,
                        upper: Some(2),
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('c'),
                    qualifier: RegexQualifier {
                        lower: 2,
                        upper: Some(2),
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('d'),
                    qualifier: RegexQualifier {
                        lower: 1,
                        upper: None,
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('e'),
                    qualifier: RegexQualifier {
                        lower: 0,
                        upper: Some(3),
                    },
                },
                RegexToken {
                    value: RegexLiteral::Char('f'),
                    qualifier: RegexQualifier {
                        lower: 4,
                        upper: Some(6),
                    },
                },
            ],
        };

        assert!(
            r.tokens
                .iter()
                .zip(expected.tokens.iter())
                .all(|(v1, v2)| v1 == v2)
        );
    }

    #[test]
    pub fn test_readme() {
        let regex: SimpleRegex = SimpleRegex::new("Hello .+!").expect("Regex should parse");
        assert!(regex.check("Hello world!"));
        assert!(regex.check("Hello Geoff!"));
        assert!(!regex.check("Hello ?"));
    }

    #[test]
    pub fn test_success_1() {
        let r: SimpleRegex = "a".parse().expect("Should parse");
        assert_eq!(false, r.check("aa"));
    }

    #[test]
    pub fn test_success_2() {
        let r: SimpleRegex = "a*".parse().expect("Should parse");
        assert_eq!(true, r.check("aa"));
    }

    #[test]
    pub fn test_success_3() {
        let r: SimpleRegex = ".*".parse().expect("Should parse");
        assert_eq!(true, r.check("ab"));
    }

    #[test]
    pub fn test_success_4() {
        let r: SimpleRegex = "a*b+c{1,2}.{3,7}d?d?d?d?d?".parse().expect("Should parse");
        let goods: Vec<&str> = vec![
            "aaaaaaaabbbbbcc1234567ddddd",
            "bc123",
            "bccc123",
            "bccccccccc",
            "bcc23456ddddddd",
        ];
        let bads: Vec<&str> = vec![
            "c123",
            "b123",
            "bc12",
            "bc12345678",
            "bc1234567dddddd",
            "bcc23456dddddddd",
            "bc123456ddddddd",
            "bccc23456ddddddd",
            "bcc123456ddddddd",
            "bcccccccccc",
        ];

        for good in goods {
            assert_eq!(true, r.check(good), "testing good: {good}");
        }

        for bad in bads {
            assert_eq!(false, r.check(bad), "testing bad: {bad}");
        }
    }

    #[test]
    pub fn test_success_5() {
        let r: SimpleRegex = "\\a\\.\\??\\++\\**\\{{0,1}\\,{,1}\\}{1,}"
            .parse()
            .expect("Should parse");
        assert_eq!(true, r.check("a.+}"), "testing: specials #1");
        assert_eq!(true, r.check("a.?++++****{,}}}}}"), "testing: specials #2");
    }

    #[test]
    pub fn test_failure_1() {
        "*".parse::<SimpleRegex>().expect_err("Should not parse");
    }

    #[test]
    pub fn test_failure_2() {
        "?".parse::<SimpleRegex>().expect_err("Should not parse");
    }

    #[test]
    pub fn test_failure_3() {
        "?".parse::<SimpleRegex>().expect_err("Should not parse");
    }

    #[test]
    pub fn test_failure_4() {
        "abcd{"
            .parse::<SimpleRegex>()
            .expect_err("Should not parse");
    }

    #[test]
    pub fn test_failure_5() {
        r"\".parse::<SimpleRegex>().expect_err("Should not parse");
    }

    #[test]
    pub fn test_failure_6() {
        r"a{,".parse::<SimpleRegex>().expect_err("Should not parse");
    }
}
