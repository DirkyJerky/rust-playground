/// An error encountered while parsing a hand
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Unknown rank
    InvalidRank(char),
    /// Unknown suit
    InvalidSuit(char),
    /// Unknown card format (should be a rank + a suit)
    InvalidCard(String),
    /// Invalid hand sequence (should be exactly 5 cards seperated by whitespace)
    InvalidHand(String),
    /// Invalid number of cards,
    InvalidCardCount,
    /// A duplicate card was found, you cheat!
    DuplicateCard(Card),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl TryFrom<char> for Rank {
    type Error = ParseError;

    fn try_from(from: char) -> Result<Self, Self::Error> {
        let rank = match from {
            '2' => Rank::Two,
            '3' => Rank::Three,
            '4' => Rank::Four,
            '5' => Rank::Five,
            '6' => Rank::Six,
            '7' => Rank::Seven,
            '8' => Rank::Eight,
            '9' => Rank::Nine,
            '0' => Rank::Ten,
            'J' => Rank::Jack,
            'Q' => Rank::Queen,
            'K' => Rank::King,
            'A' => Rank::Ace,
            _ => return Err(ParseError::InvalidRank(from)),
        };

        Ok(rank)
    }
}

impl Rank {
    pub fn next(&self) -> Option<Self> {
        use Rank::*;

        Some(match self {
            Two => Three,
            Three => Four,
            Four => Five,
            Five => Six,
            Six => Seven,
            Seven => Eight,
            Eight => Nine,
            Nine => Ten,
            Ten => Jack,
            Jack => Queen,
            Queen => King,
            King => Ace,
            Ace => return None,
        })
    }

    pub fn prev(&self) -> Option<Self> {
        use Rank::*;

        Some(match self {
            Two => return None,
            Three => Two,
            Four => Three,
            Five => Four,
            Six => Five,
            Seven => Six,
            Eight => Seven,
            Nine => Eight,
            Ten => Nine,
            Jack => Ten,
            Queen => Jack,
            King => Queen,
            Ace => King,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Spades,
    Hearts,
    Diamonds,
}

impl TryFrom<char> for Suit {
    type Error = ParseError;

    fn try_from(from: char) -> Result<Self, Self::Error> {
        let rank = match from {
            'C' => Suit::Clubs,
            'S' => Suit::Spades,
            'H' => Suit::Hearts,
            'D' => Suit::Diamonds,
            _ => return Err(ParseError::InvalidSuit(from)),
        };

        Ok(rank)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl TryFrom<&str> for Card {
    type Error = ParseError;

    fn try_from(from: &str) -> Result<Self, Self::Error> {
        let mut chars = from.chars();

        if let (Some(c1), Some(c2), None) = (chars.next(), chars.next(), chars.next()) {
            Ok(Card {
                rank: Rank::try_from(c1)?,
                suit: Suit::try_from(c2)?,
            })
        } else {
            Err(ParseError::InvalidCard(String::from(from)))
        }
    }
}

/// A poker hand of 5 *unique* cards.
///
/// This hand will always be sorted with the highest value cards first.
///
/// Cards are ranked by the Rank, suit is irrelevant for ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hand {
    hand: [Card; 5],
}

impl TryFrom<String> for Hand {
    type Error = ParseError;

    fn try_from(from: String) -> Result<Self, Self::Error> {
        let mut tokens = from.split_whitespace();

        let mut hand: [Card; 5] = tokens
            .by_ref()
            .take(5)
            .map(Card::try_from)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| ParseError::InvalidCardCount)?;

        // Sort in reverse
        hand.sort_by(|a, b| b.rank.cmp(&a.rank));

        // Check for duplicaates
        for index in 0..hand.len() {
            if hand[index + 1..].contains(&hand[index]) {
                return Err(ParseError::DuplicateCard(hand[index]));
            }
        }

        // If there is more tokens left after taking 5, then the hand is in a bad state.
        if tokens.next().is_some() {
            return Err(ParseError::InvalidHand(from));
        }

        Ok(Hand { hand })
    }
}

impl Hand {
    pub fn cards(&self) -> [Card; 5] {
        self.hand
    }
}
