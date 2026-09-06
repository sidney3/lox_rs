use thiserror::Error;

use super::Span;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Unterminated escape character: '{0}")]
  UnterminatedEscape(String),

  #[error("Unterminated regex: '{0}")]
  UnterminatedRegex(String),

  #[error("Malformatted range: '{0}")]
  MalformattedRange(String),

  #[error("Unordered range: '{0}'-'{1}'")]
  UnorderedRange(char, char),

  #[error("At pos {0}, no matching token")]
  NoMatchingToken(Span),
}

pub type Result<T> = std::result::Result<T, Error>;
