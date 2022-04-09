use crate::async_main::async_main_impl;
use crate::resources::{derive_resource_impl, init_resources_impl};

mod resources;
mod async_main;

pub(crate) const CRATE_NAME: &str = "fishfight_core";

#[proc_macro_attribute]
pub fn main(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    async_main_impl(args, input)
}

#[proc_macro_derive(Resource, attributes(resource))]
pub fn derive_resource(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_resource_impl(input)
}

#[proc_macro]
pub fn init_resources(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    init_resources_impl(input)
}