mod codegen;
mod core;
mod frontend;
mod lexer;
mod parse;
mod vm;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "lox", version, about = "A Lox Runtime")]
struct Cli {
  /// Path to the .lox source file (omit for REPL)
  #[arg(short, long)]
  script: PathBuf,

  #[arg(long, default_value_t = false)]
  disasm: bool,
}

fn main() {
  env_logger::init();

  let cli = Cli::parse();

  let lexer = frontend::token::LoxLexer::new().unwrap();
  let parser = frontend::ast::LoxParser::new();

  let source_code = std::fs::read_to_string(cli.script).unwrap();
  let token_result = lexer.lex(&source_code).unwrap();
  let ast = parser.parse(token_result).unwrap();
  let chunk = codegen::Chunk::new(&ast);

  if (cli.disasm) {
    println!("{chunk}");
  }

  let mut vm = vm::Vm::new();
  vm.run(&chunk).unwrap();
}
