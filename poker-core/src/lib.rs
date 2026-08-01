use itertools::Itertools;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{cmp::max, collections::HashMap, fmt, sync::LazyLock};

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

const SUITS: [Suit; 4] = [Suit::Spades, Suit::Diamonds, Suit::Hearts, Suit::Clubs];
const RANKS: [Rank; 13] = [
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

static FULL_DECK: LazyLock<[Card; 52]> = LazyLock::new(|| {
    let mut deck = [Card {
        suit: Suit::Spades,
        rank: Rank::Two,
    }; 52];
    let mut i = 0;
    for &suit in &SUITS {
        for &rank in &RANKS {
            deck[i] = Card { suit, rank };
            i += 1;
        }
    }
    deck
});

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

pub const MIN_PLAYERS: usize = 3;
pub const MAX_PLAYERS: usize = 10;

/// Upper bound on a starting stack. Chosen so a whole pot (at most
/// `MAX_PLAYERS * MAX_STACK`) fits in both `u64` and `i64`, which makes the pot
/// sums and the `as i64` casts in settlement lossless and the zero-sum PnL
/// invariant exact.
pub const MAX_STACK: u64 = 1 << 40;
const _: () = assert!(MAX_STACK * MAX_PLAYERS as u64 <= i64::MAX as u64);

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
    /// Chips still behind, available to commit. `allin` means this is zero.
    stack: u64,
    /// Chips committed to the pot so far this hand, cumulative across streets.
    bet: u64,
    folded: bool,
    allin: bool,
}

