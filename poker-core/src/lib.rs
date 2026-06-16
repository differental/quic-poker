use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{cmp::max, collections::HashMap, fmt};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
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

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
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
    Ace,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Card {
    suit: Suit,
    rank: Rank,
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Serialize, Deserialize)]
enum PokerHand {
    HighCard(Rank, Rank, Rank, Rank, Rank),
    OnePair(Rank, Rank, Rank, Rank), // (pair, ...)
    TwoPair(Rank, Rank, Rank),       // (large_pair, small_pair, other_card)
    Trips(Rank, Rank, Rank),         // (trips, fourth_card, fifth_card)
    Straight(Rank),                  // Only highest card rank for straight
    Flush(Rank, Rank, Rank, Rank, Rank),
    FullHouse(Rank, Rank), // (trip_rank, pair_rank)
    Quads(Rank, Rank),     // (quad_rank, fifth_card)
    StraightFlush(Rank),   // Only highest card rank
    RoyalFlush,
}

impl fmt::Display for PokerHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PokerHand::HighCard(hi, ..) => write!(f, "high card, {hi} high"),
            PokerHand::OnePair(pair, ..) => write!(f, "pair of {pair}s"),
            PokerHand::TwoPair(hi, lo, _) => write!(f, "two pair, {hi}s and {lo}s"),
            PokerHand::Trips(trips, ..) => write!(f, "three of a kind, {trips}s"),
            PokerHand::Straight(hi) => write!(f, "straight, {hi} high"),
            PokerHand::Flush(hi, ..) => write!(f, "flush, {hi} high"),
            PokerHand::FullHouse(trips, pair) => write!(f, "full house, {trips}s full of {pair}s"),
            PokerHand::Quads(quads, _) => write!(f, "four of a kind, {quads}s"),
            PokerHand::StraightFlush(hi) => write!(f, "straight flush, {hi} high"),
            PokerHand::RoyalFlush => write!(f, "royal flush"),
        }
    }
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
                } else if unique_ranks
                    .iter()
                    .map(|x| Into::<u8>::into(*x))
                    .collect::<Vec<u8>>()
                    == [2, 3, 4, 5, 14]
                {
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
                        unique_ranks[4],
                        unique_ranks[3],
                        unique_ranks[2],
                        unique_ranks[1],
                        unique_ranks[0],
                    )
                } else {
                    PokerHand::HighCard(
                        unique_ranks[4],
                        unique_ranks[3],
                        unique_ranks[2],
                        unique_ranks[1],
                        unique_ranks[0],
                    )
                }
            }
            2 => {
                // Full houses or quads
                let count_first = cards.iter().filter(|c| c.rank == unique_ranks[0]).count();

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
                    let count = cards.iter().filter(|c| c.rank == rank).count();
                    if count == 2 {
                        pairs.push(rank);
                    } else if count == 3 {
                        trips = Some(rank);
                    } else {
                        singulars.push(rank);
                    }
                }

                if let Some(trips) = trips {
                    return PokerHand::Trips(trips, singulars[1], singulars[0]);
                }
                PokerHand::TwoPair(pairs[1], pairs[0], singulars[0])
            }
            4 => {
                // One pair
                let mut double = None;
                let mut singulars = Vec::with_capacity(4);
                for rank in unique_ranks {
                    let count = cards.iter().filter(|c| c.rank == rank).count();
                    if count == 2 {
                        double = Some(rank);
                    } else {
                        singulars.push(rank);
                    }
                }
                PokerHand::OnePair(double.unwrap(), singulars[2], singulars[1], singulars[0])
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

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Action {
    Fold,
    Check,
    Call,
    Raise { to: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerData {
    id: PlayerId,
    hole_cards: Vec<Card>,
    bet: u64,
    folded: bool,
    allin: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum PokerRound {
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PokerGame {
    current_round: PokerRound,
    current_bet: u64,
    community_cards: Vec<Card>,
    player_data: Vec<PlayerData>,
    pub table_max_bet: u64,
    pub big_blind: u64,
    pub small_blind: u64,
    player_to_action_idx: usize,
    last_raise_player_idx: usize,
    player_ids_to_idx_map: HashMap<PlayerId, usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerDataView {
    id: PlayerId,
    bet: u64,
    folded: bool,
    allin: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PokerGameView {
    current_round: PokerRound,
    current_bet: u64,
    player_current_bet: u64,
    hole_cards: Vec<Card>,
    drawn_community_cards: Vec<Card>,
    player_view: Vec<PlayerDataView>,
    player_to_action_idx: usize,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum RuleError {
    NotYourTurn,
    CheckOnBet,
    CallOnNoBet,
    BelowMinRaise,
    ExceedTableMax,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PokerGameResult {
    winners: Vec<PlayerId>,
    total_pot: u64,
    community_cards: Vec<Card>,
    player_hands: Vec<(PlayerId, Vec<Card>, bool, PokerHand)>,
}

impl fmt::Display for PokerRound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PokerRound::PreFlop => "pre-flop",
            PokerRound::Flop => "flop",
            PokerRound::Turn => "turn",
            PokerRound::River => "river",
            PokerRound::Showdown => "showdown",
        };
        write!(f, "{name}")
    }
}

impl fmt::Display for PokerGameView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "round: {} | table bet: {} | your bet: {}",
            self.current_round, self.current_bet, self.player_current_bet
        )?;

        write!(f, "board:")?;
        if self.drawn_community_cards.is_empty() {
            write!(f, " (none)")?;
        } else {
            for card in &self.drawn_community_cards {
                write!(f, " {card}")?;
            }
        }
        writeln!(f)?;

        write!(f, "hand:")?;
        for card in &self.hole_cards {
            write!(f, " {card}")?;
        }
        writeln!(f)?;

        writeln!(f, "players:")?;
        for (idx, player) in self.player_view.iter().enumerate() {
            let to_act = if idx == self.player_to_action_idx {
                " <- to act"
            } else {
                ""
            };
            let status = if player.folded {
                " (folded)"
            } else if player.allin {
                " (all-in)"
            } else {
                ""
            };
            writeln!(
                f,
                "  player {}: bet {}{}{}",
                player.id.0, player.bet, status, to_act
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for PokerGameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "board:")?;
        if self.community_cards.is_empty() {
            write!(f, " (none)")?;
        } else {
            for card in &self.community_cards {
                write!(f, " {card}")?;
            }
        }
        writeln!(f)?;

        writeln!(f, "pot: {}", self.total_pot)?;

        let label = if self.winners.len() > 1 {
            "winners:"
        } else {
            "winner:"
        };
        write!(f, "{label}")?;
        for id in &self.winners {
            write!(f, " player {}", id.0)?;
        }
        writeln!(f)?;

        // player_hands is sorted strongest first, so this reads as a ranking.
        writeln!(f, "hands (best first):")?;
        for (id, hole_cards, folded, hand) in &self.player_hands {
            let marker = if self.winners.contains(id) { " *" } else { "" };
            let folded_marker = if *folded { " (folded)" } else { "" };
            write!(f, "  player {}:", id.0)?;
            for card in hole_cards {
                write!(f, " {card}")?;
            }
            writeln!(f, " - {hand}{folded_marker}{marker}")?;
        }

        Ok(())
    }
}

impl PokerGame {
    pub fn new(players: &[PlayerId], small_blind: u64, big_blind: u64, table_max_bet: u64) -> Self {
        // Start new poker game
        assert!(players.len() >= 3); // min 3 players per game for now
        assert!(players.len() <= 10); // max 10 players per game for now

        let mut rng = rand::rng();
        let mut deck = Vec::from(FULL_DECK);
        deck.shuffle(&mut rng);

        let mut player_data = Vec::<PlayerData>::with_capacity(players.len());
        let mut player_ids_to_idx_map = HashMap::new();

        for (i, player) in players.iter().enumerate() {
            player_data.push(PlayerData {
                id: *player,
                hole_cards: vec![deck.pop().unwrap(), deck.pop().unwrap()],
                bet: 0,
                folded: false,
                allin: false,
            });
            player_ids_to_idx_map.insert(*player, i);
        }

        player_data[1].bet = small_blind;
        player_data[2].bet = big_blind;

        let community_cards = vec![
            deck.pop().unwrap(),
            deck.pop().unwrap(),
            deck.pop().unwrap(),
            deck.pop().unwrap(),
            deck.pop().unwrap(),
        ];

        // players[0] is button, [1] is small blind, [2] is big blind
        // Pre-flop betting starts from 3 (or 0 if 3 people) and ends with big blind 2
        // Post-flop betting starts from 1 and ends with button
        let pre_flop_first_player = if players.len() == 3 { 0usize } else { 3usize };
        PokerGame {
            current_round: PokerRound::PreFlop,
            current_bet: big_blind,
            community_cards,
            table_max_bet,
            big_blind,
            small_blind,
            player_data,
            player_to_action_idx: pre_flop_first_player,
            last_raise_player_idx: pre_flop_first_player,
            player_ids_to_idx_map,
        }
    }

    pub fn get_showdown_results(&self) -> PokerGameResult {
        // Can only be called on a showdown-round game.
        assert_eq!(self.current_round, PokerRound::Showdown);

        // Calculate player hands and sort (id, card, folded, hand) in decreasing order of hand
        let player_hands: Vec<(PlayerId, Vec<Card>, bool, PokerHand)> = self
            .player_data
            .iter()
            .map(|x| {
                (
                    x.id,
                    x.hole_cards.clone(),
                    x.folded,
                    self.community_cards
                        .iter()
                        .chain(&x.hole_cards)
                        .cloned()
                        .collect::<Vec<Card>>(),
                )
            })
            .map(|(id, hole_cards, folded, all_cards)| (id, hole_cards, folded, evaluate_holdem_hand(&all_cards)))
            .sorted_by(|x, y| Ord::cmp(&x.2, &y.2).then(Ord::cmp(&x.3, &y.3).reverse()))
            .collect();

        let total_pot: u64 = self.player_data.iter().map(|x| x.bet).sum();

        let mut winners = vec![player_hands[0].0];

        for i in 1..player_hands.len() {
            if player_hands[i].3 == player_hands[0].3 && !player_hands[i].2 {
                winners.push(player_hands[i].0);
            } else {
                break;
            }
        }

        PokerGameResult {
            winners,
            total_pot,
            community_cards: self.community_cards.clone(),
            player_hands,
        }
    }

    fn advance_round(&mut self) -> bool {
        // Advance to next round. Executes showdown if called on a river-round game.
        // Returns if we've completed showdown. true if we have.
        match self.current_round {
            PokerRound::PreFlop => {
                self.current_round = PokerRound::Flop;
                self.last_raise_player_idx = 1;

                // Find first player starting from small blind (1) that can action
                for idx in (1..self.player_data.len()).chain(std::iter::once(0)) {
                    let player = &self.player_data[idx];
                    if !player.folded && !player.allin {
                        // Found next player
                        self.player_to_action_idx = idx;
                        return false;
                    }
                }

                // All players folded or all-ined. Skip to showdown
                self.current_round = PokerRound::Showdown;
                true
            }
            PokerRound::Flop => {
                self.current_round = PokerRound::Turn;
                self.last_raise_player_idx = 1;

                // Find first player starting from small blind (1) that can action
                for idx in (1..self.player_data.len()).chain(std::iter::once(0)) {
                    let player = &self.player_data[idx];
                    if !player.folded && !player.allin {
                        // Found next player
                        self.player_to_action_idx = idx;
                        return false;
                    }
                }

                // All players folded or all-ined. Skip to showdown
                self.current_round = PokerRound::Showdown;
                true
            }
            PokerRound::Turn => {
                self.current_round = PokerRound::River;
                self.last_raise_player_idx = 1;

                // Find first player starting from small blind (1) that can action
                for idx in (1..self.player_data.len()).chain(std::iter::once(0)) {
                    let player = &self.player_data[idx];
                    if !player.folded && !player.allin {
                        // Found next player
                        self.player_to_action_idx = idx;
                        return false;
                    }
                }

                // All players folded or all-ined. Skip to showdown
                self.current_round = PokerRound::Showdown;
                true
            }
            PokerRound::River => {
                self.current_round = PokerRound::Showdown;
                true
            }
            PokerRound::Showdown => unreachable!(),
        }
    }

    pub fn action(&mut self, player_id: PlayerId, action: Action) -> Result<bool, RuleError> {
        // Returns Ok(bool) signalling if the game is over (true if it's over).
        let curr_player = &mut self.player_data[self.player_to_action_idx];

        if player_id != curr_player.id {
            return Err(RuleError::NotYourTurn);
        }

        match action {
            Action::Check => {
                // Check only legal if player already has matching bet
                if curr_player.bet != self.current_bet {
                    return Err(RuleError::CheckOnBet);
                }
                // Check: Do nothing
            }
            Action::Fold => {
                // Fold always legal
                curr_player.folded = true;
            }
            Action::Call => {
                // Call only legal if player bet different from matching bet
                if curr_player.bet == self.current_bet {
                    return Err(RuleError::CallOnNoBet);
                }
                // Call: Set new bet
                curr_player.bet = self.current_bet;

                // If calling a max bet, it's also an all-in
                if self.current_bet == self.table_max_bet {
                    curr_player.allin = true;
                }
            }
            Action::Raise { to: new_bet } => {
                // Raise only legal if new_bet larger than current bet
                if new_bet <= self.current_bet {
                    return Err(RuleError::BelowMinRaise);
                }

                // Illegal to raise more than table max
                if new_bet > self.table_max_bet {
                    return Err(RuleError::ExceedTableMax);
                }

                // Raise: Set user bet as well as table current bet
                self.current_bet = new_bet;
                curr_player.bet = new_bet;
                // Update last raise to the current player
                self.last_raise_player_idx = self.player_to_action_idx;

                // If user bet is same as table max bet, it's effectively an "all-in"
                if new_bet == self.table_max_bet {
                    curr_player.allin = true;
                }
            }
        }

        // Advance to next person
        let mut idx = (self.player_to_action_idx + 1) % self.player_data.len();

        while idx != self.last_raise_player_idx {
            let player = &self.player_data[idx];
            if !player.folded && !player.allin {
                // Found next player
                self.player_to_action_idx = idx;
                return Ok(false);
            }

            idx = (idx + 1) % self.player_data.len();
        }

        // Current betting round finished. Advance to next round
        let showdown_finished = self.advance_round();

        Ok(showdown_finished)
    }

    pub fn get_player_ids(&self) -> Vec<PlayerId> {
        self.player_data.iter().map(|x| x.id).collect()
    }

    pub fn is_next_player(&self, player_id: PlayerId) -> bool {
        let player_idx = self.player_ids_to_idx_map[&player_id];
        player_idx == self.player_to_action_idx
    }

    pub fn view_for(&self, player_id: PlayerId) -> PokerGameView {
        let player_idx = *(self.player_ids_to_idx_map.get(&player_id).unwrap());

        let drawn_community_cards = match self.current_round {
            PokerRound::PreFlop => vec![],
            PokerRound::Flop => {
                let mut cards = self.community_cards.clone();
                cards.truncate(3);
                cards
            }
            PokerRound::Turn => {
                let mut cards = self.community_cards.clone();
                cards.truncate(4);
                cards
            }
            _ => self.community_cards.clone(),
        };

        PokerGameView {
            current_round: self.current_round,
            current_bet: self.current_bet,
            player_current_bet: self.player_data[player_idx].bet,
            hole_cards: self.player_data[player_idx].hole_cards.clone(),
            drawn_community_cards,
            player_view: self
                .player_data
                .iter()
                .map(|x| PlayerDataView {
                    id: x.id,
                    bet: x.bet,
                    folded: x.folded,
                    allin: x.allin,
                })
                .collect(),
            player_to_action_idx: self.player_to_action_idx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    /// Builds a fresh pre-flop game with `n` seats, no shuffling, so individual
    /// fields can be controlled. Mirrors what `PokerGame::new` would produce for
    /// the parts the betting/round logic cares about.
    fn game_with_players(n: usize) -> PokerGame {
        let player_data: Vec<PlayerData> = (0..n)
            .map(|i| PlayerData {
                id: PlayerId(i as u32),
                hole_cards: vec![],
                bet: 0,
                folded: false,
                allin: false,
            })
            .collect();

        let player_ids_to_idx_map = player_data
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, i))
            .collect();

        PokerGame {
            current_round: PokerRound::PreFlop,
            current_bet: 100,
            community_cards: vec![],
            player_data,
            big_blind: 100,
            small_blind: 50,
            table_max_bet: 10_000,
            player_to_action_idx: 0,
            last_raise_player_idx: 0,
            player_ids_to_idx_map,
        }
    }

    // Hand categories rank from RoyalFlush (best) down to HighCard (worst).
    #[test]
    fn hand_categories_rank_in_order() {
        assert!(
            PokerHand::RoyalFlush > PokerHand::StraightFlush(Rank::King)
                && PokerHand::StraightFlush(Rank::King) > PokerHand::Quads(Rank::Ace, Rank::King)
                && PokerHand::Quads(Rank::Ace, Rank::King)
                    > PokerHand::FullHouse(Rank::Ace, Rank::King)
                && PokerHand::FullHouse(Rank::Ace, Rank::King)
                    > PokerHand::Flush(Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine)
                && PokerHand::Flush(Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine)
                    > PokerHand::Straight(Rank::Ace)
                && PokerHand::Straight(Rank::Ace)
                    > PokerHand::Trips(Rank::Ace, Rank::King, Rank::Queen)
                && PokerHand::Trips(Rank::Ace, Rank::King, Rank::Queen)
                    > PokerHand::TwoPair(Rank::Ace, Rank::King, Rank::Queen)
                && PokerHand::TwoPair(Rank::Ace, Rank::King, Rank::Queen)
                    > PokerHand::OnePair(Rank::Ace, Rank::King, Rank::Queen, Rank::Jack)
                && PokerHand::OnePair(Rank::Ace, Rank::King, Rank::Queen, Rank::Jack)
                    > PokerHand::HighCard(
                        Rank::Ace,
                        Rank::King,
                        Rank::Queen,
                        Rank::Jack,
                        Rank::Nine
                    )
        );
    }

    // A higher straight flush beats a lower one within the same category.
    #[test]
    fn higher_straight_flush_wins() {
        assert!(PokerHand::StraightFlush(Rank::King) > PokerHand::StraightFlush(Rank::Six));
    }

    // T-J-Q-K-A suited evaluates to a royal flush.
    #[test]
    fn evaluate_detects_royal_flush() {
        let cards = [
            card(Rank::Ten, Suit::Hearts),
            card(Rank::Jack, Suit::Hearts),
            card(Rank::Queen, Suit::Hearts),
            card(Rank::King, Suit::Hearts),
            card(Rank::Ace, Suit::Hearts),
            card(Rank::Two, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
        ];
        assert_eq!(evaluate_holdem_hand(&cards), PokerHand::RoyalFlush);
    }

    // Five consecutive suited cards evaluate to a straight flush.
    #[test]
    fn evaluate_detects_straight_flush() {
        let cards = [
            card(Rank::Five, Suit::Spades),
            card(Rank::Six, Suit::Spades),
            card(Rank::Seven, Suit::Spades),
            card(Rank::Eight, Suit::Spades),
            card(Rank::Nine, Suit::Spades),
        ];
        assert!(matches!(
            evaluate_holdem_hand(&cards),
            PokerHand::StraightFlush(..)
        ));
    }

    // Four of a kind evaluates to quads.
    #[test]
    fn evaluate_detects_quads() {
        let cards = [
            card(Rank::Nine, Suit::Spades),
            card(Rank::Nine, Suit::Hearts),
            card(Rank::Nine, Suit::Diamonds),
            card(Rank::Nine, Suit::Clubs),
            card(Rank::King, Suit::Spades),
        ];
        assert!(matches!(evaluate_holdem_hand(&cards), PokerHand::Quads(..)));
    }

    // Trips plus a pair evaluates to a full house.
    #[test]
    fn evaluate_detects_full_house() {
        let cards = [
            card(Rank::Nine, Suit::Spades),
            card(Rank::Nine, Suit::Hearts),
            card(Rank::Nine, Suit::Diamonds),
            card(Rank::King, Suit::Clubs),
            card(Rank::King, Suit::Spades),
        ];
        assert!(matches!(
            evaluate_holdem_hand(&cards),
            PokerHand::FullHouse(..)
        ));
    }

    // Five cards of one suit evaluate to a flush.
    #[test]
    fn evaluate_detects_flush() {
        let cards = [
            card(Rank::Ace, Suit::Spades),
            card(Rank::Jack, Suit::Spades),
            card(Rank::Nine, Suit::Spades),
            card(Rank::Six, Suit::Spades),
            card(Rank::Three, Suit::Spades),
        ];
        assert!(matches!(evaluate_holdem_hand(&cards), PokerHand::Flush(..)));
    }

    // Five consecutive off-suit cards evaluate to a straight.
    #[test]
    fn evaluate_detects_straight() {
        let cards = [
            card(Rank::Five, Suit::Spades),
            card(Rank::Six, Suit::Hearts),
            card(Rank::Seven, Suit::Diamonds),
            card(Rank::Eight, Suit::Clubs),
            card(Rank::Nine, Suit::Spades),
        ];
        assert!(matches!(
            evaluate_holdem_hand(&cards),
            PokerHand::Straight(..)
        ));
    }

    // The A-2-3-4-5 wheel counts as a straight (high card Five).
    #[test]
    fn evaluate_detects_wheel_straight() {
        let cards = [
            card(Rank::Ace, Suit::Spades),
            card(Rank::Two, Suit::Hearts),
            card(Rank::Three, Suit::Diamonds),
            card(Rank::Four, Suit::Clubs),
            card(Rank::Five, Suit::Spades),
        ];
        assert_eq!(
            evaluate_holdem_hand(&cards),
            PokerHand::Straight(Rank::Five)
        );
    }

    // Three of a kind evaluates to trips.
    #[test]
    fn evaluate_detects_trips() {
        let cards = [
            card(Rank::Nine, Suit::Spades),
            card(Rank::Nine, Suit::Hearts),
            card(Rank::Nine, Suit::Diamonds),
            card(Rank::King, Suit::Clubs),
            card(Rank::Two, Suit::Spades),
        ];
        assert!(matches!(evaluate_holdem_hand(&cards), PokerHand::Trips(..)));
    }

    // Two pairs evaluate to two pair.
    #[test]
    fn evaluate_detects_two_pair() {
        let cards = [
            card(Rank::Nine, Suit::Spades),
            card(Rank::Nine, Suit::Hearts),
            card(Rank::Five, Suit::Diamonds),
            card(Rank::Five, Suit::Clubs),
            card(Rank::King, Suit::Spades),
        ];
        assert!(matches!(
            evaluate_holdem_hand(&cards),
            PokerHand::TwoPair(..)
        ));
    }

    // A single pair evaluates to one pair.
    #[test]
    fn evaluate_detects_one_pair() {
        let cards = [
            card(Rank::Nine, Suit::Spades),
            card(Rank::Nine, Suit::Hearts),
            card(Rank::King, Suit::Diamonds),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Two, Suit::Spades),
        ];
        assert!(matches!(
            evaluate_holdem_hand(&cards),
            PokerHand::OnePair(..)
        ));
    }

    // Five unconnected off-suit cards evaluate to high card.
    #[test]
    fn evaluate_detects_high_card() {
        let cards = [
            card(Rank::Ace, Suit::Spades),
            card(Rank::King, Suit::Hearts),
            card(Rank::Queen, Suit::Diamonds),
            card(Rank::Jack, Suit::Clubs),
            card(Rank::Nine, Suit::Spades),
        ];
        assert_eq!(
            evaluate_holdem_hand(&cards),
            PokerHand::HighCard(Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine)
        );
    }

    // From seven cards the best five-card hand is chosen (flush over a pair).
    #[test]
    fn evaluate_picks_best_of_seven() {
        let cards = [
            card(Rank::Ace, Suit::Spades),
            card(Rank::King, Suit::Spades),
            card(Rank::Queen, Suit::Spades),
            card(Rank::Jack, Suit::Spades),
            card(Rank::Nine, Suit::Spades),
            card(Rank::Two, Suit::Hearts),
            card(Rank::Two, Suit::Diamonds),
        ];
        assert!(matches!(evaluate_holdem_hand(&cards), PokerHand::Flush(..)));
    }

    // PreFlop: action falls through to the button when both blinds are out.
    #[test]
    fn advance_preflop_finds_button_when_blinds_cannot_act() {
        let mut game = game_with_players(3);
        game.player_data[1].folded = true;
        game.player_data[2].allin = true;
        game.advance_round();
        assert_eq!(game.player_to_action_idx, 0);
    }

    // Flop: action falls through to the button when both blinds are out.
    #[test]
    fn advance_flop_finds_button_when_blinds_cannot_act() {
        let mut game = game_with_players(3);
        game.current_round = PokerRound::Flop;
        game.player_data[1].folded = true;
        game.player_data[2].allin = true;
        game.advance_round();
        assert_eq!(game.player_to_action_idx, 0);
    }

    // Turn: action falls through to the button when both blinds are out.
    #[test]
    fn advance_turn_finds_button_when_blinds_cannot_act() {
        let mut game = game_with_players(3);
        game.current_round = PokerRound::Turn;
        game.player_data[1].folded = true;
        game.player_data[2].allin = true;
        game.advance_round();
        assert_eq!(game.player_to_action_idx, 0);
    }

    // Bets are cumulative across streets by design and are not reset on advance.
    #[test]
    fn cumulative_bets_persist_across_rounds() {
        let mut game = game_with_players(3);
        game.current_bet = 100;
        for p in &mut game.player_data {
            p.bet = 100;
        }
        game.advance_round();
        assert_eq!(game.current_bet, 100);
        assert_eq!(game.player_data[1].bet, 100);
    }

    // A raise above table_max_bet is rejected and leaves state untouched.
    #[test]
    fn raise_above_table_max_is_rejected() {
        let mut game = game_with_players(3);
        game.table_max_bet = 1_000;
        game.current_bet = 100;
        let res = game.action(PlayerId(0), Action::Raise { to: 5_000 });
        assert!(res.is_err());
        assert_eq!(game.current_bet, 100);
        assert_eq!(game.player_data[0].bet, 0);
    }

    // Cards compare by rank only; suit is intentionally ignored.
    #[test]
    fn cards_compare_by_rank_only() {
        assert_eq!(card(Rank::Ace, Suit::Spades), card(Rank::Ace, Suit::Hearts));
        assert!(card(Rank::Ace, Suit::Spades) > card(Rank::King, Suit::Spades));
    }

    // A fixed five-card board for view_for tests.
    fn full_board() -> Vec<Card> {
        vec![
            card(Rank::Two, Suit::Spades),
            card(Rank::Seven, Suit::Hearts),
            card(Rank::Ten, Suit::Diamonds),
            card(Rank::Jack, Suit::Clubs),
            card(Rank::Ace, Suit::Spades),
        ]
    }

    // view_for: pre-flop reveals none of the board.
    #[test]
    fn view_hides_board_preflop() {
        let mut game = game_with_players(3);
        game.community_cards = full_board();
        assert!(game.view_for(PlayerId(0)).drawn_community_cards.is_empty());
    }

    // view_for: the flop reveals exactly the first three community cards.
    #[test]
    fn view_reveals_three_on_flop() {
        let mut game = game_with_players(3);
        game.current_round = PokerRound::Flop;
        game.community_cards = full_board();
        assert_eq!(
            game.view_for(PlayerId(0)).drawn_community_cards,
            full_board()[..3].to_vec()
        );
    }

    // view_for: the turn reveals exactly the first four community cards.
    #[test]
    fn view_reveals_four_on_turn() {
        let mut game = game_with_players(3);
        game.current_round = PokerRound::Turn;
        game.community_cards = full_board();
        assert_eq!(
            game.view_for(PlayerId(0)).drawn_community_cards,
            full_board()[..4].to_vec()
        );
    }

    // view_for: the river reveals the whole board.
    #[test]
    fn view_reveals_full_board_on_river() {
        let mut game = game_with_players(3);
        game.current_round = PokerRound::River;
        game.community_cards = full_board();
        assert_eq!(
            game.view_for(PlayerId(0)).drawn_community_cards,
            full_board()
        );
    }

    // view_for: showdown reveals the whole board.
    #[test]
    fn view_reveals_full_board_on_showdown() {
        let mut game = game_with_players(3);
        game.current_round = PokerRound::Showdown;
        game.community_cards = full_board();
        assert_eq!(
            game.view_for(PlayerId(0)).drawn_community_cards,
            full_board()
        );
    }

    // view_for: a player id is resolved to the right seat via the id->index map.
    #[test]
    fn view_resolves_player_id_to_correct_seat() {
        let mut game = game_with_players(3);
        let theirs = vec![
            card(Rank::Queen, Suit::Spades),
            card(Rank::Jack, Suit::Hearts),
        ];
        game.player_data[2].hole_cards = theirs.clone();
        game.player_data[2].bet = 250;

        let view = game.view_for(PlayerId(2));
        assert_eq!(view.hole_cards, theirs);
        assert_eq!(view.player_current_bet, 250);
    }

    // view_for: only the requesting player's hole cards are exposed.
    #[test]
    fn view_exposes_only_requesters_hole_cards() {
        let mut game = game_with_players(3);
        let mine = vec![
            card(Rank::Ace, Suit::Spades),
            card(Rank::King, Suit::Hearts),
        ];
        game.player_data[0].hole_cards = mine.clone();
        game.player_data[1].hole_cards = vec![
            card(Rank::Two, Suit::Clubs),
            card(Rank::Three, Suit::Diamonds),
        ];

        let view = game.view_for(PlayerId(0));
        assert_eq!(view.hole_cards, mine);
        // player_view carries public state only — there is no hole_cards field at all.
        assert_eq!(view.player_view.len(), 3);
    }

    // view_for: player_current_bet reflects the requesting player's own bet.
    #[test]
    fn view_reports_requesters_own_bet() {
        let mut game = game_with_players(3);
        game.player_data[2].bet = 500;
        assert_eq!(game.view_for(PlayerId(2)).player_current_bet, 500);
    }

    // view_for: the view surfaces whose turn it is to act.
    #[test]
    fn view_reports_player_to_action() {
        let mut game = game_with_players(3);
        game.player_to_action_idx = 2;
        assert_eq!(game.view_for(PlayerId(0)).player_to_action_idx, 2);
    }

    // view_for: player_view mirrors every player's public state in seat order.
    #[test]
    fn view_player_view_mirrors_public_state() {
        let mut game = game_with_players(3);
        game.player_data[1].folded = true;
        game.player_data[2].allin = true;
        game.player_data[2].bet = 9_000;

        let view = game.view_for(PlayerId(0));
        assert_eq!(view.player_view.len(), 3);
        assert_eq!(view.player_view[0].id, PlayerId(0));
        assert!(view.player_view[1].folded);
        assert!(view.player_view[2].allin);
        assert_eq!(view.player_view[2].bet, 9_000);
    }
}
