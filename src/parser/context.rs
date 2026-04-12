use crate::core::interner::Interner;
use either::Either;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::hash::Hash;
use std::usize;

/*
 * The core parsing context.
 *
 * To avoid being burdened with generics, this module expects you to have
 * already interned your rules and tokens into `RuleId` and `TokenId`
 *
 * The thinking here is that only at the API boundaries will our system
 * think in terms of Rules and Tokens. Everywhere else, we just use
 * ids.
 *
 * Moreover, both the TokenIds and RuleIds should be sequences having
 * disjoint ranges.
 *
 */

macro_rules! usize_id {
    ($name:ident) => {
        #[derive(Hash, Eq, PartialEq, Clone, Copy)]
        pub struct $name(usize);

        impl From<$name> for usize {
            fn from(t: $name) -> usize {
                t.0
            }
        }
        impl From<usize> for $name {
            fn from(i: usize) -> $name {
                $name(i)
            }
        }
    };
}

usize_id!(ProductionId);
usize_id!(RuleId);
usize_id!(TokenId);
usize_id!(StateId);
usize_id!(SymbolId);

const MAX_PRODUCTION_LENGTH: usize = 10;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Symbol {
    Terminal(TokenId),
    NonTerminal(RuleId),
}

pub type SymbolVec = SmallVec<[Symbol; 10]>;
pub type RuleVec = SmallVec<[RuleId; 10]>;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct Item {
    pub production: ProductionId,
    // position is not the end - all Items are in progress
    pub position: usize,
}

pub type ItemVec = SmallVec<[Item; 10]>;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct State {
    pub in_progress: ItemVec,
    pub done: RuleVec,
}

pub struct Context {
    pub states: Interner<StateId, State>,
    productions: Interner<ProductionId, Vec<Symbol>>,
    // rules are 1-N with productions
    rule_definitions: HashMap<RuleId, Vec<ProductionId>>,
    production_rules: HashMap<ProductionId, RuleId>,
    num_rules: usize,
    num_tokens: usize,
}

impl Context {
    pub fn new(rules: &Vec<(RuleId, Vec<Symbol>)>, num_rules: usize, num_tokens: usize) -> Self {
        let max_rule_id = rules
            .iter()
            .map(|(rule_id, _)| usize::from(*rule_id))
            .max()
            .unwrap_or(0 as usize);

        let mut res = Self {
            productions: Interner::new(),
            states: Interner::new(),
            rule_definitions: HashMap::new(),
            production_rules: HashMap::new(),
            num_rules,
            num_tokens,
        };

        for x in rules {
            res.alloc_production(x.clone());
        }

        res
    }

    pub fn rule_definition(&self, r: RuleId) -> &Vec<ProductionId> {
        self.rule_definitions.get(&r).unwrap()
    }
    pub fn production_definition(&self, p: ProductionId) -> &Vec<Symbol> {
        self.productions.get_left(p)
    }

    pub fn symbol_id(&self, s: Symbol) -> SymbolId {
        match s {
            Symbol::NonTerminal(r) => SymbolId(r.0),
            Symbol::Terminal(t) => SymbolId(t.0 + self.num_rules),
        }
    }
    pub fn num_symbols(&self) -> usize {
        self.num_rules + self.num_tokens
    }

    fn alloc_production(&mut self, rule_def: (RuleId, Vec<Symbol>)) -> ProductionId {
        let (rule_id, productions) = rule_def;

        let production_id = self.productions.intern(productions);

        self.rule_definitions
            .entry(rule_id)
            .or_default()
            .push(production_id);

        self.production_rules.insert(production_id, rule_id);

        production_id
    }

    pub fn item_head(&self, item: Item) -> Symbol {
        self.productions.get_left(item.production)[item.position]
    }

    pub fn symbol_production(&self, sym: Symbol) -> impl Iterator<Item = &ProductionId> {
        match sym {
            Symbol::NonTerminal(rule_id) => Either::Left(self.rule_definitions[&rule_id].iter()),
            Symbol::Terminal(_) => Either::Right(std::iter::empty()),
        }
    }
    pub fn production_rule(&self, production_id: ProductionId) -> RuleId {
        self.production_rules.get(&production_id).unwrap().clone()
    }
}
