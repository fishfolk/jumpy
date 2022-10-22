use futures_lite::{future, AsyncWriteExt};
use jumpy_matchmaker_proto::matchmaker::{MatchmakerRequest, MatchmakerResponse};
use quinn::Connection;

pub async fn handle_matchmaker_conn(conn: Connection) {
    debug!("Accepted matchmaker connection");

    if let Err(e) = impl_matchmaker(conn).await {
        error!("Error in matchmaker connection: {e:?}");
    }
}

/// After a matchmaker connection is established, it will open a bi-directional channel with the
/// client.
///
/// At this point the client is free to engage in the matchmaking protocol over that channel.
async fn impl_matchmaker(conn: Connection) -> anyhow::Result<()> {
    loop {
        let (mut send, recv) = conn.accept_bi().await?;

        let request: MatchmakerRequest = postcard::from_bytes(&recv.read_to_end(256).await?)?;

        info!(?request, "Got matchmaker request");

        let message = postcard::to_allocvec(&MatchmakerResponse::MatchId(7))?;
        send.write_all(&message).await?;
        send.flush().await?; // This doesn't help for some reason

        // Must be called to write message
        send.finish().await?;

        info!("waiting...");
        info!(closed=?conn.closed().await);
    }
}