/// Moves up to `amount` from the stack into the bet, flagging all-in when the
/// stack empties. Returns what was actually committed, which is less than
/// `amount` when the stack is too short to cover it.
fn commit(player: &mut PlayerData, amount: u64) -> u64 {
    let paid = amount.min(player.stack);
    player.stack -= paid;
    player.bet += paid;
    if player.stack == 0 {
        player.allin = true;
    }
    paid
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
    pub big_blind: u64,
    pub small_blind: u64,
    player_to_action_idx: usize,
    last_raise_player_idx: usize,
    player_ids_to_idx_map: HashMap<PlayerId, usize>,
    fold_counter: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerDataView {
    id: PlayerId,
    stack: u64,
    bet: u64,
    folded: bool,
    allin: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PokerGameView {
    current_round: PokerRound,
    current_bet: u64,
    player_current_bet: u64,
    player_current_stack: u64,
    hole_cards: Vec<Card>,
    drawn_community_cards: Vec<Card>,
    player_view: Vec<PlayerDataView>,
    player_to_action_idx: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RuleError {
    NotYourTurn,
    CheckOnBet,
    CallOnNoBet,
    BelowMinRaise,
    InsufficientChips,
}

/// One layer of the pot. With no all-ins there is a single layer holding
/// everything; each player who is all-in for less than the final bet splits off
/// another layer that only the players who matched it can win.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pot {
    pub amount: u64,
    pub winners: Vec<PlayerId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PokerGameResult {
    winners: Vec<PlayerId>,
    total_pot: u64,
    community_cards: Vec<Card>,
    player_hands: Vec<(PlayerId, Vec<Card>, bool, PokerHand)>,
    /// Pot layers, main pot first.
    pots: Vec<Pot>,
    /// Net chips won or lost this hand, in seat order (index i is seat i).
    /// Always sums to exactly zero.
    player_pnl: Vec<(PlayerId, i64)>,
    /// What each seat is paid out of the pots, in seat order. Applied to stacks
    /// by [`PokerGame::settle`].
    payouts: Vec<u64>,
}

impl PokerGameResult {
    pub fn player_pnl(&self) -> &[(PlayerId, i64)] {
        &self.player_pnl
    }

    pub fn pots(&self) -> &[Pot] {
        &self.pots
    }

    pub fn winners(&self) -> &[PlayerId] {
        &self.winners
    }

    pub fn total_pot(&self) -> u64 {
        self.total_pot
    }
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
            "round: {} | table bet: {} | your bet: {} | your stack: {}",
            self.current_round,
            self.current_bet,
            self.player_current_bet,
            self.player_current_stack
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
            let blind = match idx {
                0 => "[Button]",
                1 => "    [SB]",
                2 => "    [BB]",
                3 => "   [UTG]",
                _ => "        ",
            };

            writeln!(
                f,
                "{} player {}: stack {}, bet {}{}{}",
                blind, player.id.0, player.stack, player.bet, status, to_act
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

        // A single layer is the ordinary case and needs no breakdown; more than
        // one means someone was all-in for less, so show who could win what.
        if self.pots.len() > 1 {
            for (i, pot) in self.pots.iter().enumerate() {
                let label = if i == 0 {
                    "  main pot".to_string()
                } else {
                    format!("  side pot {i}")
                };
                write!(f, "{label}: {} to", pot.amount)?;
                for id in &pot.winners {
                    write!(f, " player {}", id.0)?;
                }
                writeln!(f)?;
            }
        }

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
            // player_pnl is in seat order while player_hands is in rank order,
            // so look up rather than zip. At most 10 seats, so the scan is free.
            let pnl = self
                .player_pnl
                .iter()
                .find(|(p, _)| p == id)
                .map(|(_, v)| *v)
                .unwrap_or(0);
            write!(f, "  player {}:", id.0)?;
            for card in hole_cards {
                write!(f, " {card}")?;
            }
            writeln!(f, " - {hand}{folded_marker}{marker} ({pnl:+})")?;
        }

        Ok(())
    }
}

impl PokerGame {
    /// `players` is `(id, starting_stack)` in seat order: seat 0 is the button,
    /// 1 the small blind, 2 the big blind.
    pub fn new(players: &[(PlayerId, u64)], small_blind: u64, big_blind: u64) -> Self {
        // Start new poker game
        assert!(players.len() >= MIN_PLAYERS);
        assert!(players.len() <= MAX_PLAYERS);
        assert!(players.iter().all(|(_, stack)| *stack <= MAX_STACK));

        let mut rng = rand::rng();
        let mut deck = Vec::from(*FULL_DECK);
        deck.shuffle(&mut rng);

        let mut player_data = Vec::<PlayerData>::with_capacity(players.len());
        let mut player_ids_to_idx_map = HashMap::new();

        for (i, (player, stack)) in players.iter().enumerate() {
            player_data.push(PlayerData {
                id: *player,
                hole_cards: vec![deck.pop().unwrap(), deck.pop().unwrap()],
                stack: *stack,
                bet: 0,
                folded: false,
                allin: false,
            });
            player_ids_to_idx_map.insert(*player, i);
        }

        // A stack shorter than its blind posts all-in for less rather than going
        // negative, which is exactly the situation side pots exist to settle.
        commit(&mut player_data[1], small_blind);
        commit(&mut player_data[2], big_blind);

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
            big_blind,
            small_blind,
            player_data,
            player_to_action_idx: pre_flop_first_player,
            last_raise_player_idx: pre_flop_first_player,
            player_ids_to_idx_map,
            fold_counter: 0,
        }
    }

    /// Works out who wins what, splitting the pot into side pots when players
    /// are all-in for different amounts. Pure — call [`PokerGame::settle`] to
    /// actually move the chips into stacks.
    pub fn get_showdown_results(&self) -> PokerGameResult {
        // Can only be called on a showdown-round game.
        assert_eq!(self.current_round, PokerRound::Showdown);

        let num_players = self.player_data.len();

        // Best hand per seat, indexed by seat.
        let hands: Vec<PokerHand> = self
            .player_data
            .iter()
            .map(|x| {
                let all_cards: Vec<Card> = self
                    .community_cards
                    .iter()
                    .chain(&x.hole_cards)
                    .cloned()
                    .collect();
                evaluate_holdem_hand(&all_cards)
            })
            .collect();

        // The same hands sorted for display: live players first, then strongest
        // first, so the recap reads as a ranking.
        let player_hands: Vec<(PlayerId, Vec<Card>, bool, PokerHand)> = self
            .player_data
            .iter()
            .zip(&hands)
            .map(|(x, hand)| (x.id, x.hole_cards.clone(), x.folded, *hand))
            .sorted_by(|x, y| Ord::cmp(&x.2, &y.2).then(Ord::cmp(&x.3, &y.3).reverse()))
            .collect();

        let total_pot: u64 = self.player_data.iter().map(|x| x.bet).sum();

        // Slice the pot into one layer per distinct contribution level. A player
        // all-in for less can only win up to what they matched, which is the
        // layer at their own level and everything below it. Folded players'
        // chips are in the pot too: forfeited, but still real money.
        let mut levels: Vec<u64> = self
            .player_data
            .iter()
            .map(|x| x.bet)
            .filter(|bet| *bet > 0)
            .collect();
        levels.sort_unstable();
        levels.dedup();

        struct Layer {
            amount: u64,
            eligible: Vec<usize>,
            contributions: Vec<u64>, // by seat
        }

        let mut layers: Vec<Layer> = Vec::with_capacity(levels.len());
        let mut prev_level = 0u64;
        for level in levels {
            let contributions: Vec<u64> = self
                .player_data
                .iter()
                .map(|x| x.bet.min(level) - x.bet.min(prev_level))
                .collect();
            let eligible: Vec<usize> = (0..num_players)
                .filter(|&seat| {
                    !self.player_data[seat].folded && self.player_data[seat].bet >= level
                })
                .collect();
            layers.push(Layer {
                amount: contributions.iter().sum(),
                eligible,
                contributions,
            });
            prev_level = level;
        }

        // Eligibility only shrinks as the level rises, so a layer with no live
        // claimant is necessarily at the top with only other orphans above it.
        // Fold those chips down into the highest layer someone can still win.
        while layers.len() > 1 && layers.last().unwrap().eligible.is_empty() {
            let orphan = layers.pop().unwrap();
            layers.last_mut().unwrap().amount += orphan.amount;
        }

        let mut payouts = vec![0u64; num_players];
        let mut pots: Vec<Pot> = Vec::with_capacity(layers.len());

        for layer in &layers {
            if layer.eligible.is_empty() {
                // Nobody live ever matched even this level. Hand the chips back
                // to whoever put them in rather than vanishing them.
                for (seat, amount) in layer.contributions.iter().enumerate() {
                    payouts[seat] += amount;
                }
                continue;
            }

            let best = layer
                .eligible
                .iter()
                .map(|&seat| hands[seat])
                .max()
                .unwrap();
            let mut winner_seats: Vec<usize> = layer
                .eligible
                .iter()
                .copied()
                .filter(|&seat| hands[seat] == best)
                .collect();

            // Odd chips that don't divide evenly go one at a time to the seats
            // nearest the button's left: seat 1 (small blind) first, the button
            // at seat 0 last. `eligible` is built in seat order, so this only
            // has to rotate seat 0 to the back.
            winner_seats.sort_by_key(|&seat| (seat + num_players - 1) % num_players);

            let share = layer.amount / winner_seats.len() as u64;
            let remainder = layer.amount % winner_seats.len() as u64;
            for (i, &seat) in winner_seats.iter().enumerate() {
                payouts[seat] += share + if (i as u64) < remainder { 1 } else { 0 };
            }

            pots.push(Pot {
                amount: layer.amount,
                winners: winner_seats
                    .iter()
                    .map(|&seat| self.player_data[seat].id)
                    .collect(),
            });
        }

        let player_pnl: Vec<(PlayerId, i64)> = self
            .player_data
            .iter()
            .enumerate()
            .map(|(seat, x)| (x.id, payouts[seat] as i64 - x.bet as i64))
            .collect();

        // Everyone who took a share of any layer, main pot first, deduped.
        let mut winners: Vec<PlayerId> = Vec::new();
        for pot in &pots {
            for id in &pot.winners {
                if !winners.contains(id) {
                    winners.push(*id);
                }
            }
        }

        PokerGameResult {
            winners,
            total_pot,
            community_cards: self.community_cards.clone(),
            player_hands,
            pots,
            player_pnl,
            payouts,
        }
    }

    /// Pays a showdown result into the players' stacks. Call exactly once per
    /// hand, with the result from [`PokerGame::get_showdown_results`].
    pub fn settle(&mut self, result: &PokerGameResult) {
        assert_eq!(self.current_round, PokerRound::Showdown);
        assert_eq!(result.payouts.len(), self.player_data.len());
        for (seat, player) in self.player_data.iter_mut().enumerate() {
            player.stack += result.payouts[seat];
        }
    }

    /// Final stacks in seat order, for carrying chips into the next hand.
    pub fn stacks(&self) -> Vec<(PlayerId, u64)> {
        self.player_data.iter().map(|x| (x.id, x.stack)).collect()
    }

    fn advance_round(&mut self) -> bool {
        // Advance to next round. Executes showdown if called on a river-round game.
        // Returns if we've completed showdown. true if we have.

        // If only one player hasn't folded, skip to showdown
        if self.fold_counter == self.player_data.len() as u64 - 1 {
            self.current_round = PokerRound::Showdown;
            return true;
        }

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
                self.fold_counter += 1;

                // Check if there's only one player left - if so, proceed directly to showdown through advance_round
                if self.fold_counter == self.player_data.len() as u64 - 1 {
                    let showdown_finished = self.advance_round();
                    return Ok(showdown_finished);
                }
            }
            Action::Call => {
                // Call only legal if player bet different from matching bet
                if curr_player.bet == self.current_bet {
                    return Err(RuleError::CallOnNoBet);
                }
                // A stack too short to cover the call goes all-in for less,
                // which is legal and is what creates a side pot.
                let to_call = self.current_bet - curr_player.bet;
                commit(curr_player, to_call);
            }
            Action::Raise { to: new_bet } => {
                // Raise only legal if new_bet larger than current bet
                if new_bet <= self.current_bet {
                    return Err(RuleError::BelowMinRaise);
                }

                // Can't put in more than is behind. Shoving the exact stack is
                // fine; anything past it is not.
                let to_put_in = new_bet - curr_player.bet;
                if to_put_in > curr_player.stack {
                    return Err(RuleError::InsufficientChips);
                }

                commit(curr_player, to_put_in);
                self.current_bet = new_bet;
                // Update last raise to the current player
                self.last_raise_player_idx = self.player_to_action_idx;
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

    pub fn fold_disconnected(&mut self, player_id: PlayerId) -> bool {
        // Returns a bool signalling whether the game is over
        if player_id == self.player_data[self.player_to_action_idx].id {
            // If it's the current player, we process it as if they folded themselves to handle next player update.
            // Fold is always legal for the current player, so it is safe to call unwrap() here.
            return self.action(player_id, Action::Fold).unwrap();
        }

        let player_idx = self.player_ids_to_idx_map[&player_id];
        let player = &mut self.player_data[player_idx];

        // Don't fold a DC'd player which has already folded, or is all-in.
        if !player.folded && !player.allin {
            player.folded = true;
            self.fold_counter += 1;

            // Check if there's only one player left
            if self.fold_counter == self.player_data.len() as u64 - 1 {
                let showdown_finished = self.advance_round();
                return showdown_finished;
            }
        }

        false
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
            player_current_stack: self.player_data[player_idx].stack,
            hole_cards: self.player_data[player_idx].hole_cards.clone(),
            drawn_community_cards,
            player_view: self
                .player_data
                .iter()
                .map(|x| PlayerDataView {
                    id: x.id,
                    stack: x.stack,
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
                stack: 10_000,
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
            player_to_action_idx: 0,
            last_raise_player_idx: 0,
            player_ids_to_idx_map,
            fold_counter: 0,
        }
    }

    /// Seats a finished hand on `full_board()` with explicit bets, hole cards
    /// and folds. Stacks start at zero so that after `settle` a seat's stack is
    /// exactly what it was paid.
    fn showdown_with(bets: &[u64], hands: &[Vec<Card>], folded: &[usize]) -> PokerGame {
        let mut game = game_with_players(bets.len());
        game.current_round = PokerRound::Showdown;
        game.community_cards = full_board();
        for (seat, player) in game.player_data.iter_mut().enumerate() {
            player.bet = bets[seat];
            player.stack = 0;
            player.hole_cards = hands[seat].clone();
        }
        for &seat in folded {
            game.player_data[seat].folded = true;
        }
        game
    }

    // Against full_board() (2♠ 7♥ T♦ J♣ A♠) these rank aces > kings > queens >
    // rags. Cards compare by rank only, so the two king hands tie exactly.
    fn aces() -> Vec<Card> {
        vec![
            card(Rank::Ace, Suit::Hearts),
            card(Rank::Ace, Suit::Diamonds),
        ]
    }
    fn kings_a() -> Vec<Card> {
        vec![
            card(Rank::King, Suit::Spades),
            card(Rank::King, Suit::Hearts),
        ]
    }
    fn kings_b() -> Vec<Card> {
        vec![
            card(Rank::King, Suit::Diamonds),
            card(Rank::King, Suit::Clubs),
        ]
    }
    fn queens() -> Vec<Card> {
        vec![
            card(Rank::Queen, Suit::Spades),
            card(Rank::Queen, Suit::Hearts),
        ]
    }
    // Three hands that all play the board identically, for three-way ties.
    fn rags(n: usize) -> Vec<Card> {
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs];
        vec![card(Rank::Three, suits[n]), card(Rank::Five, suits[n])]
    }

    fn pnl(result: &PokerGameResult) -> Vec<i64> {
        result.player_pnl().iter().map(|(_, v)| *v).collect()
    }

    // A lone winner takes the pot and is up by everyone else's contributions.
    #[test]
    fn pnl_single_winner_takes_pot_minus_own_bet() {
        let game = showdown_with(&[100, 100, 100], &[aces(), kings_a(), queens()], &[]);
        let result = game.get_showdown_results();
        assert_eq!(pnl(&result), vec![200, -100, -100]);
        assert_eq!(result.winners(), &[PlayerId(0)]);
    }

    // Tied winners split the pot evenly when it divides.
    #[test]
    fn pnl_splits_evenly_between_tied_winners() {
        let game = showdown_with(
            &[100, 100, 100, 100],
            &[queens(), kings_a(), kings_b(), queens()],
            &[],
        );
        let result = game.get_showdown_results();
        assert_eq!(pnl(&result), vec![-100, 100, 100, -100]);
    }

    // An odd chip goes to the small blind, the first seat left of the button.
    // The folded seat's extra chip is orphaned above every live bet, so it also
    // covers the orphan layer merging down instead of vanishing.
    #[test]
    fn pnl_odd_chip_goes_to_seat_left_of_button() {
        let game = showdown_with(&[100, 100, 101], &[kings_a(), kings_b(), aces()], &[2]);
        let result = game.get_showdown_results();
        assert_eq!(result.total_pot(), 301);
        assert_eq!(pnl(&result), vec![50, 51, -101]);
    }

    // With two odd chips the button is served last, not first. This is the test
    // that fails if the winners are left in plain seat order.
    #[test]
    fn pnl_odd_chips_reach_the_button_last() {
        let game = showdown_with(
            &[100, 100, 100, 2],
            &[rags(0), rags(1), rags(2), aces()],
            &[3],
        );
        let result = game.get_showdown_results();
        assert_eq!(result.total_pot(), 302);
        assert_eq!(pnl(&result), vec![0, 1, 1, -2]);
    }

    // A folded player forfeits exactly what they put in.
    #[test]
    fn pnl_folded_player_loses_their_contribution() {
        let game = showdown_with(&[100, 100, 50], &[aces(), kings_a(), queens()], &[2]);
        let result = game.get_showdown_results();
        assert_eq!(pnl(&result), vec![150, -100, -50]);
    }

    // Holding the best hand is worth nothing once you have folded.
    #[test]
    fn pnl_folded_player_never_wins_a_tie() {
        let game = showdown_with(&[100, 100, 100], &[kings_a(), kings_b(), aces()], &[2]);
        let result = game.get_showdown_results();
        assert!(!result.winners().contains(&PlayerId(2)));
        assert_eq!(pnl(&result)[2], -100);
    }

    // A short all-in wins the main pot only; the side pot goes to the best of
    // the players who kept betting.
    #[test]
    fn side_pot_short_allin_wins_only_main_pot() {
        let game = showdown_with(&[100, 500, 500], &[aces(), kings_a(), queens()], &[]);
        let result = game.get_showdown_results();
        assert_eq!(result.pots().len(), 2);
        assert_eq!(result.pots()[0].amount, 300);
        assert_eq!(result.pots()[0].winners, vec![PlayerId(0)]);
        assert_eq!(result.pots()[1].amount, 800);
        assert_eq!(result.pots()[1].winners, vec![PlayerId(1)]);
        assert_eq!(pnl(&result), vec![200, 300, -500]);
    }

    // Three all-ins at three different levels build three layers.
    #[test]
    fn side_pot_three_way_allin_builds_three_layers() {
        let game = showdown_with(&[100, 300, 500], &[queens(), kings_a(), aces()], &[]);
        let result = game.get_showdown_results();
        assert_eq!(result.pots().len(), 3);
        assert_eq!(
            result.pots().iter().map(|p| p.amount).collect::<Vec<_>>(),
            vec![300, 400, 200]
        );
        // Seat 2 has the best hand and matched every level, so it wins them all.
        assert_eq!(pnl(&result), vec![-100, -300, 400]);
    }

    // Chips nobody live can claim are returned rather than lost: an uncalled
    // shove comes back to the player who made it.
    #[test]
    fn uncalled_portion_of_a_bet_is_returned() {
        let game = showdown_with(&[500, 100, 100], &[aces(), kings_a(), queens()], &[]);
        let result = game.get_showdown_results();
        // Seat 0 wins the 300 everyone matched and gets its extra 400 back.
        assert_eq!(pnl(&result), vec![200, -100, -100]);
    }

    // Every fixture settles to exactly zero: chips are conserved.
    #[test]
    fn pnl_sums_to_zero() {
        let fixtures = vec![
            showdown_with(&[100, 100, 100], &[aces(), kings_a(), queens()], &[]),
            showdown_with(&[100, 100, 101], &[kings_a(), kings_b(), aces()], &[2]),
            showdown_with(
                &[100, 100, 100, 2],
                &[rags(0), rags(1), rags(2), aces()],
                &[3],
            ),
            showdown_with(&[100, 500, 500], &[aces(), kings_a(), queens()], &[]),
            showdown_with(&[100, 300, 500], &[queens(), kings_a(), aces()], &[]),
            showdown_with(&[500, 100, 100], &[aces(), kings_a(), queens()], &[]),
            showdown_with(&[0, 0, 0], &[aces(), kings_a(), queens()], &[]),
        ];
        for game in &fixtures {
            let result = game.get_showdown_results();
            assert_eq!(pnl(&result).iter().sum::<i64>(), 0);
        }
    }

    // The pot layers always add back up to the total pot.
    #[test]
    fn pots_sum_to_total_pot() {
        let game = showdown_with(&[100, 300, 500], &[queens(), kings_a(), aces()], &[]);
        let result = game.get_showdown_results();
        assert_eq!(
            result.pots().iter().map(|p| p.amount).sum::<u64>(),
            result.total_pot()
        );
    }

    // Nobody who wins a share ends the hand down on it.
    #[test]
    fn pnl_winner_is_never_negative() {
        for game in [
            showdown_with(&[100, 500, 500], &[aces(), kings_a(), queens()], &[]),
            showdown_with(&[100, 300, 500], &[queens(), kings_a(), aces()], &[]),
            showdown_with(&[100, 100, 101], &[kings_a(), kings_b(), aces()], &[2]),
        ] {
            let result = game.get_showdown_results();
            for (id, delta) in result.player_pnl() {
                if result.winners().contains(id) {
                    assert!(*delta >= 0, "winner {id:?} finished at {delta}");
                }
            }
        }
    }

    // player_pnl is in seat order so it lines up with player_data positionally.
    #[test]
    fn pnl_covers_every_seat_in_seat_order() {
        let game = showdown_with(&[100, 100, 100], &[aces(), kings_a(), queens()], &[]);
        let result = game.get_showdown_results();
        let ids: Vec<PlayerId> = result.player_pnl().iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![PlayerId(0), PlayerId(1), PlayerId(2)]);
    }

    // settle moves the winnings into the stacks, once.
    #[test]
    fn settle_pays_winnings_into_stacks() {
        let mut game = showdown_with(&[100, 500, 500], &[aces(), kings_a(), queens()], &[]);
        let result = game.get_showdown_results();
        game.settle(&result);
        assert_eq!(
            game.stacks(),
            vec![(PlayerId(0), 300), (PlayerId(1), 800), (PlayerId(2), 0)]
        );
    }

    // A stack too short to cover its blind posts what it has and is all-in.
    #[test]
    fn stack_shorter_than_blind_posts_allin_for_less() {
        let game = PokerGame::new(
            &[(PlayerId(0), 1_000), (PlayerId(1), 3), (PlayerId(2), 1_000)],
            5,
            10,
        );
        assert_eq!(game.player_data[1].bet, 3);
        assert_eq!(game.player_data[1].stack, 0);
        assert!(game.player_data[1].allin);
        // The big blind had enough and is not all-in.
        assert_eq!(game.player_data[2].bet, 10);
        assert!(!game.player_data[2].allin);
    }

    // Calling with too few chips commits the whole stack and goes all-in.
    #[test]
    fn short_call_commits_whole_stack_and_flags_allin() {
        let mut game = game_with_players(3);
        game.player_data[0].stack = 40;
        game.current_bet = 100;
        assert!(game.action(PlayerId(0), Action::Call).is_ok());
        assert_eq!(game.player_data[0].bet, 40);
        assert_eq!(game.player_data[0].stack, 0);
        assert!(game.player_data[0].allin);
    }

    // Chips are never created or destroyed by an action: stack + bet holds.
    #[test]
    fn stack_plus_bet_is_conserved_across_actions() {
        let mut game = game_with_players(3);
        let before: Vec<u64> = game.player_data.iter().map(|p| p.stack + p.bet).collect();

        let _ = game.action(PlayerId(0), Action::Raise { to: 500 });
        let _ = game.action(PlayerId(1), Action::Call);
        let _ = game.action(PlayerId(2), Action::Call);

        let after: Vec<u64> = game.player_data.iter().map(|p| p.stack + p.bet).collect();
        assert_eq!(before, after);
    }

    // A whole hand driven through `action`: a short stack is priced in for less
    // than the others, so settlement has to split into a main and a side pot.
    // Hole cards are shuffled, so this asserts the shape rather than a winner.
    #[test]
    fn full_hand_with_short_allin_produces_a_side_pot() {
        let mut game = PokerGame::new(
            &[
                (PlayerId(0), 1_000),
                (PlayerId(1), 1_000),
                (PlayerId(2), 120),
            ],
            5,
            10,
        );

        // Seat 0 opens, seat 1 calls, and seat 2 can only cover 120 of it.
        assert!(!game.action(PlayerId(0), Action::Raise { to: 500 }).unwrap());
        assert!(!game.action(PlayerId(1), Action::Call).unwrap());
        assert!(!game.action(PlayerId(2), Action::Call).unwrap());

        assert_eq!(game.player_data[2].bet, 120);
        assert_eq!(game.player_data[2].stack, 0);
        assert!(game.player_data[2].allin);

        // Everyone left checks it down to showdown.
        let mut over = false;
        for _ in 0..20 {
            if over {
                break;
            }
            let id = game.player_data[game.player_to_action_idx].id;
            over = game.action(id, Action::Check).unwrap();
        }
        assert!(over, "hand should have reached showdown");

        let result = game.get_showdown_results();
        assert_eq!(result.total_pot(), 1_120);
        // Main pot is 120 from each; the side pot is what the other two put in
        // beyond what the short stack could match.
        assert_eq!(result.pots().len(), 2);
        assert_eq!(result.pots()[0].amount, 360);
        assert_eq!(result.pots()[1].amount, 760);
        // The short stack is never eligible for the side pot.
        assert!(!result.pots()[1].winners.contains(&PlayerId(2)));

        let deltas = pnl(&result);
        assert_eq!(deltas.iter().sum::<i64>(), 0);
        // Seat 2 can lose at most what it put in and win at most the main pot.
        assert!((-120..=240).contains(&deltas[2]));

        game.settle(&result);
        assert_eq!(
            game.stacks().iter().map(|(_, stack)| *stack).sum::<u64>(),
            2_120
        );
    }

    // A starting stack above the settlement ceiling is rejected outright.
    #[test]
    #[should_panic]
    fn new_rejects_stack_above_max() {
        PokerGame::new(
            &[
                (PlayerId(0), MAX_STACK + 1),
                (PlayerId(1), 100),
                (PlayerId(2), 100),
            ],
            5,
            10,
        );
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

    // A raise beyond what the player has behind is rejected, untouched state.
    #[test]
    fn raise_beyond_stack_is_rejected() {
        let mut game = game_with_players(3);
        game.player_data[0].stack = 1_000;
        game.current_bet = 100;
        let res = game.action(PlayerId(0), Action::Raise { to: 5_000 });
        assert!(matches!(res, Err(RuleError::InsufficientChips)));
        assert_eq!(game.current_bet, 100);
        assert_eq!(game.player_data[0].bet, 0);
        assert_eq!(game.player_data[0].stack, 1_000);
    }

    // Shoving exactly the stack is legal and flags the player all-in.
    #[test]
    fn raise_of_exactly_the_stack_is_an_allin() {
        let mut game = game_with_players(3);
        game.player_data[0].stack = 1_000;
        game.current_bet = 100;
        assert!(
            game.action(PlayerId(0), Action::Raise { to: 1_000 })
                .is_ok()
        );
        assert_eq!(game.player_data[0].bet, 1_000);
        assert_eq!(game.player_data[0].stack, 0);
        assert!(game.player_data[0].allin);
        assert_eq!(game.current_bet, 1_000);
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
