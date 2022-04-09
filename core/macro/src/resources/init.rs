use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Block, bracketed, Ident, ExprBlock, ItemFn, Lit, LitBool, LitStr, parse_quote, Path, Token, token, Type, Stmt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

struct Signature {
    crate_name: Ident,
    extension: String,
    types: Vec<Type>,
}

struct Syntax {
    crate_name: LitStr,
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
                crate_name: stream.parse()?,
                _comma_1: stream.parse()?,
                extension: stream.parse()?,
                _comma_2: stream.parse()?,
                _bracket: bracketed!(content in stream),
                types: content.parse_terminated(Type::parse)?,
            };

            Ok(Signature {
                crate_name: Ident::from_value(&Lit::Str(syntax.crate_name)).unwrap(),
                extension: syntax.extension.value(),
                types: syntax.types.into_iter().collect(),
            })
        }
    }
}

pub(crate) fn init_resources_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let signature = syn::parse_macro_input!(input as Signature);

    let crate_name = signature.crate_name;
    let extension = signature.extension;
    let custom_types = signature.types;

    let mut load_resources_from = {
        let tokens = proc_macro::TokenStream::from(quote! {
            async fn load_resources_from<P: AsRef<std::path::Path>>(path: P, is_required: bool, should_overwrite: bool) -> #crate_name::Result<()> {
                Ok(())
            }
        });

        syn::parse_macro_input!(tokens as ItemFn)
    };

    let mut stmts: Vec<Stmt> = vec![
        parse_quote! { let path = path.as_ref(); },
        parse_quote! { #crate_name::resources::load_particle_effects(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_audio(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_textures(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_decoration(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_maps(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_images(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { #crate_name::resources::load_fonts(&path, #extension, is_required, should_overwrite).await?; },
        parse_quote! { use #crate_name::resources::{ResourceVec, ResourceMap}; },
        parse_quote! { let path = path.to_str().unwrap().to_string(); },
    ];

    for custom_type in custom_types {
        stmts.push(parse_quote! { #custom_type::load(&path, #extension, is_required, should_overwrite).await?; });
    }

    for (i, stmt) in stmts.into_iter().enumerate() {
        load_resources_from.block.stmts.insert(i, stmt);
    }

    let mut res = load_resources_from.to_token_stream();

    let load_mods_from = quote! {
        async fn load_mods_from<P: AsRef<std::path::Path>>(path: P) -> #crate_name::Result<()> {
            let mut iter = #crate_name::resources::ModLoadingIterator::new(path, #extension).await?;
            while let Some((mod_path, _meta)) = iter.next().await? {
                load_resources_from(&mod_path, false, false).await?;
            }

            Ok(())
        }
    };

    res.extend(load_mods_from);

    let load = quote! {
        pub async fn load_resources<P: AsRef<std::path::Path>>(assets_dir: P, mods_dir: P) -> #crate_name::Result<()> {
            let assets_dir = assets_dir.as_ref();
            let mods_dir = mods_dir.as_ref();

            #crate_name::resources::set_assets_dir(&assets_dir);
            #crate_name::resources::set_mods_dir(&mods_dir);

            load_resources_from(&assets_dir, true, true).await?;
            load_mods_from(&mods_dir).await?;

            Ok(())
        }
    };

    res.extend(load);

    let reload = quote! {
        pub async fn reload_resources() -> #crate_name::Result<()> {
            let assets_dir = #crate_name::resources::assets_dir();
            let mods_dir = #crate_name::resources::mods_dir();

            load_resources_from(&assets_dir, true, true).await?;
            load_mods_from(&mods_dir).await?;

            Ok(())
        }
    };

    res.extend(reload);

    res.into()
}