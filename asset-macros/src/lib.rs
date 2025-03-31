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
/// # Example
///
/// ```ignore
/// use asset_macros::assets;
///
/// assets!(UiAssets, "assets/ui", include: r"\.(png|jpg)$");
/// ```
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
