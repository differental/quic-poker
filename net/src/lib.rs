use quinn::{Connection, Endpoint, SendStream};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use protocol::{ClientMessage, ServerMessage};

mod cert;
mod error;

pub use cert::configure_server;

#[cfg(feature = "dev")]
pub use cert::dev::configure_client;

#[cfg(not(feature = "dev"))]
pub use cert::prod::configure_client;

use error::NetError;

pub fn make_server_endpoint(
    addr: SocketAddr,
    cert: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
) -> Result<Endpoint, NetError> {
    let config = configure_server(cert, key)?;
    Ok(Endpoint::server(config, addr)?)
}

pub fn make_client_endpoint() -> Result<Endpoint, NetError> {
    let client_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
    let mut endpoint = Endpoint::client(client_addr)?;
    endpoint.set_default_client_config(configure_client()?);
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
    let (mut send, mut recv) = conn.open_bi().await?;
    let encoded = protocol::encode(msg);
    let encoded_bytes = encoded.into_bytes();

    send.write_all(&encoded_bytes).await?;
    send.finish().unwrap();

    let received = recv.read_to_end(5120).await?;
    let received_str = String::from_utf8(received).unwrap();

    Ok(protocol::decode(&received_str))
}

pub async fn reply(mut send: SendStream, msg: &ServerMessage) -> Result<(), NetError> {
    let encoded = protocol::encode(msg);
    let encoded_bytes = encoded.into_bytes();

    send.write_all(&encoded_bytes).await?;
    send.finish().unwrap();

    Ok(())
}
