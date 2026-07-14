use lox_rs::{Config, run};
use std::path::Path;

#[test]
fn test_expression() {
  let config = Config {
    script: Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/samples/expressions.lox"),
    disasm: false,
  };

  let out = run(config);

  out.expect("expression test failed");
}
