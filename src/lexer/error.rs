use crate::frontend::diagnostics::{Diagnostic, ToDiagnostic};
use crate::lexer::Span;
use thiserror::Error;

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

impl ToDiagnostic for Error {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::UnterminatedEscape(_) => Diagnostic::from_message(self),
      Self::UnterminatedRegex(_) => Diagnostic::from_message(self),
      Self::MalformattedRange(_) => Diagnostic::from_message(self),
      Self::UnorderedRange(_, _) => Diagnostic::from_message(self),

      &Self::NoMatchingToken(span) => Diagnostic::from_span("No matching token", span),
    }
  }
}
