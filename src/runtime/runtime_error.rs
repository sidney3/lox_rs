use crate::frontend::diagnostics::{Diagnostic, ToDiagnostic};

#[derive(Debug)]
pub struct RuntimeError {
  pub msg: String,
}

impl RuntimeError {
  pub fn new(msg: &str) -> Self {
    RuntimeError {
      msg: msg.to_string(),
    }
  }
  pub fn from_str(msg: String) -> Self {
    RuntimeError { msg }
  }
}

impl ToDiagnostic for RuntimeError {
  fn to_diagnostic(&self) -> Diagnostic {
    // TODO: RuntimeError should really carry the span. This will
    // be the big win of propagating our Spans through our AST nodes.
    Diagnostic::from_message(self.msg.clone())
  }
}
