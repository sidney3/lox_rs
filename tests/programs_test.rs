use std::path::Path;

use lox_rs::{Config, run};

const SAMPLE_PATH: &str = "tests/samples";

fn run_test_file(file_name: &str) {
  let _ = env_logger::builder().is_test(true).try_init();
  let config = Config {
    script: Path::new(env!("CARGO_MANIFEST_DIR"))
      .join(SAMPLE_PATH)
      .join(file_name),
  };

  let out = run(config);

  out.expect("expression test failed");
}

#[test]
pub fn test_expression() {
  run_test_file("expressions.lox");
}

#[test]
pub fn test_conditionals() {
  run_test_file("conditionals.lox");
}

#[test]
pub fn test_variables() {
  run_test_file("variables.lox");
}

#[test]
pub fn test_functions() {
  run_test_file("functions.lox");
}

#[test]
pub fn test_closures() {
  run_test_file("closures.lox");
}

#[test]
pub fn test_classes() {
  run_test_file("class.lox");
}

#[test]
pub fn test_inheritance() {
  run_test_file("inheritance.lox");
}

#[test]
pub fn test_long_file() {
  run_test_file("long_file.lox");
}
