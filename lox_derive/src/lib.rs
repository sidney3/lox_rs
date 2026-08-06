mod ordinal_impl;

#[proc_macro_derive(Ordinal)]
pub fn ordinal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  ordinal_impl::ordinal_impl(input)
}
