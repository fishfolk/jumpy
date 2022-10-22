use dashmap::DashMap;
use futures_lite::future;
use jumpy_matchmaker_proto::matchmaker::{MatchInfo, MatchmakerRequest, MatchmakerResponse};
use once_cell::sync::Lazy;
use quinn::{Connection, ConnectionError, SendStream};
use ulid::Ulid;

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

/// The matchmaker state
#[derive(Default)]
struct State {
    /// The mapping of match info to the vector connected clients in the waiting room.
    rooms: DashMap<MatchInfo, Vec<(Connection, SendStream)>>,
}

static STATE: Lazy<State> = Lazy::new(State::default);

/// After a matchmaker connection is established, it will open a bi-directional channel with the
/// client.
///
/// At this point the client is free to engage in the matchmaking protocol over that channel.
async fn impl_matchmaker(conn: Connection) -> anyhow::Result<()> {
    let connection_id = conn.stable_id();

    loop {
        // Get the next channel open or connection close event
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
                let (send, recv) = bi?;

                // Parse matchmaker request
                let request: MatchmakerRequest =
                    postcard::from_bytes(&recv.read_to_end(256).await?)?;

                info!(connection_id, ?request, "Got matchmaker request");

                match request {
                    MatchmakerRequest::RequestMatch(match_info) => {
                        debug!(connection_id, ?match_info, "Request for match");

                        let player_count = match_info.player_count;
                        let mut members = STATE.rooms.entry(match_info.clone()).or_default();

                        // Add the current client to the room
                        members.push((conn.clone(), send));

                        // Remove any dropped connections from the list
                        members.retain(|(conn, _send)| {
                            if let Some(reason) = conn.close_reason() {
                                let connection_id = conn.stable_id();
                                debug!(
                                    connection_id,
                                    "Removing disconnected client from room: {reason}"
                                );

                                false
                            } else {
                                true
                            }
                        });

                        // If we have a complete room
                        let member_count = members.len();
                        debug!(?match_info, "Room now has {}/{} members", member_count, player_count);
                        if member_count >= player_count as _ {
                            let match_id = Ulid::new();
                            debug!(%match_id, "Creating new match ID");

                            // Create a new match ID
                            let message = postcard::to_allocvec(&MatchmakerResponse::MatchId(match_id))?;

                            // Send the match ID to all of the clients in the room
                            for (_conn, mut send) in members.drain(..) {
                                send.write_all(&message).await?;
                                send.finish().await?;
                            }
                        }
                    }
                }
            }
        }
    }
}
