use darling::ast::Data;
use darling::{ast, util, FromDeriveInput, FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type};

use crate::CORE_CRATE_NAME;

#[derive(Debug, FromField)]
#[darling(attributes(resource))]
pub struct ResourceField {
    ident: Option<Ident>,
    #[allow(dead_code)]
    ty: Type,
    #[darling(default)]
    id: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(resource), forward_attrs(resource), supports(struct_named))]
pub struct ResourceDeriveArgs {
    #[allow(dead_code)]
    ident: Ident,
    data: ast::Data<util::Ignored, ResourceField>,
    //attrs: Vec<Attribute>,
    name: String,
    #[darling(default)]
    name_plural: Option<String>,
    #[darling(default)]
    path_index: bool,
    #[darling(default)]
    iter_only: bool,
    #[darling(default)]
    mutable: bool,
    #[darling(default)]
    crate_name: Option<String>,
}

pub(crate) fn derive_resource_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut res = TokenStream::new();

    let input = parse_macro_input!(input as DeriveInput);

    let attr_args: ResourceDeriveArgs = match ResourceDeriveArgs::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(e.write_errors());
        }
    };

    let crate_name_str = attr_args
        .crate_name
        .unwrap_or_else(|| CORE_CRATE_NAME.to_string());
    let name_str = attr_args.name;
    let name_plural_str = attr_args
        .name_plural
        .unwrap_or_else(|| format!("{}s", &name_str));
    let storage_name_str = name_plural_str.to_uppercase();

    let is_path_index = attr_args.path_index;
    let is_iter_only = attr_args.iter_only;
    let is_mutable = attr_args.mutable;

    let mut id_name = None;

    if let Data::Struct(fields) = attr_args.data {
        for field in fields {
            if !is_iter_only {
                if field.id {
                    id_name = Some(field.ident.unwrap());
                    break;
                } else if let Some(ident) = field.ident {
                    if format!("{}", ident) == "id" {
                        id_name = Some(ident);
                        break;
                    }
                }
            }
        }
    }

    let type_name = input.ident;
    let crate_name = Ident::from_string(&crate_name_str).unwrap();
    let storage_name = Ident::from_string(&storage_name_str).unwrap();
    let resource_name = Ident::from_string(&name_str).unwrap();
    let resource_mut_name = Ident::from_string(&format!("{}_mut", &name_str)).unwrap();
    let iter_name = Ident::from_string(&format!("iter_{}", &name_plural_str)).unwrap();
    let iter_mut_name = Ident::from_string(&format!("iter_{}_mut", &name_plural_str)).unwrap();
    let try_get_name = Ident::from_string(&format!("try_get_{}", &name_str)).unwrap();
    let get_name = Ident::from_string(&format!("get_{}", &name_str)).unwrap();
    let try_get_mut_name = Ident::from_string(&format!("try_get_{}_mut", &name_str)).unwrap();
    let get_mut_name = Ident::from_string(&format!("get_{}_mut", &name_str)).unwrap();
    let load_name = Ident::from_string(&format!("load_{}_resources", &resource_name)).unwrap();

    let common = quote! {
        impl #crate_name::resources::Resource for #type_name {}
    };

    res.extend(common);

    if let Some(id_name) = id_name {
        let base = quote! {
            impl #crate_name::resources::ResourceId for #type_name {
                fn id(&self) -> String {
                    self.#id_name.to_string()
                }
            }

            pub(crate) mod resource_impl {
                use super::*;
                use #crate_name::resources::ResourceId;

                static mut #storage_name: Option<std::collections::HashMap<String, #type_name>> = None;

                pub fn #resource_name() -> &'static std::collections::HashMap<String, #type_name> {
                    unsafe { #storage_name.get_or_insert_with(std::collections::HashMap::new) }
                }

                pub fn #iter_name() -> std::collections::hash_map::Iter<'static, String, #type_name> {
                    #resource_name().iter()
                }

                pub fn #try_get_name(id: &str) -> Option<&'static #type_name> {
                    #resource_name().get(id)
                }

                pub fn #get_name(id: &str) -> &'static #type_name {
                    #try_get_name(id).unwrap()
                }

                pub async fn #load_name<P, E>(path: P, ext: E, is_required: bool, should_overwrite: bool) -> #crate_name::result::Result<()>
                    where
                        P: AsRef<std::path::Path>,
                        E: Into<Option<&'static str>>,
                {
                    let path = path.as_ref();

                    let ext = ext.into().unwrap_or(#crate_name::resources::DEFAULT_RESOURCE_FILE_EXTENSION);

                    let mut storage = unsafe { #storage_name.get_or_insert_with(std::collections::HashMap::new) };

                    if should_overwrite {
                        storage.clear();
                    }

                    let file_path = path
                        .join(#name_plural_str)
                        .with_extension(ext);

                    match #crate_name::file::read_from_file(&file_path).await {
                        Err(err) => if is_required {
                            return Err(err.into());
                        }
                        Ok(bytes) => {
                            if #is_path_index {
                                let paths: Vec<String> = #crate_name::parsing::deserialize_bytes_by_extension(ext, &bytes)?;

                                for resource_path in paths {
                                    let resource_path = path
                                        .join(resource_path);

                                    let ext = resource_path
                                        .extension()
                                        .unwrap()
                                        .to_str()
                                        .unwrap();

                                    let bytes = #crate_name::file::read_from_file(&resource_path).await?;

                                    let resource: #type_name = #crate_name::parsing::deserialize_bytes_by_extension(ext, &bytes)?;

                                    storage.insert(resource.id(), resource);
                                }
                            } else {
                                let ext = file_path
                                    .extension()
                                    .unwrap()
                                    .to_str()
                                    .unwrap();

                                let resources: Vec<#type_name> = #crate_name::parsing::deserialize_bytes_by_extension(ext, &bytes)?;

                                for resource in resources {
                                    storage.insert(resource.id(), resource);
                                }
                            }
                        }
                    }

                    Ok(())
                }
            }

            pub use resource_impl::{#resource_name, #load_name, #try_get_name, #get_name, #iter_name};

            #[#crate_name::async_trait]
            impl #crate_name::resources::ResourceMap for #type_name {
                fn storage() -> &'static std::collections::HashMap<String, #type_name> {
                    #resource_name()
                }

                async fn load<P, E>(path: P, ext: E, is_required: bool, should_overwrite: bool) -> #crate_name::result::Result<()>
                    where
                        P: AsRef<std::path::Path> + Send,
                        E: Into<Option<&'static str>> + Send,
                {
                    #load_name(path, ext, is_required, should_overwrite).await?;

                    Ok(())
                }
            }
        };

        res.extend(base);

        if is_mutable {
            let mutable = quote! {
                pub(crate) mod resource_impl {
                    pub fn #resource_mut_name() -> &'static mut std::collections::HashMap<String, #type_name> {
                        unsafe { #storage_name.get_or_insert_with(std::collections::HashMap::new) }
                    }

                    pub fn #iter_mut_name() -> std::collections::hash_map::IterMut<'static, String, #type_name> {
                        #resource_mut_name().iter_mut()
                    }

                    pub fn #try_get_mut_name(id: &str) -> Option<&'static mut #type_name> {
                        #resource_mut_name().get_mut(id)
                    }

                    pub fn #get_mut_name(id: &str) -> &'static mut #type_name {
                        #try_get_mut_name(id).unwrap()
                    }
                }

                pub use resource_impl::{#resource_mut_name, #try_get_mut_name, #get_mut_name, #iter_mut_name};

                impl #crate_name::resources::ResourceMapMut for #type_name {
                    fn storage_mut() -> &'static mut std::collections::HashMap<String, #type_name> {
                        #resource_mut_name()
                    }
                }
            };

            res.extend(mutable);
        }
    } else {
        let base = quote! {
            pub(crate) mod resource_impl {
                use super::*;
                use super::#type_name;

                static mut #storage_name: Vec<#type_name> = Vec::new();

                pub fn #resource_name() -> &'static Vec<#type_name> {
                    unsafe { #storage_name.as_ref() }
                }

                pub fn #resource_mut_name() -> &'static mut Vec<#type_name> {
                    unsafe { #storage_name.as_mut() }
                }

                pub fn #iter_name() -> std::slice::Iter<'static, #type_name> {
                    #resource_name().iter()
                }

                pub fn #iter_mut_name() -> std::slice::IterMut<'static, #type_name> {
                    #resource_mut_name().iter_mut()
                }

                pub fn #try_get_name(index: usize) -> Option<&'static #type_name> {
                    #resource_name().get(index)
                }

                pub fn #try_get_mut_name(index: usize) -> Option<&'static mut #type_name> {
                    #resource_mut_name().get_mut(index)
                }

                pub fn #get_name(index: usize) -> &'static #type_name {
                    #try_get_name(index).unwrap()
                }

                pub fn #get_mut_name(index: usize) -> &'static mut #type_name {
                    #try_get_mut_name(index).unwrap()
                }

                pub async fn #load_name<P, E>(path: P, ext: E, is_required: bool, should_overwrite: bool) -> #crate_name::result::Result<()>
                    where
                        P: AsRef<std::path::Path>,
                        E: Into<Option<&'static str>>,
                {
                    let path = path.as_ref();

                    let ext = ext.into().unwrap_or(#crate_name::resources::DEFAULT_RESOURCE_FILE_EXTENSION);

                    let storage = unsafe { &mut #storage_name };

                    if should_overwrite {
                        storage.clear();
                    }

                    let file_path = path
                        .join(#name_plural_str)
                        .with_extension(ext);

                    match #crate_name::file::read_from_file(&file_path).await {
                        Err(err) => if is_required {
                            return Err(err.into());
                        }
                        Ok(bytes) => {
                            if #is_path_index {
                                let paths: Vec<String> = #crate_name::parsing::deserialize_bytes_by_extension(ext, &bytes)?;

                                for resource_path in paths {
                                    let resource_path = path
                                        .join(&resource_path);

                                    let ext = resource_path
                                        .extension()
                                        .unwrap()
                                        .to_str()
                                        .unwrap();

                                    let bytes = #crate_name::file::read_from_file(&resource_path).await?;

                                    let resource: #type_name = #crate_name::parsing::deserialize_bytes_by_extension(ext, &bytes)?;

                                    storage.push(resource);
                                }
                            } else {
                                let ext = file_path
                                    .extension()
                                    .unwrap()
                                    .to_str()
                                    .unwrap();

                                let mut resources: Vec<#type_name> = #crate_name::parsing::deserialize_bytes_by_extension(ext, &bytes)?;

                                storage.append(&mut resources);
                            }
                        }
                    }

                    Ok(())
                }
            }

            pub use resource_impl::{#resource_name, #load_name, #try_get_name, #get_name, #iter_name};

            #[#crate_name::async_trait]
            impl #crate_name::resources::ResourceVec for #type_name {
                fn storage() -> &'static Vec<#type_name> {
                    #resource_name()
                }

                async fn load<P, E>(path: P, ext: E, is_required: bool, should_overwrite: bool) -> #crate_name::result::Result<()>
                    where
                        P: AsRef<std::path::Path> + Send,
                        E: Into<Option<&'static str>> + Send,
                {
                    #load_name(path, ext, is_required, should_overwrite).await?;

                    Ok(())
                }
            }
        };

        res.extend(base);

        if is_mutable {
            let mutable = quote! {
                pub use resource_impl::{#resource_mut_name, #try_get_mut_name, #get_mut_name, #iter_mut_name};

                impl #crate_name::resources::ResourceVecMut for #type_name {
                    fn storage_mut() -> &'static mut Vec<#type_name> {
                        #resource_mut_name()
                    }
                }
            };

            res.extend(mutable);
        }
    }

    proc_macro::TokenStream::from(res)
}
