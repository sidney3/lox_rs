use std::path::Path;

use lox_rs::{Config, run};

#[test]
fn test_expression() {
  let config = Config {
    script: Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/samples/variables.lox"),
    disasm: false,
  };

  let out = run(config);

  out.expect("variable test failed");
}
