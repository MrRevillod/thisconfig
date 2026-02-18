use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Meta, parse_macro_input};

#[proc_macro_attribute]
pub fn config(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match config_impl(args, &input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn config_impl(args: TokenStream, input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let args = syn::parse::<Meta>(args)?;

    let key = match args {
        Meta::NameValue(nv) => {
            if !nv.path.is_ident("key") {
                return Err(Error::new_spanned(nv.path, "expected `key` attribute"));
            }

            match nv.value {
                syn::Expr::Lit(expr_lit) => match expr_lit.lit {
                    syn::Lit::Str(lit_str) => lit_str.value(),
                    _ => {
                        return Err(Error::new_spanned(
                            expr_lit,
                            "expected string literal for key",
                        ));
                    }
                },
                _ => {
                    return Err(Error::new_spanned(
                        nv.value,
                        "expected string literal for key",
                    ));
                }
            }
        }
        _ => {
            return Err(Error::new_spanned(
                args,
                "expected format: #[config(key = \"section_name\")]",
            ));
        }
    };

    let expanded = quote! {
        #input

        impl thisconfig::ConfigItem for #name {
            fn key() -> &'static str {
                #key
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
