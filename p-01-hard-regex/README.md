# Regex Engine

A simple Regex engine, which supports the following features:

- "." wildcard literals
- "?", "+", and "\*" qualifiers
- '{123,456}' range qualifiers
- `\` escape sequences
- Full string matching only (i.e. assumed '^' and '$')

## Usage

```rust
use crate::SimpleRegex;

fn main() {
    let regex: SimpleRegex = SimpleRegex::new("Hello .+!").expect("Regex should parse");
    assert!(regex.check("Hello world!"));
    assert!(regex.check("Hello Geoff!"));
    assert!(!regex.check("Hello ?"));
}
```

## Future feature additions

- Capture groups
- Substring matching, w/ support for '^' and '$'
- `match()` iterators

## Future architecture upgrades

- Use a featured syntax engine (like `nom`) to perform string parsing
