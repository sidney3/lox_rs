use log::debug;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use either::Either;
use nonempty::NonEmpty;
use smallvec::{SmallVec, smallvec};

use super::debug::DisplayWithGrammar;
use super::rule::Rule;
use crate::core::Ordinal;
use crate::lexer::TokenType;
use crate::usize_id;

#[derive(Debug, Hash, Eq, Clone, PartialEq)]
pub enum Symbol<R: Rule> {
  Token(R::TokenType),
  Rule(R),
}

impl<R: Rule> Copy for Symbol<R> {}

pub struct Production<R: Rule> {
  pub rule: R,
  pub definition: Vec<Symbol<R>>,
}

impl<R: Rule> Production<R> {
  pub fn len(&self) -> usize {
    self.definition.len()
  }
}

impl<R: Rule> Display for Symbol<R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Token(token_type) => token_type.fmt(f),
      Self::Rule(rule_type) => std::fmt::Display::fmt(&rule_type, f),
    }
  }
}

impl<R: Rule> Ordinal for Symbol<R> {
  const COUNT: usize = R::TokenType::COUNT + R::COUNT;

  fn ord(&self) -> usize {
    match self {
      Self::Token(token_type) => token_type.ord(),
      Self::Rule(rule_type) => R::TokenType::COUNT + rule_type.ord(),
    }
  }
}

impl<R: Rule> Symbol<R> {
  pub fn productions(self, grammar: &Grammar<R>) -> impl Iterator<Item = ProductionId> {
    match self {
      Self::Rule(rule_type) => {
        Either::Right(grammar.productions_for_rule(rule_type).iter().cloned())
      }
      Self::Token(_) => Either::Left(std::iter::empty()),
    }
  }
}

usize_id!(ProductionId);

impl<R: Rule> DisplayWithGrammar<R> for ProductionId {
  fn fmt_with(&self, grammar: &Grammar<R>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    for sym in &grammar.production(*self).definition {
      write!(f, "{}", sym)?;
    }

    Ok(())
  }
}

pub type ProductionList = SmallVec<[ProductionId; 10]>;

pub struct Grammar<R: Rule> {
  productions: Vec<Production<R>>, // indexed by ProductionId

  // Trivial derived properties of our grammar that we eagerly compute
  // More involved properties (goto table) get pulled off
  tokens: Vec<R::TokenType>,
  rules: Vec<R>,
  productions_for_rule: Vec<ProductionList>,
  goal: R,
}

impl<R: Rule> Grammar<R> {
  pub fn new(goal: R, productions: Vec<Production<R>>) -> Self {
    let mut productions_for_rule = Vec::new();
    let mut tokens = HashSet::new();
    let mut rules = HashSet::new();

    for (i, production) in productions.iter().enumerate() {
      if production.rule.ord() >= productions_for_rule.len() {
        productions_for_rule.resize_with(production.rule.ord() + 1, ProductionList::new);
      }
      productions_for_rule[production.rule.ord()].push(ProductionId(i));
      rules.insert(production.rule);
      for symbol in &production.definition {
        match *symbol {
          Symbol::Token(token_type) => tokens.insert(token_type),
          Symbol::Rule(rule_type) => rules.insert(rule_type),
        };
      }
    }

    tokens.insert(R::TokenType::eof());

    Self {
      productions,
      tokens: tokens.into_iter().collect(),
      rules: rules.into_iter().collect(),
      productions_for_rule,
      goal,
    }
  }

  pub fn production(&self, p: ProductionId) -> &Production<R> {
    &self.productions[usize::from(p)]
  }
  pub fn productions_for_rule(&self, r: R) -> &ProductionList {
    &self.productions_for_rule[r.ord()]
  }
  pub fn productions(&self) -> impl Iterator<Item = &Production<R>> {
    self.productions.iter()
  }
  pub fn production_ids(&self) -> impl Iterator<Item = ProductionId> {
    (0..self.productions.len()).map(ProductionId)
  }

  pub fn trivial_production_ids(&self) -> impl Iterator<Item = ProductionId> {
    self
      .production_ids()
      .filter(|p| self.production(*p).definition.is_empty())
  }
  pub fn tokens(&self) -> &Vec<R::TokenType> {
    &self.tokens
  }
  pub fn rules(&self) -> &Vec<R> {
    &self.rules
  }
  pub fn target_rule(&self) -> R {
    self.goal
  }
}
