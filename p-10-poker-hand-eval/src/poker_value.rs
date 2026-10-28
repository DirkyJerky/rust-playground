use crate::poker_hand::{Card, Hand, Rank};

/// A poker hand's value
///
/// All variants will be strictly ordered.
///
/// The ranks within each variant will be ordered, with the most significant rank first.
/// The most significant rank corresponds to the
/// The most significant value to the hand will come first in the tuple
/// This will ensure that the derived `Ord` implementation will correctly compare hand values.
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandValue {
    HighCard([Rank; 5]),
    Pair(Rank, [Rank; 3]),
    TwoPair(Rank, Rank, [Rank; 1]),
    ThreeOfAKind(Rank, [Rank; 2]),
    Straight(Rank),
    Flush([Rank; 5]),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank, [Rank; 1]),
    StraightFlush(Rank),
    RoyalFlush,
}

enum RankGroup {
    Single(Card),
    Double(Card),
    Triple(Card),
    Quadlet(Card),
}

impl RankGroup {
    fn count(&self) -> usize {
        match self {
            RankGroup::Single(..) => 1,
            RankGroup::Double(..) => 2,
            RankGroup::Triple(..) => 3,
            RankGroup::Quadlet(..) => 4,
        }
    }

    fn rank(&self) -> Rank {
        match self {
            RankGroup::Single(card)
            | RankGroup::Double(card, ..)
            | RankGroup::Triple(card, ..)
            | RankGroup::Quadlet(card, ..) => card.rank,
        }
    }
}

impl Hand {
    fn straight_high(&self) -> Option<Rank> {
        use Rank::*;

        let cards = self.cards();
        let ranks = [
            cards[0].rank,
            cards[1].rank,
            cards[2].rank,
            cards[3].rank,
            cards[4].rank,
        ];

        // In an ace-low straight, the ace acts below the two.
        if ranks == [Ace, Five, Four, Three, Two] {
            return Some(Five);
        }

        ranks
            .windows(2)
            .all(|adjacent| adjacent[0].prev() == Some(adjacent[1]))
            .then_some(ranks[0])
    }

    fn is_flush(&self) -> bool {
        let cards = self.cards();
        cards[1..].iter().all(|card| card.suit == cards[0].suit)
    }

    fn group(&self) -> Vec<RankGroup> {
        use RankGroup::*;

        let mut groups: Vec<RankGroup> = self
            .cards()
            .chunk_by(|left, right| left.rank == right.rank)
            .map(|cards| match cards {
                [a] => Single(*a),
                [a, _] => Double(*a),
                [a, _, _] => Triple(*a),
                [a, _, _, _] => Quadlet(*a),
                _ => unreachable!("At most 4 cards of each rank may exist within a hand"),
            })
            .collect();

        groups.sort_by(|left, right| {
            right
                .count()
                .cmp(&left.count())
                .then_with(|| right.rank().cmp(&left.rank()))
        });

        groups
    }
}

impl From<&Hand> for HandValue {
    fn from(value: &Hand) -> Self {
        use HandValue::*;
        use RankGroup::*;

        let cards = value.cards();
        let straight_high = value.straight_high();
        let is_flush = value.is_flush();

        // Royal Flush and Straight Flush
        if let (true, Some(high)) = (is_flush, straight_high) {
            return match high {
                Rank::Ace => RoyalFlush,
                rank => StraightFlush(rank),
            };
        }

        let groups = value.group();

        // FourOfAKind or FullHouse
        match groups.as_slice() {
            [Quadlet(quad, ..), Single(kicker)] => return FourOfAKind(quad.rank, [kicker.rank]),

            [Triple(triple, ..), Double(pair, ..)] => return FullHouse(triple.rank, pair.rank),

            _ => {}
        }

        // Flush or Straight
        if is_flush {
            return Flush([
                cards[0].rank,
                cards[1].rank,
                cards[2].rank,
                cards[3].rank,
                cards[4].rank,
            ]);
        } else if let Some(high) = straight_high {
            return Straight(high);
        }

        // The rest
        match groups.as_slice() {
            [Triple(triple, ..), Single(high), Single(low)] => {
                ThreeOfAKind(triple.rank, [high.rank, low.rank])
            }

            [Double(high_pair, ..), Double(low_pair, ..), Single(kicker)] => {
                TwoPair(high_pair.rank, low_pair.rank, [kicker.rank])
            }

            [Double(pair, ..), Single(k1), Single(k2), Single(k3)] => {
                Pair(pair.rank, [k1.rank, k2.rank, k3.rank])
            }

            [Single(c1), Single(c2), Single(c3), Single(c4), Single(c5)] => {
                HighCard([c1.rank, c2.rank, c3.rank, c4.rank, c5.rank])
            }

            _ => unreachable!("validated hands have one of the known grouping shapes"),
        }
    }
}
