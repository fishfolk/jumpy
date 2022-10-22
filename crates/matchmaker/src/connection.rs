use anyhow::Context;
use jumpy_matchmaker_proto::ConnectionType;

use quinn::Connection;

use crate::EXE;

mod game;
mod matchmaker;

/// Handle a new connection
///
/// The first step after connecting to the matchmaker is to open a bidirectional channel and send a
/// message containing the [`ConnectionType`].
///
/// The the connection will be handed off to either the matchmaking server, or a new game
/// connection.
pub async fn handle_new_connection(conn: Connection) -> anyhow::Result<()> {
    let (mut send, recv) = conn
        .accept_bi()
        .await
        .context("Error opening initial connection channel")?;

    let message = recv.read_to_end(100).await?;

    let message: ConnectionType =
        postcard::from_bytes(&message).context("Deserialize connection type message")?;

    match message {
        ConnectionType::Matchmaker => {
            EXE.spawn(matchmaker::handle_matchmaker_conn(conn)).detach();
        }
        ConnectionType::Game => {
            warn!("Match connection not implemented yet");
        }
    }

    // Confirm accepted connection
    send.write_all(&[0]).await?;
    send.finish().await?;

    Ok(())
}
