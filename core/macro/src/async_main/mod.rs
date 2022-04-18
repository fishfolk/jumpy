use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, token, AttributeArgs, ItemFn, ItemMod, Path, Type};

use crate::async_main::internal::internal_main;
use crate::async_main::macroquad::macroquad_main;

pub(crate) mod internal;
pub(crate) mod macroquad;

use crate::event::custom_events;
use crate::resources::resource_loading;
use crate::util::prepend_crate;
use crate::CORE_CRATE_NAME;

const DEFAULT_WINDOW_TITLE: &str = "Game";
const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Clone, Copy, Eq, PartialEq, FromMeta)]
#[darling(default)]
pub enum Backend {
    Internal,
    Macroquad,
}

impl Default for Backend {
    fn default() -> Self {
        Self::Internal
    }
}

#[derive(Debug, FromMeta)]
struct MacroArgs {
    #[darling(default)]
    core_rename: Option<String>,
    #[darling(default)]
    backend: Backend,
    #[darling(default)]
    config_path_fn: Option<String>,
    #[darling(default)]
    window_title: Option<String>,
    #[darling(default)]
    window_icon_fn: Option<String>,
    #[darling(default)]
    event_type: Option<String>,
    #[darling(default)]
    resource_extension: Option<String>,
    #[darling(default)]
    custom_resources: Option<String>,
    #[darling(default)]
    error_type: Option<String>,
}

pub(crate) fn async_main_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut main_inner = parse_macro_input!(input as ItemFn);
    let attr_args = parse_macro_input!(args as AttributeArgs);

    let args: MacroArgs = match MacroArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(err) => {
            return proc_macro::TokenStream::from(err.write_errors());
        }
    };

    if main_inner.sig.asyncness.is_none() {
        panic!("main must be an async function!");
    }

    main_inner.sig.ident = Ident::from_string("main_inner").unwrap();

    let core_crate = Ident::from_string(&args.core_rename.unwrap_or(CORE_CRATE_NAME.to_string()))
        .unwrap_or_else(|err| {
            panic!(
                "{}::async_main: Error when building core_crate ident: {}",
                CORE_CRATE_NAME, err
            )
        });

    let core_impl = Ident::from_string(&format!("{}_impl", &core_crate)).unwrap_or_else(|err| {
        panic!(
            "{}::async_main: Error when building core_impl ident: {}",
            core_crate, err
        )
    });

    let config_path_fn = args.config_path_fn.map(|s| {
        Path::from_string(&prepend_crate(s)).unwrap_or_else(|err| {
            panic!(
                "{}::async_main: Error when building config_path_fn path: {}",
                core_crate, err
            )
        })
    });

    let window_title = args
        .window_title
        .unwrap_or_else(|| DEFAULT_WINDOW_TITLE.to_string());

    let window_icon_fn = args.window_icon_fn.map(|s| {
        Path::from_string(&prepend_crate(s)).unwrap_or_else(|err| {
            panic!(
                "{}::async_main: Error when building window_icon_fn path: {}",
                core_crate, err
            )
        })
    });

    let custom_resources = args
        .custom_resources
        .map(|mut s| {
            assert!(s.starts_with('[') && s.ends_with(']'), "{}::async_main: Error when building custom_resources: Expected `\"[Path1, Path2, ..]\"`", &core_crate);
            s.remove(s.len() - 1);
            s.remove(0);
            s.split(',')
                .into_iter()
                .map(|s| s.trim().to_string())
                .map(|s| Type::from_string(&prepend_crate(s))
                    .unwrap_or_else(|err| panic!(
                        "{}::async_main: Error when building custom_resources paths: {}",
                        core_crate, err
                    )))
                .collect::<Vec<_>>()
        }).unwrap_or_default();

    let error_type = Path::from_string(
        &args
            .error_type
            .map(|s| prepend_crate(s))
            .unwrap_or_else(|| format!("{}::error::Error", &core_crate)),
    )
    .unwrap_or_else(|err| {
        panic!(
            "{}::async_main: Error when building error_type path: {}",
            core_crate, err
        )
    });

    let mut context = TokenStream::new();

    let resources = TokenStream::from(resource_loading(
        &core_crate,
        args.resource_extension,
        &custom_resources,
    ));

    context.extend(resources);

    let config_path = if let Some(config_path_fn) = config_path_fn {
        quote! {
            pub use #config_path_fn as config_path;
        }
    } else {
        quote! {
            pub fn config_path() -> String {
                #DEFAULT_CONFIG_PATH.to_string()
            }
        }
    };

    context.extend(config_path);

    let window_icon = if let Some(window_icon_fn) = window_icon_fn {
        quote! { pub use #window_icon_fn as window_icon; }
    } else {
        quote! {
            pub fn window_icon() -> Option<#core_crate::window::WindowIcon> {
                None
            }
        }
    };

    context.extend(window_icon);

    let mut res = main_inner.to_token_stream();

    let mut context_use = quote! {
        pub use #core_impl::{load_resources, reload_resources};
    };

    let main_impl = match args.backend {
        Backend::Internal => {
            let event_type = Type::from_string(
                &args
                    .event_type
                    .map(|s| prepend_crate(s))
                    .unwrap_or_else(|| format!("{}::event::DefaultCustomEvent", core_crate)),
            )
            .unwrap_or_else(|err| {
                panic!(
                    "{}::async_main: Error when building event_type path: {}",
                    core_crate, err
                )
            });

            let events = TokenStream::from(custom_events(&core_crate, event_type));

            context.extend(events);

            let events_use = quote! {
                pub use #core_impl::{Event, new_event_loop, dispatch_event};
            };

            context_use.extend(events_use);

            internal_main(&core_crate, &core_impl, &error_type)
        }
        Backend::Macroquad => {
            println!("WARNING: {}::async_main: An event type was specified but this is not supported when using the Macroquad backend!", core_crate);

            macroquad_main(&core_crate, &core_impl, window_title, &error_type)
        }
    };

    let impl_module = quote! {
        pub mod #core_impl {
            #context
        }

        #context_use
    };

    res.extend(impl_module);

    res.extend(main_impl);

    proc_macro::TokenStream::from(res)
}
