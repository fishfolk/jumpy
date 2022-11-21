use bevy_tasks::IoTaskPool;
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

        // Spawn a task for handling this client's connections
        task_pool
            .spawn(async move {
                let result = async {
                    loop {
                        // Wait for an incomming connection
                        let accept = conn.accept_uni().await?;

                        // Parse the message
                        let message = accept.read_to_end(1024).await?;
                        let message = postcard::from_bytes::<SendProxyMessage>(&message)?;
                        let target_client = message.target_client;
                        let message = message.message;

                        // Get the connection to the client that the message should be proxied to
                        let Some(target_client_conn) = peers.get(target_client as usize) else {
                            warn!("Tried to send message to non-existent client: {target_client}");
                            continue;
                        };

                        // Send the message to the target client
                        let mut send = target_client_conn.open_uni().await?;
                        let send_message = RecvProxyMessage {
                            from_client: i as u8,
                            message,
                        };
                        let send_message = postcard::to_allocvec(&send_message)?;
                        send.write_all(&send_message).await?;
                        send.finish().await?;
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
