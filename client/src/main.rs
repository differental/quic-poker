use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use poker_core::Action;
use protocol::{ClientMessage, ServerMessage, TableId};

/// Result of parsing a line of user input.
enum Command {
    Send(ClientMessage),
    Help,
    Quit,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let endpoint = net::make_client_endpoint()?;
    let server_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5000));
    let connection = net::connect_to_server(&endpoint, server_addr, "localhost").await?;

    println!("Connected to {server_addr}.");
    print_help();

    // Listen for server-initiated notifications (state updates, turn alerts) in
    // the background while the main loop drives request/response input.
    let notify_conn = connection.clone();
    tokio::spawn(async move {
        while let Ok(msg) = net::receive_push(&notify_conn).await {
            print_notification(&msg);
        }
    });

    loop {
        print!("> ");
        io::stdout().flush()?;

        // Read a line on the blocking pool so the notification task can keep
        // printing while we wait for input.
        let (read, line) = tokio::task::spawn_blocking(|| {
            let mut line = String::new();
            io::stdin().read_line(&mut line).map(|read| (read, line))
        })
        .await??;

        if read == 0 {
            // EOF (Ctrl-D)
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg = match parse_command(trimmed) {
            Ok(Command::Send(msg)) => msg,
            Ok(Command::Help) => {
                print_help();
                continue;
            }
            Ok(Command::Quit) => break,
            Err(err) => {
                println!("error: {err}");
                continue;
            }
        };

        let response = net::request(&connection, &msg).await?;
        // ServerMessage has no Debug/Display, so reuse the wire encoding to show it.
        println!("<- {}", protocol::encode(&response));
    }

    Ok(())
}

/// Displays a server-initiated notification (a state update or a turn alert),
/// then restores the input prompt the user was typing at.
fn print_notification(msg: &ServerMessage) {
    match msg {
        ServerMessage::StateUpdate(view) => println!("\n<- state update:\n{view}"),
        ServerMessage::ItsYourTurn => {
            println!("\n<- it's your turn! (fold | check | call | raise <to>)")
        }
        ServerMessage::GameOver(result) => println!("\n<- game over:\n{result}"),
        // Pushes are only ever state/turn/game-over notifications, but fall back
        // to the wire form for anything unexpected.
        other => println!("\n<- {}", protocol::encode(other)),
    }

    print!("> ");
    let _ = io::stdout().flush();
}

/// Parses a single line of input into a [`Command`].
fn parse_command(line: &str) -> Result<Command, String> {
    let mut parts = line.split_whitespace();
    let cmd = parts.next().expect("line is non-empty");

    let msg = match cmd.to_lowercase().as_str() {
        "help" | "?" => return Ok(Command::Help),
        "quit" | "exit" | "q" => return Ok(Command::Quit),

        "hello" => ClientMessage::Hello,
        "join" => ClientMessage::JoinTable(TableId(parse_arg(&mut parts, "table_id")?)),
        "configure" => ClientMessage::ConfigureTable(
            parse_arg(&mut parts, "table_max_bet")?,
            parse_arg(&mut parts, "big_blind")?,
            parse_arg(&mut parts, "small_blind")?,
        ),
        "start" => ClientMessage::StartGame,

        "fold" => ClientMessage::Action(Action::Fold),
        "check" => ClientMessage::Action(Action::Check),
        "call" => ClientMessage::Action(Action::Call),
        "raise" => ClientMessage::Action(Action::Raise {
            to: parse_arg(&mut parts, "to")?,
        }),

        other => return Err(format!("unknown command `{other}` (try `help`)")),
    };

    Ok(Command::Send(msg))
}

/// Pulls the next whitespace-separated token and parses it, naming the argument
/// in any error message.
fn parse_arg<'a, T>(parts: &mut impl Iterator<Item = &'a str>, name: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let raw = parts
        .next()
        .ok_or_else(|| format!("missing argument `{name}`"))?;
    raw.parse().map_err(|e| format!("invalid `{name}`: {e}"))
}

fn print_help() {
    println!(
        "\
Commands:
  hello                                  send Hello (register with the server)
  join <table_id>                        join a table
  configure <max_bet> <big> <small>      configure current table
  start                                  start the game
  fold | check | call                    take an action
  raise <to>                             raise to an amount
  help                                   show this help
  quit                                   exit"
    );
}
