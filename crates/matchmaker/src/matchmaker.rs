use futures_lite::future;
use jumpy_matchmaker_proto::{MatchInfo, MatchmakerRequest, MatchmakerResponse};
use once_cell::sync::Lazy;
use quinn::{Connection, ConnectionError};
use scc::HashMap;

use crate::game_server::start_game_server;

pub async fn handle_connection(conn: Connection) {
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
    rooms: HashMap<MatchInfo, Vec<Connection>>,
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
                let (mut send, recv) = bi?;

                // Parse matchmaker request
                let request: MatchmakerRequest =
                    postcard::from_bytes(&recv.read_to_end(256).await?)?;

                match request {
                    MatchmakerRequest::RequestMatch(match_info) => {
                        debug!(connection_id, ?match_info, "Got request for match");

                        // Accept request
                        let message = postcard::to_allocvec(&MatchmakerResponse::Accepted)?;
                        send.write_all(&message).await?;
                        send.finish().await?;

                        let player_count = match_info.player_count;

                        let mut members_to_join = Vec::new();
                        let mut members_to_notify = Vec::new();

                        // Make sure room exists
                        STATE
                            .rooms
                            .insert_async(match_info.clone(), Vec::new())
                            .await
                            .ok();

                        STATE
                            .rooms
                            .update_async(&match_info, |match_info, members| {
                                debug!("Testing");
                                // Add the current client to the room
                                members.push(conn.clone());

                                // Remove any dropped connections from the list
                                members.retain(|conn| {
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

                                let member_count = members.len();

                                // If we have a complete room
                                debug!(
                                    ?match_info,
                                    "Room now has {}/{} members", member_count, player_count
                                );

                                if member_count >= player_count as _ {
                                    // Clear the room
                                    members_to_join.append(members);
                                } else {
                                    members_to_notify = members.clone();
                                }
                            })
                            .await;

                        if !members_to_notify.is_empty() {
                            let message = postcard::to_allocvec(&MatchmakerResponse::PlayerCount(
                                members_to_notify.len() as u8,
                            ))?;
                            let mut send = conn.open_uni().await?;
                            send.write_all(&message).await?;
                            send.finish().await?;
                        }

                        if !members_to_join.is_empty() {
                            // Respond with success
                            let message = postcard::to_allocvec(&MatchmakerResponse::Success)?;

                            // Send the match ID to all of the clients in the room
                            let mut clients = Vec::with_capacity(player_count as usize);
                            for conn in members_to_join.drain(..) {
                                let mut send = conn.open_uni().await?;
                                send.write_all(&message).await?;
                                send.finish().await?;

                                clients.push(conn);
                            }

                            // Hand the clients off to the game manager
                            start_game_server(match_info, clients).await;
                        }
                    }
                }
            }
        }
    }
}