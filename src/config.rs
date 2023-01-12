use once_cell::sync::Lazy;

pub const SERVER_MODE_ENV_VAR: &str = "JUMPY_SERVER_MODE";
pub const ASSET_DIR_ENV_VAR: &str = "JUMPY_ASSET_DIR";

const DEFAULT_LOG_LEVEL: &str = "info,wgpu=error,bevy_fluent=warn,symphonia_core=warn,symphonia_format_ogg=warn,symphonia_bundle_mp3=warn";

pub static ENGINE_CONFIG: Lazy<EngineConfig> = Lazy::new(|| {
    #[cfg(not(target_arch = "wasm32"))]
    return <EngineConfig as clap::Parser>::parse();

    #[cfg(target_arch = "wasm32")]
    return EngineConfig::from_web_params();
});

#[derive(Clone, Debug, clap::Parser)]
#[command(author, version, about)]
pub struct EngineConfig {
    /// Hot reload assets
    #[arg(short = 'R', long)]
    pub hot_reload: bool,

    /// Sets the sync test distance during local play.
    ///
    /// This is useful for developers for testing game rollback determinism without having to start
    /// a network game.
    #[arg(short = 'C', long, default_value = "0")]
    pub sync_test_check_distance: usize,

    /// The directory to load assets from
    #[arg(short, long, env = ASSET_DIR_ENV_VAR)]
    pub asset_dir: Option<String>,

    /// The .game.yaml asset to load at startup
    #[arg(default_value = "default.game.yaml")]
    pub game_asset: String,

    /// Set the log level
    ///
    /// May additionally specify log levels for specific modules as a comma-separated list of
    /// `module=level` items.
    #[arg(short = 'l', long, default_value = DEFAULT_LOG_LEVEL)]
    pub log_level: String,
}

impl EngineConfig {
    #[cfg(target_arch = "wasm32")]
    pub fn from_web_params() -> Self {
        if let Some(query) = web_sys::window().and_then(|w| w.location().search().ok()) {
            let mut config = Self::web_default();

            if let Some(asset_dir) = parse_url_query_string(&query, "asset_url") {
                config.asset_dir = Some(asset_dir.into());
            }

            if let Some(game_asset) = parse_url_query_string(&query, "game_asset") {
                config.game_asset = game_asset.into();
            }

            if let Some(log_level) = parse_url_query_string(&query, "log_level") {
                config.log_level = log_level.into();
            }

            config
        } else {
            Self::web_default()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn web_default() -> Self {
        // Note: It's unfortunate that we have to manually synchronize this with the defaults set
        // with structopt. If we find a way around that we should use it.
        Self {
            hot_reload: false,
            asset_dir: None,
            game_asset: "default.game.yaml".into(),
            log_level: DEFAULT_LOG_LEVEL.into(),
            sync_test_check_distance: 0,
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
/// Parse the query string as returned by `web_sys::window()?.location().search()?` and get a
/// specific key out of it.
pub fn parse_url_query_string<'a>(query: &'a str, search_key: &str) -> Option<&'a str> {
    let query_string = query.strip_prefix('?')?;

    for pair in query_string.split('&') {
        let (key, value) = if let Some(idx) = pair.find('=') {
            let key = &pair[0..idx];
            let value = &pair[(idx + 1)..];

            (key, value)
        } else {
            (pair, "")
        };

        if key == search_key {
            return Some(value);
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_url_query_string() {
        assert_eq!(
            Some("info"),
            parse_url_query_string("?RUST_LOG=info", "RUST_LOG")
        );
        assert_eq!(
            Some("debug"),
            parse_url_query_string("?RUST_LOG=debug&hello=world&foo=bar", "RUST_LOG")
        );
        assert_eq!(
            Some("debug,wgpu=warn"),
            parse_url_query_string("?RUST_LOG=debug,wgpu=warn&hello=world&foo=bar", "RUST_LOG")
        );
        assert_eq!(
            Some("trace"),
            parse_url_query_string("?hello=world&RUST_LOG=trace&foo=bar", "RUST_LOG")
        );
        assert_eq!(
            None,
            parse_url_query_string("?hello=world&foo=bar", "RUST_LOG")
        );
    }
}
