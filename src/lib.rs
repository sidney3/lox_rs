mod asm;
mod compile;
mod executor;
mod frontend;
mod gc;
mod native_function;
mod obj;
mod runtime;

use std::path::PathBuf;

use log::error;

use crate::executor::Executor;
use crate::obj::Function;
use crate::runtime::{Root, Runtime, RuntimeError};
use lox_core::diagnostics::ToDiagnostic;

pub struct Config {
  pub script: PathBuf,
}

#[derive(Debug)]
pub enum LoxError {
  Compile(frontend::Error),
  Runtime(RuntimeError),
}

fn compile(program: &str, rt: &mut Runtime) -> Result<Root<Function>, frontend::Error> {
  let lexer = frontend::token::make_lox_lexer().expect("Token definition error");
  // let parser = frontend::ast::make_lox_parser();
  let parser = frontend::parser::LParseParser::new();

  let tokens = lexer.lex(program)?;
  let (lexeme_arena, program) = parser.parse(tokens)?;
  let ast = frontend::ast::Ast {
    lexeme_arena,
    root: program,
  };
  Ok(compile::Compiler::new(&ast, rt).compile())
}

pub fn run(config: Config) -> Result<(), LoxError> {
  let source_code = std::fs::read_to_string(config.script).expect("Source code reading error");

  let diagnostic_renderer = lox_core::diagnostics::DiagnosticRenderer::new(&source_code);
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
