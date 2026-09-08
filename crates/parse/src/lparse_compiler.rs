use lasso::Rodeo;
use log::info;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use thiserror::Error;

use crate::lparse_grammar::{LNode, LRule};

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

  fn ident_tokens(&self, ident: &Ident) -> TokenStream {
    self
      .lexeme_arena
      .resolve(ident)
      .parse()
      .expect("TODO: this should be a result type")
  }

  fn preamble(&self) -> TokenStream {
    let imports = self.grammar.preamble.iter().map(|s| self.ident_tokens(s));

    quote! {
      #(#imports)*
    }
  }

  fn rule_type(&self) -> TokenStream {
    quote! {
      ParseRule
    }
  }

  fn token_type(&self) -> TokenStream {
    self.ident_tokens(&self.grammar.token_type)
  }

  fn rule_enum_def(&self) -> TokenStream {
    let rule_type = self.rule_type();
    let rule_names = self
      .grammar
      .rules
      .iter()
      .map(|r| self.ident_tokens(&r.name));

    quote! {
      #[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
      enum #rule_type {
        #(#rule_names,)*
      }
    }
  }

  fn rule_function_function_name(&self, rule: &LRule) -> TokenStream {
    let fn_name = format_ident!("__make_{}", self.lexeme_arena.resolve(&rule.name));

    quote! {
      #fn_name
    }
  }

  // e.x. Symbol::Token(LParseToken::LSquareBracket)
  fn node_type(&self, node: &LNode) -> TokenStream {
    match node {
      LNode::Leaf(leaf) => {
        let token_type = self.token_type();
        let this_token_kind = self.ident_tokens(&leaf.token);

        quote! {
          Symbol::Token(#token_type::#this_token_kind)
        }
      }
      LNode::Rule(rule) => {
        let rule_tokens = self.ident_tokens(&rule.rule);
        let rule_type = self.rule_type();

        quote! {
          Symbol::Rule(#rule_type::#rule_tokens)
        }
      }
    }
  }

  fn node_bind(&self, node: &LNode) -> TokenStream {
    let rule_type = self.rule_type();

    match node {
      LNode::Leaf(bound_leaf) => {
        let bound_to = self.ident_tokens(&bound_leaf.bind_to);
        quote! {crate::Node::<#rule_type>::Leaf(#bound_to)}
      }
      LNode::Rule(bound_rule) => {
        let bound_to = self.ident_tokens(&bound_rule.bind_to);
        quote! {crate::Node::<#rule_type>::Parent(#bound_to)}
      }
    }
  }

  fn node_predicate(&self, node: &LNode) -> TokenStream {
    match node {
      LNode::Leaf(leaf) => {
        let bound_ident = self.ident_tokens(&leaf.bind_to);
        let token_ident = self.ident_tokens(&leaf.token);
        let token_type = self.token_type();

        quote! {
          #bound_ident.token_type == #token_type::#token_ident
        }
      }
      LNode::Rule(_) => {
        // I'm lazy and in practice this isn't important for
        // disambiguating.
        quote! {
          true
        }
      }
    }
  }

  fn production_grammatical_definition(
    &self,
    parent_rule: &LRule,
    production: &lparse_grammar::ProductionDefinition,
  ) -> TokenStream {
    let rule_type = self.rule_type();
    let rule_name = self.ident_tokens(&parent_rule.name);

    let rule_definition = production.definition.iter().map(|n| self.node_type(n));

    quote! {
      Production::<#rule_type> {
        rule: #rule_type::#rule_name,
        definition: Vec::from([
          #(#rule_definition),*
        ])
      }
    }
  }

  fn rule_factory_function(&self, rule: &LRule) -> TokenStream {
    let func_name = self.rule_function_function_name(rule);
    let rule_type = self.rule_type();
    let return_type = self.resolve_embedded_rust(rule.return_type);

    let parent_node = quote! {
      crate::Parent<#rule_type>
    };

    let match_branches = rule
      .productions
      .iter()
      .map(|p| self.production_match_statement(rule, p));

    quote! {
      fn #func_name(node: &#parent_node) -> #return_type {
        match (&node.rule, node.children.as_slice()) {
          #(#match_branches)*
        }
      }
    }
  }

  fn production_match_statement(
    &self,
    parent_rule: &LRule,
    production: &lparse_grammar::ProductionDefinition,
  ) -> TokenStream {
    let node_bindings = production.definition.iter().map(|n| self.node_bind(n));
    let node_predicates = production
      .definition
      .iter()
      .map(|node| self.node_predicate(node));

    // TODO: let's pull the lexeme out and bind it to what the user wants
    let semantic_action = self
      .resolve_embedded_rust(production.semantic_action)
      .unwrap();

    let rule_type = self.rule_type();
    let rule_name = self.ident_tokens(&parent_rule.name);

    quote! {
      (
        #rule_type::#rule_name,
        [
          #(#node_bindings),*
        ],
      ) if true #(&& #node_predicates)* => {
        #semantic_action
      },
    }
  }

  fn rule_grammatical_definitions(&self, rule: &LRule) -> TokenStream {
    let production_definitions = rule
      .productions
      .iter()
      .map(|p| self.production_grammatical_definition(rule, p));

    quote! {
      #(#production_definitions,)*
    }
  }

  fn make_grammar_name(&self) -> TokenStream {
    quote! {
      __make_grammar
    }
  }
  fn make_grammar(&self) -> TokenStream {
    let make_grammar_name = self.make_grammar_name();
    let rule_type = self.rule_type();

    let rule_definitions = self
      .grammar
      .rules
      .iter()
      .map(|r| self.rule_grammatical_definitions(r));

    let goal_rule = self.ident_tokens(&self.grammar.goal_rule);

    quote! {
      fn #make_grammar_name() -> Grammar<#rule_type> {
        Grammar::new(
          #rule_type::#goal_rule, //<goal rule
          Vec::from([
            #(#rule_definitions)*
          ]),
        )
      }
    }
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
    let make_grammar = self.make_grammar();
    let rule_factories = self
      .grammar
      .rules
      .iter()
      .map(|rule| self.rule_factory_function(rule));

    Ok(quote! {
      #preamble

      #rule_enum_def

      #make_grammar

      #(#rule_factories)*
    })
  }
}
