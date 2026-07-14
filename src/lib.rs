mod codegen;
mod core;
mod frontend;
mod gc;
mod lexer;
mod parse;
mod vm;

use std::path::PathBuf;

pub struct Config {
  pub script: PathBuf,
  pub disasm: bool,
}

pub fn run(config: Config) {
  let lexer = frontend::token::LoxLexer::new().unwrap();
  let parser = frontend::ast::LoxParser::new();

  let source_code = std::fs::read_to_string(config.script).unwrap();
  let token_result = lexer.lex(&source_code).unwrap();
  let ast = parser.parse(token_result).unwrap();
  let chunk = codegen::Chunk::new(&ast);

  if config.disasm {
    println!("{chunk}");
  }

  let mut vm = vm::Vm::new();
  vm.run(&chunk).unwrap();
}
