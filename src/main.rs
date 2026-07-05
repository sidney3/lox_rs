mod codegen;
mod core;
mod frontend;
mod lexer;
mod parse;
mod vm;

use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(name = "lox", version, about = "A Lox Runtime")]
struct Cli {
    /// Path to the .lox source file (omit for REPL)
    script: String,
}

fn main() {
    let cli = Cli::parse();

    let lexer = frontend::token::LoxLexer::new().unwrap();
    let parser = frontend::ast::LoxParser::new();

    let source_code = std::fs::read_to_string(cli.script).unwrap();
    let token_result = lexer.lex(&source_code).unwrap();
    let ast = parser.parse(token_result).unwrap();
    let chunk = codegen::Chunk::new(&ast);

    let mut vm = vm::Vm::new();
    vm.run(&chunk).unwrap();
}
