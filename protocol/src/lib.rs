use poker_core::{Action, PlayerId, PokerGameView, RuleError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    // All messages from server to client.
    TableJoinSuccess(PlayerId),
    TableJoinFailed,
    StateUpdate(PokerGameView),
    ItsYourTurn,
    ActionRejected(RuleError),
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    // All messages from client to server.
    JoinTableRequest,
    Action(Action),
}

pub fn encode<T: Serialize>(message: &T) -> String {
    serde_json::to_string(message).unwrap()
}

pub fn decode<'a, T: Deserialize<'a>>(message: &'a str) -> T {
    serde_json::from_str(message).unwrap()
}
