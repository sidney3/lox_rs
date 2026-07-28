mod asm;
mod codegen;
mod core;
mod frontend;
mod gc;
mod lexer;
mod parse;
mod vm;

use std::path::PathBuf;

use crate::vm::RuntimeError;

pub struct Config {
  pub script: PathBuf,
  pub disasm: bool,
}

pub fn run(config: Config) -> Result<(), RuntimeError> {
  let lexer = frontend::token::LoxLexer::new().unwrap();
  let parser = frontend::ast::LoxParser::new();

  let source_code = std::fs::read_to_string(config.script).unwrap();
  let token_result = lexer.lex(&source_code).unwrap();
  let ast = parser.parse(token_result).unwrap();
  let compiled = codegen::Compiler::new(&ast)
    .compile()
    .expect("Compilation failed");

  if config.disasm {
    println!("{:?}", compiled.main.chunk);
  }

  let mut vm = vm::Vm::new();
  vm.run(&compiled)
}
