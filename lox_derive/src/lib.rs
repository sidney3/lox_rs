use quote::quote;
use syn::Data;

#[proc_macro_derive(Ordinal)]
pub fn ordinal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  expand(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn expand(input: syn::DeriveInput) -> syn::Result<proc_macro::TokenStream> {
  let name = &input.ident;

  let variants = match &input.data {
    Data::Enum(e) => &e.variants,
    _ => {
      return Err(syn::Error::new_spanned(
        &input.ident,
        "Ordinal can only be used on enums",
      ));
    }
  };

  let variant_count = variants.len();

  let expanded = quote! {
      // TODO: figure out how to refer to the
      // actual crate
      impl crate::core::Ordinal for #name {
          const COUNT: usize = #variant_count;

          fn ord(&self) -> usize {
              *self as usize
          }
      }

      impl Copy for #name {}
      impl Clone for #name {
          fn clone (&self) -> Self {*self}
      }
  };

  Ok(expanded.into())
}
