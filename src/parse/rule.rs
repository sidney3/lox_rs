use crate::core::Ordinal;
use crate::lexer::TokenType;

pub trait Rule: Ordinal {
    type TokenType: Ordinal + TokenType;

    fn goal() -> Self;
}
