mod asm;
mod compile;
mod executor;
mod frontend;
mod gc;
mod native_function;
mod obj;
mod parse;
mod runtime;

use std::path::PathBuf;

use log::error;

use crate::executor::Executor;
use crate::frontend::ToDiagnostic;
use crate::obj::Function;
use crate::runtime::{Root, Runtime, RuntimeError};

pub struct Config {
  pub script: PathBuf,
}

#[derive(Debug)]
pub enum LoxError {
  Compile(frontend::Error),
  Runtime(RuntimeError),
}

fn compile(program: &str, rt: &mut Runtime) -> Result<Root<Function>, frontend::Error> {
  let lexer = frontend::token::LoxLexer::new().expect("Token definition error");
  let parser = frontend::ast::LoxParser::new();

  let tokens = lexer.lex(program)?;
  let ast = parser.parse(tokens)?;
  Ok(compile::Compiler::new(&ast, rt).compile())
}

pub fn run(config: Config) -> Result<(), LoxError> {
  let source_code = std::fs::read_to_string(config.script).expect("Source code reading error");

  let diagnostic_renderer = frontend::diagnostics::DiagnosticRenderer::new(&source_code);
  let mut rt = runtime::Runtime::new();

  let main = compile(source_code.as_str(), &mut rt).map_err(|e| {
    error!("{}", diagnostic_renderer.render(&e.to_diagnostic()));
    LoxError::Compile(e)
  })?;

  native_function::load_native_functions(&mut rt);

  Executor::new(&mut rt, main).run().map_err(|e| {
    error!("{}", diagnostic_renderer.render(&e.to_diagnostic()));
    LoxError::Runtime(e)
  })
}
