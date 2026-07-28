#[derive(Debug)]
pub struct RuntimeError {
  pub msg: String,
}

impl RuntimeError {
  pub fn new(msg: &str) -> Self {
    RuntimeError {
      msg: msg.to_string(),
    }
  }
  pub fn from_str(msg: String) -> Self {
    RuntimeError { msg }
  }
}
