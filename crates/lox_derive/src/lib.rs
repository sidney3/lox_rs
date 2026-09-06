mod lox_obj_impl;
mod ordinal_impl;

#[proc_macro_derive(Ordinal)]
pub fn ordinal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  ordinal_impl::ordinal_impl(input)
}

#[proc_macro_derive(LoxObj)]
pub fn lox_obj(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  lox_obj_impl::lox_obj_impl(input)
}
