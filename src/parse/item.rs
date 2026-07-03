use super::debug::DisplayWithGrammar;
use super::grammar::{Grammar, ProductionId, Symbol};
use super::rule::Rule;
use either::{Either, Left, Right};
use smallvec::SmallVec;
use std::hash::Hash;

// All items are in-progress items
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Item {
    pos: usize,
    production_id: ProductionId,
}

pub type AdvanceResult = Option<Either<Item, ProductionId>>;

impl Item {
    pub fn new(production_id: ProductionId) -> Self {
        Self {
            pos: 0,
            production_id,
        }
    }
    pub fn current_symbol<R: Rule>(&self, grammar: &Grammar<R>) -> Symbol<R> {
        let production = grammar.production(self.production_id);

        assert!(self.pos < production.definition.len());

        production.definition[self.pos]
    }
    /// Attempt to advance the item by a position
    ///
    /// None if the symbol does not match ``current_symbol``
    ///
    /// Return either the next item, or the completed production, in the case
    /// that this advance finished the item.
    pub fn advance<R: Rule>(&self, grammar: &Grammar<R>, by: Symbol<R>) -> AdvanceResult {
        if self.current_symbol(grammar) != by {
            None
        } else {
            let next_pos = self.pos + 1;

            if next_pos < grammar.production(self.production_id()).definition.len() {
                Some(Left(Item {
                    pos: next_pos,
                    production_id: self.production_id,
                }))
            } else {
                Some(Right(self.production_id()))
            }
        }
    }
    pub fn production_id(&self) -> ProductionId {
        self.production_id
    }
}

impl<R: Rule> DisplayWithGrammar<R> for Item {
    fn fmt_with(&self, grammar: &Grammar<R>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let production = &grammar.production(self.production_id);

        let before = production
            .definition
            .iter()
            .take(usize::from(self.production_id()));

        let after = production
            .definition
            .iter()
            .skip(usize::from(self.production_id()));

        for sym in before {
            write!(f, "{}", sym)?;
        }

        write!(f, "•")?;

        for sym in after {
            write!(f, "{}", sym)?;
        }

        Ok(())
    }
}

pub type ItemList = SmallVec<[Item; 10]>;
