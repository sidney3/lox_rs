use super::{Grammar, Rule, Symbol};
use smallvec::{SmallVec, smallvec};
use std::collections::HashSet;

pub(super) fn first<R: Rule>(grammar: &Grammar<R>) -> Vec<HashSet<R::TokenType>> {
  let mut changed = true;
  let mut first_table: Vec<HashSet<R::TokenType>> = (0..R::COUNT).map(|_| HashSet::new()).collect();

  while changed {
    changed = false;

    for production in grammar.productions() {
      let fst_set: SmallVec<[R::TokenType; 8]> = match production.definition.first() {
        None => smallvec![],
        Some(&Symbol::Token(t)) => smallvec![t],
        Some(&Symbol::Rule(r)) => first_table[r.ord()].iter().cloned().collect(),
      };
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
