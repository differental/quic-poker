# Quic-Poker

This is an implementation of a multi-player system used for simple games (e.g., poker, blackjack, number-based games) using a Rust implementation of QUIC (aka HTTP/3), `quinn`. QUIC is a UDP-based transport protocol designed to be more efficient than TCP, with security built in (so no extra TLS layer needed).


## Structure

The codebase is currently separated into five crates:

- `poker-core`: Core Poker logic and game loop
- `protocol`: `ClientMessage` and `ServerMessage` types (enums) which list all possible client (server) messages, along with a generic `encode` and `decode` helper function
- `net`: Network layer stuff, helpers to establish and read/write through bidirectional or unidirectional QUIC streams
- `server`: Server binary
- `client`: Client binary

Note that the Poker logic may differ slightly from conventional poker. More crates may be added to expand support to other sample games.

By default, to deploy an instance, a certificate from a trusted certificate authority (CA) for the server is required. This is most commonly done with Let's Encrypt using `certbot`. Alternatively, the server and client binaries accept a feature flag `dev` (default: off), which for the server would auto-generate a self-signed certificate on start-up, and for the client would disable certificate verification. Using the `dev` mode can be dangerous, as the connection is suspectible to a man-in-the-middle attack.


## Lobby Implementation

The current "lobby" implementation is quite simple. User can join a "table" which hasn't started a game yet, then configure the parameters of the game, before starting the game (after which the players must take turns to action). After each game finishes, all players are released to the "lobby" of the table.

Joining a table buys the player in for the table's starting stack. Chips carry over from one hand to the next, and the dealer button rotates by one seat after each hand so the blinds move around. A player who can no longer cover the big blind is automatically topped back up to the starting stack before the next hand is dealt.


## Chips and PnL

Every player has a finite chip stack. Betting is capped by what is actually behind them: calling with too few chips puts them all-in for less, and the pot is then split into a main pot and side pots so that an all-in player can only win up to what they matched. Odd chips that do not divide evenly among tied winners go to the seats nearest the button's left, the small blind first.

Each player has a running profit-and-loss figure, tracked per hand as what they were paid out of the pots minus what they committed. Per-hand PnL always sums to exactly zero, and appears next to each player in the showdown recap. Cumulative totals for everyone at your table are shown after each hand and can be requested at any time with the `standings` command.

PnL is kept per **account**, keyed by the name given to `hello <name>`. A `PlayerId` is minted per connection and restarts at 0 whenever the server does, so it cannot be used to key anything that has to outlive a session. Accounts persist across disconnects, table changes, and server restarts.

Two files are written under `DATA_DIR` (default `./data`):

- `accounts.json` — the ledger, rewritten after each hand via a temporary file and a rename, so a crash part-way through cannot truncate it
- `hands.jsonl` — one JSON object per settled hand, appended and never rewritten. This is the durable record the ledger can be rebuilt from; summing its `pnl` column per name reproduces each account's stored total.

A rebuy is deliberately not a PnL event: it hands over chips and an equal debt, which is what keeps `pnl == stack - total_bought_in` true for a seated player.


## Connections and Messages 

The current implementation uses both uni-directional messages and bi-directional messages through QUIC. Bi-directional messages are always sent by the client, always represents an action (either an in-game action or some other action), with the server replying the result of the action afterwards. Uni-directional messages are always sent by the server, and always represents some form of notification. This generally includes a notification of a new game state (e.g., some other player made a move), or a notification to the user supposed to make the next move.

For more details on uni-directional messages and bi-directional messages through QUIC, see [the docs for quinn](https://quinn-rs.github.io/quinn/quinn/data-transfer.html).


