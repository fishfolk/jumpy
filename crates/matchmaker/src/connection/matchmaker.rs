use futures_lite::future;
use jumpy_matchmaker_proto::matchmaker::{MatchmakerRequest, MatchmakerResponse};
use quinn::{Connection, ConnectionError};

pub async fn handle_matchmaker_conn(conn: Connection) {
    let connection_id = conn.stable_id();
    debug!(connection_id, "Accepted matchmaker connection");

    if let Err(e) = impl_matchmaker(conn).await {
        match e.downcast::<ConnectionError>() {
            Ok(conn_err) => match conn_err {
                ConnectionError::ApplicationClosed(e) => {
                    debug!(connection_id, "Application close connection: {e:?}");
                }
                e => {
                    error!(connection_id, "Error in matchmaker connection: {e:?}");
                }
            },
            Err(e) => {
                error!(connection_id, "Error in matchmaker connection: {e:?}");
            }
        }
    }
}

/// After a matchmaker connection is established, it will open a bi-directional channel with the
/// client.
///
/// At this point the client is free to engage in the matchmaking protocol over that channel.
async fn impl_matchmaker(conn: Connection) -> anyhow::Result<()> {
    let connection_id = conn.stable_id();

    loop {
        let event = future::or(async { either::Left(conn.closed().await) }, async {
            either::Right(conn.accept_bi().await)
        })
        .await;

        match event {
            either::Either::Left(close) => {
                debug!("Connection closed {close:?}");
                return Ok(());
            }
            either::Either::Right(bi) => {
                let (mut send, recv) = bi?;

                let request: MatchmakerRequest =
                    postcard::from_bytes(&recv.read_to_end(256).await?)?;

                info!(connection_id, ?request, "Got matchmaker request");

                let message = postcard::to_allocvec(&MatchmakerResponse::MatchId(7))?;
                send.write_all(&message).await?;
                send.finish().await?;
            }
        }
    }
}
