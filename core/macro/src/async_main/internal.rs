use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

pub(crate) fn internal_main(
    core_crate: &Ident,
    core_impl: &Ident,
    error_type: &Path,
) -> TokenStream {
    quote! {
        fn main() -> std::result::Result<(), #error_type> {
            let rt  = #core_crate::tokio::runtime::Builder::new_current_thread()
                .build()?;

            let mut res = Ok(());

            rt.block_on(async {
                if let Err(err) = #core_crate::config::load_config(#core_impl::config_path()).await {
                    res = Err(err);
                    return;
                }

                if let Err(err) = main_inner().await {
                    res = Err(err);
                    return;
                }
            });

            res
        }
    }
}
