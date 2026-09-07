use lasso::Rodeo;
use log::info;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use thiserror::Error;

use crate::lparse_grammar::LRule;

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
  Compiler::new(lexeme_arena, grammar).compile()
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

  fn token_type(&self) -> TokenStream {
    self
      .lexeme_arena
      .resolve(&self.grammar.token_type)
      .parse()
      .expect("TokenType regex allowed invalid rust")
  }

  fn rule_name(&self, rule: &LRule) -> TokenStream {
    self
      .lexeme_arena
      .resolve(&rule.name)
      .parse()
      .expect("RuleName regex allowed invalid regex")
  }

  fn preamble(&self) -> TokenStream {
    self
      .grammar
      .preamble
      .iter()
      .map(|s| self.lexeme_arena.resolve(s))
      .collect::<Vec<_>>()
      .join("")
      .parse()
      .expect("Preamble regex allowed invalid rust")
  }

  fn rule_enum_def(&self) -> TokenStream {
    let rule_names = self.grammar.rules.iter().map(|r| self.rule_name(r));

    info!("{:?}", self.grammar.rules);

    quote! {
      #[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
      enum ParseRule {
        #(#rule_names,)*
      }
    }
  }

  fn grammar(&self) -> TokenStream {
    todo!();
  }

  fn rule_function_function_name(&self, rule: Ident) -> TokenStream {
    todo!();
  }

  fn rule_factory_function(&self, rule: Ident) -> TokenStream {
    todo!();
  }

  fn resolve_embedded_rust(&self, s: Ident) -> Option<TokenStream> {
    self
      .lexeme_arena
      .resolve(&s)
      .strip_prefix("%")
      .and_then(|s| s.strip_suffix("%"))
      .and_then(|s| s.parse().ok())
  }

  pub fn compile(&self) -> Result<TokenStream, Error> {
    let preamble = self.preamble();

    let rule_enum_def = self.rule_enum_def();

    Ok(quote! {
      #preamble

      #rule_enum_def
    })
  }
}
