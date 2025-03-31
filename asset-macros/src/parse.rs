use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream};

/// Input parameters for the `assets!` macro.
pub struct AssetsInput {
    pub enum_name: Ident,
    pub dir_path_lit: LitStr,
    pub include_pattern_lit: Option<LitStr>,
    pub ignore_pattern_lit: Option<LitStr>,
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
