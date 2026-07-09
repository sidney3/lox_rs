use thiserror::Error;

use crate::lexer::Error as LexError;
use crate::parse::Error as ParseError;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  Lex(#[from] LexError),

  #[error(transparent)]
  Parse(#[from] ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
