mod asm;
mod compile;
mod core;
mod executor;
mod frontend;
mod gc;
mod lexer;
mod parse;
mod runtime;

use std::path::PathBuf;

use log::error;
use thiserror::Error;

use crate::executor::Executor;
use crate::frontend::{Diagnostic, ToDiagnostic};
use crate::runtime::{Function, Obj, Runtime, RuntimeError};

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

fn compile(program: &str, rt: &mut Runtime) -> Result<Obj<Function>, CompileError> {
  let lexer = frontend::token::LoxLexer::new().expect("Token definition error");
  let parser = frontend::ast::LoxParser::new();

  let tokens = lexer.lex(program)?;
  let ast = parser.parse(tokens)?;
  Ok(compile::Compiler::new(&ast, rt).compile()?)
}

pub fn run(config: Config) -> Result<(), LoxError> {
  let source_code = std::fs::read_to_string(config.script).expect("Source code reading error");

  let diagnostic_renderer = frontend::diagnostics::DiagnosticRenderer::new(&source_code);
  let mut rt = runtime::Runtime::new();

  let main = compile(source_code.as_str(), &mut rt).map_err(|e| {
    error!("{}", diagnostic_renderer.render(&e.to_diagnostic()));
    LoxError::Compile(e)
  })?;

  Executor::new(&mut rt, main).run().map_err(|e| {
    error!("{}", diagnostic_renderer.render(&e.to_diagnostic()));
    LoxError::Runtime(e)
  })
}
