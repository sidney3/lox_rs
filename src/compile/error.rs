use crate::frontend::diagnostics::{Diagnostic, ToDiagnostic};
use thiserror::Error;

use crate::asm;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  AssemblerError(#[from] asm::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl ToDiagnostic for Error {
  fn to_diagnostic(&self) -> Diagnostic {
    Diagnostic::from_message(format!("{self}"))
  }
}
