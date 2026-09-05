use quote::quote;

pub fn lox_obj_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  expand(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn expand(input: syn::DeriveInput) -> syn::Result<proc_macro::TokenStream> {
  let name = &input.ident;

  // TODO: take this as an argument name somehow
  let obj_data_variant = name;

  let expanded = quote! {
      impl crate::obj::ObjKind for #name {
          fn project(obj: &ObjData) -> Option<&Self> {
              match obj {
                  ObjData::#obj_data_variant(s) => Some(s),
                  _ => None
              }
          }
          fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
              match obj {
                  ObjData::#obj_data_variant(s) => Some(s),
                  _ => None
              }
          }

          fn embed(self) -> ObjData {
              ObjData::#obj_data_variant(self)
          }
      }
  };

  Ok(expanded.into())
}
