use super::action::Action;
use super::grammar::{Grammar, ProductionId, Symbol};
use super::item::{Item, ItemList};
use super::rule::Rule;
use crate::core::Ordinal;
use crate::lexer::TokenType;
use crate::parse::debug::{DisplayWithGrammar, DisplayWithGrammarExt};
use crate::usize_id;
use itertools::Itertools;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::hash::Hash;

usize_id!(StateId);

type ProductionList = SmallVec<[ProductionId; 10]>;

#[derive(Hash, Debug, Eq, PartialEq, Clone)]
pub struct State {
    in_progress: ItemList,
    complete: ProductionList,
}

impl State {
    /// Construct a new state.
    ///
    /// Note: we take the closure over in_progress.
    pub fn new<R: Rule>(
        grammar: &Grammar<R>,
        in_progress: impl Iterator<Item = Item>,
        complete: impl Iterator<Item = ProductionId>,
    ) -> Self {
        let mut in_progress: ItemList = closure(grammar, in_progress.collect());
        let mut complete: ProductionList = complete.collect();
        in_progress.sort();
        complete.sort();

        in_progress.dedup();
        complete.dedup();

        Self {
            in_progress,
            complete,
        }
    }

    pub fn initial<R: Rule>(grammar: &Grammar<R>) -> Self {
        let initial_productions: ItemList = grammar
            .productions_for_rule(grammar.target_rule())
            .iter()
            .map(|production_id| Item::new(*production_id))
            .collect();

        State::new(
            grammar,
            initial_productions.into_iter(),
            ProductionList::new().into_iter(),
        )
    }

    pub fn in_progress(&self) -> &ItemList {
        &self.in_progress
    }

    pub fn complete(&self) -> &ProductionList {
        &self.complete
    }

    pub fn is_accepting(&self) -> bool {
        self.in_progress().is_empty() && self.complete().is_empty()
    }

    pub fn edges<R: Rule>(&self, grammar: &Grammar<R>) -> impl Iterator<Item = Symbol<R>> {
        self.in_progress()
            .iter()
            .map(|item| item.current_symbol(grammar))
            .chain(
                self.complete()
                    .iter()
                    .map(|p| Symbol::Rule(grammar.production(*p).rule)),
            )
            .chain(
                self.in_progress()
                    .iter()
                    .map(|item| Symbol::Rule(grammar.production(item.production_id()).rule)),
            )
    }
    pub fn transition<R: Rule>(&self, grammar: &Grammar<R>, edge: Symbol<R>) -> Self {
        let (next_items, finished_productions): (ItemList, ProductionList) = self
            .in_progress()
            .iter()
            .filter_map(|item| item.advance(grammar, edge))
            .partition_map(|x| x);

        State::new(
            grammar,
            next_items.into_iter(),
            finished_productions.into_iter(),
        )
    }

    pub fn action<R: Rule>(
        &self,
        grammar: &Grammar<R>,
        follow: &Vec<HashSet<R::TokenType>>,
        token_type: &R::TokenType,
    ) -> Action {
        if *token_type == R::TokenType::eof() && self.is_accepting() {
            Action::Accept
        } else {
            let longest_complete = self
                .complete()
                .iter()
                .sorted() // to disambiguate
                .filter(|production_id| {
                    let rule_type = grammar.production(**production_id).rule;
                    follow[rule_type.ord()].contains(token_type)
                })
                .max_by_key(|production_id| grammar.production(**production_id).definition.len());

            match longest_complete {
                Some(&production_id) => Action::Reduce(production_id),
                None => Action::Shift,
            }
        }
    }
}

impl<R: Rule> DisplayWithGrammar<R> for State {
    fn fmt_with(&self, grammar: &Grammar<R>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{ in_progress: [")?;

        for (i, item) in self.in_progress().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item.with(grammar))?;
        }

        write!(f, "], \n")?;

        write!(f, "done: [")?;
        for production_id in self.complete() {
            production_id.fmt_with(grammar, f)?
        }
        write!(f, "] }}")?;

        Ok(())
    }
}

/// Compute all reachable items from an initial set of items.
///
/// For example:
///
/// I := 'b'B;
/// B := C;
/// C := 'a';
///
/// Note: the '.' signifies the current position in each of the below productions
///
/// items = {('b'.B)}
/// closure(items) = {('b'.B), (.C), (.'a')}
///
///
fn closure<R: Rule>(grammar: &Grammar<R>, mut items: ItemList) -> ItemList {
    let mut worklist: Vec<ProductionId> = items
        .iter()
        .flat_map(|i| i.current_symbol(grammar).productions(grammar))
        .collect();

    let mut visited: HashSet<ProductionId> = HashSet::new();

    while let Some(production_id) = worklist.pop() {
        if visited.contains(&production_id) {
            continue;
        } else {
            visited.insert(production_id);
        }

        items.push(Item::new(production_id));
        worklist.extend(
            grammar
                .production(production_id)
                .definition
                .first()
                .productions(grammar),
        );
    }

    items
}
