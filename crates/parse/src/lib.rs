extern crate self as parse;

mod action;
mod debug;
mod error;
mod first;
mod follow;
mod goto;
mod grammar;
mod item;
mod parser;
mod rule;
mod state;

mod generated_lparse_parser;
mod lparse_compiler;
mod lparse_frontend;

pub use error::Error;
pub use grammar::{Grammar, Production, Symbol};
use lox_core::diagnostics::{self, Diagnostic, ToDiagnostic};
pub use parser::{Node, Parent, Parser, Tree};
pub use rule::Rule;

use std::path::Path;
use thiserror::Error;

use generated_lparse_parser::LParseParser;

#[derive(Debug, Error)]
pub enum ParseGenerateError {
  #[error(transparent)]
  Lex(#[from] lexer::Error),

  #[error(transparent)]
  Parse(#[from] Error),

  #[error(transparent)]
  Compile(#[from] lparse_compiler::Error),

  #[error(transparent)]
  IoError(#[from] std::io::Error),
}

pub fn run_generate_parser(input: &Path, output: &Path) -> Result<(), ParseGenerateError> {
  let lexer = lparse_frontend::lexer().expect("Ill-formed LParse tokens ");

  let program = std::fs::read_to_string(input)?;

  let tokens = lexer.lex(program.as_str())?;

  let (rodeo, ast) = LParseParser::new().parse(tokens)?;

  let output_tokens = lparse_compiler::compile(&rodeo, &ast)?;
  let file: syn::File = syn::parse2(output_tokens).expect("Ill-formed output file");

  std::fs::write(output, prettyplease::unparse(&file))?;

  Ok(())
}

impl ToDiagnostic for ParseGenerateError {
  fn to_diagnostic(&self) -> Diagnostic {
    match self {
      Self::Lex(l) => l.to_diagnostic(),
      Self::Parse(p) => p.to_diagnostic(),
      Self::IoError(io) => Diagnostic::from_message(io.to_string()),
      Self::Compile(compile) => Diagnostic::from_message(compile.to_string()),
    }
  }
}

pub fn generate_parser(input: &Path, output: &Path) -> Result<(), ParseGenerateError> {
  let input_text = std::fs::read_to_string(input)?;
  run_generate_parser(input, output).inspect_err(|e| {
    let renderer = diagnostics::DiagnosticRenderer::new(input_text.as_str());
    eprintln!("{}", renderer.render(&e.to_diagnostic()));
  })
}
