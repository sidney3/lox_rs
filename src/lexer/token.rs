use lasso::{Rodeo, Spur};
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;

pub trait TokenType: Hash + Eq + Clone + Copy + PartialEq + Display {}
impl<T: Hash + Eq + Clone + Copy + PartialEq + Display> TokenType for T {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<T: TokenType> {
    pub lexeme: Spur,
    pub token_type: T,
    pub line: usize,
}
