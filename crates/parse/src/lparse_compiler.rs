use lasso::Rodeo;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use thiserror::Error;

use super::lparse_grammar::{self, Ident};

#[derive(Debug, Error)]
pub enum Error {
  #[error("Referenced rule {0} does not exist")]
  UnboundRule(String),
}

pub fn compile<'ast>(
  lexeme_arena: &'ast Rodeo,
  grammar: &'ast lparse_grammar::LGrammar,
) -> Result<TokenStream, Error> {
  let mut compiler = Compiler::new(lexeme_arena, grammar);
  compiler.compile()
}

struct Compiler<'ast> {
  grammar: &'ast lparse_grammar::LGrammar,
  rules_by_ident: HashMap<Ident, &'ast lparse_grammar::LRule>,
  lexeme_arena: &'ast Rodeo,
}

impl<'ast> Compiler<'ast> {
  pub fn new(lexeme_arena: &'ast Rodeo, grammar: &'ast lparse_grammar::LGrammar) -> Self {
    let rules_by_ident: HashMap<Ident, &'ast lparse_grammar::LRule> =
      grammar.rules.iter().map(|rule| (rule.name, rule)).collect();

    Self {
      grammar,
      rules_by_ident,
      lexeme_arena,
    }
  }

  pub fn compile(&mut self) -> Result<TokenStream, Error> {
    Ok(quote! {})
  }
}
