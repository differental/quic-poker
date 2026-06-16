use poker_core::{Action, PlayerId, PokerGame};
use protocol::{ActionError, ClientMessage, ServerMessage, TableError, TableId};
use quinn::Connection;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::sync::Mutex;

use std::collections::HashMap;

struct Lobby {
    players: Vec<PlayerId>,
    table_max_bet: u64,
    big_blind: u64,
    small_blind: u64,
}

enum TableState {
    Lobby(Lobby),
    Game(PokerGame),
}

struct ServerState {
    tables: HashMap<TableId, TableState>,
    next_table_id: TableId,
    players: HashMap<usize, PlayerId>, // keys: connection.stable_id()
    next_player_id: PlayerId,
    player_id_to_table_map: HashMap<PlayerId, TableId>,
    connections: HashMap<PlayerId, Connection>, // Connection objects required to notify all players
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            next_table_id: TableId(0),
            players: HashMap::new(),
            next_player_id: PlayerId(0),
            player_id_to_table_map: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    pub fn new_player(&mut self, conn: &Connection) -> Option<PlayerId> {
        let key = conn.stable_id();
        if self.players.contains_key(&key) {
            return None;
        }
        let player_id = self.next_player_id;
        self.players.insert(key, player_id);
        self.connections.insert(player_id, conn.clone());
        self.next_player_id.0 += 1;
        println!("New player #{player_id:?}: {key}");
        Some(player_id)
    }

    pub fn lookup_player(&self, conn: &Connection) -> Option<PlayerId> {
        let key = conn.stable_id();
        if !self.players.contains_key(&key) {
            return None;
        }
        Some(self.players[&key])
    }

    pub fn new_table(&mut self) -> TableId {
        let table_id = self.next_table_id;
        self.next_table_id.0 += 1;
        self.tables.insert(
            table_id,
            TableState::Lobby(Lobby {
                players: vec![],
                table_max_bet: 1000u64,
                big_blind: 10u64,
                small_blind: 5u64,
            }),
        );
        table_id
    }

    pub fn join_table(&mut self, player: PlayerId, table: TableId) -> Result<(), TableError> {
        if self.player_id_to_table_map.contains_key(&player) {
            // Player already in table
            return Err(TableError::PlayerAlreadyInTable);
        }
        if !self.tables.contains_key(&table) {
            return Err(TableError::TableNotFound);
        }

        let curr_table = self.tables.get_mut(&table).unwrap();
        match curr_table {
            TableState::Game(_) => return Err(TableError::GameInProgress),
            TableState::Lobby(lobby) => {
                self.player_id_to_table_map.insert(player, table);
                lobby.players.push(player);
            }
        };

        Ok(())
    }

    pub fn configure_table(
        &mut self,
        player: PlayerId,
        table_max_bet: u64,
        big_blind: u64,
        small_blind: u64,
    ) -> Result<(), TableError> {
        if !self.player_id_to_table_map.contains_key(&player) {
            return Err(TableError::NotInTable);
        }
        let table = self.player_id_to_table_map[&player];
        let curr_table = self.tables.get_mut(&table).unwrap();

        match curr_table {
            TableState::Game(_) => return Err(TableError::GameInProgress),
            TableState::Lobby(lobby) => {
                lobby.table_max_bet = table_max_bet;
                lobby.big_blind = big_blind;
                lobby.small_blind = small_blind;
            }
        };

        Ok(())
    }

    pub fn start_game(&mut self, player: PlayerId) -> Result<(), TableError> {
        if !self.player_id_to_table_map.contains_key(&player) {
            return Err(TableError::NotInTable);
        }
        let table = self.player_id_to_table_map[&player];
        let curr_table = self.tables.get_mut(&table).unwrap();

        match curr_table {
            TableState::Lobby(lobby) => {
                if lobby.players.len() < 3 {
                    return Err(TableError::TooFewPlayers);
                }
                if lobby.players.len() > 10 {
                    return Err(TableError::TooManyPlayers);
                }

                *curr_table = TableState::Game(PokerGame::new(
                    &lobby.players,
                    lobby.small_blind,
                    lobby.big_blind,
                    lobby.table_max_bet,
                ));

                Ok(())
            }
            TableState::Game(_) => Err(TableError::GameInProgress),
        }
    }

    pub fn execute_poker_action(
        &mut self,
        player: PlayerId,
        action: Action,
    ) -> Result<bool, ActionError> {
        // Returns Ok(bool) if no errors, where bool indicates if game has completed
        if !self.player_id_to_table_map.contains_key(&player) {
            return Err(ActionError::Table(TableError::NotInTable));
        }
        let table = self.player_id_to_table_map[&player];
        let curr_table = self.tables.get_mut(&table).unwrap();

        match curr_table {
            TableState::Lobby(_) => Err(ActionError::Table(TableError::GameNotStarted)),
            TableState::Game(game) => {
                game.action(player, action).map_err(|re| ActionError::Rule(re))
            }
        }
    }

