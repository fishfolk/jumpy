use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_quote, token, Block, ExprBlock, Ident, ItemFn, Lit, LitBool, LitStr, Path,
    Stmt, Token, Type,
};

use crate::resources::DEFAULT_EXTENSION;

struct Signature {
    core_crate: Ident,
    extension: String,
    types: Vec<Type>,
}

struct Syntax {
    core_crate: LitStr,
    _comma_1: Token![,],
    extension: LitStr,
    _comma_2: Token![,],
    _bracket: token::Bracket,
    types: Punctuated<Type, Token![,]>,
}

impl Parse for Signature {
    fn parse(stream: ParseStream) -> syn::Result<Self> {
        if stream.is_empty() {
            panic!("The init_resources macro requires three arguments!");
        } else {
            let content;
            let syntax = Syntax {
                core_crate: stream.parse()?,
                _comma_1: stream.parse()?,
                extension: stream.parse()?,
                _comma_2: stream.parse()?,
                _bracket: bracketed!(content in stream),
                types: content.parse_terminated(Type::parse)?,
            };

            Ok(Signature {
                core_crate: Ident::from_value(&Lit::Str(syntax.core_crate)).unwrap(),
                extension: syntax.extension.value(),
                types: syntax.types.into_iter().collect(),
            })
        }
    }
}

pub(crate) fn setup_resources_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let signature = syn::parse_macro_input!(input as Signature);

    let core_crate = signature.core_crate;
    let extension = signature.extension;
    let custom_resources = signature.types;

    resource_loading(&core_crate, Some(extension), &custom_resources)
}

pub(crate) fn resource_loading(
    core_crate: &Ident,
    extension: Option<String>,
    custom_resources: &[Type],
) -> proc_macro::TokenStream {
    let mut extension = extension.unwrap_or_else(|| DEFAULT_EXTENSION.to_string());

    if extension.starts_with('.') {
        extension.remove(0);
    }

    let mut load_resources_from = {
        let tokens = proc_macro::TokenStream::from(quote! {
            async fn load_resources_from<P: AsRef<std::path::Path>>(path: P, is_required: bool, should_overwrite: bool) -> #core_crate::result::Result<()> {
                Ok(())
            }
        });

        syn::parse_macro_input!(tokens as ItemFn)
    };

    let mut stmts: Vec<Stmt> = vec![
        parse_quote! { let path = path.as_ref(); },
        parse_quote! { #core_crate::particles::load_particle_effects(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #core_crate::audio::load_audio(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #core_crate::texture::load_textures(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #core_crate::map::load_decoration(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #core_crate::map::load_maps(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #core_crate::image::load_images(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #core_crate::text::load_fonts(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { use #core_crate::resources::{ResourceVec, ResourceMap}; },
        parse_quote! { let path = path.to_str().unwrap().to_string(); },
    ];

    for custom_type in custom_resources {
        stmts.push(parse_quote! { #custom_type::load(&path, #extension, is_required, should_overwrite).await?; });
    }

    for (i, stmt) in stmts.into_iter().enumerate() {
        load_resources_from.block.stmts.insert(i, stmt);
    }

    let mut res = load_resources_from.to_token_stream();

    let load_mods_from = quote! {
        async fn load_mods_from<P: AsRef<std::path::Path>>(path: P) -> #core_crate::result::Result<()> {
            let mut iter = #core_crate::resources::ModLoadingIterator::new(path, #extension).await?;
            while let Some((mod_path, _meta)) = iter.next().await? {
                load_resources_from(&mod_path, false, false).await?;
            }

            Ok(())
        }
    };

    res.extend(load_mods_from);

    let load = quote! {
        pub async fn load_resources() -> #core_crate::result::Result<()> {
            let assets_dir = #core_crate::resources::assets_dir();
            let mods_dir = #core_crate::resources::mods_dir();

            #core_crate::resources::set_assets_dir(&assets_dir);
            #core_crate::resources::set_mods_dir(&mods_dir);

            load_resources_from(&assets_dir, true, true).await?;
            load_mods_from(&mods_dir).await?;

            Ok(())
        }
    };

    res.extend(load);

    let reload = quote! {
        pub async fn reload_resources() -> #core_crate::result::Result<()> {
            let assets_dir = #core_crate::resources::assets_dir();
            let mods_dir = #core_crate::resources::mods_dir();

            load_resources_from(&assets_dir, true, true).await?;
            load_mods_from(&mods_dir).await?;

            Ok(())
        }
    };

    res.extend(reload);

    proc_macro::TokenStream::from(res)
}
