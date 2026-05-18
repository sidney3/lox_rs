use crate::core::ordinal::Ordinal;
use crate::lexer::TokenType;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;

pub trait Rule: Ordinal {
    type TokenType: Ordinal + TokenType;

    fn goal() -> Self;
}
