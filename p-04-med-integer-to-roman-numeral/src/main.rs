use std::iter::FusedIterator;

struct Numeral {
    char: char,
    value: u32,
}

impl Numeral {
    const I: Numeral = Numeral {
        char: 'I',
        value: 1,
    };
    const V: Numeral = Numeral {
        char: 'V',
        value: 5,
    };
    const X: Numeral = Numeral {
        char: 'X',
        value: 10,
    };
    const L: Numeral = Numeral {
        char: 'L',
        value: 50,
    };
    const C: Numeral = Numeral {
        char: 'C',
        value: 100,
    };
    const D: Numeral = Numeral {
        char: 'D',
        value: 500,
    };
    const M: Numeral = Numeral {
        char: 'M',
        value: 1000,
    };
}

enum Direction {
    Plus,
    Minus,
}

impl Direction {
    fn apply(self, from: u32, change: u32) -> u32 {
        match self {
            Direction::Minus => from - change,
            Direction::Plus => from + change,
        }
    }
}

fn emit_roman_numeral(from: u32) -> Option<(char, u32)> {
    let change = match from {
        // Zero
        0 => None,
        // Single digits
        1..4 => Some((Numeral::I, Direction::Minus)),
        4 => Some((Numeral::I, Direction::Plus)),
        5..9 => Some((Numeral::V, Direction::Minus)),
        9 => Some((Numeral::I, Direction::Plus)),

        // Double digits
        10..40 => Some((Numeral::X, Direction::Minus)),
        40..50 => Some((Numeral::X, Direction::Plus)),
        50..90 => Some((Numeral::L, Direction::Minus)),
        90..100 => Some((Numeral::X, Direction::Plus)),

        // Triple digits
        100..400 => Some((Numeral::C, Direction::Minus)),
        400..500 => Some((Numeral::C, Direction::Plus)),
        500..900 => Some((Numeral::D, Direction::Minus)),
        900..1000 => Some((Numeral::C, Direction::Plus)),

        // Quad digits
        1000..4000 => Some((Numeral::M, Direction::Minus)),

        // Excessive size
        4000.. => None,
    };

    if let Some((numeral, direction)) = change {
        Some((numeral.char, direction.apply(from, numeral.value)))
    } else {
        None
    }
}

struct RomanNumeralIterator {
    val: u32,
}

impl RomanNumeralIterator {
    fn new(val: u32) -> Self {
        RomanNumeralIterator { val }
    }
}

impl FusedIterator for RomanNumeralIterator {}

impl Iterator for RomanNumeralIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        emit_roman_numeral(self.val).map(|(char, next)| {
            self.val = next;
            char
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cases() {
        let cases: [(u32, &str); _] = [
            (0, ""),
            (1, "I"),
            (2, "II"),
            (3, "III"),
            (4, "IV"),
            (5, "V"),
            (6, "VI"),
            (7, "VII"),
            (8, "VIII"),
            (9, "IX"),
            (10, "X"),
            (11, "XI"),
            (14, "XIV"),
            (19, "XIX"),
            (20, "XX"),
            (21, "XXI"),
            (40, "XL"),
            (44, "XLIV"),
            (50, "L"),
            (59, "LIX"),
            (90, "XC"),
            (99, "XCIX"),
            (100, "C"),
            (101, "CI"),
            (154, "CLIV"),
            (300, "CCC"),
            (400, "CD"),
            (500, "D"),
            (600, "DC"),
            (900, "CM"),
            (1000, "M"),
            (2000, "MM"),
            (3494, "MMMCDXCIV"),
            (3999, "MMMCMXCIX"),
            (4000, ""),
            (1000000, ""),
        ];

        for (input, output) in cases {
            assert_eq!(
                RomanNumeralIterator::new(input)
                    .collect::<String>()
                    .as_str(),
                output,
                "{input} -> {output}"
            );
        }
    }
}
