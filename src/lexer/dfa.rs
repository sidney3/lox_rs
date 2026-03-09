use super::nfa;
use super::nfa::NFA;
use super::regex::Regex;
use super::subset::{EpsilonClosure, Subset};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::option::Option;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateId(pub usize);

impl<T> Index<StateId> for DFA<T> {
    type Output = State;

    fn index(&self, id: StateId) -> &Self::Output {
        &self.states[id.0]
    }
}

impl<T> IndexMut<StateId> for DFA<T> {
    fn index_mut(&mut self, id: StateId) -> &mut Self::Output {
        &mut self.states[id.0]
    }
}

pub struct State {
    pub transitions: HashMap<char, StateId>,
}

impl State {
    pub fn trivial() -> Self {
        State {
            transitions: HashMap::new(),
        }
    }
}

pub struct DFA<T> {
    pub states: Vec<State>,
    pub terminal_states: HashMap<StateId, T>,
    pub initial_state: StateId,
}

impl<T> Display for DFA<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "DFA {{")?;

        for (i, state) in self.states.iter().enumerate() {
            let id = StateId(i);

            // Header: state id + flags
            write!(f, "  {:?} ", id)?;

            if id == self.initial_state {
                write!(f, "[START] ")?;
            }

            if let Some(tok) = self.terminal_states.get(&id) {
                write!(f, "[ACCEPT: {}]", tok)?;
            }

            writeln!(f)?;

            // Transitions
            for (c, next) in &state.transitions {
                writeln!(f, "    '{}' → {:?}", c, next)?;
            }

            // Optional: show dead states explicitly
            if state.transitions.is_empty() {
                writeln!(f, "    ∅")?;
            }

            writeln!(f)?;
        }

        writeln!(f, "}}")
    }
}

impl<T> DFA<T>
where
    T: Hash + Copy + Clone + Eq + PartialEq,
{
    pub fn make(nfa: NFA<T>) -> Self {
        DFABuilder::make().build(nfa)
    }

    fn test(&self, input: &str) -> Option<T> {
        let mut current_state = self.initial_state;

        for c in input.chars() {
            match self[current_state].transitions.get(&c) {
                Some(state) => {
                    current_state = *state;
                }
                None => {
                    return None;
                }
            }
        }

        self.terminal_states.get(&current_state).cloned()
    }
}

// Basically the DFA but easier to construct (we find out
// the initial node pretty late).
struct DFABuilder<T> {
    states: Vec<State>,
    terminal_states: HashMap<StateId, T>,
    subset_state: HashMap<Subset, StateId>,
}

impl<T> DFABuilder<T>
where
    T: Hash + Copy + Clone + Eq + PartialEq,
{
    pub fn make() -> Self {
        DFABuilder {
            states: Vec::new(),
            terminal_states: HashMap::new(),
            subset_state: HashMap::new(),
        }
    }

    pub fn build(mut self, nfa: NFA<T>) -> DFA<T> {
        let mut closure = EpsilonClosure::make();

        let initial_state: Subset =
            closure.compute(&nfa, std::iter::once(nfa.initial_state).collect());
        let initial_node = self.get_node(initial_state);
        let mut visiting: VecDeque<_> = std::iter::once(initial_state).collect();
        let mut visited = HashSet::new();

        while let Some(subset) = visiting.pop_front() {
            let node = self.get_node(subset);
            if visited.contains(&node) {
                continue;
            }

            visited.insert(node);
            self.subset_state.insert(subset, node);
            let transitioning_chars = subset
                .into_iter()
                .flat_map(|node| nfa[node].transitions.keys());

            self.states[node.0].transitions = transitioning_chars
                .map(|c| {
                    let directly_reachable: Subset = subset
                        .into_iter()
                        .filter_map(|node| nfa[node].transitions.get(c).cloned())
                        .collect();
                    let reachable = closure.compute(&nfa, directly_reachable);
                    visiting.push_back(reachable);

                    (*c, self.get_node(reachable))
                })
                .collect();
        }

        self.terminal_states = self
            .subset_state
            .iter()
            .filter_map(|(subset, node)| self.subset_token(&nfa, subset).map(|t| (*node, t)))
            .collect();
        DFA {
            states: self.states,
            terminal_states: self.terminal_states,
            initial_state: initial_node,
        }
    }

    fn subset_token(&self, nfa: &NFA<T>, subset: &Subset) -> Option<T> {
        subset
            .into_iter()
            .filter_map(|x: nfa::StateId| nfa.terminal_states.get(&x).copied())
            .max_by_key(|t| nfa.rank.get(t).copied().unwrap_or(0))
    }

    // lookup the node for a given subset, allocating a new one
    // if necessary.
    fn get_node(&mut self, subset: Subset) -> StateId {
        self.subset_state.get(&subset).cloned().unwrap_or_else(|| {
            let node = self.alloc();
            self.subset_state.insert(subset, node);
            node
        })
    }
    fn alloc(&mut self) -> StateId {
        let node = StateId(self.states.len());
        self.states.push(State::trivial());

        node
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    enum Token {
        Literal,
        BetterLiteral,
        A,
        AStar,
        AB,
        Alt,
    }

    impl Display for Token {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Token::Literal => write!(f, "Literal"),
                Token::BetterLiteral => write!(f, "BetterLiteral"),
                Token::A => write!(f, "A"),
                Token::AStar => write!(f, "AStar"),
                Token::AB => write!(f, "AB"),
                Token::Alt => write!(f, "Alt"),
            }
        }
    }

    fn make_dfa(tokens: Vec<(Token, &str)>) -> DFA<Token> {
        let as_regex = tokens
            .into_iter()
            .map(|(token, regex_str)| (token, Regex::make(regex_str).unwrap()))
            .collect();

        let nfa = NFA::make(as_regex);
        DFA::make(nfa)
    }

    #[test]
    fn test_literals() {
        let dfa = make_dfa(vec![(Token::Literal, ("abc"))]);

        assert!(dfa.test("").is_none());
        assert!(dfa.test("ab").is_none());
        assert!(dfa.test("abcd").is_none());
        assert_eq!(dfa.test("abc").unwrap(), Token::Literal);
    }

    #[test]
    fn test_ambiguous() {
        let dfa = make_dfa(vec![(Token::Literal, ("a*a"))]);

        assert!(dfa.test("").is_none());
        assert!(dfa.test("b").is_none());
        assert_eq!(dfa.test("a").unwrap(), Token::Literal);
        assert_eq!(dfa.test("aa").unwrap(), Token::Literal);
        assert_eq!(dfa.test("aaaaaaaa").unwrap(), Token::Literal);
    }

    #[test]
    fn test_token_precedence() {
        let dfa = make_dfa(vec![
            (Token::BetterLiteral, ("[a-z]*")),
            (Token::Literal, ("[a-z]*")),
        ]);

        assert_eq!(dfa.test("asbjasdflasdf").unwrap(), Token::BetterLiteral);
        assert_eq!(dfa.test("slasdf").unwrap(), Token::BetterLiteral);
        assert!(dfa.test("slasdf1alfs2").is_none());
    }

    #[test]
    fn test_everything() {
        let dfa = make_dfa(vec![
            (Token::A, ("a")),
            (Token::AStar, ("a*")),
            (Token::AB, ("ab")),
            (Token::Alt, ("a|b")),
        ]);

        assert_eq!(dfa.test("").unwrap(), Token::AStar);
        assert_eq!(dfa.test("a").unwrap(), Token::A);
        assert_eq!(dfa.test("ab").unwrap(), Token::AB);
    }
}
