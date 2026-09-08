use heck::ToSnakeCase;
use lasso::Rodeo;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use thiserror::Error;

use crate::lparse_frontend::{LNode, LRule};

use super::lparse_frontend::{self, Ident};

#[derive(Debug, Error)]
pub enum Error {
  #[error("Referenced rule {0} does not exist")]
  UnboundRule(String),
}

pub fn compile<'ast>(
  lexeme_arena: &'ast Rodeo,
  grammar: &'ast lparse_frontend::LGrammar,
) -> Result<TokenStream, Error> {
  Compiler::new(lexeme_arena, grammar).compile()
}

struct Compiler<'ast> {
  grammar: &'ast lparse_frontend::LGrammar,
  rules_by_ident: HashMap<Ident, &'ast lparse_frontend::LRule>,
  goal_rule: &'ast lparse_frontend::LRule,
  lexeme_arena: &'ast Rodeo,
}

impl<'ast> Compiler<'ast> {
  pub fn new(lexeme_arena: &'ast Rodeo, grammar: &'ast lparse_frontend::LGrammar) -> Self {
    let rules_by_ident: HashMap<Ident, &'ast lparse_frontend::LRule> =
      grammar.rules.iter().map(|rule| (rule.name, rule)).collect();

    let goal_rule = *rules_by_ident.get(&grammar.goal_rule).expect("GoalRule");
    Self {
      grammar,
      rules_by_ident,
      goal_rule,
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

  fn rule_factory_function_name(&self, rule: &LRule) -> TokenStream {
    let fn_name = format_ident!(
      "__rule_factory_function_{}",
      self.lexeme_arena.resolve(&rule.name).to_snake_case()
    );

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
          parse::Symbol::Token(#token_type::#this_token_kind)
        }
      }
      LNode::Rule(rule) => {
        let rule_tokens = self.ident_tokens(&rule.rule);
        let rule_type = self.rule_type();

        quote! {
          parse::Symbol::Rule(#rule_type::#rule_tokens)
        }
      }
    }
  }

  fn production_grammatical_definition(
    &self,
    parent_rule: &LRule,
    production: &lparse_frontend::ProductionDefinition,
  ) -> TokenStream {
    let rule_type = self.rule_type();
    let rule_name = self.ident_tokens(&parent_rule.name);

    let rule_definition = production.definition.iter().map(|n| self.node_type(n));

    quote! {
      parse::Production::<#rule_type> {
        rule: #rule_type::#rule_name,
        definition: Vec::from([
          #(#rule_definition),*
        ])
      }
    }
  }

