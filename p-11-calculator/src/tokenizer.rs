pub enum Token {
    Whitespace,
    Number(i64),
    Float(f64),
    Plus,
    Minus,
    Times,
    Divide,
    LeftParen,
    RightParen,
    Eof,
}

#[derive(Debug)]
pub struct ParseError(String);

impl TryFrom<&str> for Token {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(val) = value.parse::<i64>() {
            return Ok(Token::Number(val));
        } else if let Ok(val) = value.parse::<f64>() {
            return Ok(Token::Float(val));
        } else if value.chars().all(|c| c.is_whitespace()) {
            return Ok(Token::Whitespace);
        }

        Ok(match value {
            "+" => Token::Plus,
            "-" => Token::Minus,
            "*" => Token::Times,
            "/" => Token::Divide,
            "(" => Token::LeftParen,
            ")" => Token::RightParen,
            _ => return Err(ParseError(value.to_owned())),
        })
    }
}

pub struct Tokenizer {
    tokens: Vec<Token>,
}

fn chunk_chars(a: &char, b: &char) -> bool {
    if a.is_whitespace() && b.is_whitespace() {
        return true;
    }

    // Group the parts that make up an integer or float definition
    match (a, b) {
        ('0'..='9', '0'..='9') => true,
        ('+' | '-', '0'..='9') => true,
        ('0'..='9', '.') => true,
        ('.', '0'..='9') => true,
        (_, _) => false,
    }
}

impl Tokenizer {
    pub fn tokenize(from: String) -> Result<Tokenizer, ParseError> {
        let chars: Vec<char> = from.chars().collect();

        let tokens = chars
            .chunk_by(chunk_chars)
            .map(|seq| {
                let string = seq.iter().collect::<String>();
                Token::try_from(string.as_ref())
            })
            .collect::<Result<Vec<Token>, ParseError>>()?;

        Ok(Tokenizer { tokens })
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop()
    }
}
