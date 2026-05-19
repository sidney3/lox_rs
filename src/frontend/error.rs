use crate::lexer::Error as LexError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Lex(#[from] LexError),
}

pub type Result<T> = std::result::Result<T, Error>;
