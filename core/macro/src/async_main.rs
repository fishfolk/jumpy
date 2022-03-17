use darling::FromMeta;
use syn::{AttributeArgs, Expr, ExprCall, ItemFn, Ident, LitStr, parse_macro_input, Path};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use crate::CRATE_NAME;

const DEFAULT_WINDOW_TITLE: &str = "Game";

#[derive(Debug, Clone, Copy, FromMeta)]
#[darling(default)]
enum Backend {
    Macroquad,
    Internal,
}

impl Default for Backend {
    fn default() -> Self {
        Self::Internal
    }
}

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    crate_name: Option<String>,
    #[darling(default)]
    config_path_fn: Option<String>,
    #[darling(default)]
    window_icon_fn: Option<String>,
    #[darling(default)]
    window_title: Option<String>,
    #[darling(default)]
    backend: Backend,
}

pub(crate) fn async_main_impl(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let mut input = parse_macro_input!(input as ItemFn);

    let args: MacroArgs = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => { return proc_macro::TokenStream::from(e.write_errors()); }
    };

    let crate_name = Ident::from_string(&args.crate_name
        .unwrap_or(CRATE_NAME.to_string()))
        .unwrap();

    if input.sig.asyncness.is_none() {
        panic!("[{}::main] Must be an async function!", &crate_name);
    }

    input.sig.ident = Ident::from_string("inner_main").unwrap();

    let window_title = args.window_title.unwrap_or(DEFAULT_WINDOW_TITLE.to_string());

    let window_icon_fn = Path::from_string(&args.window_icon_fn
        .unwrap_or(format!("{}::window::default_window_icon", &crate_name)))
        .unwrap();

    let config_path_fn = Path::from_string(&format!("{}", args.config_path_fn
        .unwrap_or(format!("{}::config::default_config_path", &crate_name))))
        .unwrap();

    let mut res = input.into_token_stream();

    let prelude = match args.backend {
        Backend::Macroquad => {
            let macroquad_rename = format!("{}::macroquad", &crate_name);

            quote! {
                fn window_conf() -> #crate_name::macroquad::window::Conf {
                    let config = #crate_name::config::load_config_sync(#config_path_fn()).unwrap();
                    config.as_macroquad_window_conf(#window_title, #window_icon_fn(), true)
                }

                #[#crate_name::macroquad::main(window_conf)]
                #[macroquad(crate_rename = #macroquad_rename)]
                async fn main() -> #crate_name::Result<()> {
                    inner_main().await?;

                    Ok(())
                }
            }
        }
        Backend::Internal => {
            #[cfg(target_arch = "wasm32")]
            let prelude = quote! {
                #[#crate_name::wasm_bindgen]
                async fn main() -> #crate_name::Result<()> {
                    #crate_naem::config::load_config(#config_path_fn()).await?;

                    inner_main().await?;

                    Ok(())
                }
            };

            #[cfg(not(target_arch = "wasm32"))]
            let prelude = quote! {
                #[#crate_name::tokio::main(flavor = "current_thread")]
                async fn main() -> #crate_name::Result<()> {
                    #crate_name::config::load_config(#config_path_fn()).await?;

                    inner_main().await?;

                    Ok(())
                }
            };

            prelude
        }
    };

    res.extend(prelude);

    proc_macro::TokenStream::from(res)
}