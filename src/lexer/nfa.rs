use super::fa_test::{FA, TokenType, run_tests};
use super::regex::Regex;
use super::subset::{EpsilonClosure, Subset};
use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::option::Option;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateId(pub usize);

impl<T> Index<StateId> for NFA<T> {
    type Output = State;

    fn index(&self, id: StateId) -> &Self::Output {
        &self.states[id.0]
    }
}

impl<T> IndexMut<StateId> for NFA<T> {
    fn index_mut(&mut self, id: StateId) -> &mut Self::Output {
        &mut self.states[id.0]
    }
}

pub struct State {
    pub epsilon_transitions: Vec<StateId>,
    pub transitions: HashMap<char, StateId>,
}

impl State {
    pub fn trivial() -> Self {
        State {
            epsilon_transitions: vec![],
            transitions: HashMap::new(),
        }
    }
}

pub struct NFA<T> {
    pub states: Vec<State>,
    pub terminal_states: HashMap<StateId, T>,
    pub rank: HashMap<T, usize>,
    pub initial_state: StateId,
}

// End is reachable from start, and has no outbound nodes
// (when returned)
struct Subgraph {
    pub start: StateId,
    pub end: StateId,
}

impl<T> NFA<T>
where
    T: Eq + Hash + Copy + Clone,
{
    fn alloc(&mut self) -> StateId {
        self.states.push(State::trivial());
        StateId(self.states.len() - 1)
    }

    fn parse(&mut self, regex: &Regex) -> Subgraph {
        let start = self.alloc();
        let end = self.alloc();

        match regex {
            Regex::Or(lhs, rhs) => {
                let lhs_subgraph = self.parse(lhs);
                let rhs_subgraph = self.parse(rhs);

                self[start]
                    .epsilon_transitions
                    .extend([lhs_subgraph.start, rhs_subgraph.start]);

                self[lhs_subgraph.end].epsilon_transitions.push(end);
                self[rhs_subgraph.end].epsilon_transitions.push(end);
            }

            Regex::Kleene(kleened) => {
                let start_prime = self.alloc();
                let subgraph = self.parse(kleened);

                self[start].epsilon_transitions.push(start_prime);

                self[start_prime].epsilon_transitions.push(end);

                self[subgraph.end].epsilon_transitions.push(start_prime);

                self[start_prime].epsilon_transitions.push(subgraph.start);
            }

            Regex::CharClass(class) => {
                for c in class.chars() {
                    self[start].transitions.insert(c, end);
                }
            }

            Regex::Literal(c) => {
                self[start].transitions.insert(*c, end);
            }
            Regex::Concat(lhs, rhs) => {
                let lhs_subgraph = self.parse(lhs);
                let rhs_subgraph = self.parse(rhs);

                self[start].epsilon_transitions.push(lhs_subgraph.start);
                self[lhs_subgraph.end]
                    .epsilon_transitions
                    .push(rhs_subgraph.start);
                self[rhs_subgraph.end].epsilon_transitions.push(end);
            }
        }

        Subgraph { start, end }
    }

    pub fn make(token_definitions: Vec<(T, Regex)>) -> Self {
        let states = vec![State::trivial()];

        let initial_state = StateId(0);

        let rank: HashMap<_, _> = token_definitions
            .iter()
            .map(|&(token, _)| token)
            .unique()
            .enumerate()
            .map(|(i, token)| (token, token_definitions.len() - i))
            // TODO:how does this magic work?
            .collect();

        let mut result = Self {
            states,
            terminal_states: HashMap::new(),
            rank,
            initial_state,
        };

        for (token, regex) in &token_definitions {
            let subgraph = result.parse(&regex);
            result[initial_state]
                .epsilon_transitions
                .push(subgraph.start);
            result.terminal_states.insert(subgraph.end, token.clone());
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl FA<TokenType> for NFA<TokenType> {
        fn make(token_defs: Vec<(TokenType, Regex)>) -> Self {
            Self::make(token_defs)
        }
        fn classify(&self, input: &str) -> Option<TokenType> {
            let mut closure = EpsilonClosure::make();
            let mut active_states =
                closure.compute(self, Subset::make(std::iter::once(self.initial_state)));

            for c in input.chars() {
                let next_states = active_states
                    .into_iter()
                    .filter_map(|s| self[s].transitions.get(&c).map(|x| *x))
                    .collect();

                active_states = closure.compute(self, next_states);
            }

            active_states
                .into_iter()
                .filter_map(|x| self.terminal_states.get(&x))
                .max_by_key(|t| self.rank.get(&t).copied().unwrap_or(0))
                .copied()
        }
    }

    #[test]
    fn test_nfa() {
        run_tests::<NFA<TokenType>>();
    }
}
