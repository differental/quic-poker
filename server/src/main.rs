use poker_core::{Action, PlayerId, PokerGame};
use protocol::{
    ActionError, ClientMessage, ServerMessage, Standings, StandingsEntry, TableError, TableId,
};
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
mod store;

use store::{Account, HandLogPlayer};

#[cfg(not(feature = "dev"))]
use constants::{FULLCHAIN_PATH, PRIVKEY_PATH};

struct TableConfig {
    starting_stack: u64,
    big_blind: u64,
    small_blind: u64,
}

enum TableState {
    Lobby,
    Game(PokerGame),
}

/// The roster, settings and chips live on the table rather than inside the
/// state enum, because they have to survive the lobby/game transition in both
/// directions — chips carried between hands are the whole point.
struct Table {
    /// Seat order: 0 is the button, 1 the small blind, 2 the big blind. Rotated
    /// left by one after each hand so the blinds move around.
    players: Vec<PlayerId>,
    config: TableConfig,
    state: TableState,
    /// Chips in front of each seat, carried between hands. The running PnL
    /// lives on the account, not here.
    stacks: HashMap<PlayerId, u64>,
    hands_played: u64,
}

impl Table {
    fn in_game(&self) -> bool {
        matches!(self.state, TableState::Game(_))
    }
}

struct ServerState {
    tables: HashMap<TableId, Table>,
    next_table_id: TableId,
    players: HashMap<usize, PlayerId>, // keys: connection.stable_id()
    next_player_id: PlayerId,
    player_id_to_table_map: HashMap<PlayerId, TableId>,
    connections: HashMap<PlayerId, Connection>, // Connection objects required to notify all players
    /// The persistent PnL ledger, keyed by player name. `PlayerId` is minted per
    /// connection and restarts at 0 each boot, so it cannot key anything that
    /// has to outlive a session. Accounts are never removed.
    accounts: HashMap<String, Account>,
    /// Which account each live connection is playing on. Dropped on disconnect;
    /// the `Account` behind it is not.
    player_to_account: HashMap<PlayerId, String>,
}

impl ServerState {
    pub fn new(accounts: HashMap<String, Account>) -> Self {
        Self {
            tables: HashMap::new(),
            next_table_id: TableId(0),
            players: HashMap::new(),
            next_player_id: PlayerId(0),
            player_id_to_table_map: HashMap::new(),
            connections: HashMap::new(),
            accounts,
            player_to_account: HashMap::new(),
        }
    }

    pub fn new_player(&mut self, conn: &Connection, name: &str) -> Result<PlayerId, TableError> {
        let key = conn.stable_id();
        if self.players.contains_key(&key) {
            return Err(TableError::PlayerAlreadyInTable);
        }

        let name = name.trim();
        if name.is_empty() || name.len() > protocol::MAX_NAME_LEN {
            return Err(TableError::InvalidName);
        }

        // One live connection per account, or two sessions would fight over the
        // same running total.
        if self.player_to_account.values().any(|n| n == name) {
            return Err(TableError::PlayerAlreadyInTable);
        }

        let player_id = self.next_player_id;
        self.players.insert(key, player_id);
        self.connections.insert(player_id, conn.clone());
        self.player_to_account.insert(player_id, name.to_string());
        let account = self.accounts.entry(name.to_string()).or_default();
        let pnl = account.pnl;
        self.next_player_id.0 += 1;

        info!(?player_id, name, pnl, stable_id = key, "Player signed in");
        Ok(player_id)
    }

    /// Adds `amount` to a player's buy-in total. Not a PnL event: it hands over
    /// chips and an equal debt, which is what keeps `pnl == stack - bought_in`.
    fn record_buy_in(&mut self, player: PlayerId, amount: u64) {
        if let Some(name) = self.player_to_account.get(&player)
            && let Some(account) = self.accounts.get_mut(name)
        {
            account.total_bought_in = account.total_bought_in.saturating_add(amount);
        }
    }

    /// Records chips leaving the table with a player. The counterpart to
    /// [`ServerState::record_buy_in`]: without it `total_bought_in` would only
    /// ever grow and would stop lining up with `pnl`.
    fn record_cash_out(&mut self, player: PlayerId, amount: u64) {
        if let Some(name) = self.player_to_account.get(&player)
            && let Some(account) = self.accounts.get_mut(name)
        {
            account.total_bought_in = account.total_bought_in.saturating_sub(amount);
        }
    }

    /// Corrects a buy-in already recorded, when a player is re-seated at a
    /// different starting stack before playing a hand.
    fn adjust_buy_in(&mut self, player: PlayerId, old: u64, new: u64) {
        if let Some(name) = self.player_to_account.get(&player)
            && let Some(account) = self.accounts.get_mut(name)
        {
            account.total_bought_in = account
                .total_bought_in
                .saturating_sub(old)
                .saturating_add(new);
        }
    }

