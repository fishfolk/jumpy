use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Block, bracketed, Ident, ExprBlock, ItemFn, Lit, LitBool, LitStr, parse_quote, Path, Token, token, Type, Stmt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Token};

struct InitResourcesArgs {
    crate_name: Ident,
    extension: String,
    custom_types: Vec<Type>,
}

impl Parse for InitResourcesArgs {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        if stream.is_empty() {
            panic!("No arguments!");
        } else {
            let crate_name: LitStr = stream.parse()?;
            stream.parse::<Token![,]>()?;

            let extension: LitStr = stream.parse()?;
            stream.parse::<Token![,]>()?;

            let types;
            bracketed!(types in stream);

            let custom_types: Punctuated<Type, Token![,]> = types.parse_terminated(Type::parse)?;

            Ok(InitResourcesArgs {
                crate_name: Ident::from_value(&Lit::Str(crate_name)).unwrap(),
                extension: extension.value(),
                custom_types: custom_types.into_iter().collect(),
            })
        }
    }
}

pub(crate) fn init_resources_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let signature = syn::parse_macro_input!(input as InitResourcesArgs);

    let crate_name = signature.crate_name;
    let extension = signature.extension;
    let custom_types = signature.custom_types;

    let mut function = {
        let function_tokens = quote! {
            async fn load_resources_from(path: &str, is_required: bool, should_overwrite: bool) -> #crate_name::Result<()> {
                Ok(())
            }
        };

        let stream = function_tokens.into();

        syn::parse_macro_input!(stream as ItemFn)
    };

    let mut stmts: Vec<Stmt> = vec![
        parse_quote! { use #crate_name::resources::{ResourceVec, ResourceMap}; },
        parse_quote! { let path = path.to_string(); },
        parse_quote! { #crate_name::resources::load_particle_effects(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_audio(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_textures(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_decoration(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_maps(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_images(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_fonts(&path, #extension, is_required, should_overwrite).await?; },
    ];

    for custom_type in custom_types {
        stmts.push(parse_quote! { #custom_type::load(&path, #extension, is_required, should_overwrite).await?; });
    }

    for (i, stmt) in stmts.into_iter().enumerate() {
        function.block.stmts.insert(i, stmt);
    }

    let mut res = function.to_token_stream();

    let mod_tokens = quote! {
        pub async fn load_resources<P: AsRef<std::path::Path>>(assets_dir: P, mods_dir: P) -> #crate_name::Result<()> {
            #crate_name::resources::set_assets_dir(assets_dir);

            let assets_dir = #crate_name::resources::assets_dir().to_string();

            load_resources_from(&assets_dir, true, true).await?;

            #crate_name::resources::set_mods_dir(&mods_dir);

            let mut iter = #crate_name::resources::ModLoadingIterator::new(mods_dir, #extension).await?;
            while let Some((mod_path, _meta)) = iter.next().await? {
                load_resources_from(&mod_path, false, false).await?;
            }

            Ok(())
        }
    };

    res.extend(mod_tokens);

    proc_macro::TokenStream::from(res)
}