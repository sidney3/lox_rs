use crate::core::ordinal::Ordinal;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Rule: Debug + Sized + Hash + Eq + Clone {
    type RuleType: Ordinal;
    type TokenType: Ordinal;
    fn get_type(&self) -> Self::RuleType;
}
