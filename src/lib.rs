mod asm;
mod compile;
mod core;
mod executor;
mod frontend;
mod gc;
mod lexer;
mod parse;
mod runtime;

use log::error;
use std::path::PathBuf;
use thiserror::Error;

use crate::executor::Executor;
use crate::frontend::{Diagnostic, ToDiagnostic};
use crate::runtime::RuntimeError;

pub struct Config {
  pub script: PathBuf,
}

#[derive(Debug, Error)]
pub enum CompileError {
  #[error(transparent)]
  Frontend(#[from] frontend::Error),

  #[error(transparent)]
  Backend(#[from] compile::Error),
}

#[derive(Debug)]
pub enum LoxError {
  Compile(CompileError),
  Runtime(RuntimeError),
}

impl ToDiagnostic for CompileError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::Frontend(err) => err.to_diagnostic(),
      Self::Backend(err) => err.to_diagnostic(),
    }
  }
}

fn compile(program: &str) -> Result<compile::Compilation, CompileError> {
  let lexer = frontend::token::LoxLexer::new().expect("Token definition error");
  let parser = frontend::ast::LoxParser::new();

  let tokens = lexer.lex(program)?;
  let ast = parser.parse(tokens)?;
  Ok(compile::Compiler::new(&ast).compile()?)
}

pub fn run(config: Config) -> Result<(), LoxError> {
  let source_code = std::fs::read_to_string(config.script).expect("Source code reading error");

  let diagnostic_renderer = frontend::diagnostics::DiagnosticRenderer::new(&source_code);
  let mut vm = runtime::Runtime::new();

  let compiled = match compile(source_code.as_str()) {
    Ok(c) => c,
    Err(e) => {
      error!("{}", diagnostic_renderer.render(&e.to_diagnostic()));
      return Err(LoxError::Compile(e));
    }
  };

  match Executor::new(&mut vm, &compiled).run() {
    Ok(()) => Ok(()),
    Err(e) => {
      error!("{}", diagnostic_renderer.render(&e.to_diagnostic()));
      Err(LoxError::Runtime(e))
    }
  }
}
