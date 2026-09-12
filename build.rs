use lparse::generate_parser;
use std::env;
use std::path::Path;

fn main() {
  let grammar_path = "src/frontend/grammar.lparse";
  println!("cargo::rerun-if-changed={}", grammar_path);

  let out_dir = env::var("OUT_DIR").unwrap();
  let dest = Path::new(&out_dir).join("parser.rs");

  generate_parser(Path::new(grammar_path), &dest).unwrap();
}
