use itertools::iproduct;
use ndarray::Array2;

use super::follow::follow;
use super::grammar::{Grammar, ProductionId};
use super::rule::Rule;
use super::state::{State, StateId};
use crate::core::Ordinal;
use crate::core::interner::Interner;

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
