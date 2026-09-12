use itertools::Itertools;
use lasso::{Rodeo, Spur};
use lexer::TokenType;
use lox_derive::Ordinal;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use strum::Display;

pub(crate) trait Stage {
  type Kleene: Debug;
}

pub(crate) struct Raw {}
impl Stage for Raw {
  type Kleene = ();
}

pub(crate) struct NoKleene {}
impl Stage for NoKleene {
  type Kleene = Infallible;
}

pub type Ident = Spur;

pub fn lexer() -> Result<lexer::Lexer<LParseToken>, lexer::Error> {
  lexer::Lexer::<LParseToken>::new(LEX_SPEC)
}

#[derive(Debug, Clone)]
pub struct BoundLeaf {
  pub token: Ident,
  pub bind_to: Ident,
}
#[derive(Debug, Clone)]
pub struct BoundRule {
  pub rule: Ident,
  pub bind_to: Ident,
}
#[derive(Debug, Clone)]
pub enum LNode<S: Stage = Raw> {
  Leaf(BoundLeaf),
  Rule(BoundRule),
  Kleene(BoundRule, S::Kleene),
}

#[derive(Debug)]
pub struct ProductionDefinition<S: Stage = Raw> {
  pub definition: Vec<LNode<S>>,
  pub semantic_action: Ident,
}

#[derive(Debug)]
pub struct LRule<S: Stage = Raw> {
  pub name: Ident,
  pub return_type: Ident,
  pub productions: Vec<ProductionDefinition<S>>,
}

#[derive(Debug)]
pub struct LGrammar<S: Stage = Raw> {
  pub preamble: Vec<Spur>,
  pub goal_rule: Ident,
  pub token_type: Ident,
  pub rules: Vec<LRule<S>>,
}

pub fn try_unwrap_embedded_rust(s: &str) -> Option<&str> {
  s.strip_prefix("%").and_then(|s| s.strip_suffix("%"))
}
pub fn format_embedded_rust(s: &str) -> String {
  format!("%{s}%")
}

impl<S: Stage> LGrammar<S> {
  pub fn get_rule(&self, ident: Ident) -> Option<&LRule<S>> {
    self.rules.iter().find(|r| r.name == ident)
  }

  pub fn all_productions(&self) -> impl Iterator<Item = &ProductionDefinition<S>> {
    self.rules.iter().flat_map(|rule| rule.productions.iter())
  }
  pub fn all_nodes(&self) -> impl Iterator<Item = &LNode<S>> {
    self.all_productions().flat_map(|p| p.definition.iter())
  }
}

impl LGrammar {
  pub fn parse(self, rodeo: &mut Rodeo) -> LGrammar<NoKleene> {
    let kleene_rules = self.make_kleene_rules(rodeo);

    let mut rules: Vec<LRule<NoKleene>> = self
      .rules
      .iter()
      .map(|r| self.parse_rule(r, &kleene_rules))
      .collect();

    rules.extend(kleene_rules.into_values());
    rules.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    LGrammar::<NoKleene> {
      preamble: self.preamble,
      goal_rule: self.goal_rule,
      token_type: self.token_type,
      rules,
    }
  }

  fn parse_rule(
    &self,
    rule: &LRule,
    kleene_rules: &HashMap<Ident, LRule<NoKleene>>,
  ) -> LRule<NoKleene> {
    let parse_node = |node: &LNode| match node {
      LNode::Rule(rule) => LNode::<NoKleene>::Rule(rule.clone()),
      LNode::Leaf(leaf) => LNode::<NoKleene>::Leaf(leaf.clone()),
      LNode::Kleene(rule, _) => LNode::<NoKleene>::Rule(BoundRule {
        bind_to: rule.bind_to,
        rule: kleene_rules.get(&rule.rule).expect("Unhandled kleene").name,
      }),
    };

    let parse_production = |production: &ProductionDefinition| ProductionDefinition::<NoKleene> {
      definition: production.definition.iter().map(parse_node).collect(),
      semantic_action: production.semantic_action,
    };

    LRule::<NoKleene> {
      name: rule.name,
      return_type: rule.return_type,
      productions: rule.productions.iter().map(parse_production).collect(),
    }
  }

