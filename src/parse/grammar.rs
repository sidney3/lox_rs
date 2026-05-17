use super::rule::Rule;
use crate::core::ordinal::Ordinal;
use crate::lexer::{Token, TokenType};
use crate::usize_id;
use either::Either;
use nonempty::NonEmpty;
use smallvec::SmallVec;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug)]
pub enum Node<R: Rule> {
    NonTerminal(R),
    Terminal(Token<R::TokenType>),
}

#[derive(Debug, Hash, Eq, Clone, PartialEq)]
pub enum Symbol<R: Rule> {
    Token(R::TokenType),
    Rule(R::RuleType),
}

impl<R: Rule> Copy for Symbol<R> {}

pub struct Production<R: Rule> {
    pub rule: R::RuleType,

    /// What makes me sad about this code is that there is a runtime constraint established between definition and make_rule - the symbols of the definition always _should_ "match up" with the nodes that actually get passed in.
    ///
    /// I would _love_ to have some way to express this constraint, but I just
    /// can't think of a great way in the rust type system to do this.
    /// So we just panic if ``make_rule`` gets called with nodes
    /// that don't match ``definition``
    pub make_rule: Box<dyn Fn(Vec<Node<R>>) -> R>,
    pub definition: NonEmpty<Symbol<R>>,
}

impl<R: Rule> Production<R> {
    pub fn len(&self) -> usize {
        self.definition.len()
    }
}

impl<R: Rule> Display for Symbol<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(token_type) => token_type.fmt(f),
            Self::Rule(rule_type) => rule_type.fmt(f),
        }
    }
}

impl<R: Rule> Ordinal for Symbol<R> {
    const COUNT: usize = R::TokenType::COUNT + R::RuleType::COUNT;

    fn ord(&self) -> usize {
        match self {
            Self::Token(token_type) => token_type.ord(),
            Self::Rule(rule_type) => R::TokenType::COUNT + rule_type.ord(),
        }
    }
}

impl<R: Rule> Symbol<R> {
    pub fn productions(self, grammar: &Grammar<R>) -> impl Iterator<Item = ProductionId> {
        match self {
            Self::Rule(rule_type) => {
                Either::Right(grammar.productions_for_rule(rule_type).iter().cloned())
            }
            Self::Token(_) => Either::Left(std::iter::empty()),
        }
    }
}

usize_id!(ProductionId);
pub type ProductionList = SmallVec<[ProductionId; 10]>;

pub struct Grammar<R: Rule> {
    productions: Vec<Production<R>>, // indexed by ProductionId
    target_rule: R::RuleType,

    // Trivial derived properties of our grammar that we eagerly compute
    // More involved properties (goto table) get pulled off
    tokens: Vec<R::TokenType>,
    rules: Vec<R::RuleType>,
    productions_for_rule: Vec<ProductionList>,
}

impl<R: Rule> Grammar<R> {
    pub fn new(productions: Vec<Production<R>>, target_rule: R::RuleType) -> Self {
        let mut productions_for_rule = Vec::new();
        let mut tokens = HashSet::new();
        let mut rules = HashSet::new();

        for (i, production) in productions.iter().enumerate() {
            if production.rule.ord() >= productions_for_rule.len() {
                productions_for_rule.resize_with(production.rule.ord() + 1, ProductionList::new);
            }
            productions_for_rule[production.rule.ord()].push(ProductionId(i));
            rules.insert(production.rule);
            for symbol in &production.definition {
                match *symbol {
                    Symbol::Token(token_type) => tokens.insert(token_type),
                    Symbol::Rule(rule_type) => rules.insert(rule_type),
                };
            }
        }

        tokens.insert(R::TokenType::eof());

        Self {
            productions,
            target_rule,
            tokens: tokens.into_iter().collect(),
            rules: rules.into_iter().collect(),
            productions_for_rule,
        }
    }

    pub fn production(&self, p: ProductionId) -> &Production<R> {
        &self.productions[usize::from(p)]
    }
    pub fn productions_for_rule(&self, r: R::RuleType) -> &ProductionList {
        &self.productions_for_rule[r.ord()]
    }
    pub fn productions(&self) -> impl Iterator<Item = &Production<R>> {
        self.productions.iter()
    }
    pub fn tokens(&self) -> &Vec<R::TokenType> {
        &self.tokens
    }
    pub fn rules(&self) -> &Vec<R::RuleType> {
        &self.rules
    }
    pub fn target_rule(&self) -> R::RuleType {
        self.target_rule
    }
}
