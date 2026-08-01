use thiserror::Error;

use crate::frontend::diagnostics::{Diagnostic, ToDiagnostic};
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

impl ToDiagnostic for Error {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::Lex(err) => err.to_diagnostic(),
      Self::Parse(err) => err.to_diagnostic(),
    }
  }
}
