use p_11_calculator::Calculator;
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    loop {
        print!(": ");
        if let Err(err) = io::stdout().flush() {
            eprintln!("Failed to flush prompt to stdin {err}");
            break;
        }

        let mut input = String::new();
        if let Err(err) = stdin.read_line(&mut input) {
            eprintln!("Failed to read expression: {err}");
        }

        let input = input.trim().to_owned();

        if input.len() == 0 {
            break;
        }

        let result = match Calculator::eval(input) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("Invalid expression: {err:?}");
                continue;
            }
        };

        println!("-> {result}");
    }
}
