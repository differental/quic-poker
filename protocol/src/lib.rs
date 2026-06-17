use poker_core::{Action, PlayerId, PokerGameResult, PokerGameView, RuleError};
pub use rmp_serde::decode::Error as DecodeError;
pub use rmp_serde::encode::Error as EncodeError;
use serde::{Deserialize, Serialize};

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
}

#[derive(Clone, Copy, Eq, Hash, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableId(pub u32);

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    // All messages from client to server.
    Hello,
    JoinTable(TableId),
    ConfigureTable(u64, u64, u64), // (max_hand_size, big_blind, small_blind)
    StartGame,
    Action(Action),
}

pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec(message)
}

pub fn decode<'a, T: Deserialize<'a>>(message: &'a [u8]) -> Result<T, rmp_serde::decode::Error> {
    rmp_serde::from_slice(message)
}