    pub async fn notify_table(&self, table: TableId) -> Result<(), anyhow::Error> {
        let curr_table = self.tables.get(&table).unwrap();

        match curr_table {
            TableState::Lobby(_) => {
                panic!("Cannot notify participants of a game that's not ongoing!")
            }
            TableState::Game(game) => {
                let players = game.get_player_ids();

                for player_id in players {
                    let view = game.view_for(player_id);
                    let conn = &self.connections[&player_id];
                    net::push(conn, &ServerMessage::StateUpdate(view)).await?;

                    // Notify next player
                    if game.is_next_player(player_id) {
                        net::push(conn, &ServerMessage::ItsYourTurn).await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn end_game(&mut self, table: TableId) -> Result<(), anyhow::Error> {
        // Showdown, send showdown messages, eject all players, return to lobby
        let curr_table = self.tables.get_mut(&table).unwrap();

        match curr_table {
            TableState::Lobby(_) => {
                panic!("Cannot showdown in a lobby!")
            }
            TableState::Game(game) => {
                let players = game.get_player_ids();

                let showdown_result = game.get_showdown_results();

                for player_id in players {
                    let conn = &self.connections[&player_id];
                    net::push(conn, &ServerMessage::GameOver(showdown_result.clone())).await?;
                }

                // Reset table to lobby, keeping all players and settings
                *curr_table = TableState::Lobby(Lobby {
                    players: game.get_player_ids(),
                    table_max_bet: game.table_max_bet,
                    big_blind: game.big_blind,
                    small_blind: game.small_blind,
                });
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (cert, key) = net::cert::dev::generate_self_signed_cert()?;

    const SERVER_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5000));

    let endpoint = net::make_server_endpoint(SERVER_ADDR, cert, key)?;

    let state = Arc::new(Mutex::new(ServerState::new()));

    // Start a new table first
    {
        let mut state = state.lock().await;
        state.new_table();
    }

    while let Some(conn) = endpoint.accept().await {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let connection = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("connection failed: {e}");
                    return;
                }
            };

            while let Ok((mut send, mut recv)) = connection.accept_bi().await {
                let recv_bytes = recv
                    .read_to_end(1280)
                    .await
                    .expect("Error in receiving message");
                let client_msg: ClientMessage =
                    protocol::decode(&String::from_utf8(recv_bytes).unwrap());
                let msg: ServerMessage = match client_msg {
                    ClientMessage::Hello => {
                        let mut state = state.lock().await;
                        let player_id = state.new_player(&connection);
                        // If connection already established: Decline message
                        match player_id {
                            Some(player_id) => ServerMessage::Welcome(player_id),
                            None => ServerMessage::AlreadyConnected,
                        }
                    }
                    ClientMessage::JoinTable(table_id) => {
                        let mut state = state.lock().await;
                        match state.lookup_player(&connection) {
                            Some(player_id) => match state.join_table(player_id, table_id) {
                                Ok(()) => ServerMessage::TableJoinSuccess(table_id),
                                Err(err) => ServerMessage::TableJoinFailed(err),
                            },
                            None => ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                        }
                    }
                    ClientMessage::ConfigureTable(table_max_bet, big_blind, small_blind) => {
                        let mut state = state.lock().await;
                        match state.lookup_player(&connection) {
                            Some(player_id) => {
                                match state.configure_table(
                                    player_id,
                                    table_max_bet,
                                    big_blind,
                                    small_blind,
                                ) {
                                    Ok(()) => ServerMessage::TableConfigureSuccess,
                                    Err(err) => ServerMessage::TableConfigureFailed(err),
                                }
                            }
                            None => ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                        }
                    }
                    ClientMessage::StartGame => {
                        let mut state = state.lock().await;
                        match state.lookup_player(&connection) {
                            Some(player_id) => match state.start_game(player_id) {
                                Ok(()) => ServerMessage::TableConfigureSuccess,
                                Err(err) => ServerMessage::TableConfigureFailed(err),
                            },
                            None => ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                        }
                    }
                    ClientMessage::Action(action) => {
                        let mut state = state.lock().await;
                        match state.lookup_player(&connection) {
                            Some(player_id) => {
                                match state.execute_poker_action(player_id, action) {
                                    Ok(game_over) => {
                                        let table_id = state.player_id_to_table_map[&player_id];
                                        if game_over {
                                            // Hand finished: Notify showdown results to entire table
                                            let _ = state.end_game(table_id).await;
                                        } else {
                                            // Notify entire table (and next player)
                                            let _ = state.notify_table(table_id).await; // Update fails silently
                                        }

                                        ServerMessage::ActionAccepted
                                    }
                                    Err(err) => ServerMessage::ActionRejected(err),
                                }
                            }
                            None => ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                        }
                    }
                };

                net::reply(send, &msg).await.expect("Error in replying");
            }
        });
    }

    Ok(())
}
