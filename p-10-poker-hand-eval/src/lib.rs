mod poker_hand;
mod poker_value;

pub use poker_hand::{Card, Hand, Rank, Suit};
pub use poker_value::HandValue;

#[cfg(test)]
mod tests {
    use crate::{
        poker_hand::{Card, Hand, ParseError, Rank, Suit},
        poker_value::HandValue,
    };

    fn evaluate(input: &str) -> HandValue {
        let hand = Hand::try_from(input.to_owned())
            .unwrap_or_else(|error| panic!("failed to parse {input:?}: {error:?}"));

        HandValue::from(&hand)
    }

    mod parsing {
        use super::*;

        #[test]
        fn rejects_too_few_cards() {
            let result = Hand::try_from("2C 3D 4H 5S".to_owned());

            assert_eq!(result, Err(ParseError::InvalidCardCount));
        }

        #[test]
        fn rejects_too_many_cards() {
            let input = "2C 3D 4H 5S 6C 7D";
            let result = Hand::try_from(input.to_owned());

            assert_eq!(result, Err(ParseError::InvalidHand(input.to_owned())));
        }

        #[test]
        fn rejects_invalid_rank() {
            let result = Hand::try_from("1C 3D 4H 5S 6C".to_owned());

            assert_eq!(result, Err(ParseError::InvalidRank('1')));
        }

        #[test]
        fn rejects_invalid_suit() {
            let result = Hand::try_from("2X 3D 4H 5S 6C".to_owned());

            assert_eq!(result, Err(ParseError::InvalidSuit('X')));
        }

        #[test]
        fn rejects_malformed_card() {
            let result = Hand::try_from("2CC 3D 4H 5S 6C".to_owned());

            assert_eq!(result, Err(ParseError::InvalidCard("2CC".to_owned())));
        }

        #[test]
        fn rejects_duplicate_cards() {
            let result = Hand::try_from("AC AH AC 5S 6C".to_owned());

            assert_eq!(
                result,
                Err(ParseError::DuplicateCard(Card {
                    rank: Rank::Ace,
                    suit: Suit::Clubs,
                })),
            );
        }

        #[test]
        fn rejects_non_adjacent_duplicate_cards() {
            let result = Hand::try_from("AC 2D KH AC 7S".to_owned());

            assert!(matches!(
                result,
                Err(ParseError::DuplicateCard(Card {
                    rank: Rank::Ace,
                    suit: Suit::Clubs,
                }))
            ));
        }

        #[test]
        fn accepts_arbitrary_whitespace_between_cards() {
            assert_eq!(
                evaluate("  2C\t3D   4H\n5S  6C  "),
                HandValue::Straight(Rank::Six),
            );
        }
    }

    mod classification {
        use super::*;

        #[test]
        fn classifies_every_hand_category() {
            use HandValue::*;
            use Rank::*;

            let cases = [
                ("AS KD 9H 5C 3D", HighCard([Ace, King, Nine, Five, Three])),
                ("KH KD AS 7C 2D", Pair(King, [Ace, Seven, Two])),
                ("AH AD KC KS 2D", TwoPair(Ace, King, [Two])),
                ("QH QD QS AC 2D", ThreeOfAKind(Queen, [Ace, Two])),
                ("2C 3H 4D 5S 6C", Straight(Six)),
                ("AC JC 8C 4C 2C", Flush([Ace, Jack, Eight, Four, Two])),
                ("9C 9D 9H KS KD", FullHouse(Nine, King)),
                ("7C 7D 7H 7S AC", FourOfAKind(Seven, [Ace])),
                ("5C 6C 7C 8C 9C", StraightFlush(Nine)),
                ("0C JC QC KC AC", RoyalFlush),
            ];

            for (input, expected) in cases {
                assert_eq!(evaluate(input), expected, "failed for {input}");
            }
        }

        #[test]
        fn recognizes_ace_low_straight() {
            assert_eq!(evaluate("AC 2D 3H 4S 5C"), HandValue::Straight(Rank::Five),);
        }

        #[test]
        fn recognizes_ace_low_straight_flush() {
            assert_eq!(
                evaluate("AC 2C 3C 4C 5C"),
                HandValue::StraightFlush(Rank::Five),
            );
        }

