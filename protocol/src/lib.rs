use poker_core::{Action, PlayerId, PokerGameView, RuleError};
use serde::{Deserialize, Serialize};

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

pub fn encode<T: Serialize>(message: &T) -> String {
    serde_json::to_string(message).unwrap()
}

pub fn decode<'a, T: Deserialize<'a>>(message: &'a str) -> T {
    serde_json::from_str(message).unwrap()
}
