use lox_core::usize_id;
use std::collections::HashSet;
use std::hash::Hash;

use either::Either;
use itertools::Itertools;
use smallvec::SmallVec;

use super::action::Action;
use super::debug::{DisplayWithGrammar, DisplayWithGrammarExt};
use super::grammar::{Grammar, ProductionId, Symbol};
use super::item::{Item, ItemList};
use super::rule::Rule;
use lexer::TokenType;

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
    let (mut in_progress, extra_complete) = closure(grammar, in_progress.collect());
    let mut complete: ProductionList = complete.collect();
    complete.extend(extra_complete);
    in_progress.sort();
    complete.sort();

    assert!(
      !in_progress
        .iter()
        .any(|i| grammar.production(i.production_id()).definition.is_empty()),
      "Cannot construct a state with an in progress empty item. These are trivial",
    );

    in_progress.dedup();
    complete.dedup();

    Self {
      in_progress,
      complete,
    }
  }

  pub fn initial<R: Rule>(grammar: &Grammar<R>) -> Self {
    let (initial_productions, trivially_complete): (ItemList, ProductionList) = grammar
      .productions_for_rule(grammar.target_rule())
      .iter()
      .partition_map(|production_id| Item::new(grammar, *production_id));

    State::new(
      grammar,
      initial_productions.into_iter(),
      trivially_complete.into_iter(),
    )
  }

  pub fn in_progress(&self) -> &ItemList {
    &self.in_progress
  }

  pub fn complete(&self) -> &ProductionList {
    &self.complete
  }

  pub fn edges<R: Rule>(&self, grammar: &Grammar<R>) -> impl Iterator<Item = Symbol<R>> {
    self
      .in_progress()
      .iter()
      .map(|item| item.current_symbol(grammar))
      .chain(
        self
          .complete()
          .iter()
          .map(|p| Symbol::Rule(grammar.production(*p).rule)),
      )
      .chain(
        self
          .in_progress()
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
    follow: &[HashSet<R::TokenType>],
    token_type: &R::TokenType,
  ) -> Action {
    let complete_target = self
      .complete()
      .iter()
      .cloned()
      .find(|&p| grammar.production(p).rule == grammar.target_rule());

    if let Some(p) = complete_target
      && *token_type == R::TokenType::eof()
    {
      Action::Accept(p)
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

    writeln!(f, "]")?;

    write!(f, "done: [")?;
    for production_id in self.complete() {
      let production = grammar.production(*production_id);

      write!(f, "{}", production.rule)?;
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
/// Closure also returns a set of potentially now completed items (which could
/// arise with epsilon items).
///
fn closure<R: Rule>(grammar: &Grammar<R>, mut items: ItemList) -> (ItemList, ProductionList) {
  let mut worklist: Vec<ProductionId> = items
    .iter()
    .flat_map(|i| i.current_symbol(grammar).productions(grammar))
    .collect();

  let mut visited: HashSet<ProductionId> = HashSet::new();
  let mut now_finished = ProductionList::new();

  while let Some(production_id) = worklist.pop() {
    if visited.contains(&production_id) {
      continue;
    } else {
      visited.insert(production_id);
    }

    if let Either::Left(item) = Item::new(grammar, production_id) {
      items.push(item);
    }
    match Item::new(grammar, production_id) {
      Either::Left(item) => items.push(item),
      Either::Right(completed_production) => now_finished.push(completed_production),
    };
    worklist.extend(
      grammar
        .production(production_id)
        .definition
        .first()
        .iter()
        .flat_map(|g| g.productions(grammar)),
    );
  }

  (items, now_finished)
}
