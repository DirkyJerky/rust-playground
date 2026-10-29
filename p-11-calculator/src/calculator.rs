#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalError {}

pub struct Calculator {}

impl Calculator {
    pub fn eval(expr: String) -> Result<i64, EvalError> {
        return Ok(42);
        todo!()
    }
}

struct Tokenizer {
    source: String
}

imply

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator() {
        assert_eq!(Calculator::eval("1+1".to_owned()), Ok(2));
    }
}
