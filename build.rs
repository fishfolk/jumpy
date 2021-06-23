use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("nakama_credentials.rs");

    fs::write(
        &dest_path,
        &format!(
            "pub const NAKAMA_SERVER: &str = \"{}\";
             pub const NAKAMA_KEY: &str = \"{}\";
             pub const NAKAMA_PORT: u32 = {};
             pub const NAKAMA_PROTOCOL: &str = \"{}\";",
            option_env!("NAKAMA_SERVER").unwrap_or("/127.0.0.1"),
            option_env!("NAKAMA_KEY").unwrap_or("defaultkey"),
            option_env!("NAKAMA_PORT").unwrap_or(&format!("{}", 7350)),
            option_env!("NAKAMA_PROTOCOL").unwrap_or("http")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
