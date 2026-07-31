use std::hash::Hash;

use super::Span;
use lasso::Spur;

pub trait TokenType: Hash + Eq + Clone + Copy + PartialEq {
  fn eof() -> Self;
}

pub const TRIVIAL_SPAN: Span = Span::new(0, 0);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Token<T: TokenType> {
  pub lexeme: Spur,
  pub token_type: T,
  pub span: Span,
}

impl<T: TokenType> Token<T> {
  pub fn canonical(self) -> Self {
    Token {
      lexeme: self.lexeme,
      token_type: self.token_type,
      span: TRIVIAL_SPAN,
    }
  }
}
