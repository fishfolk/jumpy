use clap::Parser;
use tracing::metadata::LevelFilter;

use crate::EXE;

pub fn start() {
    configure_logging();

    let args = crate::Config::parse();

    futures_lite::future::block_on(EXE.run(async move {
        if let Err(e) = super::server(args).await {
            eprintln!("Error: {e}");
        }
    }));
}

fn configure_logging() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .finish(),
    )
    .unwrap();
}
