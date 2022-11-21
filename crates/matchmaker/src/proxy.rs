use anyhow::format_err;
use bevy_tasks::IoTaskPool;
use bytes::Bytes;
use jumpy_matchmaker_proto::{MatchInfo, RecvProxyMessage, SendProxyMessage};
use quinn::Connection;

pub async fn start_proxy(match_info: MatchInfo, clients: Vec<Connection>) {
    info!(?match_info, "Starting match");
    let client_ids = clients.iter().map(|x| x.stable_id()).collect::<Vec<_>>();

    if let Err(e) = impl_proxy(match_info.clone(), clients).await {
        error!(?match_info, ?client_ids, "Error running match: {e}");
    }
}

async fn impl_proxy(match_info: MatchInfo, clients: Vec<Connection>) -> anyhow::Result<()> {
    let task_pool = IoTaskPool::get();

    // For each connected client
    for (i, conn) in clients.iter().enumerate() {
        // Get the client connection
        let conn = conn.clone();

        // And all the connections to it's peers
        let peers = clients.clone();

        // Spawn a task for handling this client's reliable connections
        let conn_ = conn.clone();
        let peers_ = peers.clone();
        task_pool
            .spawn(async move {
                let result = async {
                    loop {
                        // Wait for an incomming connection
                        let accept = conn_.accept_uni().await?;

                        // Parse the message
                        let message = accept.read_to_end(1024).await?;
                        let message = postcard::from_bytes::<SendProxyMessage>(&message)?;
                        let target_client = message.target_client;
                        let message = message.message;

                        let targets = match target_client {
                            jumpy_matchmaker_proto::TargetClient::All => peers_.clone(),
                            jumpy_matchmaker_proto::TargetClient::One(i) => vec![peers_
                                .get(i as usize)
                                .cloned()
                                .ok_or_else(|| format_err!("Invalid target client: {i}"))?],
                        };

                        // Send message to all target clients
                        let mut send_tasks = Vec::with_capacity(targets.len());
                        for target_client in targets {
                            let message_ = message.clone();
                            let task = task_pool.spawn(async move {
                                // Send the message to the target client
                                let mut send = target_client.open_uni().await?;
                                let send_message = RecvProxyMessage {
                                    from_client: i as u8,
                                    message: message_,
                                };
                                let send_message = postcard::to_allocvec(&send_message)?;
                                send.write_all(&send_message).await?;
                                send.finish().await?;

                                Ok::<_, anyhow::Error>(())
                            });

                            send_tasks.push(task);
                        }
                        futures::future::try_join_all(send_tasks).await?;
                    }

                    #[allow(unreachable_code)]
                    Ok::<_, anyhow::Error>(())
                };

                if let Err(e) = result.await {
                    warn!("Error in client connection: {e:?}");
                }
            })
            .detach();

        // Spawn task for handling the client's unreliable messages
        // TODO: De-duplicate this code a little?
        task_pool
            .spawn(async move {
                let result = async {
                    loop {
                        // Wait for an incomming connection
                        let bytes = conn.read_datagram().await?;

                        // Parse the message
                        let message = postcard::from_bytes::<SendProxyMessage>(&bytes)?;
                        let target_client = message.target_client;
                        let message = message.message;

                        let targets = match target_client {
                            jumpy_matchmaker_proto::TargetClient::All => peers.clone(),
                            jumpy_matchmaker_proto::TargetClient::One(i) => vec![peers
                                .get(i as usize)
                                .cloned()
                                .ok_or_else(|| format_err!("Invalid target client: {i}"))?],
                        };

                        // Send message to all target clients
                        let mut send_tasks = Vec::with_capacity(targets.len());
                        // Send the message to the target client
                        let send_message = RecvProxyMessage {
                            from_client: i as u8,
                            message,
                        };
                        let send_message = Bytes::from(postcard::to_allocvec(&send_message)?);
                        for target_client in targets {
                            let send_message = send_message.clone();
                            let task = task_pool.spawn(async move {
                                target_client.send_datagram(send_message)?;

                                Ok::<_, anyhow::Error>(())
                            });

                            send_tasks.push(task);
                        }
                        futures::future::try_join_all(send_tasks).await?;
                    }

                    #[allow(unreachable_code)]
                    Ok::<_, anyhow::Error>(())
                };

                if let Err(e) = result.await {
                    warn!("Error in client connection: {e:?}");
                }
            })
            .detach();
    }

    info!(?match_info, "Match finished");

    Ok(())
}