        #[test]
        fn ace_does_not_wrap_in_other_sequences() {
            assert!(matches!(evaluate("AC 2D 3H 4S 6C"), HandValue::HighCard(_),));
        }
    }

    mod ordering {
        use super::*;

        #[test]
        fn hand_categories_have_correct_order() {
            let values = [
                evaluate("AS KD 9H 5C 3D"),
                evaluate("KH KD AS 7C 2D"),
                evaluate("AH AD KC KS 2D"),
                evaluate("QH QD QS AC 2D"),
                evaluate("2C 3H 4D 5S 6C"),
                evaluate("AC JC 8C 4C 2C"),
                evaluate("9C 9D 9H KS KD"),
                evaluate("7C 7D 7H 7S AC"),
                evaluate("5C 6C 7C 8C 9C"),
                evaluate("0C JC QC KC AC"),
            ];

            for adjacent in values.windows(2) {
                assert!(
                    adjacent[0] < adjacent[1],
                    "{:?} should be less than {:?}",
                    adjacent[0],
                    adjacent[1],
                );
            }
        }

        #[test]
        fn higher_pair_wins() {
            assert!(evaluate("KH KD AS 7C 2D") > evaluate("QH QD AS KC 2D"));
        }

        #[test]
        fn pair_uses_kickers_in_descending_order() {
            assert!(evaluate("KH KD AS 7C 2D") > evaluate("KS KC QH 7D 2C"));
        }

        #[test]
        fn pair_compares_all_kickers() {
            assert!(evaluate("KH KD AS 7C 2D") > evaluate("KS KC AH 6D 5C"));
        }

        #[test]
        fn two_pair_compares_high_pair_first() {
            assert!(evaluate("AH AD 2C 2S 3D") > evaluate("KH KD QC QS AD"));
        }

        #[test]
        fn two_pair_compares_low_pair_then_kicker() {
            assert!(evaluate("AH AD KC KS 2D") > evaluate("AS AC QH QD KD"));
        }

        #[test]
        fn full_house_compares_triple_before_pair() {
            assert!(evaluate("QH QD QS 2C 2D") > evaluate("JH JD JS AC AD"));
        }

        #[test]
        fn four_of_a_kind_compares_quad_rank_first() {
            assert!(evaluate("KH KD KS KC 2D") > evaluate("QH QD QS QC AD"));
        }

        #[test]
        fn high_card_compares_each_rank_in_order() {
            assert!(evaluate("AS KD 9H 5C 3D") > evaluate("AH KC 9D 5S 2C"));
        }
    }

    mod invariants {
        use super::*;

        #[test]
        fn suits_do_not_break_ties() {
            assert_eq!(evaluate("AS KD 9H 5C 3D"), evaluate("AH KC 9D 5S 3C"),);
        }

        #[test]
        fn equal_pairs_with_equal_kickers_tie() {
            assert_eq!(evaluate("AH AD KC 7S 2D"), evaluate("AS AC KH 7D 2C"),);
        }

        #[test]
        fn input_order_does_not_change_value() {
            let expected = evaluate("AH AD KC KS 2D");

            assert_eq!(expected, evaluate("2D KS AH KC AD"));
            assert_eq!(expected, evaluate("KC 2D AD AH KS"));
        }
    }

    mod ranks {
        use super::*;

        #[test]
        fn rank_navigation_handles_boundaries() {
            assert_eq!(Rank::Two.prev(), None);
            assert_eq!(Rank::Two.next(), Some(Rank::Three));
            assert_eq!(Rank::King.next(), Some(Rank::Ace));
            assert_eq!(Rank::Ace.next(), None);
            assert_eq!(Rank::Ace.prev(), Some(Rank::King));
        }

        #[test]
        fn rank_navigation_is_consistent() {
            let ranks = [
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ];

            for adjacent in ranks.windows(2) {
                assert_eq!(adjacent[0].next(), Some(adjacent[1]));
                assert_eq!(adjacent[1].prev(), Some(adjacent[0]));
            }
        }
    }
}
