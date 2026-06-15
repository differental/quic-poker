use std::{cmp::max, fmt};
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spades => write!(f, "♠"),
            Self::Diamonds => write!(f, "♦"),
            Self::Hearts => write!(f, "♥"),
            Self::Clubs => write!(f, "♣"),
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
enum Rank {
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
    Ace
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
            Self::Seven => write!(f, "7"),
            Self::Eight => write!(f, "8"),
            Self::Nine => write!(f, "9"),
            Self::Ten => write!(f, "10"),
            Self::Jack => write!(f, "J"),
            Self::Queen => write!(f, "Q"),
            Self::King => write!(f, "K"),
            Self::Ace => write!(f, "A"),
        }
    }
}

impl From<Rank> for u8 {
    fn from(rank: Rank) -> Self {
        match rank {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
        }
    }
}

struct Card {
    suit: Suit,
    rank: Rank
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank
    }
}

impl Eq for Card {}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.suit {
            Suit::Spades => write!(f, "\x1b[1;37m{} {}\x1b[0m", self.suit, self.rank),
            Suit::Diamonds => write!(f, "\x1b[1;34m{} {}\x1b[0m", self.suit, self.rank),
            Suit::Clubs => write!(f, "\x1b[1;32m{} {}\x1b[0m", self.suit, self.rank),
            Suit::Hearts => write!(f, "\x1b[1;31m{} {}\x1b[0m", self.suit, self.rank),
        }
        //write!(f, "{} {}", self.suit, self.value)
    }
}

const FULL_DECK: [Card; 52] = [
    Card {
        suit: Suit::Spades,
        rank: Rank::Ace, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Two, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Three, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Four, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Five, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Six, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Seven, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Eight, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Nine, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Ten, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Jack, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::Queen, 
    },
    Card {
        suit: Suit::Spades,
        rank: Rank::King, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Ace, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Two, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Three, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Four, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Five, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Six, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Seven, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Eight, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Nine, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Ten, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Jack, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::Queen, 
    },
    Card {
        suit: Suit::Diamonds,
        rank: Rank::King, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Ace, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Two, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Three, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Four, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Five, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Six, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Seven, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Eight, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Nine, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Ten, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Jack, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::Queen, 
    },
    Card {
        suit: Suit::Hearts,
        rank: Rank::King, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Ace, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Two, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Three, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Four, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Five, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Six, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Seven, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Eight, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Nine, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Ten, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Jack, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::Queen, 
    },
    Card {
        suit: Suit::Clubs,
        rank: Rank::King, 
    },
];

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum PokerHand {
    RoyalFlush,
    StraightFlush(Rank), // Only highest card rank
    Quads(Rank, Rank), // (quad_rank, fifth_card)
    FullHouse(Rank, Rank), // (trip_rank, pair_rank)
    Flush(Rank, Rank, Rank, Rank, Rank),
    Straight(Rank), // Only highest card rank for straight
    Trips(Rank, Rank, Rank), // (trips, fourth_card, fifth_card)
    TwoPair(Rank, Rank, Rank), // (large_pair, small_pair, other_card)
    OnePair(Rank, Rank, Rank, Rank), // (pair, ...)
    HighCard(Rank, Rank, Rank, Rank, Rank)
}

fn evaluate_holdem_hand(cards: &[Card]) -> PokerHand {
    assert!(cards.len() >= 5);

    fn get_sorted_unique_ranks(cards: &[&Card]) -> Vec<Rank> {
        let mut unique_ranks = Vec::<Rank>::with_capacity(cards.len());
        for card in cards {
            if !unique_ranks.contains(&card.rank) {
                unique_ranks.push(card.rank);
            }
        }
        unique_ranks.sort();
        unique_ranks
    }

    fn evaluate_holdem_hand_of_five(cards: &[&Card]) -> PokerHand {
        assert_eq!(cards.len(), 5);
        let unique_ranks = get_sorted_unique_ranks(cards);
        match unique_ranks.len() {
            5 => {
                // Flushes or straights
                let mut straight_high: Option<Rank> = None;
                if Into::<u8>::into(unique_ranks[4]) - Into::<u8>::into(unique_ranks[0]) == 4 {
                    straight_high = Some(unique_ranks[4]);
                } else if unique_ranks.iter().map(|x| Into::<u8>::into(*x)).collect::<Vec<u8>>() == [2, 3, 4, 5, 14] {
                    // Wrap-around for A-2-3-4-5
                    straight_high = Some(Rank::Five)
                }

                let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);

                if let Some(straight_high) = straight_high
                    && is_flush
                {
                    if Into::<u8>::into(straight_high) == 14 {
                        // Royal flush
                        return PokerHand::RoyalFlush;
                    }
                    PokerHand::StraightFlush(straight_high)
                } else if let Some(straight_high) = straight_high {
                    PokerHand::Straight(straight_high)
                } else if is_flush {
                    PokerHand::Flush(
                        unique_ranks[4], unique_ranks[3],unique_ranks[2],unique_ranks[1],unique_ranks[0]
                    )
                } else {
                    PokerHand::HighCard(
                        unique_ranks[4], unique_ranks[3],unique_ranks[2],unique_ranks[1],unique_ranks[0]
                    )
                }
            }
            2 => {
                // Full houses or quads
                let count_first = cards
                    .iter()
                    .filter(|c| c.rank == unique_ranks[0])
                    .count();

                match count_first {
                    4 => {
                        // Quads first
                        PokerHand::Quads(unique_ranks[0], unique_ranks[1])
                    }
                    1 => {
                        // Quads second
                        PokerHand::Quads(unique_ranks[1], unique_ranks[0])
                    }
                    3 => {
                        // Full house first on second
                        PokerHand::FullHouse(unique_ranks[0], unique_ranks[1])
                    }
                    2 => {
                        // Full house second on first
                        PokerHand::FullHouse(unique_ranks[1], unique_ranks[0])
                    }
                    _ => unreachable!(),
                }
            }
            3 => {
                // Two pairs or one trips
                let mut pairs = Vec::with_capacity(2);
                let mut trips = None;
                let mut singulars = Vec::with_capacity(2);
                for rank in unique_ranks {
                    let count = cards
                        .iter()
                        .filter(|c| c.rank == rank)
                        .count();
                    if count == 2 {
                        pairs.push(rank);
                    } else if count == 3 {
                        trips = Some(rank);
                    } else {
                        singulars.push(rank);
                    }
                }

                if let Some(trips) = trips {
                    return PokerHand::Trips(
                        trips.into(),
                        singulars[1].into(),
                        singulars[0].into(),
                    );
                }
                PokerHand::TwoPair(pairs[1], pairs[0], singulars[0])
            }
            4 => {
                // One pair
                let mut double = None;
                let mut singulars = Vec::with_capacity(4);
                for rank in unique_ranks {
                    let count = cards
                        .iter()
                        .filter(|c| c.rank == rank)
                        .count();
                    if count == 2 {
                        double = Some(rank);
                    } else {
                        singulars.push(rank);
                    }
                }
                PokerHand::OnePair(
                    double.unwrap().into(),
                    singulars[2].into(),
                    singulars[1].into(),
                    singulars[0].into(),
                )
            }
            _ => unreachable!(),
        }
    }

    let mut best_hand = PokerHand::HighCard(Rank::Two, Rank::Two, Rank::Two, Rank::Two, Rank::Two);


    for combo in cards.iter().combinations(5) {
        let new_hand = evaluate_holdem_hand_of_five(&combo);

        best_hand = max(best_hand, new_hand)
    }

    best_hand
}







