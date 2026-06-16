use poker_core::{Action, PlayerId, PokerGameResult, PokerGameView, RuleError};
use serde::{Deserialize, Serialize};
pub use serde_json::Error as SerdeError;

#[derive(Clone, Copy, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ActionError {
    Rule(RuleError),
    Table(TableError),
}

#[derive(Serialize, Deserialize)]
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
    GameOver(PokerGameResult),
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TableId(pub u32);

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    // All messages from client to server.
    Hello,
    JoinTable(TableId),
    ConfigureTable(u64, u64, u64), // (max_hand_size, big_blind, small_blind)
    StartGame,
    Action(Action),
}

pub fn encode<T: Serialize>(message: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(message)
}

pub fn decode<'a, T: Deserialize<'a>>(message: &'a str) -> Result<T, serde_json::Error> {
    serde_json::from_str(message)
}
