use clap::Parser;
use tracing::metadata::LevelFilter;

pub fn start() {
    configure_logging();

    let args = crate::Config::parse();

    if let Err(e) = super::server(args) {
        eprintln!("Error: {e}");
    }
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
