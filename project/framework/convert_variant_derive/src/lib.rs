extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};

use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};

macro_rules! derive_error {
    ($string: tt) => {
        Error::new(Span::call_site(), $string)
            .to_compile_error()
            .into()
    };
}

// Derives a function that pattern matches over an enum and returns each variant with 'Relations::' attached.
#[proc_macro_derive(EquivRelId)]
pub fn derive_convert_to_relid(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let ref name = input.ident;
    let ref data = input.data;
    let mut variant_cases;
    match data {
        Data::Enum(data_enum) => {
            variant_cases = TokenStream2::new();
            for variant in &data_enum.variants {
                let ref variant_name = variant.ident;
                let fields_in_variant = match &variant.fields {
                    Fields::Unnamed(_) => quote_spanned! {variant.span()=> (..) },
                    Fields::Unit => quote_spanned! { variant.span()=> },
                    Fields::Named(_) => quote_spanned! {variant.span()=> {..} },
                };
                variant_cases.extend(quote_spanned! {variant.span() =>
                    #name::#variant_name #fields_in_variant => return Relations::#variant_name,
                })
            }
        }
        _ => return derive_error!("EquivRelId only implemented for enums"),
    };
    let full_function = quote! {
        impl EquivRelId for #name {
            fn get_equiv_relid(&self) -> Relations {
                match self {
                    #variant_cases
                    _ => panic!("Something went wrong with relation conversion to RelId")
                }
            }
        }
    };
    TokenStream::from(full_function)
}
