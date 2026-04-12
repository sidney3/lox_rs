use super::grammar::{Grammar, ProductionId, Symbol};
use super::item::{Item, ItemList};
use super::rule::Rule;
use crate::usize_id;
use itertools::Itertools;
use smallvec::SmallVec;
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
/// We do not add duplicate productions. If a production is already in progress, there
/// is no point adding it twice (as our ``items`` represents the possible productions
/// we can create).
///
fn closure<R: Rule>(grammar: &Grammar<R>, mut items: ItemList) -> ItemList {
    let mut worklist: Vec<ProductionId> = items
        .iter()
        .flat_map(|i| i.current_symbol(grammar).productions(grammar))
        .collect();

    while let Some(production_id) = worklist.pop() {
        if items
            .iter()
            .any(|item| item.production_id() == production_id)
        {
            continue;
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
