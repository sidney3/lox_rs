use std::collections::HashSet;

use ndarray::Array2;

use super::grammar::{Grammar, Symbol};
use super::rule::Rule;
use super::state::{State, StateId};
use lox_core::{InternPool, Ordinal};

pub fn make_goto<R: Rule>(
  grammar: &Grammar<R>,
  interner: &mut InternPool<StateId, State>,
) -> Array2<Option<StateId>> {
  let start_state = State::initial(grammar);
  let start_state_id = interner.get_or_intern(start_state);

  let mut visited: HashSet<StateId> = HashSet::new();
  let mut workset: Vec<StateId> = vec![start_state_id];
  let mut wip_goto: Vec<(StateId, Symbol<R>, StateId)> = Vec::new();

  while let Some(state_id) = workset.pop() {
    if visited.contains(&state_id) {
      continue;
    } else {
      visited.insert(state_id);
    }

    let state = interner[state_id].clone();

    for edge in state.edges(grammar) {
      let next_state = state.transition(grammar, edge);
      let next_state_id = interner.get_or_intern(next_state);

      workset.push(next_state_id);
      wip_goto.push((state_id, edge, next_state_id));
    }
  }

  let num_states = interner.len();
  let num_symbols = <Symbol<R> as Ordinal>::COUNT;
  let mut res = Array2::from_elem((num_states, num_symbols + 1), None);

  for (state_id, edge, next_state_id) in wip_goto {
    res[[state_id.0, edge.ord()]] = Some(next_state_id);
  }

  res
}
