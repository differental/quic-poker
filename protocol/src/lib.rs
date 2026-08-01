use poker_core::{Action, PlayerId, PokerGameResult, PokerGameView, RuleError};
pub use rmp_serde::decode::Error as DecodeError;
pub use rmp_serde::encode::Error as EncodeError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Longest accepted player name. Names become persisted ledger keys, and the
/// server caps an incoming message at 1280 bytes, so keep this well inside it.
pub const MAX_NAME_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TableError {
    PlayerNotFound,
    PlayerAlreadyInTable,
    TableNotFound,
    GameInProgress,
    GameNotStarted,
    NotInTable,
    TooFewPlayers,
    TooManyPlayers,
    InvalidTableConfig,
    InvalidName,
}

/// One player's line on the standings board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandingsEntry {
    pub name: String,
    pub id: PlayerId,
    /// Running profit or loss across every hand this account has ever played.
    pub pnl: i64,
    /// Chips currently in front of them at this table.
    pub stack: u64,
}

/// Cumulative PnL for everyone at one table, best first.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Standings {
    pub table: TableId,
    pub entries: Vec<StandingsEntry>,
}

impl fmt::Display for Standings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "standings (table {}):", self.table.0)?;
        if self.entries.is_empty() {
            writeln!(f, "  (nobody at this table)")?;
        }
        for entry in &self.entries {
            writeln!(
                f,
                "  {} (player {}): {:+} total, stack {}",
                entry.name, entry.id.0, entry.pnl, entry.stack
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ActionError {
    Rule(RuleError),
    Table(TableError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    // All messages from server to client.
    Welcome(PlayerId),
    AlreadyConnected,
    TableJoinSuccess(TableId),
    TableJoinFailed(TableError),
    TableConfigureSuccess,
    TableConfigureFailed(TableError),
    StateUpdate(PokerGameView),
    ItsYourTurn,
    ActionAccepted,
    ActionRejected(ActionError),
    PlayerJoinedTable(PlayerId),
    PlayerDisconnected(PlayerId),          // Lobby D/C
    PlayerDisconnectedAndFolded(PlayerId), // In-game D/C
    GameOver(PokerGameResult),
    // New variants must be appended: rmp-serde encodes enums by variant index,
    // so inserting one silently breaks peers built from another revision.
    Standings(Standings),
    StandingsFailed(TableError),
}

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableId(pub u32);

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    // All messages from client to server.
    Hello(String), // Player name: the key the persisted PnL ledger is kept under
    JoinTable(TableId),
    ConfigureTable(u64, u64, u64), // (starting_stack, big_blind, small_blind)
    StartGame,
    Action(Action),
    GetStandings,
}

pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(message)
}

pub fn decode<'a, T: Deserialize<'a>>(message: &'a [u8]) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_slice(message)
}
