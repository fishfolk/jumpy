use bevy_tasks::IoTaskPool;
use bytes::Bytes;
use futures::future::join_all;
use jumpy_matchmaker_proto::MatchInfo;
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

    for (i, conn) in clients.iter().enumerate() {
        let mut peers = clients.clone();
        peers.remove(i);

        let conn = conn.clone();

        task_pool
            .spawn(async move {
                let result = async {
                    loop {
                        let mut accept = conn.accept_uni().await?;

                        let mut peer_channels = Vec::with_capacity(peers.len());
                        for peer_conn in &peers {
                            match peer_conn.open_uni().await {
                                Ok(send) => peer_channels.push(send),
                                Err(e) => {
                                    warn!("Error opening stream to peer: {e:?}");
                                }
                            }
                        }

                        loop {
                            let mut buf = std::array::from_fn::<_, 32, _>(|_| Bytes::new());
                            if let Some(len) = accept.read_chunks(&mut buf).await? {
                                let data = buf[..len].to_vec();

                                let mut send_tasks = Vec::with_capacity(peers.len() * len);
                                for send in &mut peer_channels {
                                    let mut data = data.clone();
                                    send_tasks.push(async move {
                                        if let Err(e) = send.write_all_chunks(&mut data).await {
                                            warn!("Error sending data over channel: {e:?}");
                                        }
                                    });
                                }

                                join_all(send_tasks).await;
                            } else {
                                // Stream finished
                                break;
                            }
                        }
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
