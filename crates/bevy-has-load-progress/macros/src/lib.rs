use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned};

/// Derive macro for `HasLoadProgress`
///
/// May be used to implement `HasLoadProgress` on structs where all fields implement
/// `HasLoadProgress`.
///
/// Fields not implementing `HasLoadProgress` may be skipped with `#[has_load_progress(none)]` added
/// to the field.
///
/// `#[has_load_progress(none)]` may also be added to the struct itself to use the default
/// implementation of `HasLoadProgress` which returns no progress and nothing to load.
///
/// `#[has_load_progress(none)]` could also be added to enums to derive the default implementation
/// with no load progress, but cannot be added to enum variants.
#[proc_macro_derive(HasLoadProgress, attributes(has_load_progress))]
pub fn has_load_progress(input: TokenStream) -> TokenStream {
    let input = syn::parse(input).unwrap();

    impl_has_load_progress(&input).into()
}

fn impl_has_load_progress(input: &syn::DeriveInput) -> TokenStream2 {
    // The attribute that may be added to skip fields or use the default implementation.
    let no_load_attr: syn::Attribute = parse_quote! {
        #[has_load_progress(none)]
    };

    let item_ident = &input.ident;
    let mut impl_function_body = quote! {};

    // Check for `#[has_load_progress(none)]` on the item itself
    let mut skip_all_fields = false;
    for attr in &input.attrs {
        if attr.path == parse_quote!(has_load_progress) {
            if attr == &no_load_attr {
                skip_all_fields = true;
            } else {
                return quote_spanned!(attr.span() =>
                    compile_error!("Attribute must be `#[has_load_progress(none)]` if specified");
                );
            }
        }
    }

    // If we are skipping all fields
    if skip_all_fields {
        impl_function_body = quote! {
            bevy_has_load_progress::LoadProgress::default()
        };

    // If we should process struct fields
    } else {
        // Parse the struct
        let in_struct = match &input.data {
            syn::Data::Struct(s) => s,
            syn::Data::Enum(_) | syn::Data::Union(_) => {
                return quote_spanned! { input.ident.span() =>
                    compile_error!("You may only derive HasLoadProgress on structs");
                };
            }
        };

        // Start a list of the progresses for each field
        let mut progresses = Vec::new();
        'field: for field in &in_struct.fields {
            // Skip this field if it has `#[has_load_progress(none)]`
            for attr in &field.attrs {
                if attr.path == parse_quote!(has_load_progress) {
                    if attr == &no_load_attr {
                        continue 'field;
                    } else {
                        impl_function_body = quote_spanned! { attr.span() =>
                            compile_error!("Attribute be `#[has_load_progress(none)]` if specified");
                        }
                    }
                }
            }

            // Add this fields load progress to the list of progresses
            let field_ident = field.ident.as_ref().expect("Field ident");
            progresses.push(quote_spanned! { field_ident.span() =>
                bevy_has_load_progress::HasLoadProgress::load_progress(
                    &self.#field_ident,
                    loading_resources
                )
            })
        }

        // Retrun the merged progress result
        impl_function_body = quote! {
            #impl_function_body
            bevy_has_load_progress::LoadProgress::merged([ #( #progresses),* ])
        };
    }

    // Fill out rest of impl block
    quote! {
        impl bevy_has_load_progress::HasLoadProgress for #item_ident {
            fn load_progress(
                &self,
                loading_resources: &bevy_has_load_progress::LoadingResources
            ) -> bevy_has_load_progress::LoadProgress {
                #impl_function_body
            }
        }
    }
}
