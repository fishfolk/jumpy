use crate::async_main::async_main_impl;
use crate::event::setup_events_impl;
use crate::resources::{derive_resource_impl, setup_resources_impl};

mod async_main;
mod event;
mod resources;
mod util;

pub(crate) const CORE_CRATE_NAME: &str = "fishfight_core";

#[proc_macro_attribute]
pub fn async_main(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    async_main_impl(args, input)
}

#[proc_macro_derive(Resource, attributes(resource))]
pub fn derive_resource(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_resource_impl(input)
}

/// This can be used to setup resources if not using the `async_main` attribute macro
#[proc_macro]
pub fn setup_resources(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    setup_resources_impl(input)
}

/// This can be used to setup event handling if not using the `async_main` attribute macro
#[proc_macro]
pub fn setup_events(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    setup_events_impl(input)
}
