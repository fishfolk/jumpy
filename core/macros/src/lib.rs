use proc_macro::TokenTree;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index};

const CRATE_NAME: &str = "fishfight_core";

const RESOURCE_ID_ATTR: &str = "resource_id";

const DEFAULT_RESOURCE_ID_IDENT: &str = "id";

#[proc_macro_derive(CustomResource, attributes(resource_id))]
pub fn derive_custom_resource(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let mut attr_ident = None;
    let mut default_ident = None;

    match input.data {
        syn::Data::Struct(ref data_struct) => {
            match data_struct.fields {
                syn::Fields::Named(ref fields_named) => {
                    'outer: for field in fields_named.named.iter() {
                        let ident = field.ident.as_ref().unwrap();

                        if format!("{:?}", ident) == DEFAULT_RESOURCE_ID_IDENT {
                            default_ident = Some(ident.clone());
                        }

                        for attr in field.attrs.iter() {
                            match attr.parse_meta().unwrap() {
                                syn::Meta::Path(ref path) => {
                                    let path_str = path
                                        .get_ident()
                                        .unwrap()
                                        .to_string();

                                    if path_str == RESOURCE_ID_ATTR {
                                        attr_ident = Some(ident.clone());

                                        break 'outer;
                                    }
                                },
                                _ => {},
                            }
                        }
                    }
                }
                _ => panic!("Must be a struct with named fields"),
            }
        }
        _ => panic!("Must be a struct"),
    }

    let ident = attr_ident
        .unwrap_or_else(|| default_ident
            .unwrap_or_else(|| panic!(
                "Unable to determine what field to use as the resources id (use the '{}' if there is no field named '{}')",
                RESOURCE_ID_ATTR,
                DEFAULT_RESOURCE_ID_IDENT,
            )));

    let expanded = quote! {
        impl CustomResource for #name {
            fn id(&self) -> String {
                self.#ident.to_string()
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}