  // OriginalRuleName -> Kleene for that rule
  fn make_kleene_rules(&self, rodeo: &mut Rodeo) -> HashMap<Ident, LRule<NoKleene>> {
    self
      .all_nodes()
      .filter_map(|node| match node {
        LNode::Kleene(bound_to, _) => Some(bound_to.rule),
        _ => None,
      })
      .unique()
      .map(|rule_ident| self.get_rule(rule_ident).expect("Unbound kleene"))
      .map(|rule| (rule.name, self.make_kleene_rule(rule, rodeo)))
      .collect()
  }

  fn make_kleene_rule(&self, rule: &LRule, rodeo: &mut Rodeo) -> LRule<NoKleene> {
    let name = format!("Reserved{}Kleene", rodeo.resolve(&rule.name));
    let return_type = format_embedded_rust(
      format!(
        "Vec<{}>",
        try_unwrap_embedded_rust(rodeo.resolve(&rule.return_type))
          .expect("Return type not embedded rust")
      )
      .as_str(),
    );

    let trivial_semantic_action = quote! {
      Vec::new()
    };

    let tail = "tail";
    let first = "first";

    let tail_ident = format_ident!("{}", tail);
    let first_ident = format_ident!("{}", first);

    let recursive_semantic_action = quote! {
      let mut all = #first_ident;
      all.push(#tail_ident);
      all
    };

    let productions = vec![
      ProductionDefinition::<NoKleene> {
        definition: Vec::new(),
        semantic_action: rodeo.get_or_intern(format_embedded_rust(
          trivial_semantic_action.to_string().as_str(),
        )),
      },
      ProductionDefinition::<NoKleene> {
        definition: vec![
          LNode::<NoKleene>::Rule(BoundRule {
            rule: rodeo.get_or_intern(name.as_str()),
            bind_to: rodeo.get_or_intern(first),
          }),
          LNode::<NoKleene>::Rule(BoundRule {
            rule: rule.name,
            bind_to: rodeo.get_or_intern(tail),
          }),
        ],
        semantic_action: rodeo.get_or_intern(format_embedded_rust(
          recursive_semantic_action.to_string().as_str(),
        )),
      },
    ];

    LRule::<NoKleene> {
      name: rodeo.get_or_intern(name.as_str()),
      return_type: rodeo.get_or_intern(return_type),
      productions,
    }
  }
}

#[derive(Ordinal, Eq, PartialEq, Hash, Display, Debug, PartialOrd)]
pub enum LParseToken {
  LCurlyBrace,
  RCurlyBrace,
  LSquareBracket,
  RSquareBracket,
  LAngleBracket,
  RAngleBracket,
  Equals,
  Semicolon,
  Colon,
  Comma,
  Asterisk,
  RustImport,
  RustDirective,

  // keywords
  GoalRule,
  TokenType,

  Ident,

  EmbeddedRust,
  Whitespace,
  Eof,
}

impl TokenType for LParseToken {
  fn whitespace() -> Self {
    Self::Whitespace
  }
  fn eof() -> Self {
    Self::Eof
  }
}

const LEX_SPEC: &[(LParseToken, &str)] = &[
  (LParseToken::LCurlyBrace, "{"),
  (LParseToken::RCurlyBrace, "}"),
  (LParseToken::LSquareBracket, "\\["),
  (LParseToken::RSquareBracket, "\\]"),
  (LParseToken::LAngleBracket, "<"),
  (LParseToken::RAngleBracket, ">"),
  (LParseToken::Equals, "="),
  (LParseToken::Semicolon, ";"),
  (LParseToken::Colon, ":"),
  (LParseToken::Comma, ","),
  (LParseToken::Asterisk, "\\*"),
  (LParseToken::GoalRule, "goal_rule"),
  (LParseToken::TokenType, "token_type"),
  (LParseToken::EmbeddedRust, "%[\u{0}-\u{24}\u{26}-\u{7F}]*%"),
  (LParseToken::Whitespace, " "),
  (LParseToken::Whitespace, "\t"),
  (LParseToken::Whitespace, "\n"),
  (LParseToken::RustImport, "use ([a-zA-Z]|_|{|}|:|,| )*;"),
  (LParseToken::RustDirective, "#!\\[allow\\(clippy::all\\)\\]"),
  (LParseToken::Ident, "([a-zA-Z]|_)([a-zA-Z0-9]|_)*"),
];
