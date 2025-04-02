mod ir;
mod parse;
mod utils;

use ir::AssetEnum;
use parse::AssetsInput;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

/// A macro that generates an enum containing all assets in a directory.
///
/// # Parameters
///
/// * `enum_name` - Required. The identifier for the generated enum.
/// * `dir_path` - Required. A string literal specifying the directory path to scan for assets.
/// * `include` - Optional. A regex pattern string literal specifying which files to include.
/// * `ignore` - Optional. A regex pattern string literal specifying which files to ignore.
///
/// # Syntax
///
/// ```
/// assets!(EnumName, "directory/path"[, include: "regex_pattern"][, ignore: "regex_pattern"]);
/// ```
///
/// # Example
///
/// ```ignore
/// use asset_macros::assets;
///
/// assets!(UiAssets, "assets/ui", include: r"\.(png|jpg)$");
/// ```
///
/// This will generate an enum `UiAssets` with variants for each PNG and JPG file in the "assets/ui" directory.
#[proc_macro]
pub fn assets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AssetsInput);
    let ir = match AssetEnum::try_from(input) {
        Ok(ir) => ir,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };
    ir.into_token_stream().into()
}
