use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error, Meta};

/// Derive macro para implementar automáticamente el trait `ConfigItem`.
///
/// # Ejemplo
///
/// ```rust
/// use axum_config::config;
/// use serde::Deserialize;
///
/// #[config(key = "database")]
/// #[derive(Debug, Clone, Deserialize)]
/// pub struct DatabaseConfig {
///     pub host: String,
///     pub port: u16,
/// }
/// ```
#[proc_macro_attribute]
pub fn config(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match config_impl(args, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn config_impl(args: TokenStream, input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    // Parsear el argumento key = "value"
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

        impl axum_config::ConfigItem for #name {
            fn key() -> &'static str {
                #key
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
