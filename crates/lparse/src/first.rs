use super::{Grammar, Rule, Symbol};
use itertools::Itertools;
use smallvec::SmallVec;
use std::collections::HashSet;

pub(super) fn first<R: Rule>(grammar: &Grammar<R>) -> Vec<HashSet<R::TokenType>> {
  let mut changed = true;
  let mut first_table: Vec<HashSet<R::TokenType>> = (0..R::COUNT).map(|_| HashSet::new()).collect();

  while changed {
    changed = false;

    for production in grammar.productions() {
      let first_nodes = production
        .definition
        .iter()
        .take_while_inclusive(|sym| match sym {
          Symbol::Token(_) => false,
          &Symbol::Rule(rule) => grammar.rule_contains_epsilon(*rule),
        });

      let mut fst_set: SmallVec<[R::TokenType; 8]> = SmallVec::new();

      for node in first_nodes {
        match *node {
          Symbol::Token(t) => fst_set.push(t),
          Symbol::Rule(r) => fst_set.extend(first_table[r.ord()].iter().cloned()),
        };
      }

      let mut add_first = |rule: &R, token: R::TokenType| {
        if !first_table[rule.ord()].contains(&token) {
          changed = true;
          first_table[rule.ord()].insert(token);
        }
      };

      for t in fst_set {
        add_first(&production.rule, t);
      }
    }
  }

  first_table
}
