use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use protocol::ClientMessage;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let endpoint = net::make_client_endpoint()?;
    let server_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 5000));
    let connection = net::connect_to_server(&endpoint, server_addr, "localhost").await?;

    loop {
        println!("Input command: ");

        let msg = ClientMessage
    }

    Ok(())
}