    fn account_pnl(&self, player: PlayerId) -> i64 {
        self.player_to_account
            .get(&player)
            .and_then(|name| self.accounts.get(name))
            .map(|account| account.pnl)
            .unwrap_or(0)
    }

    fn account_name(&self, player: PlayerId) -> String {
        self.player_to_account
            .get(&player)
            .cloned()
            .unwrap_or_else(|| format!("player {}", player.0))
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
            Table {
                players: vec![],
                config: TableConfig {
                    starting_stack: 1000u64,
                    big_blind: 10u64,
                    small_blind: 5u64,
                },
                state: TableState::Lobby,
                stacks: HashMap::new(),
                hands_played: 0,
            },
        );
        debug!(?table_id, "Created table");
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
        if curr_table.in_game() {
            return Err(TableError::GameInProgress);
        }

        for existing_player in &curr_table.players {
            let conn = &self.connections[existing_player];
            messages.push((conn.clone(), ServerMessage::PlayerJoinedTable(player)));
        }

        // Sitting down buys you in for the table's starting stack.
        let buy_in = curr_table.config.starting_stack;
        curr_table.players.push(player);
        curr_table.stacks.insert(player, buy_in);
        self.player_id_to_table_map.insert(player, table);

        self.record_buy_in(player, buy_in);
        debug!(?player, ?table, buy_in, "Player joined table");
        Ok(messages)
    }

    pub fn configure_table(
        &mut self,
        player: PlayerId,
        starting_stack: u64,
        big_blind: u64,
        small_blind: u64,
    ) -> Result<(), TableError> {
        if !self.player_id_to_table_map.contains_key(&player) {
            return Err(TableError::NotInTable);
        }
        // Rejecting an oversized stack here keeps the assert in PokerGame::new
        // unreachable from the network.
        if big_blind < small_blind
            || big_blind > starting_stack
            || starting_stack > poker_core::MAX_STACK
        {
            return Err(TableError::InvalidTableConfig);
        }

        let table = self.player_id_to_table_map[&player];
        let curr_table = self.tables.get_mut(&table).unwrap();
        if curr_table.in_game() {
            return Err(TableError::GameInProgress);
        }

        let previous_stack = curr_table.config.starting_stack;
        curr_table.config = TableConfig {
            starting_stack,
            big_blind,
            small_blind,
        };

        // Before any hand is dealt everyone is still sitting on exactly the old
        // starting stack, so re-seat them at the new one — otherwise you would
        // configure a 2000 table and then play it for 1000. Once chips have
        // actually moved, a reconfigure only affects later joins and rebuys.
        let reseated: Vec<PlayerId> = if curr_table.hands_played == 0 {
            for seated in &curr_table.players {
                curr_table.stacks.insert(*seated, starting_stack);
            }
            curr_table.players.clone()
        } else {
            vec![]
        };

        for seated in reseated {
            self.adjust_buy_in(seated, previous_stack, starting_stack);
        }

        debug!(
            ?player,
            ?table,
            starting_stack,
            big_blind,
            small_blind,
            "Configured table"
        );
        Ok(())
    }

    pub fn start_game(&mut self, player: PlayerId) -> Result<(), TableError> {
        if !self.player_id_to_table_map.contains_key(&player) {
            return Err(TableError::NotInTable);
        }
        let table = self.player_id_to_table_map[&player];
        let curr_table = self.tables.get_mut(&table).unwrap();

        if curr_table.in_game() {
            return Err(TableError::GameInProgress);
        }
        if curr_table.players.len() < poker_core::MIN_PLAYERS {
            return Err(TableError::TooFewPlayers);
        }
        if curr_table.players.len() > poker_core::MAX_PLAYERS {
            return Err(TableError::TooManyPlayers);
        }

        let starting_stack = curr_table.config.starting_stack;
        let big_blind = curr_table.config.big_blind;
        let small_blind = curr_table.config.small_blind;

        // Anyone who can no longer cover a big blind is topped back up before
        // the cards come out. A rebuy is not a PnL event: it hands over chips
        // and an equal debt, which is why it only touches the buy-in total.
        let mut rebuys: Vec<(PlayerId, u64)> = Vec::new();
        for seated in &curr_table.players {
            let stack = curr_table.stacks.entry(*seated).or_insert(starting_stack);
            if *stack < big_blind {
                rebuys.push((*seated, starting_stack.saturating_sub(*stack)));
                *stack = starting_stack;
            }
        }

        let seats: Vec<(PlayerId, u64)> = curr_table
            .players
            .iter()
            .map(|seated| (*seated, curr_table.stacks[seated]))
            .collect();
        let num_players = seats.len();

        curr_table.state = TableState::Game(PokerGame::new(&seats, small_blind, big_blind));

        for (rebought, amount) in rebuys {
            debug!(player = ?rebought, ?table, amount, "Topped up a busted stack");
            self.record_buy_in(rebought, amount);
        }

        debug!(?player, ?table, num_players, "Started game");
        Ok(())
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

        match &mut curr_table.state {
            TableState::Lobby => Err(ActionError::Table(TableError::GameNotStarted)),
            TableState::Game(game) => {
                let game_over = game.action(player, action).map_err(ActionError::Rule)?;
                debug!(
                    ?player,
                    ?table,
                    ?action,
                    game_over,
                    "Player action accepted"
                );
                Ok(game_over)
            }
        }
    }

    pub fn notify_table(&self, table: TableId) -> Vec<(Connection, ServerMessage)> {
        let curr_table = self.tables.get(&table).unwrap();

        let TableState::Game(game) = &curr_table.state else {
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
                debug!(?player_id, ?table, "It's this player's turn");
                output.push((conn.clone(), ServerMessage::ItsYourTurn));
                //net::push(conn, &ServerMessage::ItsYourTurn).await?;
            }
        }

        output
    }

    pub fn handle_disconnect(&mut self, conn: &Connection) -> Vec<(Connection, ServerMessage)> {
        let Some(player_id) = self.players.remove(&conn.stable_id()) else {
            // Player never registered
            debug!(
                stable_id = conn.stable_id(),
                "Disconnect from unregistered connection"
            );
            return vec![];
        };
        self.connections.remove(&player_id);
        debug!(?player_id, "Player disconnected");

        let Some(table_id) = self.player_id_to_table_map.remove(&player_id) else {
            // Player not in table
            debug!(?player_id, "Disconnected player was not in a table");
            return vec![];
        };

        let mut messages = Vec::new();
        let mut cashed_out: Option<u64> = None;

        let curr_table = self.tables.get_mut(&table_id).unwrap();
        match &mut curr_table.state {
            TableState::Lobby => {
                // Remove player from lobby and do nothing else. Their chips
                // leave with them; their account keeps the running PnL.
                curr_table.players.retain(|p| *p != player_id);
                cashed_out = curr_table.stacks.remove(&player_id);
                debug!(
                    ?player_id,
                    ?table_id,
                    "Removed disconnected player from lobby"
                );

                for other_player in &curr_table.players {
                    let conn = &self.connections[other_player];
                    messages.push((conn.clone(), ServerMessage::PlayerDisconnected(player_id)));
                }
            }
            TableState::Game(game) => {
                let game_over = game.fold_disconnected(player_id);
                debug!(
                    ?player_id,
                    ?table_id,
                    game_over,
                    "Folded disconnected player"
                );

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

        if let Some(stack) = cashed_out {
            self.record_cash_out(player_id, stack);
        }

        // If a hand this player is part of is still in flight, keep the account
        // link alive: their chips are in that pot, and dropping the link now
        // would leave that loss unrecorded and the hand's ledger entries no
        // longer summing to zero. `end_game` releases it once the hand settles.
        // The `Account` itself always stays — that is what persists the PnL.
        let still_in_hand = self
            .tables
            .get(&table_id)
            .is_some_and(|curr_table| curr_table.in_game());
        if !still_in_hand {
            self.player_to_account.remove(&player_id);
        }

        messages
    }

    /// Cumulative PnL for everyone at a table. Driven off the roster rather than
    /// off the ledger's keys, so a player who has just sat down shows 0 instead
    /// of being missing.
    pub fn standings_for(&self, table: TableId) -> Standings {
        let curr_table = self.tables.get(&table).unwrap();

        // Mid-hand the authoritative stacks are the game's, not the table's
        // carry-over copy, so a player sees chips actually behind them.
        let (roster, stacks): (Vec<PlayerId>, HashMap<PlayerId, u64>) = match &curr_table.state {
            TableState::Lobby => (curr_table.players.clone(), curr_table.stacks.clone()),
            TableState::Game(game) => (game.get_player_ids(), game.stacks().into_iter().collect()),
        };

        let mut entries: Vec<StandingsEntry> = roster
            .into_iter()
            .filter(|player| self.connections.contains_key(player))
            .map(|player| StandingsEntry {
                name: self.account_name(player),
                id: player,
                pnl: self.account_pnl(player),
                stack: stacks.get(&player).copied().unwrap_or(0),
            })
            .collect();

        // Best first; name breaks ties so the board doesn't shuffle around.
        entries.sort_by(|a, b| b.pnl.cmp(&a.pnl).then(a.name.cmp(&b.name)));

        Standings { table, entries }
    }

    pub fn end_game(&mut self, table: TableId) -> Vec<(Connection, ServerMessage)> {
        // Showdown: pay the pots into stacks, fold the hand into the ledger,
        // then drop back to the lobby with chips and settings carried over.
        let (showdown_result, final_stacks, players) = {
            let curr_table = self.tables.get_mut(&table).unwrap();
            let TableState::Game(game) = &mut curr_table.state else {
                panic!("Cannot notify participants of a game that's not ongoing!");
            };

            let showdown_result = game.get_showdown_results();
            game.settle(&showdown_result);
            let final_stacks: HashMap<PlayerId, u64> = game.stacks().into_iter().collect();
            (showdown_result, final_stacks, game.get_player_ids())
        };

        debug!(
            ?table,
            num_players = players.len(),
            pot = showdown_result.total_pot(),
            "Hand finished, settling and resetting to lobby"
        );

        // Fold this hand into each player's running total. Accounts are keyed on
        // name and outlive the connection, so someone who dropped mid-hand still
        // has it recorded against them.
        let mut logged = Vec::with_capacity(players.len());
        for &(player_id, delta) in showdown_result.player_pnl() {
            let Some(name) = self.player_to_account.get(&player_id).cloned() else {
                continue;
            };
            let account = self.accounts.entry(name.clone()).or_default();
            account.pnl = account.pnl.saturating_add(delta);
            account.hands_played += 1;
            logged.push(HandLogPlayer {
                name,
                pnl: delta,
                stack_after: final_stacks.get(&player_id).copied().unwrap_or(0),
            });
        }

        let survivors: Vec<PlayerId> = players
            .iter()
            .copied()
            .filter(|player| self.connections.contains_key(player))
            .collect();

        // Anyone who dropped during the hand has now had it settled against
        // their ledger, so their chips leave the table and the account link
        // `handle_disconnect` deliberately held open can finally be released.
        for player_id in &players {
            if survivors.contains(player_id) {
                continue;
            }
            self.record_cash_out(
                *player_id,
                final_stacks.get(player_id).copied().unwrap_or(0),
            );
            self.player_to_account.remove(player_id);
        }

        // Persist only once the ledger is fully settled for this hand, cash-outs
        // included. One append and one rewrite per hand, both small. This runs
        // under the state mutex, which is fine at this cadence but is why it
        // stays out of the per-message path.
        store::append_hand(table.0, showdown_result.total_pot(), &logged);
        store::save_accounts(&self.accounts);

        {
            let curr_table = self.tables.get_mut(&table).unwrap();
            curr_table.stacks = survivors
                .iter()
                .map(|player| (*player, final_stacks.get(player).copied().unwrap_or(0)))
                .collect();
            curr_table.players = survivors;
            if !curr_table.players.is_empty() {
                // Move the button. Without this, seat 1 posts the small blind
                // every hand and the PnL board would mostly measure seating.
                curr_table.players.rotate_left(1);
            }
            curr_table.hands_played += 1;
            curr_table.state = TableState::Lobby;
        }

        let standings = self.standings_for(table);

        let mut output: Vec<(Connection, ServerMessage)> = Vec::with_capacity(players.len() * 2);
        for player_id in &players {
            // Skip DC'd players
            let Some(conn) = self.connections.get(player_id) else {
                continue;
            };
            output.push((
                conn.clone(),
                ServerMessage::GameOver(showdown_result.clone()),
            ));
            output.push((conn.clone(), ServerMessage::Standings(standings.clone())));
        }

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

    let accounts = store::load_accounts();
    info!(
        accounts = accounts.len(),
        data_dir = %*constants::DATA_DIR,
        "Loaded PnL ledger"
    );

    let state = Arc::new(Mutex::new(ServerState::new(accounts)));

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
                    ClientMessage::Hello(name) => {
                        let mut state = state.lock().await;
                        match state.new_player(&connection, &name) {
                            Ok(player_id) => ServerMessage::Welcome(player_id),
                            // Already signed in, or this name is in use elsewhere
                            Err(TableError::PlayerAlreadyInTable) => {
                                ServerMessage::AlreadyConnected
                            }
                            Err(err) => ServerMessage::TableJoinFailed(err),
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
                    ClientMessage::ConfigureTable(starting_stack, big_blind, small_blind) => {
                        let mut state = state.lock().await;
                        match state.lookup_player(&connection) {
                            Some(player_id) => {
                                match state.configure_table(
                                    player_id,
                                    starting_stack,
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
                    ClientMessage::GetStandings => {
                        // Read-only, so no deferred-I/O dance is needed.
                        let state = state.lock().await;
                        match state.lookup_player(&connection) {
                            Some(player_id) => match state.player_id_to_table_map.get(&player_id) {
                                Some(&table_id) => {
                                    ServerMessage::Standings(state.standings_for(table_id))
                                }
                                None => ServerMessage::StandingsFailed(TableError::NotInTable),
                            },
                            None => ServerMessage::StandingsFailed(TableError::PlayerNotFound),
                        }
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
