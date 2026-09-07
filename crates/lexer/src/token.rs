use std::hash::Hash;

use lasso::Spur;

use lox_core::Span;

pub trait TokenType: Hash + Eq + Clone + Copy + PartialEq {
  fn eof() -> Self;
  fn whitespace() -> Self;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<T: TokenType> {
  pub lexeme: Spur,
  pub token_type: T,
  pub span: Span,
}
