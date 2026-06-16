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


## Connections and Messages 

The current implementation uses both uni-directional messages and bi-directional messages through QUIC. Bi-directional messages are always sent by the client, always represents an action (either an in-game action or some other action), with the server replying the result of the action afterwards. Uni-directional messages are always sent by the server, and always represents some form of notification. This generally includes a notification of a new game state (e.g., some other player made a move), or a notification to the user supposed to make the next move.

For more details on uni-directional messages and bi-directional messages through QUIC, see [the docs for quinn](https://quinn-rs.github.io/quinn/quinn/data-transfer.html).


