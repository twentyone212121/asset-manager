use proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use regex::Regex;
use std::path::Path;
use syn::Ident;

use crate::parse::AssetsInput;
use crate::utils::{collect_files, path_to_variant_name};

pub(crate) struct AssetEnum {
    enum_name: Ident,
    entries: Vec<AssetEntry>,
}

pub(crate) struct AssetEntry {
    variant_ident: Ident,
    full_path: String,
    rel_path: String,
}

impl TryFrom<AssetsInput> for AssetEnum {
    type Error = syn::Error;

    fn try_from(value: AssetsInput) -> Result<Self, Self::Error> {
        let AssetsInput {
            enum_name,
            dir_path_lit,
            include_pattern_lit,
            ignore_pattern_lit,
        } = value;

        let dir_path_str = dir_path_lit.value();
        let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| syn::Error::new(
            Span::call_site(),
            "CARGO_MANIFEST_DIR environment variable not set. Are you running inside a Cargo build?",
        ))?;
        let dir_path = Path::new(&cargo_manifest_dir).join(&dir_path_str);

        let include_regex = include_pattern_lit
            .map(|pattern| Regex::new(&pattern.value()).expect("Invalid include regex pattern"));

        let ignore_regex = ignore_pattern_lit
            .map(|pattern| Regex::new(&pattern.value()).expect("Invalid ignore regex pattern"));

        let mut valid_files = Vec::new();
        collect_files(&dir_path, &mut valid_files, &include_regex, &ignore_regex).map_err(|e| {
            syn::Error::new(
                dir_path_lit.span(),
                format!("Failed to read directory '{}': {}", dir_path_str, e),
            )
        })?;

        if valid_files.is_empty() {
            return Err(syn::Error::new(
                dir_path_lit.span(),
                format!("No matching files found in directory '{}'", dir_path_str),
            ));
        }

        let entries = valid_files
            .into_iter()
            .map(|path| {
                let rel_path = path.strip_prefix(&dir_path).unwrap();
                let variant_ident = format_ident!("{}", path_to_variant_name(&rel_path));
                let full_path = path.to_string_lossy().into_owned();
                let rel_path = rel_path.to_string_lossy().into_owned();

                AssetEntry {
                    variant_ident,
                    full_path,
                    rel_path,
                }
            })
            .collect();

        Ok(Self { enum_name, entries })
    }
}

impl ToTokens for AssetEnum {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { enum_name, entries } = self;
        let (variant_idents, (full_paths, rel_paths)): (Vec<_>, (Vec<_>, Vec<_>)) = entries
            .iter()
            .map(|entry| (&entry.variant_ident, (&entry.full_path, &entry.rel_path)))
            .unzip();

        let output = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_name {
                #(#variant_idents),*
            }

            impl #enum_name {
                fn path_and_bytes(&self) -> (&'static str, &'static [u8]) {
                    match self {
                        #(#enum_name::#variant_idents => {
                            const BYTES: &'static [u8] = include_bytes!(#full_paths);
                            (#rel_paths, BYTES)
                        }),*
                    }
                }

                /// Get all assets of this type.
                pub fn all() -> &'static [#enum_name] {
                    static ALL_ASSETS: &[#enum_name] = &[#(#enum_name::#variant_idents),*];
                    ALL_ASSETS
                }
            }

            impl asset_traits::Asset for #enum_name {
                fn path(&self) -> &'static str {
                    self.path_and_bytes().0
                }

                fn bytes(&self) -> &'static [u8] {
                    self.path_and_bytes().1
                }
            }

            impl asset_traits::AssetCollection for #enum_name {
                fn all() -> &'static [Self] {
                    Self::all()
                }
            }
        };

        tokens.extend(output);
    }
}
