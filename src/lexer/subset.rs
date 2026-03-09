use super::nfa::{NFA, StateId};
use bitvec::prelude::{BitArr, bitarr};
use std::collections::VecDeque;

const MAX_CONCURRENT_STATES: usize = 512;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Subset {
    storage: BitArr!(for MAX_CONCURRENT_STATES),
}

impl FromIterator<StateId> for Subset {
    fn from_iter<I: IntoIterator<Item = StateId>>(iter: I) -> Self {
        Subset::make(iter)
    }
}

impl Subset {
    pub fn make<I: IntoIterator<Item = StateId>>(states: I) -> Self {
        let mut res = Self::make_empty();
        for state in states {
            res.insert(state);
        }
        res
    }

    pub fn make_empty() -> Self {
        return Subset {
            storage: bitarr![0; MAX_CONCURRENT_STATES],
        };
    }

    pub fn insert(&mut self, state: StateId) {
        self.storage.set(state.0, true);
    }

    pub fn contains(&self, state: StateId) -> bool {
        self.storage[state.0]
    }

    pub fn clear(&mut self) {
        self.storage.fill(false);
    }

    pub fn into_iter(&self) -> impl Iterator<Item = StateId> + '_ {
        self.storage.iter_ones().map(StateId)
    }
}

pub struct EpsilonClosure {
    visiting: VecDeque<StateId>,
    visited: Subset,
}

impl EpsilonClosure {
    pub fn make() -> Self {
        EpsilonClosure {
            visiting: VecDeque::new(),
            visited: Subset::make_empty(),
        }
    }

    pub fn compute<T>(&mut self, nfa: &NFA<T>, over: Subset) -> Subset {
        self.visited.clear();
        self.visiting = over.into_iter().collect();

        while let Some(x) = self.visiting.pop_front() {
            self.visited.insert(x);
            let closure = nfa[x]
                .epsilon_transitions
                .iter()
                .filter(|y| !self.visited.contains(**y));
            self.visiting.extend(closure);
        }

        self.visited
    }
}
