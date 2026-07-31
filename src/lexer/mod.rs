mod dfa;
mod engine;
mod error;
mod nfa;
mod regex;
mod span;
mod subset;
mod token;

#[cfg(test)]
mod fa_test;

pub use engine::{Lexer, Tokens};
pub use error::Error;
pub use span::Span;
pub use token::{TRIVIAL_SPAN, Token, TokenType};
