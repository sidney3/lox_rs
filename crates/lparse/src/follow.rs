use super::first::first;
use super::{Grammar, Rule, Symbol};
use itertools::Itertools;
use lexer::TokenType;
use log::debug;
use smallvec::SmallVec;
use std::collections::HashSet;

pub(super) fn follow<R: Rule>(grammar: &Grammar<R>) -> Vec<HashSet<R::TokenType>> {
  let mut changed = true;
  let mut follow_table: Vec<HashSet<R::TokenType>> =
    (0..R::COUNT).map(|_| HashSet::new()).collect();

  let _initial_symbols: Vec<_> = grammar
    .productions()
    .filter_map(|production| match production.definition.first() {
      Some(&Symbol::Token(t)) => Some(t),
      Some(Symbol::Rule(_)) => None,
      _ => None,
    })
    .collect();

  follow_table[grammar.target_rule().ord()].insert(R::TokenType::eof());
  let first_table = first(grammar);

  // classic fixed point calculation
  while changed {
    changed = false;
    for production in grammar.productions() {
      let follows: SmallVec<[R::TokenType; 10]> = follow_table[production.rule.ord()]
        .iter()
        .cloned()
        .collect();

      let mut new_follows: SmallVec<[(R, R::TokenType); 8]> = SmallVec::new();

      let def = &production.definition;

      for (i, sym) in def.iter().enumerate() {
        let Symbol::Rule(before) = sym else { continue };

        let include_first_of: SmallVec<[Symbol<R>; 8]> = def[i + 1..]
          .iter()
          .copied()
          .take_while_inclusive(
            |s| matches!(s, Symbol::Rule(r2) if grammar.rule_contains_epsilon(*r2)),
          )
          .collect();

        for after in &include_first_of {
          match after {
            Symbol::Token(t) => {
              new_follows.push((*before, *t));
            }
            Symbol::Rule(r2) => {
              for fst_token in &first_table[r2.ord()] {
                new_follows.push((*before, *fst_token));
              }
            }
          }
        }

        if let Some(last) = include_first_of.last()
          && let &Symbol::Rule(r) = last
          && grammar.rule_contains_epsilon(r)
        {
          for follow_token in &follow_table[r.ord()] {
            new_follows.push((*before, *follow_token));
          }
        }
      }

      // Suppose we have
      //
      // alpha = 'u';
      // beta = '(' alpha ')'
      //
      // Our stack is '(', 'u', and the next token is ')'. Then clearly
      // we should reduce to alpha, so ')' should be in follow(alpha) -
      //
      // rigorously, we say that if
      //
      // A = ....B;
      //
      // Then B should contain FOLLOW(A)
      if let Some(Symbol::Rule(r)) = production.definition.last() {
        for t in follows {
          new_follows.push((*r, t));
        }
      }

      for (r, t) in new_follows {
        changed = changed || follow_table[r.ord()].insert(t);
      }
    }
  }

  for r in grammar.rules() {
    let follow = &follow_table[r.ord()];

    debug!("Follow({r:?}) := {follow:?}");
  }

  follow_table
}
