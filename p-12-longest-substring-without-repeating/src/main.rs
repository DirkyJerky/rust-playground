use std::io::{self, Write};

use p_12_longest_substring_without_repeating::LongestSubstringWithoutRepeatingCharacters;

fn main() {
    print!("Please enter a string: ");
    io::stdout().flush().expect("Unable to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Unable to read from stdin");

    input = input.trim().to_owned();
    let solver = LongestSubstringWithoutRepeatingCharacters::new(input);
    let result = solver.solve();

    println!("-> {result}");
}