  fn rule_factory_function(&self, rule: &LRule) -> TokenStream {
    let func_name = self.rule_factory_function_name(rule);
    let rule_type = self.rule_type();
    let return_type = self.resolve_embedded_rust(rule.return_type);

    let parent_node = quote! {
      parse::Parent<#rule_type>
    };

    let match_branches = rule
      .productions
      .iter()
      .map(|p| self.production_match_statement(rule, p));

    quote! {
      fn #func_name(node: &#parent_node) -> #return_type {
        match (&node.rule, node.children.as_slice()) {
          #(#match_branches)*
          _ => panic!("Unreachable"),
        }
      }
    }
  }

  fn parser_struct_decl(&self) -> TokenStream {
    let grammar_func = self.make_grammar_name();
    let rule_name = self.rule_type();
    let goal_rule_factory = self.rule_factory_function_name(self.goal_rule);
    let token_type = self.token_type();
    let goal_rule_type = self.resolve_embedded_rust(self.goal_rule.return_type);

    // TODO: rename from GeneratedParser to just...
    // parser :)
    quote! {
      pub struct LParseParser {
        parser: parse::Parser<#rule_name>,
      }

      impl LParseParser {
        pub fn new() -> Self {
          Self {
            parser: parse::Parser::new(#grammar_func()),
          }
        }

        pub fn parse(&self, tokens: Tokens<#token_type>) -> Result<(Rodeo, #goal_rule_type), parse::Error> {
          let cst = self.parser.parse(tokens)?;

          if let parse::Node::Parent(root) = &cst.root {
            Ok((cst.lexeme_arena, #goal_rule_factory(&root)))
          } else {
            panic!("Unreachable, root of CST is a token");
          }
        }
      }
    }
  }

  fn production_match_statement(
    &self,
    parent_rule: &LRule,
    production: &lparse_frontend::ProductionDefinition,
  ) -> TokenStream {
    let rule_type = self.rule_type();
    let rule_name = self.ident_tokens(&parent_rule.name);
    let token_type = self.token_type();

    let anonymous_node_binding = |pos| format_ident!("__node_{pos}");

    // Not the user defined bindings, because they need transforming
    let node_match_bindings = production.definition.iter().enumerate().map(|(i, node)| {
      let node_kind = match node {
        LNode::Leaf(_) => format_ident!("Leaf"),
        LNode::Rule(_) => format_ident!("Parent"),
      };

      let anonymous_binding = anonymous_node_binding(i);

      quote! {
        parse::Node::#node_kind(#anonymous_binding)
      }
    });

    let node_user_bindings = production.definition.iter().enumerate().map(|(i, node)| {
      let anonymous_binding = anonymous_node_binding(i);

      match node {
        LNode::Leaf(leaf) => {
          let bind_to = self.ident_tokens(&leaf.bind_to);
          quote! {
            let #bind_to = #anonymous_binding.lexeme;
          }
        }
        LNode::Rule(rule) => {
          let bind_to = self.ident_tokens(&rule.bind_to);
          let factory_function = self
            .rule_factory_function_name(self.rules_by_ident.get(&rule.rule).expect("Missing rule"));
          if self.lexeme_arena.resolve(&rule.bind_to) == "_" {
            quote! {}
          } else {
            quote! {
              let #bind_to = #factory_function(#anonymous_binding);
            }
          }
        }
      }
    });

    let node_predicates = production.definition.iter().enumerate().map(|(i, node)| {
      let anonymous_binding = anonymous_node_binding(i);
      match node {
        LNode::Leaf(leaf) => {
          let token = self.ident_tokens(&leaf.token);

          quote! {
            #anonymous_binding.token_type == #token_type::#token
          }
        }
        // In practice this isn't important for
        // disambiguation
        LNode::Rule(rule) => {
          let rule = self.ident_tokens(&rule.rule);
          let rule_type = self.rule_type();

          quote! {
            #anonymous_binding.rule == #rule_type::#rule
          }
        }
      }
    });

    let semantic_action = self
      .resolve_embedded_rust(production.semantic_action)
      .unwrap();

    quote! {
      (
        #rule_type::#rule_name,
        [
          #(#node_match_bindings),*
        ],
      ) if true #(&& #node_predicates)* => {
        #(#node_user_bindings)*
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
      fn #make_grammar_name() -> parse::Grammar<#rule_type> {
        parse::Grammar::new(
          #rule_type::#goal_rule, //<goal rule
          Vec::from([
            #(#rule_definitions)*
          ]),
        )
      }
    }
  }

  fn rule_impl(&self) -> TokenStream {
    let rule_type = self.rule_type();
    let token_type = self.token_type();
    quote! {
      impl parse::Rule for #rule_type {
        type TokenType = #token_type;
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
    let parser_decl = self.parser_struct_decl();
    let preamble = self.preamble();
    let rule_enum_def = self.rule_enum_def();
    let make_grammar = self.make_grammar();
    let rule_trait_impl = self.rule_impl();
    let rule_factories = self
      .grammar
      .rules
      .iter()
      .map(|rule| self.rule_factory_function(rule));

    Ok(quote! {
      #preamble

      #rule_enum_def

      #rule_trait_impl

      #parser_decl

      #make_grammar

      #(#rule_factories)*
    })
  }
}
