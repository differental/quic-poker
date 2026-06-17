use poker_core::{Action, PlayerId, PokerGame};
use protocol::{ActionError, ClientMessage, ServerMessage, TableError, TableId};
use quinn::Connection;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use std::{
    collections::HashMap,
    env,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

mod constants;

#[cfg(not(feature = "dev"))]
use constants::{FULLCHAIN_PATH, PRIVKEY_PATH};

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
        info!(?player_id, stable_id = key, "Registered new player");
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

    pub fn join_table(
        &mut self,
        player: PlayerId,
        table: TableId,
    ) -> Result<Vec<(Connection, ServerMessage)>, TableError> {
        if self.player_id_to_table_map.contains_key(&player) {
            // Player already in table
            return Err(TableError::PlayerAlreadyInTable);
        }
        if !self.tables.contains_key(&table) {
            return Err(TableError::TableNotFound);
        }

        let mut messages = vec![];

        let curr_table = self.tables.get_mut(&table).unwrap();
        match curr_table {
            TableState::Game(_) => Err(TableError::GameInProgress),
            TableState::Lobby(lobby) => {
                for existing_player in &lobby.players {
                    let conn = &self.connections[existing_player];
                    messages.push((conn.clone(), ServerMessage::PlayerJoinedTable(player)));
                }
                self.player_id_to_table_map.insert(player, table);
                lobby.players.push(player);

                Ok(messages)
            }
        }
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
        if big_blind < small_blind || big_blind > table_max_bet {
            return Err(TableError::InvalidTableConfig);
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
            TableState::Game(game) => game.action(player, action).map_err(ActionError::Rule),
        }
    }

    pub fn notify_table(&self, table: TableId) -> Vec<(Connection, ServerMessage)> {
        let curr_table = self.tables.get(&table).unwrap();

        let TableState::Game(game) = curr_table else {
            panic!("Cannot notify participants of a game that's not ongoing!");
        };

        let players = game.get_player_ids();
        let mut output: Vec<(Connection, ServerMessage)> = Vec::with_capacity(players.len() + 1);

        for player_id in players {
            // Skip DC'd players
            if !self.connections.contains_key(&player_id) {
                continue;
            }

            let view = game.view_for(player_id);
            let conn = &self.connections[&player_id];
            output.push((conn.clone(), ServerMessage::StateUpdate(view)));
            //net::push(conn, &ServerMessage::StateUpdate(view)).await?;

            // Notify next player
            if game.is_next_player(player_id) {
                output.push((conn.clone(), ServerMessage::ItsYourTurn));
                //net::push(conn, &ServerMessage::ItsYourTurn).await?;
            }
        }

        output
    }

    pub fn handle_disconnect(&mut self, conn: &Connection) -> Vec<(Connection, ServerMessage)> {
        let Some(player_id) = self.players.remove(&conn.stable_id()) else {
            // Player never registered
            return vec![];
        };
        self.connections.remove(&player_id);

        let Some(table_id) = self.player_id_to_table_map.remove(&player_id) else {
            // Player not in table
            return vec![];
        };

        let mut messages = Vec::new();

        match self.tables.get_mut(&table_id) {
            None => unreachable!(),
            Some(TableState::Lobby(lobby)) => {
                // Remove player from lobby and do nothing else
                lobby.players.retain(|p| *p != player_id);

                for other_player in &lobby.players {
                    let conn = &self.connections[other_player];
                    messages.push((conn.clone(), ServerMessage::PlayerDisconnected(player_id)));
                }
            }
            Some(TableState::Game(game)) => {
                let game_over = game.fold_disconnected(player_id);

                // Notify the D/C
                for other_player in &game.get_player_ids() {
                    if !self.connections.contains_key(other_player) {
                        // Don't send message to DC'd players
                        continue;
                    }

                    let conn = &self.connections[other_player];
                    messages.push((
                        conn.clone(),
                        ServerMessage::PlayerDisconnectedAndFolded(player_id),
                    ));
                }

                // Send notifications about the updated table status
                if game_over {
                    // Hand finished: Notify showdown results to entire table
                    messages.extend(self.end_game(table_id));
                } else {
                    // Notify entire table (and next player)
                    messages.extend(self.notify_table(table_id));
                }
            }
        };

        messages
    }

    pub fn end_game(&mut self, table: TableId) -> Vec<(Connection, ServerMessage)> {
        // Showdown, send showdown messages, eject all players, return to lobby
        let curr_table = self.tables.get_mut(&table).unwrap();

        let TableState::Game(game) = curr_table else {
            panic!("Cannot notify participants of a game that's not ongoing!");
        };

        let players = game.get_player_ids();
        let mut output: Vec<(Connection, ServerMessage)> = Vec::with_capacity(players.len() + 1);

        let showdown_result = game.get_showdown_results();

        for player_id in players {
            // Skip DC'd players
            if !self.connections.contains_key(&player_id) {
                continue;
            }

            let conn = &self.connections[&player_id];
            //net::push(conn, &ServerMessage::GameOver(showdown_result.clone())).await?;
            output.push((
                conn.clone(),
                ServerMessage::GameOver(showdown_result.clone()),
            ));
        }

        // Reset table to lobby, keeping all players and settings
        *curr_table = TableState::Lobby(Lobby {
            players: game
                .get_player_ids()
                .into_iter()
                .filter(|x| self.connections.contains_key(x))
                .collect(),
            table_max_bet: game.table_max_bet,
            big_blind: game.big_blind,
            small_blind: game.small_blind,
        });

        output
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Use RUST_LOG if set, otherwise default to "info"
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    #[cfg(feature = "dev")]
    let (cert_chain, key) = net::cert::dev::generate_self_signed_cert()?;
    #[cfg(not(feature = "dev"))]
    let (cert_chain, key) =
        net::cert::prod::load_certs_from_file(&*FULLCHAIN_PATH, &*PRIVKEY_PATH)?;

    let port: u16 = env::var("SERVER_PORT")
        .expect("SERVER_PORT must be set")
        .parse()
        .expect("SERVER_PORT must be an integer");

    let server_addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));

    let endpoint = net::make_server_endpoint(server_addr, cert_chain, key)?;
    info!(%server_addr, "Server started");

    let state = Arc::new(Mutex::new(ServerState::new()));

    // Start ten new tables first
    {
        let mut state = state.lock().await;
        for _ in 0..10 {
            let _ = state.new_table();
        }
        info!("Table created");
    }

    while let Some(conn) = endpoint.accept().await {
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            let connection = match conn.await {
                Ok(c) => c,
                Err(e) => {
                    error!(error = %e, "Failed to establish connection");
                    return;
                }
            };
            let remote = connection.remote_address();
            info!(%remote, "Connection established");

            while let Ok((mut send, mut recv)) = connection.accept_bi().await {
                let recv_bytes = match recv.read_to_end(1280).await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!(%remote, error = %e, "Failed to receive message");
                        break;
                    }
                };
                let client_msg: ClientMessage = match protocol::decode(&recv_bytes) {
                    Ok(msg) => msg,
                    Err(e) => {
                        warn!(%remote, error = %e, "Failed to decode message");
                        break;
                    }
                };
                debug!(%remote, ?client_msg, "received message");
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
                        let (result, messages_to_send) = {
                            let mut state = state.lock().await;
                            match state.lookup_player(&connection) {
                                Some(player_id) => match state.join_table(player_id, table_id) {
                                    Ok(messages) => {
                                        (ServerMessage::TableJoinSuccess(table_id), messages)
                                    }
                                    Err(err) => (ServerMessage::TableJoinFailed(err), vec![]),
                                },
                                None => (
                                    ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                                    vec![],
                                ),
                            }
                        };

                        // Network I/O after dropping mutex guard
                        for (conn, message) in messages_to_send {
                            let _ = net::push(&conn, &message).await; // updates fail silently
                        }

                        result
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
                        let (result, messages_to_send) = {
                            let mut state = state.lock().await;
                            match state.lookup_player(&connection) {
                                Some(player_id) => match state.start_game(player_id) {
                                    Ok(()) => {
                                        let table_id = state.player_id_to_table_map[&player_id];
                                        let messages_to_send = state.notify_table(table_id);
                                        (ServerMessage::TableConfigureSuccess, messages_to_send)
                                    }
                                    Err(err) => (ServerMessage::TableConfigureFailed(err), vec![]),
                                },
                                None => (
                                    ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                                    vec![],
                                ),
                            }
                        };

                        // Network I/O after dropping mutex guard
                        for (conn, message) in messages_to_send {
                            let _ = net::push(&conn, &message).await; // updates fail silently
                        }

                        result
                    }
                    ClientMessage::Action(action) => {
                        let (result, messages_to_send) = {
                            let mut state = state.lock().await;
                            match state.lookup_player(&connection) {
                                Some(player_id) => {
                                    match state.execute_poker_action(player_id, action) {
                                        Ok(game_over) => {
                                            let table_id = state.player_id_to_table_map[&player_id];
                                            let messages = {
                                                if game_over {
                                                    // Hand finished: Notify showdown results to entire table
                                                    state.end_game(table_id)
                                                } else {
                                                    // Notify entire table (and next player)
                                                    state.notify_table(table_id)
                                                }
                                            };

                                            (ServerMessage::ActionAccepted, messages)
                                        }
                                        Err(err) => (ServerMessage::ActionRejected(err), vec![]),
                                    }
                                }
                                None => (
                                    ServerMessage::TableJoinFailed(TableError::PlayerNotFound),
                                    vec![],
                                ),
                            }
                        };

                        // Network I/O after dropping mutex guard
                        for (conn, message) in messages_to_send {
                            let _ = net::push(&conn, &message).await; // updates fail silently
                        }

                        result
                    }
                };

                if let Err(e) = net::reply(&mut send, &msg).await {
                    error!(%remote, error = %e, "Failed to send reply");
                    break;
                }
            }

            let messages_to_send = {
                let mut state = state.lock().await;
                state.handle_disconnect(&connection)
            };
            for (conn, message) in messages_to_send {
                let _ = net::push(&conn, &message).await; // updates fail silently
            }

            info!(%remote, "Connection closed");
        });
    }

    Ok(())
}
