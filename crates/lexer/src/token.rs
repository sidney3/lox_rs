use std::hash::Hash;

use lasso::Spur;

use super::Span;

pub trait TokenType: Hash + Eq + Clone + Copy + PartialEq {
  fn eof() -> Self;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<T: TokenType> {
  pub lexeme: Spur,
  pub token_type: T,
  pub span: Span,
}
