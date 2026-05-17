use crate::core::ordinal::Ordinal;
use crate::lexer::TokenType;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Rule: Debug + Sized + Hash + Eq + Clone {
    type RuleType: Ordinal;
    type TokenType: Ordinal + TokenType;
}
