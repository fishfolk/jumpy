use blocking::unblock;
use jumpy_matchmaker_proto::MatchInfo;
use quinn::Connection;

pub async fn start_game_server(match_info: MatchInfo, clients: Vec<Connection>) {
    info!(?match_info, "Starting match");
    let client_ids = clients.iter().map(|x| x.stable_id()).collect::<Vec<_>>();

    if let Err(e) = impl_game_server(match_info.clone(), clients).await {
        error!(?match_info, ?client_ids, "Error running match: {e}");
    }
}

async fn impl_game_server(match_info: MatchInfo, clients: Vec<Connection>) -> anyhow::Result<()> {
    unblock(|| {
        jumpy::build_app(clients).run();
    })
    .await;

    info!(?match_info, "Game server finished");

    Ok(())
}
