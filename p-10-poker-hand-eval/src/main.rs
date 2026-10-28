use std::io::{self, Write};

use p_10_poker_hand_eval::Hand;
use p_10_poker_hand_eval::HandValue;

fn main() {
    let stdin = io::stdin();
    let mut hands: Vec<(String, HandValue)> = Vec::new();

    loop {
        print!("Enter a poker hand: ");
        if let Err(error) = io::stdout().flush() {
            eprintln!("Failed to write prompt: {error}");
            break;
        }

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Failed to read hand: {error}");
                continue;
            }
        }

        let input = input.trim().to_owned();
        if input.len() == 0 {
            break;
        }

        let hand = match Hand::try_from(input.clone()) {
            Ok(hand) => hand,
            Err(error) => {
                eprintln!("Invalid hand: {error:?}");
                continue;
            }
        };

        let value = HandValue::from(&hand);
        println!("Value: {value:?}");
        hands.push((input, value));
        hands.sort_by(|(_, left), (_, right)| right.cmp(left));

        println!("Hands seen:");
        for (index, (hand, value)) in hands.iter().enumerate() {
            println!("{}. {hand}: {value:?}", index + 1);
        }
    }
}
