use thiserror::Error;

use crate::frontend::diagnostics::{Diagnostic, ToDiagnostic};
use crate::lexer::Span;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Expected token")]
  ExpectedToken(Span),

  #[error("Unexpected token")]
  UnexpectedToken(Span), // A token lead you to a bad place

  #[error("Excess program")]
  ExcessProgram(Span), // you tried to accept with >1 remaining rule

  #[error("Incomplete program")]
  IncompleteProgram, // you didn't complete the target rule
}

pub type Result<T> = std::result::Result<T, Error>;

impl ToDiagnostic for Error {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      &Self::ExpectedToken(span) => Diagnostic::from_span("Expected token", span),
      &Self::UnexpectedToken(span) => Diagnostic::from_span("Unexpected token", span),
      &Self::ExcessProgram(span) => Diagnostic::from_span("Unexpected excess characters", span),
      Self::IncompleteProgram => Diagnostic::from_message("Incomplete program"),
    }
  }
}
