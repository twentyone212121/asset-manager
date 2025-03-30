use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

/// Input parameters for the `assets!` macro.
struct AssetsInput {
    enum_name: Ident,
    dir_path_lit: LitStr,
    include_pattern_lit: Option<LitStr>,
    ignore_pattern_lit: Option<LitStr>,
}

impl Parse for AssetsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let enum_name = input.parse()?;
        input.parse::<Token![,]>()?;
        let dir_path_lit = input.parse()?;

        let mut include_pattern_lit = None;
        let mut ignore_pattern_lit = None;

        // Parse optional parameters
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let keyword: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match keyword.to_string().as_str() {
                "include" => {
                    include_pattern_lit = Some(input.parse()?);
                }
                "ignore" => {
                    ignore_pattern_lit = Some(input.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        keyword.span(),
                        "Expected 'include' or 'ignore'",
                    ));
                }
            }
        }

        Ok(AssetsInput {
            enum_name,
            dir_path_lit,
            include_pattern_lit,
            ignore_pattern_lit,
        })
    }
}

/// A macro that generates an enum containing all assets in a directory.
///
/// # Example
///
/// ```rust
/// use asset_macros::assets;
///
/// assets!(UiAssets, "assets/ui", include: r"\.(png|jpg)$");
/// ```
#[proc_macro]
pub fn assets(input: TokenStream) -> TokenStream {
    let AssetsInput {
        enum_name,
        dir_path_lit,
        include_pattern_lit,
        ignore_pattern_lit,
    } = parse_macro_input!(input as AssetsInput);

    let dir_path_str = dir_path_lit.value();
    let Ok(cargo_manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        return syn::Error::new(
            Span::call_site(),
            "CARGO_MANIFEST_DIR environment variable not set. Are you running inside a Cargo build?"
        )
        .to_compile_error()
        .into();
    };
    let dir_path = Path::new(&cargo_manifest_dir).join(&dir_path_str);

    let include_regex = include_pattern_lit
        .map(|pattern| Regex::new(&pattern.value()).expect("Invalid include regex pattern"));

    let ignore_regex = ignore_pattern_lit
        .map(|pattern| Regex::new(&pattern.value()).expect("Invalid ignore regex pattern"));

    let mut valid_files = Vec::new();
    if let Err(e) = collect_files(&dir_path, &mut valid_files, &include_regex, &ignore_regex) {
        return syn::Error::new(
            dir_path_lit.span(),
            format!("Failed to read directory '{}': {}", dir_path_str, e),
        )
        .to_compile_error()
        .into();
    }

    if valid_files.is_empty() {
        return syn::Error::new(
            dir_path_lit.span(),
            format!("No matching files found in directory '{}'", dir_path_str),
        )
        .to_compile_error()
        .into();
    }

    let (variant_idents, (full_path_strs, rel_path_strs)): (Vec<_>, (Vec<_>, Vec<_>)) = valid_files
        .iter()
        .map(|path| {
            let variant_ident = format_ident!("{}", path_to_variant_name(path, &dir_path));
            let full_path_str = path.to_string_lossy();
            let rel_path_str = path.strip_prefix(&dir_path).unwrap().to_string_lossy();
            (variant_ident, (full_path_str, rel_path_str))
        })
        .unzip();

    let expanded = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #enum_name {
            #(#variant_idents),*
        }

        impl #enum_name {
            fn path_and_bytes(&self) -> (&'static str, &'static [u8]) {
                match self {
                    #(#enum_name::#variant_idents => {
                        const BYTES: &'static [u8] = include_bytes!(#full_path_strs);
                        (#rel_path_strs, BYTES)
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

    expanded.into()
}

/// Helper function to collect files recursively while applying filters
fn collect_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include_regex: &Option<Regex>,
    ignore_regex: &Option<Regex>,
) -> std::io::Result<()> {
    if !dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory not found: {}", dir.display()),
        ));
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let path_str = path.to_string_lossy();

        if ignore_regex
            .as_ref()
            .is_some_and(|regex| regex.is_match(&path_str))
        {
            continue;
        }

        if path.is_dir() {
            collect_files(&path, files, include_regex, ignore_regex)?;
        } else {
            if include_regex
                .as_ref()
                .is_none_or(|regex| regex.is_match(&path_str))
            {
                files.push(path);
            }
        }
    }

    Ok(())
}

/// Convert file path to a valid enum variant name
fn path_to_variant_name(path: &Path, base_dir: &Path) -> String {
    let rel_path = path.strip_prefix(base_dir).unwrap();
    let mut name = String::new();

    for component in rel_path.components() {
        let component_str = component.as_os_str().to_string_lossy();
        if !name.is_empty() {
            name.push('_');
        }
        name.push_str(&component_str.replace(|c: char| !c.is_alphanumeric(), "_"));
    }

    // Ensure it's a valid Rust identifier
    if let Some(first_char) = name.chars().next() {
        if first_char.is_numeric() {
            name = format!("_{}", name);
        }
    }

    name
}
