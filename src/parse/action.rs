use std::collections::HashSet;

use itertools::Itertools;
use itertools::iproduct;
use log::debug;
use ndarray::Array2;
use smallvec::SmallVec;

use super::grammar::{Grammar, ProductionId, Symbol};
use super::rule::Rule;
use super::state::{State, StateId};
use crate::core::Ordinal;
use crate::core::interner::Interner;
use crate::lexer::TokenType;

#[derive(Debug, Clone)]
pub enum Action {
  Reduce(ProductionId), // reduce to this production
  Accept(ProductionId), // reduce to this production and accept it as
  // final.
  Shift, // shift
}

// Indexed by (StateId, TokenId)
pub fn make_action<R: Rule>(
  grammar: &Grammar<R>,
  state_table: &Interner<StateId, State>,
) -> Array2<Action> {
  let num_states = state_table.len();
  let num_tokens = R::TokenType::COUNT;

  let mut res = Array2::from_elem((num_states, num_tokens), Action::Shift);

  let f = follow(grammar);

  for (state_id, token) in iproduct!(state_table.keys(), grammar.tokens().iter()) {
    let state = state_table.get_left(*state_id);

    res[[state_id.0, token.ord()]] = state.action(grammar, &f, token);
  }

  res
}

fn follow<R: Rule>(grammar: &Grammar<R>) -> Vec<HashSet<R::TokenType>> {
  let mut changed = true;
  let mut follow_table: Vec<HashSet<R::TokenType>> =
    (0..R::COUNT).map(|_| HashSet::new()).collect();

  let initial_symbols: Vec<_> = grammar
    .productions()
    .filter_map(|production| match production.definition.first() {
      &Symbol::Token(t) => Some(t),
      Symbol::Rule(_) => None,
    })
    .collect();

  for r in grammar.rules() {
    follow_table[r.ord()].insert(R::TokenType::eof());
    follow_table[r.ord()].extend(initial_symbols.iter());
  }

  // classic fixed point calculation
  while changed {
    changed = false;
    for production in grammar.productions() {
      let follows: SmallVec<[R::TokenType; 10]> = follow_table[production.rule.ord()]
        .iter()
        .cloned()
        .collect();

      let mut add_follow = |rule: &R, token: R::TokenType| {
        if !follow_table[rule.ord()].contains(&token) {
          changed = true;
          follow_table[rule.ord()].insert(token);
        }
      };
      for (s1, s2) in production.definition.iter().tuple_windows() {
        if let (Symbol::Rule(r), &Symbol::Token(t)) = (s1, s2) {
          add_follow(r, t);
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
      if let Symbol::Rule(r) = production.definition.last() {
        let _base = production.rule;
        for t in follows {
          add_follow(r, t);
        }
      }
    }
  }

  for r in grammar.rules() {
    let follow = &follow_table[r.ord()];

    debug!("Follow({r:?}) := {follow:?}");
  }

  follow_table
}
