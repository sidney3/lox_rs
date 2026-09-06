use lexer::TokenType;
use lox_core::Ordinal;

pub trait Rule: Ordinal {
  type TokenType: Ordinal + TokenType;
}
