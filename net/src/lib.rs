use quinn::{Connection, Endpoint, SendStream, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use protocol::{ClientMessage, ServerMessage};

pub mod cert;
mod error;

pub use cert::configure_server;

#[cfg(feature = "dev")]
pub use cert::dev::configure_client;

#[cfg(not(feature = "dev"))]
pub use cert::prod::configure_client;

use error::NetError;

/// Idle period after which QUIC tears down a silent connection. The effective
/// timeout is the minimum of what both peers advertise.
const MAX_IDLE_TIMEOUT: Duration = Duration::from_secs(180);
/// How often a peer sends PING frames to keep an otherwise-silent connection
/// alive. Kept well under half of [`MAX_IDLE_TIMEOUT`] so a dropped PING won't
/// trip the idle timeout. Players who sit thinking between actions would
/// otherwise be disconnected.
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(10);

/// Transport config shared by the server and client.
fn transport_config() -> TransportConfig {
    let mut transport = TransportConfig::default();
    transport
        .keep_alive_interval(Some(KEEP_ALIVE_INTERVAL))
        .max_idle_timeout(Some(
            MAX_IDLE_TIMEOUT
                .try_into()
                .expect("MAX_IDLE_TIMEOUT fits in a QUIC varint"),
        ));
    transport
}

pub fn make_server_endpoint(
    addr: SocketAddr,
    cert: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
) -> Result<Endpoint, NetError> {
    let mut config = configure_server(cert, key)?;
    config.transport_config(Arc::new(transport_config()));
    Ok(Endpoint::server(config, addr)?)
}

pub fn make_client_endpoint() -> Result<Endpoint, NetError> {
    let client_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
    let mut endpoint = Endpoint::client(client_addr)?;
    let mut client_config = configure_client()?;
    client_config.transport_config(Arc::new(transport_config()));
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

pub async fn connect_to_server(
    endpoint: &Endpoint,
    server: SocketAddr,
    server_name: &str,
) -> Result<Connection, NetError> {
    Ok(endpoint.connect(server, server_name)?.await?)
}

pub async fn request(conn: &Connection, msg: &ClientMessage) -> Result<ServerMessage, NetError> {
    // Client encodes a message and sends a request (e.g., join table, or poker action) to server, await server response, then decode the response.
    let (mut send, mut recv) = conn.open_bi().await?;
    let encoded = protocol::encode(msg);
    let encoded_bytes = encoded.into_bytes();

    send.write_all(&encoded_bytes).await?;
    send.finish().unwrap();

    let received = recv.read_to_end(5120).await?;
    let received_str = String::from_utf8(received).unwrap();

    Ok(protocol::decode(&received_str))
}

pub async fn reply(send: &mut SendStream, msg: &ServerMessage) -> Result<(), NetError> {
    // Server encodes a message and uses the end stream to send it to client.
    let encoded = protocol::encode(msg);
    let encoded_bytes = encoded.into_bytes();

    send.write_all(&encoded_bytes).await?;
    send.finish().unwrap();

    Ok(())
}

pub async fn push(conn: &Connection, msg: &ServerMessage) -> Result<(), NetError> {
    // Server pushes a unidirectional message to client.
    let mut send = conn.open_uni().await?;
    let encoded = protocol::encode(msg);
    let encoded_bytes = encoded.into_bytes();

    send.write_all(&encoded_bytes).await?;
    send.finish().unwrap();

    Ok(())
}

pub async fn receive_push(conn: &Connection) -> Result<ServerMessage, NetError> {
    // Client receives unidirectional pushes from server.
    let mut recv = conn.accept_uni().await?;

    let received = recv.read_to_end(5120).await?;
    let received_str = String::from_utf8(received).unwrap();

    Ok(protocol::decode(&received_str))
}
